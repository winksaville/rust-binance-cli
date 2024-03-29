use std::{
    fmt::{self, Display},
    io::Write,
};

use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use time_ms_conversions::utc_now_to_time_ms;

use crate::{
    binance_account_info::{get_account_info, AccountInfo},
    binance_avg_price::get_avg_price,
    binance_exchange_info::{get_exchange_info, ExchangeInfo},
    binance_order_response::TradeResponse,
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    binance_trade::order_log_file,
    binance_verify_order::{adj_quantity_verify_lot_size, verify_quanity_is_less_than_or_eq_free},
    common::InternalErrorRec,
    configuration::Configuration,
    ier_new,
};
use crate::{
    binance_order_response::WithdrawResponseRec,
    binance_trade::log_order_response,
    common::{post_req_get_response, ResponseErrorRec},
};

// Define an Amount as Percent, Quantity or Dollars
// Possibly extend to other fiat currencies in the future
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Amount {
    Percent(Decimal),
    Quantity(Decimal),
    Dollars(Decimal),
}

impl Default for Amount {
    fn default() -> Amount {
        Amount::Quantity(dec!(0))
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Amount::Percent(p) => write!(f, "{p:.4}%"),
            Amount::Quantity(q) => write!(f, "{q:.4}"),
            Amount::Dollars(d) => write!(f, "${d:.2}"),
        }
    }
}

impl Amount {
    pub fn new(amt_val: &str) -> Result<Amount, Box<dyn std::error::Error>> {
        //println!("amt_val={}", amt_val);
        let amount = if let Some(amt) = amt_val.strip_suffix('%') {
            //println!("Percent amt={}", amt);
            let percent = match Decimal::from_str(amt) {
                Ok(qty) => qty,
                Err(e) => {
                    return Err(format!("converting {amt} to Decimal: e={e}").into());
                }
            };

            //println!("Percent percent={}", percent);
            Amount::Percent(percent)
        } else if let Some(amt) = amt_val.strip_prefix('$') {
            //println!("Dollars amt={}", amt);
            let quantity = match Decimal::from_str(amt) {
                Ok(qty) => qty,
                Err(e) => return Err(format!("converting {amt_val} to Decimal: e={e}").into()),
            };
            //println!("Dollars quantity={}", amt);

            Amount::Dollars(quantity)
        } else {
            //println!("Quantity amt_val={}", amt_val);
            let quantity = match Decimal::from_str(amt_val) {
                Ok(qty) => qty,
                Err(e) => return Err(format!("converting {amt_val} to Decimal: e={e}").into()),
            };

            //println!("Quantity quantity={}", quantity);
            Amount::Quantity(quantity)
        };

        Ok(amount)
    }

    pub async fn to_quantity(
        &self,
        config: &Configuration,
        ai: &AccountInfo,
        sym_name: &str,
    ) -> Result<Decimal, Box<dyn std::error::Error>> {
        match self {
            Amount::Percent(p) => {
                let balance = if let Some(b) = ai.balances_map.get(sym_name) {
                    b
                } else {
                    return Err(format!("Error, {sym_name} is not in your account").into());
                };

                let q = (p / dec!(100)) * balance.free;
                trace!(
                    "to_quantity Amount::Percent: {}% {} {} balance={:?}",
                    p,
                    q,
                    sym_name,
                    balance
                );

                Ok(q)
            }
            Amount::Quantity(q) => {
                trace!("to_quantity Amount::Quantity: {} {}", q, sym_name);

                Ok(*q)
            }
            Amount::Dollars(d) => {
                let full_symbol = sym_name.to_string() + "USD";
                let avp = match get_avg_price(config, &full_symbol).await {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(
                            format!("Unable to determin avg price of {full_symbol}, {e}").into(),
                        )
                    }
                };

                let q = d / avp.price;
                trace!("to_quantity Amount::Dollars: ${} {} {}", d, q, sym_name);

                Ok(q)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithdrawParams {
    pub sym_name: String,
    pub amount: Amount,
    #[serde(default)]
    pub org_quantity: Decimal,
    pub quantity: Decimal,
    #[serde(default)]
    pub quantity_usd: Decimal,
    pub address: Option<String>,
    pub secondary_address: Option<String>,
    pub label: Option<String>,
    #[serde(default)]
    pub keep_min_amount: Option<Amount>,
}

impl Default for WithdrawParams {
    fn default() -> WithdrawParams {
        WithdrawParams {
            sym_name: "".to_string(),
            amount: Amount::default(),
            org_quantity: dec!(0),
            quantity: dec!(0),
            quantity_usd: dec!(0),
            address: None,
            secondary_address: None,
            label: None,
            keep_min_amount: None,
        }
    }
}

impl WithdrawParams {
    pub fn from_subcommand(
        sc_matches: &ArgMatches,
    ) -> Result<WithdrawParams, Box<dyn std::error::Error>> {
        let sym_name = if let Some(s) = sc_matches.value_of("SYMBOL") {
            s.to_string()
        } else {
            return Err("SYMBOL is missing".into());
        };
        let amt_val = if let Some(a) = sc_matches.value_of("AMOUNT") {
            a
        } else {
            return Err("AMOUNT is missing".into());
        };
        let amount = Amount::new(amt_val)?;

        let withdraw_addr = sc_matches.value_of("withdraw-addr").map(|s| s.to_string());

        let keep_min = sc_matches.value_of("keep-min").map(|s| s.to_string());
        let keep_min_amount = match keep_min {
            Some(v) => Some(Amount::new(&v)?),
            None => None,
        };

        let secondary_address = sc_matches.value_of("dest-sec-addr").map(|s| s.to_string());
        let label = sc_matches.value_of("dest-label").map(|s| s.to_string());

        Ok(WithdrawParams {
            sym_name,
            amount,
            org_quantity: dec!(0),
            quantity: dec!(0),
            quantity_usd: dec!(0),
            address: withdraw_addr,
            secondary_address,
            label,
            keep_min_amount,
        })
    }
}

async fn withdraw_post_and_response(
    config: &Configuration,
    mut log_writer: &mut dyn Write,
    full_path: &str,
    params: &WithdrawParams,
    mut param_tuples: Vec<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = config.keys.get_ak_or_err()?;
    let secret_key = &config.keys.get_sk_vec_u8_or_err()?;

    param_tuples.push(("recvWindow", "5000"));

    let ts_string: String = format!("{}", utc_now_to_time_ms());
    param_tuples.push(("timestamp", ts_string.as_str()));

    let mut query = query_vec_u8(&param_tuples);

    // Calculate the signature using sig_key and the data is qs and query as body
    let signature = binance_signature(secret_key, &query, &[]);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("withdraw_post_and_repsonse: query_string={}", &query_string);

    let url = config.make_url("api", &format!("{full_path}?"));
    trace!("withdraw_post_and_repsonse: url={}", url);

    let tr = if !config.test {
        let response = post_req_get_response(api_key, &url, &query_string).await?;
        trace!("withdraw_post_and_repsonse: response={:#?}", response);
        let response_headers = response.headers().clone();
        let response_status = response.status();
        let response_body = response.text().await?;
        trace!(
            "withdraw_post_and_repsonse: response_status={} response_body={}",
            response_status,
            response_body
        );

        // Process the response
        if response_status == 200 {
            let mut response: WithdrawResponseRec = serde_json::from_str(&response_body)?;
            response.test = config.test;
            response.query = query_string;
            response.params = params.clone();
            response.response_body = response_body;

            trace!(
                "withdraw_post_and_repsonse: WithdrawResponseRec={}",
                response
            );

            if response.success {
                TradeResponse::SuccessWithdraw(response)
            } else {
                TradeResponse::FailureWithdraw(response)
            }
        } else {
            let rer = ResponseErrorRec::new(
                false,
                response_status.as_u16(),
                &query_string,
                response_headers,
                &format!(r#"response_body: {response_body}"#),
            );
            trace!(
                "{}",
                format!("withdraw_post_and_repsonse: ResponseErrRec={:#?}", &rer)
            );

            TradeResponse::FailureResponse(rer)
        }
    } else {
        let wrr = WithdrawResponseRec {
            msg: "successful test".into(),
            success: true,
            id: "a-test-id".into(),
            test: config.test,
            query: query_string,
            params: params.clone(),
            response_body: "".to_string(),
        };
        trace!(
            "{}",
            format!(
                "withdraw_post_and_repsonse: test WithdrawResonseRec={:#?}",
                &wrr
            )
        );

        TradeResponse::SuccessTestWithdraw(wrr)
    };
    println!("{}", &tr);

    // Log response and return Ok or Err
    log_order_response(&mut log_writer, &tr)
}

pub async fn withdraw(
    config: &Configuration,
    ei: &ExchangeInfo,
    params: &WithdrawParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let ai = get_account_info(config, utc_now_to_time_ms()).await?;
    trace!("withdraw: Got AccountInfo: {:#?}", ai);

    let order_log_path = if let Some(olp) = &config.order_log_path {
        olp
    } else {
        return Err("No order log path, set it in the config file or use --order_log_path".into());
    };
    let mut log_writer = order_log_file(order_log_path)?;

    let full_name = params.sym_name.to_string() + "USD";
    let symbol = match ei.get_symbol(&full_name) {
        Some(s) => s,
        None => {
            return Err(ier_new!(2, &format!("No asset named {}", params.sym_name)).into());
        }
    };
    trace!("withdraw: Got symbol: {:?}", symbol);

    let org_quantity = params
        .amount
        .to_quantity(config, &ai, &params.sym_name)
        .await?;
    trace!("org_quantity: {}", org_quantity);

    let config_withdraw_addr = config.withdraw_addr.clone();
    let params_address = params.address.clone();
    let withdraw_addr = match params_address {
        Some(addr) => addr,
        None => {
            if let Some(addr) = config_withdraw_addr {
                addr
            } else {
                return Err(ier_new!(2, "No withdraw address given, expecting --withdraw-addr option or withdraw_addr in configuration toml file.").into());
            }
        }
    };
    // Guranatee that withdraw_addr is not empty
    assert!(!withdraw_addr.is_empty());

    let keep_quantity = match &params.keep_min_amount {
        Some(amount) => Some(amount.to_quantity(config, &ai, &params.sym_name).await?),
        None => None,
    };
    trace!("keep_quantity: {:?}", keep_quantity);

    let balance = if let Some(b) = ai.balances_map.get(&params.sym_name) {
        b
    } else {
        return Err(format!("Error, {} is not in your account", params.sym_name).into());
    };

    // Do not withdraw below the quantity we need to keep for transaction fees and such
    let org_quantity = if let Some(kq) = keep_quantity {
        if org_quantity > balance.free - kq {
            // Whoops org_quantity is too large
            let q = balance.free - kq;
            trace!("adj for keep quantity, new org_quantity:{} as org_quantity:{} > balance.free:{} - kq:{}", q, org_quantity, balance.free, kq);

            q
        } else {
            // org_quantity is fine
            org_quantity
        }
    } else {
        // No keep_quantity
        org_quantity
    };

    let quantity = adj_quantity_verify_lot_size(symbol, org_quantity);
    trace!(
        "withdraw: quantity={} after adjusting for lot_size",
        quantity
    );

    let mut params_x = params.clone();
    params_x.org_quantity = org_quantity;
    params_x.address = Some(withdraw_addr.clone());
    params_x.quantity = quantity;
    params_x.quantity_usd = quantity * get_avg_price(config, &full_name).await?.price;
    let params = params_x;

    verify_quanity_is_less_than_or_eq_free(&ai, symbol, quantity)?;
    let quantity_string = quantity.to_string();

    let mut param_tuples = vec![
        ("asset", params.sym_name.as_str()),
        ("address", withdraw_addr.as_str()),
        ("amount", quantity_string.as_str()),
    ];
    let sa_string: String;
    if let Some(sa) = params.secondary_address.clone() {
        sa_string = sa;
        param_tuples.push(("addressTag", sa_string.as_str()))
    }
    let label_string: String;
    if let Some(l) = params.label.clone() {
        label_string = l;
        param_tuples.push(("name", label_string.as_str()))
    }

    withdraw_post_and_response(
        config,
        &mut log_writer,
        "/wapi/v3/withdraw.html",
        &params,
        param_tuples,
    )
    .await
}

pub async fn withdraw_cmd(
    config: &Configuration,
    params: &WithdrawParams,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!(
        "withdraw_cmd:\nconfig:\n{:#?}\nparams:\n{:#?}",
        config,
        params,
    );

    let ei = &get_exchange_info(config).await?;
    withdraw(config, ei, params).await?;

    Ok(())
}
