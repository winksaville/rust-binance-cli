use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

mod binance_account_info;
mod binance_avg_price;
mod binance_context;
mod binance_exchange_info;
mod binance_open_orders;
mod binance_order_response;
mod binance_signature;
mod binance_trade;
mod common;
mod de_string_or_number;

use binance_account_info::get_account_info;
use binance_avg_price::{get_avg_price, AvgPrice};
use binance_context::BinanceContext;
use binance_exchange_info::get_exchange_info;
use binance_open_orders::{get_open_orders, OpenOrders};
use binance_trade::{binance_new_order_or_test, MarketQuantityType, TradeOrderType};
use common::{time_ms_to_utc, Side};

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
        let mut total_value = dec!(0);
        for (_, balance) in ai.balances_map {
            if balance.free > dec!(0.0) || balance.locked > dec!(0.0) {
                let price = if balance.asset != "USD" {
                    let sym = balance.asset.clone() + "USD";
                    let price = match get_avg_price(&ctx, &sym).await {
                        Ok(avgprice) => avgprice.price,
                        Err(_) => {
                            // This happens only on BCHA
                            if true {
                                // println!("unable to get_avg_price({})", sym);

                                // Ignore and just return price of 0
                                dec!(0)
                            } else {
                                // Try getting a BNB price
                                let bnbusd: Decimal = get_avg_price(&ctx, "BNBUSD").await?.price;
                                let bnbsym = balance.asset.clone() + "BNB";
                                let bnb_derived_price = match get_avg_price(&ctx, &bnbsym).await {
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
        let ap: AvgPrice = get_avg_price(&ctx, &ctx.opts.get_avg_price).await?;
        println!("ap: mins={} price={}", ap.mins, ap.price);
    }

    if ctx.opts.get_open_orders.is_some() {
        let symbol = match ctx.opts.get_open_orders.clone().unwrap() {
            Some(s) => s.clone(),
            None => "".to_string(),
        };

        let oo: OpenOrders = get_open_orders(&ctx, &symbol).await?;
        println!("oo: {:#?}", oo);
    }

    if !ctx.opts.sell.is_empty() {
        let symbol_name = ctx.opts.sell.clone();
        let mut quantity = Decimal::from_f64(ctx.opts.quantity).unwrap();
        if quantity <= dec!(0.0) {
            return Err(format!("Can't sell {} quantity", quantity).into());
        }
        trace!("symbol_name: {} quantity: {}", symbol_name, quantity);

        let ei = get_exchange_info(&ctx).await?;
        let symbol = match ei.get_symbol(&symbol_name) {
            Some(s) => s,
            None => {
                return Err(format!("There is no asset named {} to sell", symbol_name).into());
            }
        };
        trace!("Got ei");

        let ai = get_account_info(&ctx).await?;
        trace!("Got AccountInfo");

        let balance = match ai.balances_map.get(&symbol.base_asset) {
            Some(b) => b,
            None => {
                return Err(format!("No Balance for {}", symbol.base_asset).into());
            }
        };
        trace!(
            "Selling is possible as there is a balance for {}",
            balance.asset
        );

        // Verify the quantity meets the min_notional criteria
        match symbol.get_min_notional() {
            Some(mnr) => {
                let ap: AvgPrice = get_avg_price(&ctx, &symbol_name).await?;
                let min_notional_quantity = mnr.min_notional / ap.price;
                if quantity < min_notional_quantity {
                    return Err(format!(
                        "quantity: {} must be >= {} so value is >= {}",
                        quantity,
                        min_notional_quantity,
                        min_notional_quantity * ap.price,
                    )
                    .into());
                }
                trace!(
                    "quantity: {} >= min_notional_quantity: {}",
                    quantity,
                    min_notional_quantity
                );
            }
            None => {
                trace!("No min_notional for {}", symbol_name);
            }
        }

        // Verify the quantity meets the MarketLotSize criteria
        quantity = match symbol.get_market_lot_size() {
            Some(mls) => {
                trace!("mls: {:?}", mls);
                let mut mls = *mls;
                mls.step_size = dec!(0.000001);
                if mls.step_size > dec!(0.0) {
                    // Round to nearest step_size
                    let rq = quantity + (mls.step_size / dec!(2));
                    let rq_mss = rq % mls.step_size;
                    let adj_qty = rq - rq_mss;
                    //let adj_qty = (quantity + (mls.step_size / dec!(2))) % mls.step_size;
                    println!("quantity: {} adj_qty: {}", quantity, adj_qty);
                    if adj_qty < mls.min_qty {
                        return Err(format!(
                            "quanitity: {} must be >= {} MarketLotSize minimum quantity",
                            quantity, mls.min_qty,
                        )
                        .into());
                    }
                    trace!(
                        "quantity ok, adj_qty: {} >= mls.min_qty: {}",
                        adj_qty,
                        mls.min_qty
                    );
                    if adj_qty > mls.max_qty {
                        return Err(format!(
                            "quantity: {} must be <= {} MarketLotSize maximum quantity",
                            quantity, mls.max_qty,
                        )
                        .into());
                    }
                    trace!(
                        "Quantity ok, adj_qty: {} <= mls.min_qty: {}",
                        adj_qty,
                        mls.max_qty
                    );
                    //assert_eq!((adj_qty - mls.min_qty) % mls.step_size, dec!(0.0));
                    adj_qty
                } else {
                    trace!("quantity ok, as mls.step_size: {} <= 0.0", mls.step_size);
                    quantity
                }
            }
            None => {
                trace!("quantity ok, No market_lot_size for {}", symbol.base_asset);
                quantity
            }
        };

        // Verify the maximum number of orders isn't exceeded.
        // TODO: implement get_open_orders
        //   let open_orders = get_open_orders(&ctx, &symbol_name).await?
        let current_orders = 1u64; // open_orders.len()
        if let Some(max_num_orders) = symbol.get_max_num_orders() {
            if current_orders > max_num_orders {
                return Err(format!(
                    "The number of current orders is {} and thats > the maximum {}",
                    current_orders, max_num_orders,
                )
                .into());
            } else {
                trace!(
                    "current_orders: {} <= max_num_orders: {}",
                    current_orders,
                    max_num_orders
                );
            }
        } else {
            trace!("There was no get_max_num_orders for {}", symbol.base_asset);
        }

        // Verify MaxPosition
        if let Some(max_position) = symbol.get_max_position() {
            // TODO: Iterate over open_orders summing buy orders
            let sum_buy_orders = dec!(0.0); // open_orders.iter().map(|x| if x.buy {x.quantity} else { 0.0 }).sum();
            let current_holdings = balance.free + balance.locked;

            let new_position = quantity + current_holdings + sum_buy_orders;
            trace!(
                "new_position: {} = quantity: {} current_holdings: {} sum_buy_orders: {}",
                new_position,
                quantity,
                current_holdings,
                sum_buy_orders
            );
            if new_position > max_position {
                return Err(format!(
                    "The quantity: {} + current_holdings {} + sum_by_order: {} > max_position: {}",
                    quantity, current_holdings, sum_buy_orders, max_position
                )
                .into());
            }
            trace!(
                "new_position: {} <= max_position: {}",
                new_position,
                max_position
            );
        } else {
            trace!("There was no get_max_position for {}", symbol.base_asset);
        }

        // Verify balance.fre is ok
        if quantity > balance.free {
            return Err(
                format!("quantity: {} is > balance.free: {}", quantity, balance.free).into(),
            );
        }

        let response = binance_new_order_or_test(
            ctx,
            &symbol_name,
            Side::SELL,
            TradeOrderType::Market(MarketQuantityType::Quantity(quantity.to_f64().unwrap())),
            true,
        )
        .await?;
        println!("Sell reponse: {:#?}", response);
    }

    trace!("main: -");
    Ok(())
}
