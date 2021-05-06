use log::trace;

mod binance_account_info;
mod binance_auto_sell;
mod binance_avg_price;
mod binance_context;
mod binance_exchange_info;
mod binance_open_orders;
mod binance_order_response;
mod binance_sell_market;
mod binance_signature;
mod binance_trade;
mod binance_verify_order;
mod common;
mod de_string_or_number;

use binance_account_info::get_account_info;
use binance_avg_price::{get_avg_price, AvgPrice};
use binance_context::BinanceContext;
use binance_exchange_info::get_exchange_info;
use binance_open_orders::{get_open_orders, OpenOrders};
use binance_sell_market::sell_market;

use crate::binance_auto_sell::auto_sell;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    trace!("main: +");

    let ctx = &mut BinanceContext::new();

    if std::env::args().len() == 1 {
        let args: Vec<String> = std::env::args().collect();
        let prog_name = std::path::Path::new(&args[0]).file_name();
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

    if !ctx.opts.auto_sell.is_empty() {
        let config_file = &ctx.opts.auto_sell;
        auto_sell(ctx, config_file).await?;
    }

    if ctx.opts.get_exchange_info {
        let ei = get_exchange_info(ctx).await?;
        println!("ei={:#?}", ei);
    }

    if !ctx.opts.symbol.is_empty() {
        let ei = get_exchange_info(ctx).await?;
        let sym = ei.get_symbol(&ctx.opts.symbol);
        match sym {
            Some(sym) => println!("{}: {:#?}", sym.symbol, sym),
            None => println!("{} not found", ctx.opts.symbol),
        }
    }

    if !ctx.opts.sell.is_empty() {
        let ei = &get_exchange_info(ctx).await?;
        let symbol_name = &ctx.opts.sell.clone();
        let quantity = ctx.opts.quantity;

        sell_market(ctx, ei, symbol_name, quantity).await?;
    }

    if ctx.opts.get_account_info {
        let ai = get_account_info(ctx).await?;
        ai.print(ctx).await;
    }

    if !ctx.opts.get_avg_price.is_empty() {
        let ap: AvgPrice = get_avg_price(ctx, &ctx.opts.get_avg_price).await?;
        println!("ap: mins={} price={}", ap.mins, ap.price);
    }

    if ctx.opts.get_open_orders.is_some() {
        let symbol = match ctx.opts.get_open_orders.clone().unwrap() {
            Some(s) => s.clone(),
            None => "".to_string(),
        };

        let oo: OpenOrders = get_open_orders(ctx, &symbol).await?;
        println!("oo: {:#?}\nsum_buy_orders: {}", oo, oo.sum_buy_orders());
    }

    trace!("main: -");
    Ok(())
}
