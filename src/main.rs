use std::{
    //fs::OpenOptions,
    //io::{BufRead, BufReader},
    path::Path,
};

use log::trace;

mod arg_matches;
mod binance_account_info;
mod binance_auto_sell;
mod binance_avg_price;
mod binance_context;
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

use arg_matches::arg_matches;
//use binance_account_info::get_account_info;
use binance_auto_sell::auto_sell_cmd;
//use binance_avg_price::{get_avg_price, AvgPrice};
use binance_context::{
    BinanceContext, //SubCommands::AutoSell, SubCommands::BuyMarket, SubCommands::Klines,
    //SubCommands::SellMarket,
};
//use binance_exchange_info::get_exchange_info;
//use binance_get_klines_cmd::get_klines_cmd;
//use binance_market_order_cmd::buy_market_order_cmd;
//use binance_market_order_cmd::sell_market_order_cmd;
//use binance_my_trades::{get_my_trades, Trades};
//use binance_order_response::TradeResponse;
//use binance_orders::{get_all_orders, get_open_orders, Orders};
//use common::time_ms_to_utc;
use configuration::ConfigurationX;

//use rust_decimal::Decimal;
//use rust_decimal_macros::dec;

extern crate function_name;
use function_name::named;

#[named]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    trace!("main: +");

    let ctx = BinanceContext::new();

    let matches = arg_matches()?;
    let config = ConfigurationX::new(&matches);

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

    // Call subcommands
    if matches.subcommand_matches("auto-sell").is_some() {
        auto_sell_cmd(&ctx, &config).await?;
    //} else if let Some(matches) = matches.subcommand_matches("buy-market") {
        //buy_market_cmd(&config, &matches)?;
    }
 

    //if ctx.opts.get_exchange_info {
    //    let ei = get_exchange_info(ctx).await?;
    //    println!("ei={:#?}", ei);
    //}

    //if let Some(sym_name) = &ctx.opts.symbol {
    //    let ei = get_exchange_info(ctx).await?;
    //    if let Some(sym) = ei.get_symbol(sym_name) {
    //        println!("{}: {:#?}", sym.symbol, sym);
    //    } else {
    //        println!("No such symbol {}", sym_name);
    //    }
    //}

    //if ctx.opts.get_account_info {
    //    let mut ai = get_account_info(ctx).await?;
    //    ai.update_and_print(ctx).await;
    //}

    //if let Some(sym_name) = &ctx.opts.get_avg_price {
    //    let ap: AvgPrice = get_avg_price(ctx, sym_name).await?;
    //    println!("ap: mins={} price={}", ap.mins, ap.price);
    //}

    //if ctx.opts.get_open_orders.is_some() {
    //    let symbol = match ctx.opts.get_open_orders.clone().unwrap() {
    //        Some(s) => s.clone(),
    //        None => "".to_string(),
    //    };

    //    let oo: Orders = get_open_orders(ctx, &symbol).await?;
    //    println!("oo: {:#?}\nsum_buy_orders: {}", oo, oo.sum_buy_orders());
    //}

    //if ctx.opts.get_all_orders.is_some() {
    //    // TODO: Add support for getting order_id, start_date_time, end_date_time and limit
    //    let symbol = match ctx.opts.get_all_orders.clone().unwrap() {
    //        Some(s) => s.clone(),
    //        None => "".to_string(),
    //    };

    //    if symbol.is_empty() {
    //        let ei = get_exchange_info(ctx).await?;
    //        for symbol in ei.symbols_map.values() {
    //            let o: Orders = get_all_orders(ctx, &symbol.symbol, None, None, None, None).await?;
    //            println!("o: {:#?}", o);
    //        }
    //    } else {
    //        let o: Orders = get_all_orders(ctx, &symbol, None, None, None, None).await?;
    //        println!("o: {:#?}", o);
    //    }
    //}

    //if let Some(sym_name) = &ctx.opts.get_my_trades {
    //    let mt: Trades = get_my_trades(ctx, sym_name, None, None, None, None).await?;
    //    let mut total_qty: Decimal = dec!(0);
    //    let mut total_quote_value: Decimal = dec!(0);
    //    for tr in &mt.trades {
    //        println!("Date: {}", time_ms_to_utc(tr.time));
    //        println!("{:#?}", tr);
    //        total_qty += tr.qty;
    //        total_quote_value += tr.quote_qty;
    //        println!(
    //            "total_qty: {}, total_quote_value: {}",
    //            total_qty, total_quote_value
    //        );
    //    }
    //    println!(
    //        "total_qty: {}, total_quote_value: {}",
    //        total_qty, total_quote_value
    //    );
    //}

    //if let Some(log_file_path) = &ctx.opts.display_order_log {
    //    let file = OpenOptions::new().read(true).open(log_file_path)?;
    //    let reader = BufReader::new(&file);
    //    for line in reader.lines() {
    //        let tr: TradeResponse = serde_json::from_str(&line?)?;
    //        println!("{:#?}", tr);
    //    }
    //}

    //if let Some(cmd) = &ctx.opts.cmd {
    //    match cmd {
    //        Klines(rec) => {
    //            get_klines_cmd(ctx, rec).await?;
    //        }
    //        BuyMarket(rec) => {
    //            buy_market_order_cmd(ctx, rec).await?;
    //        }
    //        SellMarket(rec) => {
    //            sell_market_order_cmd(ctx, rec).await?;
    //        }
    //        AutoSell(rec) => {
    //            auto_sell_cmd(ctx, &rec).await?;
    //        }
    //    }
    //}

    trace!("main: -");
    Ok(())
}
