use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
    path::Path,
};

use log::trace;

mod binance_account_info;
mod binance_auto_sell;
mod binance_avg_price;
mod binance_context;
mod binance_exchange_info;
mod binance_klines;
mod binance_market;
mod binance_my_trades;
mod binance_order_response;
mod binance_orders;
mod binance_signature;
mod binance_trade;
mod binance_verify_order;
mod common;
mod de_string_or_number;

use binance_account_info::get_account_info;
use binance_avg_price::{get_avg_price, AvgPrice};
use binance_context::BinanceContext;
use binance_exchange_info::get_exchange_info;
use binance_market::market_order;
use binance_my_trades::{get_my_trades, Trades};
use binance_orders::{get_all_orders, get_open_orders, Orders};
use common::Side;

extern crate function_name;
use function_name::named;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    binance_auto_sell::auto_sell,
    binance_klines::{get_kline, get_klines, KlineInterval, KlineRec},
    binance_order_response::TradeResponse,
    common::{time_ms_to_utc, utc_now_to_time_ms},
};

#[named]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    trace!("main: +");

    let ctx = &BinanceContext::new();

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
        println!("Usage: {} -h or --help", name);
        return Ok(());
    }

    if let Some(config_file) = &ctx.opts.auto_sell {
        auto_sell(ctx, config_file).await?;
    }

    if ctx.opts.get_exchange_info {
        let ei = get_exchange_info(ctx).await?;
        println!("ei={:#?}", ei);
    }

    if let Some(sym_name) = &ctx.opts.symbol {
        let ei = get_exchange_info(ctx).await?;
        if let Some(sym) = ei.get_symbol(sym_name) {
            println!("{}: {:#?}", sym.symbol, sym);
        } else {
            println!("No such symbol {}", sym_name);
        }
    }

    if let Some(sym_name) = &ctx.opts.sell_market {
        let ei = &get_exchange_info(ctx).await?;
        let quantity = ctx.opts.quantity;
        market_order(ctx, ei, &sym_name, quantity, Side::SELL).await?;
    }

    if let Some(sym_name) = &ctx.opts.buy_market {
        let ei = &get_exchange_info(ctx).await?;
        let quantity = ctx.opts.quantity;
        market_order(ctx, ei, sym_name, quantity, Side::BUY).await?;
    }

    if ctx.opts.get_account_info {
        let mut ai = get_account_info(ctx).await?;
        ai.update_and_print(ctx).await;
    }

    if let Some(sym_name) = &ctx.opts.get_avg_price {
        let ap: AvgPrice = get_avg_price(ctx, sym_name).await?;
        println!("ap: mins={} price={}", ap.mins, ap.price);
    }

    if ctx.opts.get_open_orders.is_some() {
        let symbol = match ctx.opts.get_open_orders.clone().unwrap() {
            Some(s) => s.clone(),
            None => "".to_string(),
        };

        let oo: Orders = get_open_orders(ctx, &symbol).await?;
        println!("oo: {:#?}\nsum_buy_orders: {}", oo, oo.sum_buy_orders());
    }

    if ctx.opts.get_all_orders.is_some() {
        // TODO: Add support for getting order_id, start_date_time, end_date_time and limit
        let symbol = match ctx.opts.get_all_orders.clone().unwrap() {
            Some(s) => s.clone(),
            None => "".to_string(),
        };

        if symbol.is_empty() {
            let ei = get_exchange_info(ctx).await?;
            for symbol in ei.symbols_map.values() {
                let o: Orders = get_all_orders(ctx, &symbol.symbol, None, None, None, None).await?;
                println!("o: {:#?}", o);
            }
        } else {
            let o: Orders = get_all_orders(ctx, &symbol, None, None, None, None).await?;
            println!("o: {:#?}", o);
        }
    }

    if let Some(sym_name) = &ctx.opts.get_my_trades {
        let mt: Trades = get_my_trades(ctx, sym_name, None, None, None, None).await?;
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

    if let Some(log_file_path) = &ctx.opts.display_order_log {
        let file = OpenOptions::new().read(true).open(log_file_path)?;
        let reader = BufReader::new(&file);
        for line in reader.lines() {
            let tr: TradeResponse = serde_json::from_str(&line?)?;
            println!("{:#?}", tr);
        }
    }

    if let Some(sym_name) = &ctx.opts.get_klines {
        // Some constants
        const SEC: i64 = 1000;
        const MIN: i64 = 60 * SEC;
        const INTERVAL_MIN: i64 = 1;
        const MINIMUM_INTERVAL_ELAPSED_SECS: i64 = 10 * SEC;

        // Truncate st down to beginning of the current minute
        let now = utc_now_to_time_ms();
        let mut st = now;
        st = st - (st % (INTERVAL_MIN * MIN));

        // If we're <= first minimum_interval_elapsed_secsl of this
        // interval go to the previous minute, otherwise there may be
        // nothing returned as so little time has transpired. In my
        // short empherical investigation this "dead" interval was
        // about 3 or 4 seconds, so I've made it 10.
        if (st + MINIMUM_INTERVAL_ELAPSED_SECS) >= now {
            st -= MIN;
            println!("backup to previous minute");
        }
        let et = st + (INTERVAL_MIN * MIN);
        // Rounds up st, does the lesser of et and limit and if et is < KlineInterval nothing returned
        println!(
            "utc:       {} st: {} et: {} diff: {}",
            time_ms_to_utc(utc_now_to_time_ms()),
            time_ms_to_utc(st),
            time_ms_to_utc(et),
            (et - st) as f64 / MIN as f64
        );
        let krs: Vec<KlineRec> =
            get_klines(ctx, sym_name, KlineInterval::Mins1, Some(st), None, Some(1)).await?;
        for kr in &krs {
            println!(
                "Open time: {} Close time: {} diff: {}",
                time_ms_to_utc(kr.open_time),
                time_ms_to_utc(kr.close_time),
                (kr.close_time - kr.open_time) as f64 / MIN as f64
            );
            println!("{:#?}", kr);
        }
    }

    if let Some(sym_name) = &ctx.opts.get_kline {
        const MIN: f64 = 60_f64 * 1000_f64;
        let kr: KlineRec = get_kline(ctx, sym_name, utc_now_to_time_ms()).await?;
        println!(
            "Open time: {} Close time: {} diff: {}",
            time_ms_to_utc(kr.open_time),
            time_ms_to_utc(kr.close_time),
            (kr.close_time - kr.open_time) as f64 / MIN
        );
        println!("{:#?}", kr);
    }

    trace!("main: -");
    Ok(())
}
