//use clap::{AppSettings, Clap};
use log::trace;

#[allow(unused)]
use serde::{Deserialize, Serialize};

mod de_string_or_number;
#[allow(unused)]
use de_string_or_number::{
    de_string_or_number_to_f64, de_string_or_number_to_i64, de_string_or_number_to_u64,
};

mod de_vec_to_hashmap;
#[allow(unused)]
use de_vec_to_hashmap::de_vec_to_hashmap;

mod binance_context;
use binance_context::BinanceContext;

mod binance_exchange_info;
use binance_exchange_info::get_exchange_info;

mod binance_account_info;
use binance_account_info::get_account_info;

mod binance_order_response;
#[allow(unused)]
use binance_order_response::OrderResponse;

mod binance_trade;
use binance_trade::{binance_new_order_or_test, MarketQuantityType, OrderType, Side};

mod binance_avg_price;
use binance_avg_price::{get_avg_price, AvgPrice};

mod binance_signature;

mod common;
use common::time_ms_to_utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    trace!("main: +");

    let ctx = BinanceContext::new();

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

    if ctx.opts.get_exchange_info || !ctx.opts.symbol.is_empty() {
        let ei = get_exchange_info(&ctx).await?;

        if ctx.opts.get_exchange_info {
            println!("ei={:#?}", ei);
        }

        if !ctx.opts.symbol.is_empty() {
            let sym = ei.get_symbol(&ctx.opts.symbol);
            match sym {
                Some(sym) => println!("{}={:#?}", sym.symbol, sym),
                None => println!("{} not found", ctx.opts.symbol),
            }
        }
    }

    if ctx.opts.get_account_info {
        let ai = get_account_info(&ctx).await?;
        //println!("ai={:#?}", ai);
        println!("     account_type: {}", ai.account_type);
        println!("      can_deposit: {}", ai.can_deposit);
        println!("        can_trade: {}", ai.can_trade);
        println!("     can_withdraw: {}", ai.can_withdraw);
        println!(" buyer_commission: {}", ai.buyer_commission);
        println!(" maker_commission: {}", ai.maker_commission);
        println!("seller_commission: {}", ai.seller_commission);
        println!(" taker_commission: {}", ai.taker_commission);
        println!("      update_time: {}", time_ms_to_utc(ai.update_time));
        println!("      permissions: {:?}", ai.permissions);
        let mut total_value = 0.0f64;
        for balance in ai.balances {
            if balance.free > 0.0 || balance.locked > 0.0 {
                let price = if balance.asset != "USD" {
                    let sym = balance.asset.clone() + "USD";
                    let ap: AvgPrice = get_avg_price(&ctx, &sym).await?;
                    ap.price
                } else {
                    1.0
                };
                let value = price * (balance.free + balance.locked);
                println!(
                    "  {}: value: {} free: {} locked: {}",
                    balance.asset, value, balance.free, balance.locked
                );
                total_value += value;
            }
        }
        println!("total: {}", total_value);
    }

    if !ctx.opts.get_avg_price.is_empty() {
        let ap: AvgPrice = get_avg_price(&ctx, &ctx.opts.get_avg_price).await?;
        println!("ap: mins={} price={}", ap.mins, ap.price);
    }

    if !ctx.opts.sell.is_empty() {
        let symbol = ctx.opts.sell.clone();
        let quantity = ctx.opts.quantity;
        if quantity <= 0.0 {
            return Err(format!("Can't sell {} quantity", quantity).into());
        }
        let response = binance_new_order_or_test(
            ctx,
            &symbol,
            Side::SELL,
            OrderType::Market(MarketQuantityType::Quantity(quantity)),
            true,
        )
        .await?;
        println!("Sell reponse: {:#?}", response);
    }

    trace!("main: -");
    Ok(())
}
