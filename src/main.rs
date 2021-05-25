mod arg_matches;
mod binance_account_info;
mod binance_auto_sell;
mod binance_avg_price;
mod binance_exchange_info;
mod binance_get_klines_cmd;
mod binance_klines;
mod binance_market_order_cmd;
mod binance_my_trades;
mod binance_order_response;
mod binance_orders;
mod binance_signature;
mod binance_trade;
mod binance_verify_order;
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
use binance_auto_sell::auto_sell_cmd;
use configuration::Configuration;

extern crate function_name;
use function_name::named;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    binance_account_info::get_account_info,
    binance_avg_price::{get_avg_price, AvgPrice},
    binance_exchange_info::get_exchange_info,
    binance_get_klines_cmd::{get_klines_cmd, GetKlinesCmdRec},
    binance_klines::{get_kline, KlineRec},
    binance_market_order_cmd::{buy_market_order_cmd, sell_market_order_cmd},
    binance_my_trades::{get_my_trades, Trades},
    binance_order_response::TradeResponse,
    binance_orders::{get_all_orders, get_open_orders, Orders},
    common::{time_ms_to_utc, utc_now_to_time_ms},
};

fn get_configuration_and_sub_command(
) -> Result<(Configuration, Box<SubCommand<'static>>), Box<dyn std::error::Error>> {
    let the_matches = arg_matches()?;
    let config = Configuration::new(&the_matches);

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

#[named]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    trace!("main: +");

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
        println!("Usage: {} help, --help or -h", name);
        return Ok(());
    }

    let (config, subcmd) = get_configuration_and_sub_command()?;
    // println!("subcmd: {:#?}", subcmd);
    match subcmd.name.as_str() {
        "do-nothing" => {}
        "auto-sell" => {
            auto_sell_cmd(&config).await?;
        }
        "buy-market" => {
            let sym_name = subcmd
                .matches
                .value_of("SYMBOL")
                .expect("SYMBOL is missing");
            let q = subcmd
                .matches
                .value_of("QUANTITY")
                .expect("QUANTITY is missing");
            let quantity = match Decimal::from_str(q) {
                Ok(qty) => qty,
                Err(e) => return Err(format!("converting QUANTITY to Decimal: e={}", e).into()),
            };

            buy_market_order_cmd(&config, sym_name, quantity).await?;
        }
        "sell-market" => {
            let sym_name = subcmd
                .matches
                .value_of("SYMBOL")
                .expect("SYMBOL is missing");
            let q = subcmd
                .matches
                .value_of("QUANTITY")
                .expect("QUANTITY is missing");
            let quantity = match Decimal::from_str(q) {
                Ok(qty) => qty,
                Err(e) => return Err(format!("converting QUANTITY to Decimal: e={}", e).into()),
            };

            sell_market_order_cmd(&config, sym_name, quantity).await?;
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
        "st" => {
            // TODO: Add support for getting from_id, start_data_time, end_data_time and limit
            let sym_name = subcmd
                .matches
                .value_of("SYMBOL")
                .expect("SYMBOL is missing");
            let mt: Trades = get_my_trades(&config, sym_name, None, None, None, None).await?;
            let mut total_qty: Decimal = dec!(0);
            let mut total_quote_value: Decimal = dec!(0);
            for tr in &mt.trades {
                println!("Date: {}", time_ms_to_utc(tr.time));
                println!("{:#?}", tr);
                total_qty += tr.qty;
                total_quote_value += tr.quote_qty;
                println!(
                    "total_qty: {}, total_quote_value: {}",
                    total_qty, total_quote_value
                );
            }
            println!(
                "total_qty: {}, total_quote_value: {}",
                total_qty, total_quote_value
            );
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
                println!("No order log path, set it in the config file or use -l or --log_path");
            }
        },
        _ => println!("Unknown subcommand: {}", subcmd.name),
    }

    trace!("main: -");
    Ok(())
}
