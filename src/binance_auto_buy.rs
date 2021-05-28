use log::trace;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::binance_trade::{MarketQuantityType, TradeOrderType};

use crate::{
    binance_account_info::get_account_info,
    binance_exchange_info::{get_exchange_info, ExchangeInfo},
    binance_market_order_cmd::market_order,
    binance_order_response::TradeResponse,
    common::{are_you_sure_stdout_stdin, time_ms_to_utc, Side},
    configuration::Configuration,
};

pub async fn auto_buy(
    config: &Configuration,
    ei: &ExchangeInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    let test = config.test;
    trace!("auto_buy:+ test: {} config: {:?}", test, config);

    let mut ai = get_account_info(config).await?;
    ai.update_values_in_usd(config, true).await;
    //ai.print().await;

    // We assume the default_quote_asset is NOT empty
    assert!(!config.default_quote_asset.is_empty());

    let default_quote_asset_value =
        if let Some(b) = ai.balances_map.get(&config.default_quote_asset) {
            b.free
        } else {
            dec!(0)
        };

    struct ProcessRec {
        symbol_name: String,
        buy_value: Decimal,
        order_type: TradeOrderType,
    }

    let mut process_recs = Vec::<ProcessRec>::new();

    // Iterate over the BuyRec's and determine the buy_qty
    for br in config.buy.values() {
        let (quote_asset, quote_asset_value) = if !br.quote_asset.is_empty() {
            if let Some(b) = ai.balances_map.get(&br.quote_asset) {
                (br.quote_asset.as_str(), b.free)
            } else {
                (
                    config.default_quote_asset.as_str(),
                    default_quote_asset_value,
                )
            }
        } else {
            ("NONE", dec!(0))
        };

        let buy_value = br.percent * quote_asset_value;
        if buy_value > dec!(0) {
            let order_type = TradeOrderType::Market(MarketQuantityType::QuoteOrderQty(buy_value));
            let mut symbol_name = br.name.clone();
            symbol_name.push_str(quote_asset);

            process_recs.push(ProcessRec {
                symbol_name,
                buy_value,
                order_type,
            })
        }
    }

    // Print assets being bought
    for pr in &process_recs {
        print!("BUYING {:10.2} of {:10}", pr.buy_value, pr.symbol_name,);
    }

    if !process_recs.is_empty() {
        if test || are_you_sure_stdout_stdin() {
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
                                "{:8} ${} of {:10}",
                                "TEST OK", pr.order_type, pr.symbol_name,
                            );
                        }
                        TradeResponse::SuccessAck(atrr) => {
                            println!(
                                    "{:8} ${} of {:10} order_id: {}, order_list_id: {}, client_order_id: {}, transact_time: {}",
                                    "PENDING",
                                    pr.order_type,
                                    atrr.symbol,
                                    atrr.order_id,
                                    atrr.order_list_id,
                                    atrr.client_order_id,
                                    time_ms_to_utc(atrr.transact_time).to_string()
                                );
                        }
                        TradeResponse::FailureResponse(rer) => {
                            println!("{:8}, {} {}", "SKIPPING", pr.symbol_name, rer);
                        }
                        TradeResponse::FailureInternal(ier) => {
                            println!("{:8}, {} {}", "SKIPPING", pr.symbol_name, ier.msg);
                        }
                        TradeResponse::SuccessResult(_) => {}
                        TradeResponse::SuccessFull(_) => {}
                        TradeResponse::SuccessUnknown(_) => {}
                    },
                    Err(e) => println!("SKIPPING {}, {}", pr.symbol_name, e),
                }
            }
        }
    } else {
        println!("\n ** NOTHING to buy **");
    }
    println!();

    trace!("auto_buy:- test: {} config_file: {:?}", config.test, config);
    Ok(())
}

pub async fn auto_buy_cmd(config: &Configuration) -> Result<(), Box<dyn std::error::Error>> {
    trace!("auto_buy_cmd: {:#?}", config);

    let ei = get_exchange_info(config).await?;
    auto_buy(config, &ei).await?;

    Ok(())
}
