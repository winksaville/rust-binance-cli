mod arg_matches;
mod binance_account_info;
mod binance_auto_buy;
mod binance_auto_sell;
mod binance_avg_price;
mod binance_exchange_info;
mod binance_get_klines_cmd;
mod binance_history;
mod binance_klines;
mod binance_market_order_cmd;
mod binance_my_trades;
mod binance_order_response;
mod binance_orders;
mod binance_signature;
mod binance_trade;
mod binance_verify_order;
mod binance_withdraw_cmd;
mod common;
mod configuration;
mod de_string_or_number;

use clap::SubCommand;
use log::trace;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use std::{path::Path, str::FromStr};

use arg_matches::arg_matches;
use binance_auto_buy::auto_buy_cmd;
use binance_auto_sell::auto_sell_cmd;
use configuration::Configuration;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    binance_account_info::get_account_info,
    binance_avg_price::{get_avg_price, AvgPrice},
    binance_exchange_info::get_exchange_info,
    binance_get_klines_cmd::{get_klines_cmd, GetKlinesCmdRec},
    binance_history::{
        get_deposit_history, get_fiat_currency_deposit_history, get_fiat_currency_withdraw_history,
        get_withdraw_history, AssetLogRec, DepositRec, WithdrawRec,
    },
    binance_klines::{get_kline, KlineRec},
    binance_market_order_cmd::{buy_market_order_cmd, sell_market_order_cmd},
    binance_my_trades::{get_my_trades, Trades},
    binance_order_response::TradeResponse,
    binance_orders::{get_all_orders, get_open_orders, Orders},
    binance_trade::{MarketQuantityType, TradeOrderType},
    binance_withdraw_cmd::{withdraw_cmd, WithdrawParams},
    common::{
        dec_to_money_string, dec_to_separated_string, time_ms_to_utc, utc_now_to_time_ms,
        InternalErrorRec,
    },
};

fn get_configuration_and_sub_command(
) -> Result<(Configuration, Box<SubCommand<'static>>), Box<dyn std::error::Error>> {
    let the_matches = arg_matches()?;
    let config = Configuration::new(&the_matches)?;

    let subcmd = if let Some(sc) = the_matches.subcommand {
        sc
    } else {
        // Shouldn't happen because arg_matches should have returned and Error.
        // but if it does we'll panic with a "nice" message
        let vec_params: Vec<String> = std::env::args().collect();
        let parameters = vec_params.join(" ");
        unreachable!("Unexpectedly there was no subcommand: {}", parameters);
    };
    trace!("subcmd: {:#?}", subcmd);

    Ok((config, subcmd))
}

use common::APP_VERSION;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    trace!("main:+ args().len()={}", std::env::args().len());

    // If no commands display a simple usage message to terminal
    if std::env::args().len() == 1 {
        let args: Vec<String> = std::env::args().collect();
        let prog_name = Path::new(&args[0]).file_name();
        let name = match prog_name {
            Some(pn) => match pn.to_str() {
                Some(n) => n,
                None => &args[0],
            },
            None => &args[0],
        };
        println!(
            "Usage:   {} help, --help or -h\napp-ver: {}",
            name,
            APP_VERSION.as_str()
        );
        return Ok(());
    }

    let (config, subcmd) = get_configuration_and_sub_command()?;
    // println!("subcmd: {:#?}", subcmd);

    fn get_sym_qty_or_val(
        subcmd: &SubCommand,
        quantity_or_value: &str,
    ) -> Result<(String, Decimal), Box<dyn std::error::Error>> {
        let sym_name = subcmd
            .matches
            .value_of("SYMBOL")
            .unwrap_or_else(|| panic!("SYMBOL is missing"));
        let q = subcmd
            .matches
            .value_of(quantity_or_value)
            .unwrap_or_else(|| panic!("{} is missing", quantity_or_value));
        let quantity = match Decimal::from_str(q) {
            Ok(qty) => qty,
            Err(e) => {
                return Err(format!("converting {} to Decimal: e={}", quantity_or_value, e).into())
            }
        };

        Ok((sym_name.to_string(), quantity))
    }

    match subcmd.name.as_str() {
        "do-nothing" => {}
        "version" => {
            println!("{}", APP_VERSION.as_str());
        }
        "auto-sell" => {
            auto_sell_cmd(&config).await?;
        }
        "auto-buy" => {
            auto_buy_cmd(&config).await?;
        }
        "buy-market-value" => {
            let (sym_name, value) = get_sym_qty_or_val(&subcmd, "VALUE")?;
            let order_type = TradeOrderType::Market(MarketQuantityType::QuoteOrderQty(value));
            buy_market_order_cmd(&config, &sym_name, order_type).await?;
        }
        "buy-market" => {
            let (sym_name, quantity) = get_sym_qty_or_val(&subcmd, "QUANTITY")?;
            let order_type = TradeOrderType::Market(MarketQuantityType::Quantity(quantity));
            buy_market_order_cmd(&config, &sym_name, order_type).await?;
        }
        "sell-market-value" => {
            let (sym_name, value) = get_sym_qty_or_val(&subcmd, "VALUE")?;
            let order_type = TradeOrderType::Market(MarketQuantityType::QuoteOrderQty(value));
            sell_market_order_cmd(&config, &sym_name, order_type).await?;
        }
        "sell-market" => {
            let (sym_name, quantity) = get_sym_qty_or_val(&subcmd, "QUANTITY")?;
            let order_type = TradeOrderType::Market(MarketQuantityType::Quantity(quantity));
            sell_market_order_cmd(&config, &sym_name, order_type).await?;
        }
        "withdraw" => {
            let params = WithdrawParams::from_subcommand(&subcmd)?;
            withdraw_cmd(&config, &params).await?;
        }
        "ai" => {
            let mut ai = get_account_info(&config).await?;
            ai.update_and_print(&config).await;
        }
        "ei" => {
            let ei = get_exchange_info(&config).await?;
            println!("ei={:#?}", ei);
        }
        "sei" => {
            let sym_name = subcmd
                .matches
                .value_of("SYMBOL")
                .expect("SYMBOL is missing");
            let ei = get_exchange_info(&config).await?;
            if let Some(sym) = ei.get_symbol(sym_name) {
                println!("{}: {:#?}", sym.symbol, sym);
            } else {
                println!("No such symbol {}", sym_name);
            }
        }
        "sap" => {
            let sym_name = subcmd
                .matches
                .value_of("SYMBOL")
                .expect("SYMBOL is missing");
            let ap: AvgPrice = get_avg_price(&config, sym_name).await?;
            println!("ap: mins={} price={}", ap.mins, ap.price);
        }
        "skr" => {
            let sym_name = subcmd
                .matches
                .value_of("SYMBOL")
                .expect("SYMBOL is missing");
            let kr: KlineRec = get_kline(&config, sym_name, utc_now_to_time_ms()).await?;
            println!("{}", kr);
        }
        "skrs" => {
            let mut rec = GetKlinesCmdRec::default();
            let sym_name = subcmd
                .matches
                .value_of("SYMBOL")
                .expect("SYMBOL is missing");
            rec.sym_name = sym_name.to_string();
            if let Some(limit_str) = subcmd.matches.value_of("LIMIT") {
                println!("skrs: limit={}", limit_str);
                rec.limit = Some(u16::from_str(limit_str)?);
            } else {
                rec.limit = None;
            };
            if let Some(st_str) = subcmd.matches.value_of("START-TIME") {
                println!("skrs: start_date_time={}", st_str);
                rec.start_date_time = Some(st_str.to_string());
            } else {
                rec.start_date_time = None;
            }
            if let Some(interval_str) = subcmd.matches.value_of("INTERVAL") {
                println!("skrs: interval={}", interval_str);
                rec.interval = Some(interval_str.to_string());
            } else {
                rec.interval = None;
            }
            get_klines_cmd(&config, &rec).await?;
        }
        "oo" => {
            let sym_name = subcmd
                .matches
                .value_of("SYMBOL")
                .expect("SYMBOL is missing");
            let oo: Orders = get_open_orders(&config, &sym_name).await?;
            println!("oo: {:#?}\nsum_buy_orders: {}", oo, oo.sum_buy_orders());
        }
        "ao" => {
            // TODO: Add support for getting order_id, start_date_time, end_date_time and limit
            let ei = get_exchange_info(&config).await?;
            for symbol in ei.symbols_map.values() {
                let o: Orders =
                    get_all_orders(&config, &symbol.symbol, None, None, None, None).await?;
                if !o.orders.is_empty() {
                    println!("o: {:#?}", o);
                }
            }
        }
        "mt" => {
            // TODO: Add support for getting from_id, start_data_time, end_data_time and limit
            let sym_name = subcmd
                .matches
                .value_of("SYMBOL")
                .expect("SYMBOL is missing");
            let ei = get_exchange_info(&config).await?;
            let symbol = if let Some(s) = ei.get_symbol(sym_name) {
                s
            } else {
                return Err(
                    crate::ier_new!(9, &format!("Symbol {} isn't valid: ", sym_name)).into(),
                );
            };
            let asset = &symbol.base_asset;

            // Add in deposit History for the sym
            let drs: Vec<DepositRec> =
                get_deposit_history(&config, Some(asset), None, None, None).await?;
            let mut dep_qty = dec!(0);
            for dr in &drs {
                println!(
                    "Deposit: amount: {} Date: {}",
                    dr.amount,
                    time_ms_to_utc(dr.insert_time)
                );
                trace!("{:#?}", dr);
                dep_qty += dr.amount;
            }
            println!(
                "deposits: {} qty: {}",
                drs.len(),
                dec_to_separated_string(dep_qty, 4),
            );

            let wrs: Vec<WithdrawRec> =
                get_withdraw_history(&config, Some(asset), None, None, None).await?;
            let mut wd_qty = dec!(0);
            for wd in &wrs {
                println!(
                    "Withdrawl: amount: {} txs fee: {} Date: {}",
                    wd.amount,
                    wd.transaction_fee,
                    time_ms_to_utc(wd.apply_time)
                );
                trace!("{:#?}", wd);
                wd_qty += wd.amount; // - wd.transaction_fee;
            }
            println!(
                "withdrawals: {} qty: {}",
                wrs.len(),
                dec_to_separated_string(wd_qty, symbol.base_asset_precision),
            );

            let mt: Trades = get_my_trades(&config, sym_name, None, None, None, None).await?;
            let mut buy_txs: usize = 0;
            let mut buy_qty = dec!(0);
            let mut buy_quote_qty = dec!(0);
            let mut sell_txs: usize = 0;
            let mut sell_qty = dec!(0);
            let mut sell_quote_qty = dec!(0);
            let mut commission_total_usd = dec!(0); // TODO: Should allow different conversions?
            for tr in &mt.trades {
                print!("Trade: orderId: {}", tr.order_id);
                if tr.is_buyer {
                    buy_txs += 1;
                    buy_qty += tr.qty;
                    buy_quote_qty += tr.quote_qty;
                    print!(" buy ");
                } else {
                    sell_txs += 1;
                    sell_qty += tr.qty;
                    sell_quote_qty += tr.quote_qty;
                    print!(" sell");
                }
                print!(
                    " {} {} value {}",
                    symbol.base_asset,
                    dec_to_separated_string(tr.qty, symbol.base_asset_precision),
                    dec_to_money_string(tr.quote_qty),
                );

                let commission_usd = binance_trade::convert(
                    &config,
                    tr.time,
                    &tr.commission_asset,
                    tr.commission,
                    "USD",
                )
                .await?;
                commission_total_usd += commission_usd;

                // Delay so as not to be a bad binance citizen
                std::thread::sleep(std::time::Duration::from_millis(500));

                print!(
                    " commission: {} commision usd: {} commission_asset: {}",
                    tr.commission, commission_usd, tr.commission_asset
                );
                println!(" Date: {}", time_ms_to_utc(tr.time));
                trace!("{:#?}", tr);
            }

            println!(
                "total buy transactions: {} buy qty: {} buy quote_qty: {}",
                buy_txs,
                dec_to_separated_string(buy_qty, symbol.base_asset_precision),
                dec_to_money_string(buy_quote_qty),
            );
            println!(
                "total sell transactions: {} sell qty: {} sell quote_qty: {}",
                sell_txs,
                dec_to_separated_string(sell_qty, symbol.base_asset_precision),
                dec_to_money_string(sell_quote_qty),
            );
            println!(
                "commission for {} trades, value USD: {} ",
                mt.trades.len(),
                dec_to_money_string(commission_total_usd),
            );
        }
        "dh" => {
            // TODO: Add support for getting status, start_data_time, end_data_time
            let asset = subcmd.matches.value_of("ASSET");
            let dh: Vec<DepositRec> = get_deposit_history(&config, asset, None, None, None).await?;
            println!("{:#?}", dh);
        }
        "wh" => {
            // TODO: Add support for getting status, start_data_time, end_data_time
            let asset = subcmd.matches.value_of("ASSET");
            let wh: Vec<WithdrawRec> =
                get_withdraw_history(&config, asset, None, None, None).await?;
            println!("{:#?}", wh);
        }
        "fcdh" => {
            // TODO: Add support for getting status, start_data_time, end_data_time
            let asset = subcmd.matches.value_of("FIAT_CURRENCY");
            let dhfc: Vec<AssetLogRec> = get_fiat_currency_deposit_history(
                &config, asset, None, None, None, None, None, None,
            )
            .await?;
            println!("{:#?}", dhfc);
        }
        "fcwh" => {
            // TODO: Add support for getting status, start_data_time, end_data_time
            let asset = subcmd.matches.value_of("FIAT_CURRENCY");
            let whfc: Vec<AssetLogRec> = get_fiat_currency_withdraw_history(
                &config, asset, None, None, None, None, None, None,
            )
            .await?;
            println!("{:#?}", whfc);
        }
        "ol" => match config.order_log_path {
            Some(path) => {
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                for result in reader.lines() {
                    let line = result?;
                    let tr: TradeResponse = serde_json::from_str(&line)?;
                    println!("{:#?}", tr);
                }
            }
            None => {
                println!("No order log path, set it in the config file or use --order_log_path");
            }
        },
        _ => println!("Unknown subcommand: {}", subcmd.name),
    }

    trace!("main: -");
    Ok(())
}
