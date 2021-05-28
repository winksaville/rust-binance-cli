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
    common::{are_you_sure_stdout_stdin, time_ms_to_utc, Side},
    configuration::Configuration,
};

pub async fn auto_sell(
    config: &Configuration,
    ei: &ExchangeInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    let test = config.test;
    trace!("auto_sell:+ test: {} config: {:?}", test, config);

    let mut ai = get_account_info(config).await?;
    ai.update_values_in_usd(config, true).await;
    //ai.print().await;

    #[derive(Default)]
    struct ProcessRec {
        asset: String,
        price_in_usd: Decimal,
        quote_asset: String,
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

        assert!(!config.default_quote_asset.is_empty());

        let owned_qty = balance.free + balance.locked;
        if owned_qty > dec!(0) {
            if let Some(keeping) = config.keep.get(&balance.asset) {
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

            vec_process_rec.push(ProcessRec {
                asset: balance.asset.clone(),
                price_in_usd: balance.price_in_usd,
                quote_asset: quote_asset.to_string(),
                owned_qty,
                sell_value_in_usd: (sell_qty / owned_qty) * balance.value_in_usd,
                sell_qty,
                keep_value_in_usd: (keep_qty / owned_qty) * balance.value_in_usd,
                keep_qty,
            });
        }
    }

    // Print assets being kept
    let mut kept_cnt: i64 = 0;
    for kr in &vec_process_rec {
        if kr.sell_qty <= dec!(0) {
            kept_cnt += 1;
            println!(
                "Keeping {:14.6} of {:10} at about ${:10.2}/per worth ${:10.2} selling NONE",
                kr.owned_qty,
                kr.asset,
                kr.price_in_usd,
                kr.keep_value_in_usd.round_dp(2)
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
                "SELLING {:14.6} of {:10} at about ${:10.2}/per worth ${:10.2} keeping ",
                kr.sell_qty,
                kr.asset,
                kr.price_in_usd,
                kr.sell_value_in_usd.round_dp(2),
            );
            if kr.keep_qty > dec!(0) {
                println!(
                    "{:6} worth ${:10.2}",
                    kr.keep_qty,
                    kr.keep_value_in_usd.round_dp(2),
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
            "\nSELLING {} assets for ${:10.2}",
            total_assets_selling_some_or_all,
            total_sell_in_usd.round_dp(2)
        );
        if test || are_you_sure_stdout_stdin() {
            if test {
                println!();
            }
            for kr in &vec_process_rec {
                if kr.sell_qty > dec!(0) {
                    let symbol_name: String = kr.asset.clone() + &kr.quote_asset;
                    print!(
                        "{:8} {:14.6} of {:10}\r",
                        "Selling", kr.sell_qty, symbol_name
                    );
                    let _ = stdout().flush();
                    let order_type =
                        TradeOrderType::Market(MarketQuantityType::Quantity(kr.sell_qty));
                    match market_order(config, ei, &symbol_name, &order_type, Side::SELL, test)
                        .await
                    {
                        Ok(tr) => match tr {
                            TradeResponse::SuccessTest(_) => {
                                println!(
                                    "{:8} {:14.6} of {:10} at about ${:10.2}/per worth ${:10.2}",
                                    "TEST OK",
                                    kr.sell_qty,
                                    symbol_name,
                                    kr.price_in_usd,
                                    kr.sell_value_in_usd.round_dp(2)
                                );
                            }
                            TradeResponse::SuccessAck(atrr) => {
                                println!(
                                        "{:8} {:14.6} of {:10} order_id: {}, order_list_id: {}, client_order_id: {}, transact_time: {}",
                                        "PENDING",
                                        kr.sell_qty,
                                        atrr.symbol,
                                        atrr.order_id,
                                        atrr.order_list_id,
                                        atrr.client_order_id,
                                        time_ms_to_utc(atrr.transact_time).to_string()
                                    );
                            }
                            TradeResponse::FailureResponse(rer) => {
                                println!("{:8}, {} {}", "SKIPPING", symbol_name, rer);
                            }
                            TradeResponse::FailureInternal(ier) => {
                                println!("{:8}, {} {}", "SKIPPING", symbol_name, ier.msg);
                            }
                            _ => println!("{}", tr),
                        },
                        Err(e) => println!("SKIPPING {}, {}", symbol_name, e),
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
