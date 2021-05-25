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

use crate::{binance_account_info::get_account_info, binance_avg_price::{get_avg_price, AvgPrice}, binance_exchange_info::get_exchange_info, binance_get_klines_cmd::{GetKlinesCmdRec, get_klines_cmd}, binance_klines::{get_kline, KlineRec}, binance_market_order_cmd::{buy_market_order_cmd, sell_market_order_cmd}, binance_my_trades::{get_my_trades, Trades}, binance_order_response::TradeResponse, binance_orders::{get_all_orders, get_open_orders, Orders}, common::{time_ms_to_utc, utc_now_to_time_ms}};

#[named]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    trace!("main: +");

    let matches = arg_matches()?;
    let config = Configuration::new(&matches);

    // If no commands display a simple usage
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

    // Call subcommands, TODO: this is ugly :(
    if matches.subcommand_matches("auto-sell").is_some() {
        auto_sell_cmd(&config).await?;
    } else if let Some(matches) = matches.subcommand_matches("buy-market") {
        let sym_name = matches.value_of("SYMBOL").expect("SYMBOL is missing");
        let q = matches.value_of("QUANTITY").expect("QUANTITY is missing");
        let quantity = match Decimal::from_str(q) {
            Ok(qty) => qty,
            Err(e) => return Err(format!("converting QUANTITY to Decimal: e={}", e).into()),
        };

        buy_market_order_cmd(&config, sym_name, quantity).await?;
    } else if let Some(matches) = matches.subcommand_matches("sell-market") {
        let sym_name = matches.value_of("SYMBOL").expect("SYMBOL is missing");
        let q = matches.value_of("QUANTITY").expect("QUANTITY is missing");
        let quantity = match Decimal::from_str(q) {
            Ok(qty) => qty,
            Err(e) => return Err(format!("converting QUANTITY to Decimal: e={}", e).into()),
        };

        sell_market_order_cmd(&config, sym_name, quantity).await?;
    } else if matches.subcommand_matches("ai").is_some() {
        let mut ai = get_account_info(&config).await?;
        ai.update_and_print(&config).await;
    } else if matches.subcommand_matches("ei").is_some() {
        let ei = get_exchange_info(&config).await?;
        println!("ei={:#?}", ei);
    } else if let Some(matches) = matches.subcommand_matches("sei") {
        let sym_name = matches.value_of("SYMBOL").expect("SYMBOL is missing");
        let ei = get_exchange_info(&config).await?;
        if let Some(sym) = ei.get_symbol(sym_name) {
            println!("{}: {:#?}", sym.symbol, sym);
        } else {
            println!("No such symbol {}", sym_name);
        }
    } else if let Some(matches) = matches.subcommand_matches("ap") {
        let sym_name = matches.value_of("SYMBOL").expect("SYMBOL is missing");
        let ap: AvgPrice = get_avg_price(&config, sym_name).await?;
        println!("ap: mins={} price={}", ap.mins, ap.price);
    } else if let Some(matches) = matches.subcommand_matches("skr") {
        let sym_name = matches.value_of("SYMBOL").expect("SYMBOL is missing");
        let kr: KlineRec = get_kline(&config, sym_name, utc_now_to_time_ms()).await?;
        println!("{}", kr);
    } else if let Some(matches) = matches.subcommand_matches("skrs") {
        let mut rec = GetKlinesCmdRec::default();
        rec.sym_name = matches.value_of("SYMBOL").expect("SYMBOL is missing").to_string();
        if let Some(limit_str) = matches.value_of("LIMIT") {
            rec.limit = Some(u16::from_str(limit_str)?);
        } else {
            rec.limit = None;
        };
        if let Some(st_str) = matches.value_of("START-TIME") {
            rec.start_date_time = Some(st_str.to_string());
        } else {
            rec.start_date_time = None;
        }
        if let Some(interval_str) = matches.value_of("INTERVAL") {
            rec.interval = Some(interval_str.to_string());
        } else {
            rec.interval = None;
        }
        get_klines_cmd(&config, &rec).await?;
    } else if let Some(matches) = matches.subcommand_matches("oo") {
        let sym_name = matches.value_of("SYMBOL").expect("SYMBOL is missing");
        let oo: Orders = get_open_orders(&config, &sym_name).await?;
        println!("oo: {:#?}\nsum_buy_orders: {}", oo, oo.sum_buy_orders());
    } else if matches.subcommand_matches("aoo").is_some() {
        // TODO: Add support for getting order_id, start_date_time, end_date_time and limit
        let ei = get_exchange_info(&config).await?;
        for symbol in ei.symbols_map.values() {
            let o: Orders = get_all_orders(&config, &symbol.symbol, None, None, None, None).await?;
            if o.orders.len() > 0 {
                println!("o: {:#?}", o);
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("st") {
        // TODO: Add support for getting from_id, start_data_time, end_data_time and limit
        let sym_name = matches.value_of("SYMBOL").expect("SYMBOL is missing");
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
    } else if matches.subcommand_matches("ol").is_some() {
        // TODO: Add support for getting from_id, start_data_time, end_data_time and limit
        match config.order_log_path {
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
        }
    } else {
        let args: Vec<String> = std::env::args().collect();
        println!("Unknown command: {}", args.join(" "));
    }

    //if let Some(cmd) = &ctx.opts.cmd {
    //    match cmd {
    //        Klines(rec) => {
    //            get_klines_cmd(ctx, rec).await?;
    //        }
    //    }
    //}

    trace!("main: -");
    Ok(())
}
