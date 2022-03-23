use log::trace;
use rust_decimal_macros::dec;
use std::io::{stdout, Write};
//use structopt::StructOpt;
// use tokio::fs;

use rust_decimal::prelude::*;

use crate::{
    binance_account_info::get_account_info,
    binance_exchange_info::{get_exchange_info, ExchangeInfo},
    binance_market_order_cmd::market_order,
    binance_order_response::TradeResponse,
    binance_trade::{MarketQuantityType, TradeOrderType},
    common::{are_you_sure_stdout_stdin, InternalErrorRec, Side},
    configuration::Configuration,
    ier_new,
};

use dec_utils::dec_to_money_string;
use time_ms_conversions::{time_ms_to_utc, utc_now_to_time_ms};

pub async fn auto_sell(
    config: &Configuration,
    ei: &ExchangeInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    let test = config.test;
    trace!("auto_sell:+ test: {} config: {:?}", test, config);

    trace!("auto_sell: call get_account_info");
    let time_ms = utc_now_to_time_ms();
    let mut ai = get_account_info(config, time_ms).await?;
    trace!("auto_sell: call ai.update_values_in_usd");
    ai.update_values_in_usd(config, config.verbose, time_ms)
        .await;
    trace!("auto_sell: retf ai.update_values_in_usd");
    //ai.print().await;

    #[derive(Default)]
    struct ProcessRec {
        asset: String,
        precision: usize,
        symbol_name: String,
        price_in_usd: Decimal,
        owned_qty: Decimal,
        sell_value_in_usd: Decimal,
        sell_qty: Decimal,
        keep_value_in_usd: Decimal,
        keep_qty: Decimal,
    }

    let mut vec_process_rec = Vec::new();
    for balance in ai.balances_map.values() {
        let keep_qty: Decimal;
        let sell_qty: Decimal;
        let quote_asset: &str;
        let symbol_name: String;
        let asset: String = balance.asset.clone();
        trace!("auto_sell: TOL balance: {:?}", balance);

        assert!(!config.default_quote_asset.is_empty());

        let owned_qty = balance.free + balance.locked;
        if owned_qty > dec!(0) {
            let keep_recs = if let Some(krs) = &config.keep {
                krs
            } else {
                return Err(ier_new!(8, "Missing `keep` field in configuration, make it empty if everything is to be sold: `keep = []`")
                    .into());
            };

            if let Some(keeping) = keep_recs.get(&balance.asset) {
                keep_qty = if keeping.min < Decimal::MAX && keeping.min < owned_qty {
                    keeping.min
                } else {
                    owned_qty
                };
                sell_qty = owned_qty - keep_qty;

                quote_asset = if keeping.quote_asset.is_empty() {
                    &config.default_quote_asset
                } else {
                    &keeping.quote_asset
                };
            } else {
                // Selling all
                keep_qty = dec!(0);
                sell_qty = owned_qty;
                quote_asset = &config.default_quote_asset;
            }

            if asset != quote_asset {
                symbol_name = asset.clone() + quote_asset;
                if let Some(symbol) = ei.get_symbol(&symbol_name) {
                    let precision = symbol.quote_precision as usize;

                    vec_process_rec.push(ProcessRec {
                        asset,
                        precision,
                        symbol_name,
                        price_in_usd: balance.price_in_usd,
                        owned_qty,
                        sell_value_in_usd: (sell_qty / owned_qty) * balance.value_in_usd,
                        sell_qty,
                        keep_value_in_usd: (keep_qty / owned_qty) * balance.value_in_usd,
                        keep_qty,
                    });
                } else {
                    trace!("auto_sell: {} not found, must be suspended", symbol_name);
                }
            }
        }
    }

    // Print assets being kept
    let mut kept_cnt: i64 = 0;
    for kr in &vec_process_rec {
        if kr.sell_qty <= dec!(0) {
            kept_cnt += 1;
            println!(
                "{0:8} {2:14.1$} of {3:10} at about {4:10}/per worth {5:10} selling NONE",
                "Keeping",
                kr.precision,
                kr.owned_qty,
                kr.asset,
                dec_to_money_string(kr.price_in_usd),
                dec_to_money_string(kr.keep_value_in_usd.round_dp(2)),
            );
        }
    }
    if kept_cnt > 0 {
        println!();
    }

    // Print assets being sold
    let mut total_sell_in_usd = dec!(0);
    let mut total_assets_selling_some_or_all = 0i64;
    for kr in &vec_process_rec {
        if kr.sell_qty > dec!(0) {
            print!(
                "{0:8} {2:14.1$} of {3:10} at about {4:10}/per worth {5:10} keeping ",
                "SELLING",
                kr.precision,
                kr.sell_qty,
                kr.asset,
                dec_to_money_string(kr.price_in_usd),
                dec_to_money_string(kr.sell_value_in_usd.round_dp(2)),
            );
            if kr.keep_qty > dec!(0) {
                println!(
                    "{:6} worth {:10}",
                    kr.keep_qty,
                    dec_to_money_string(kr.keep_value_in_usd.round_dp(2)),
                );
            } else {
                println!("NONE");
            }
            total_sell_in_usd += kr.sell_value_in_usd;
            total_assets_selling_some_or_all += 1;
        }
    }

    if total_assets_selling_some_or_all > 0 {
        println!(
            "\nSELLING {} assets for {:10}",
            total_assets_selling_some_or_all,
            dec_to_money_string(total_sell_in_usd.round_dp(2)),
        );
        if test || !config.confirmation_required || are_you_sure_stdout_stdin() {
            if test {
                println!();
            }
            for kr in &vec_process_rec {
                if kr.sell_qty > dec!(0) {
                    print!(
                        "{:8} {:14.6} of {:10}\r",
                        "Selling", kr.sell_qty, kr.symbol_name
                    );
                    let _ = stdout().flush();
                    let order_type =
                        TradeOrderType::Market(MarketQuantityType::Quantity(kr.sell_qty));
                    match market_order(config, ei, &kr.symbol_name, &order_type, Side::SELL, test)
                        .await
                    {
                        Ok(tr) => match tr {
                            TradeResponse::SuccessTest(_) => {
                                println!(
                                    "{0:8} {2:14.1$} of {3:10} at about {4:10}/per worth {5:10}",
                                    "TEST OK",
                                    kr.precision,
                                    kr.sell_qty,
                                    kr.symbol_name,
                                    dec_to_money_string(kr.price_in_usd),
                                    dec_to_money_string(kr.sell_value_in_usd.round_dp(2)),
                                );
                            }
                            TradeResponse::SuccessAck(atrr) => {
                                println!(
                                        "{0:8} {2:14.1$} of {3:10} order_id: {4}, order_list_id: {5}, client_order_id: {6}, transact_time: {7}",
                                        "PENDING",
                                        kr.precision,
                                        kr.sell_qty,
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
                                println!("{:8}, {} {}", "SKIPPING", kr.symbol_name, rer);
                            }
                            TradeResponse::FailureInternal(ier) => {
                                println!("{:8}, {} {}", "SKIPPING", kr.symbol_name, ier.msg);
                            }
                            _ => println!("Unexpected response: {}", tr),
                        },
                        Err(e) => println!("SKIPPING {}, {}", kr.symbol_name, e),
                    }
                }
            }
        } else {
            println!("\n ** Aborted **");
        }
    } else {
        println!("\n ** NOTHING to sell **");
    }
    println!();

    trace!(
        "auto_sell:- test: {} config_file: {:?}",
        config.test,
        config
    );
    Ok(())
}

//#[derive(Debug, Clone, Default, StructOpt)]
//#[structopt(
//    about = "Auto sell keeping some assets as defined in the keep section of the config file"
//)]

pub async fn auto_sell_cmd(config: &Configuration) -> Result<(), Box<dyn std::error::Error>> {
    trace!("auto_sell_cmd: {:#?}", config);

    //let mut ctx = ctx.clone();
    //let config = update_context_from_config_file(&mut ctx, &rec.config_file).await?;
    //let ctx = &ctx;

    let ei = get_exchange_info(config).await?;
    auto_sell(config, &ei).await?;

    Ok(())
}
