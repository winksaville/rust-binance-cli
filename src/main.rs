use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

mod binance_account_info;
mod binance_avg_price;
mod binance_config_auto_sell_all;
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
use common::time_ms_to_utc;

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

    if ctx.opts.get_exchange_info || !ctx.opts.symbol.is_empty() {
        let ei = get_exchange_info(ctx).await?;

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
        let ai = get_account_info(ctx).await?;
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
        let mut total_value = dec!(0);
        for (_, balance) in ai.balances_map {
            if balance.free > dec!(0.0) || balance.locked > dec!(0.0) {
                let price = if balance.asset != "USD" {
                    let sym = balance.asset.clone() + "USD";
                    let price = match get_avg_price(ctx, &sym).await {
                        Ok(avgprice) => avgprice.price,
                        Err(_) => {
                            // This happens only on BCHA
                            if true {
                                // println!("unable to get_avg_price({})", sym);
                                // Ignore and just return price of 0
                                dec!(0)
                            } else {
                                // Try getting a BNB price
                                let bnbusd: Decimal = get_avg_price(ctx, "BNBUSD").await?.price;
                                let bnbsym = balance.asset.clone() + "BNB";
                                let bnb_derived_price = match get_avg_price(ctx, &bnbsym).await {
                                    Ok(avp) => avp.price * bnbusd,
                                    // Ignore if still no price
                                    Err(_) => {
                                        println!("No price found for {}", balance.asset);
                                        dec!(0)
                                    }
                                };
                                bnb_derived_price
                            }
                        }
                    };
                    price
                } else {
                    dec!(1.0)
                };
                let value = price * balance.free + balance.locked;
                println!(
                    "  {:6}: value: ${:10.2} free: {:15.8} locked: {}",
                    balance.asset, value, balance.free, balance.locked
                );
                total_value += value;
            }
        }
        println!("total: ${:.2}", total_value);
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

    if !ctx.opts.sell.is_empty() {
        let symbol_name = ctx.opts.sell.clone();
        let quantity = ctx.opts.quantity;

        sell_market(ctx, &symbol_name, quantity).await?;
    }

    trace!("main: -");
    Ok(())
}
