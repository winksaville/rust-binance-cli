use std::{
    fmt::{self, Display},
    io::Write,
};

use clap::SubCommand;
use serde::{Deserialize, Serialize};

use log::trace;
#[allow(unused)]
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

#[allow(unused)]
use crate::{
    binance_account_info::get_account_info,
    binance_avg_price::{get_avg_price, AvgPrice},
    binance_exchange_info::{get_exchange_info, ExchangeInfo},
    binance_order_response::TradeResponse,
    binance_orders::get_open_orders,
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    binance_trade::{
        self, binance_new_order_or_test, order_log_file, MarketQuantityType, TradeOrderType,
    },
    binance_verify_order::{
        adj_quantity_verify_lot_size, verify_max_position, verify_min_notional, verify_open_orders,
        verify_quanity_is_less_than_or_eq_free,
    },
    common::utc_now_to_time_ms,
    common::{InternalErrorRec, Side},
    configuration::Configuration,
    ier_new,
};
use crate::{
    binance_order_response::WithdrawResponseRec,
    binance_trade::log_order_response,
    common::{post_req_get_response, ResponseErrorRec},
};

// Define an Amount as Quantity or Percent
// Possibly extend to Value in USD or EUR or ...
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Amount {
    Quantity(Decimal),
    Percent(Decimal),
}

impl Default for Amount {
    fn default() -> Amount {
        Amount::Quantity(dec!(0))
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Amount::Percent(p) => write!(f, "{:.4}%", p),
            Amount::Quantity(q) => write!(f, "{:.4}", q),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithdrawParams {
    pub sym_name: String,
    pub amount: Amount,
    pub quantity: Decimal,
    pub address: String,
    pub secondary_address: Option<String>,
    pub label: Option<String>,
}

impl Default for WithdrawParams {
    fn default() -> WithdrawParams {
        WithdrawParams {
            sym_name: "".to_string(),
            amount: Amount::default(),
            quantity: dec!(0),
            address: "".to_string(),
            secondary_address: None,
            label: None,
        }
    }
}

impl WithdrawParams {
    pub fn from_subcommand(
        subcmd: &SubCommand,
    ) -> Result<WithdrawParams, Box<dyn std::error::Error>> {
        let sym_name = if let Some(s) = subcmd.matches.value_of("SYMBOL") {
            s.to_string()
        } else {
            return Err("SYMBOL is missing".into());
        };
        let amt_val = if let Some(a) = subcmd.matches.value_of("AMOUNT") {
            a
        } else {
            return Err("AMOUNT is missing".into());
        };
        let amount = if let Some(amt) = amt_val.strip_suffix('%') {
            let percent = match Decimal::from_str(amt) {
                Ok(qty) => qty,
                Err(e) => {
                    return Err(format!("converting {} to Decimal: e={}", amt, e).into());
                }
            };

            Amount::Percent(percent)
        } else {
            let quantity = match Decimal::from_str(amt_val) {
                Ok(qty) => qty,
                Err(e) => return Err(format!("converting {} to Decimal: e={}", amt_val, e).into()),
            };

            Amount::Quantity(quantity)
        };
        let address = if let Some(a) = subcmd.matches.value_of("ADDRESS") {
            a.to_string()
        } else {
            return Err("ADDRESS is missing".into());
        };

        let secondary_address = subcmd
            .matches
            .value_of("dest-sec-addr")
            .map(|s| s.to_string());
        let label = subcmd.matches.value_of("dest-label").map(|s| s.to_string());

        Ok(WithdrawParams {
            sym_name,
            amount,
            quantity: dec!(0),
            address,
            secondary_address,
            label,
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

    let url = config.make_url("api", &format!("{}?", full_path));
    trace!("withdraw_post_and_repsonse: url={}", url);

    let tr = if !config.test {
        let response = post_req_get_response(api_key, &url, &query_string).await?;
        trace!("withdraw_post_and_repsonse: response={:#?}", response);
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
                &format!(r#"response_body: {}"#, response_body),
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
    let ai = get_account_info(config).await?;
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
    trace!("withdraw: Got symbol");

    let balance = if let Some(b) = ai.balances_map.get(&params.sym_name) {
        b
    } else {
        return Err(format!("Error, {} is not in your account", params.sym_name).into());
    };
    println!("balance: {}\n{:#?}", params.sym_name, balance);

    let quantity = match params.amount {
        Amount::Percent(p) => {
            let q = (p / dec!(100)) * balance.free;
            trace!("Percent: {}", q);

            q
        }
        Amount::Quantity(q) => {
            trace!("Quantity: {}", q);

            q
        }
    };

    trace!("withdraw: quantity={}", quantity);

    let mut params_x = params.clone();
    params_x.quantity = quantity;
    let params = params_x;

    verify_quanity_is_less_than_or_eq_free(&ai, symbol, quantity)?;
    let quantity_string = quantity.to_string();

    let mut param_tuples = vec![
        ("asset", params.sym_name.as_str()),
        ("address", params.address.as_str()),
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
