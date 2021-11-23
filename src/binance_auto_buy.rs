use log::trace;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    binance_account_info::get_account_info,
    binance_exchange_info::{get_exchange_info, ExchangeInfo},
    binance_market_order_cmd::market_order,
    binance_order_response::TradeResponse,
    binance_trade::{MarketQuantityType, TradeOrderType},
    common::{are_you_sure_stdout_stdin, time_ms_to_utc, InternalErrorRec, Side},
    configuration::Configuration,
    ier_new,
};

pub async fn auto_buy(
    config: &Configuration,
    ei: &ExchangeInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    let test = config.test;
    trace!("auto_buy:+ test: {} config: {:#?}", test, config);

    let mut ai = get_account_info(config).await?;
    ai.update_values_in_usd(config, config.verbose).await;
    //ai.print().await;

    // Verify the default_quote_asset is NOT empty
    assert!(!config.default_quote_asset.is_empty());

    // Value held by the account of the default_quote_asset
    let default_quote_asset_value =
        if let Some(b) = ai.balances_map.get(&config.default_quote_asset) {
            b.free
        } else {
            dec!(0)
        };
    trace!(
        "auto-buy: default_quote_asset_value: {}",
        default_quote_asset_value
    );

    struct ProcessRec {
        symbol_name: String,
        precision: usize,
        buy_value: Decimal,
        order_type: TradeOrderType,
    }

    let mut process_recs = Vec::<ProcessRec>::new();

    let buy_recs = if let Some(brs) = &config.buy {
        brs
    } else {
        return Err(ier_new!(8, "Missing `buy` field in configuration").into());
    };

    // Iterate over the BuyRec's and determine the buy_qty
    for br in buy_recs.values() {
        trace!("auto-buy: tol adding process_recs br: {:#?}", br);
        let (quote_asset, quote_asset_value) = if br.quote_asset.is_empty() {
            (
                config.default_quote_asset.as_str(),
                default_quote_asset_value,
            )
        } else if let Some(b) = ai.balances_map.get(&br.quote_asset) {
            (br.quote_asset.as_str(), b.free)
        } else {
            println!(
                "{:8}, {} is not a valid symbol or not owned",
                "SKIPPING", br.quote_asset
            );
            ("NONE", dec!(0))
        };
        trace!(
            "auto-buy: quote_asset: {} quote_asset_value: {}",
            quote_asset,
            quote_asset_value
        );

        let buy_value = (br.percent / dec!(100)) * quote_asset_value;
        trace!("auto-buy: buy_value: {}", buy_value);

        if buy_value > dec!(0) && br.name != quote_asset {
            let symbol_name = br.name.clone() + quote_asset;
            let sym = match ei.get_symbol(&symbol_name) {
                Some(s) => s,
                None => {
                    return Err(
                        format!("{} is not a valid symbol on the exchange", symbol_name).into(),
                    )
                }
            };

            if !sym.quote_order_qty_market_allowed {
                return Err(format!("{} is not allowed to be a QuoteOrderQty", symbol_name).into());
            }
            let precision = sym.quote_precision as usize;
            let buy_value = buy_value.round_dp(sym.quote_precision);
            trace!("auto-buy: rounded buy_value: {}", buy_value);

            let order_type = TradeOrderType::Market(MarketQuantityType::QuoteOrderQty(buy_value));
            process_recs.push(ProcessRec {
                symbol_name,
                precision,
                buy_value,
                order_type,
            })
        }
    }

    // Print assets being bought
    for pr in &process_recs {
        println!(
            "{0:8} {2:14.1$} of {3:10}",
            "BUYING", pr.precision as usize, pr.buy_value, pr.symbol_name,
        );
    }

    if !process_recs.is_empty() {
        if test || !config.confirmation_required || are_you_sure_stdout_stdin() {
            // Do the auto-buy
            for pr in process_recs {
                match market_order(
                    config,
                    ei,
                    &pr.symbol_name,
                    &pr.order_type,
                    Side::BUY,
                    config.test,
                )
                .await
                {
                    Ok(tr) => match tr {
                        TradeResponse::SuccessTest(_) => {
                            println!(
                                "{0:8} {2:14.1$} of {3:10}",
                                "TEST OK", pr.precision, pr.buy_value, pr.symbol_name,
                            );
                        }
                        TradeResponse::SuccessAck(atrr) => {
                            println!(
                                    "{0:8} {2:14.1$} of {3:10} order_id: {4}, order_list_id: {5}, client_order_id: {6}, transact_time: {7}",
                                    "PENDING",
                                    pr.precision,
                                    pr.buy_value,
                                    atrr.symbol,
                                    atrr.order_id,
                                    atrr.order_list_id,
                                    atrr.client_order_id,
                                    time_ms_to_utc(atrr.transact_time)
                                );
                        }
                        TradeResponse::SuccessResult(rtrr) => {
                            println!("{}", rtrr);
                        }
                        TradeResponse::SuccessFull(ftrr) => {
                            println!("{}", ftrr);
                        }
                        TradeResponse::SuccessUnknown(utrr) => {
                            println!("{}", utrr);
                        }
                        TradeResponse::FailureResponse(rer) => {
                            println!("{:8}, {:10} {}", "SKIPPING", pr.symbol_name, rer);
                        }
                        TradeResponse::FailureInternal(ier) => {
                            println!("{:8}, {:10} {}", "SKIPPING", pr.symbol_name, ier.msg);
                        }
                        _ => println!("Unexpected response: {}", tr),
                    },
                    Err(e) => println!("{:8} {:10}, {}", "SKIPPING", pr.symbol_name, e),
                }
            }
        }
    } else {
        println!("\n ** NOTHING to buy **");
    }
    println!();

    trace!("auto_buy:- test: {}", config.test);
    Ok(())
}

pub async fn auto_buy_cmd(config: &Configuration) -> Result<(), Box<dyn std::error::Error>> {
    trace!("auto_buy_cmd: {:#?}", config);

    let ei = get_exchange_info(config).await?;
    auto_buy(config, &ei).await?;

    Ok(())
}
