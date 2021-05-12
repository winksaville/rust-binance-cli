use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use crate::{
    binance_account_info::get_account_info,
    binance_avg_price::{get_avg_price, AvgPrice},
    binance_context::BinanceContext,
    binance_exchange_info::ExchangeInfo,
    binance_orders::get_open_orders,
    binance_trade::{binance_new_order_or_test, MarketQuantityType, TradeOrderType},
    binance_verify_order::{
        adj_quantity_verify_market_lot_size, verify_max_position, verify_min_notional,
        verify_open_orders, verify_quanity_is_greater_than_free,
    },
    common::Side,
};

pub async fn market_order(
    ctx: &mut BinanceContext,
    ei: &ExchangeInfo,
    symbol_name: &str,
    quantity: Decimal,
    side: Side,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut quantity = quantity;
    if quantity <= dec!(0.0) {
        return Err(format!("order {} quantity", quantity).into());
    }
    trace!("symbol_name: {} quantity: {}", symbol_name, quantity);

    let symbol = match ei.get_symbol(&symbol_name) {
        Some(s) => s,
        None => {
            return Err(format!("There is no asset named {}", symbol_name).into());
        }
    };
    trace!("Got symbol");

    let ai = get_account_info(ctx).await?;
    trace!("Got AccountInfo");

    let open_orders = get_open_orders(ctx, &symbol.symbol).await?;

    // Verify the maximum number of orders isn't exceeded.
    verify_open_orders(&open_orders, symbol)?;

    // Adjust quantity and verify the quantity meets the MarketLotSize criteria
    quantity = adj_quantity_verify_market_lot_size(symbol, quantity)?;

    // Verify the quantity meets the min_notional criteria
    let avg_price: AvgPrice = get_avg_price(ctx, &symbol.symbol).await?;
    verify_min_notional(&avg_price, symbol, quantity)?;

    // Verify MaxPosition
    verify_max_position(&ai, &open_orders, symbol, quantity)?;

    // Could use if matches!(side, Side::SELL) but this is safer is Side changes
    match side {
        Side::SELL => {
            verify_quanity_is_greater_than_free(&ai, symbol, quantity)?;
        }
        Side::BUY => {}
    }

    let response = binance_new_order_or_test(
        ctx,
        ei,
        &symbol_name,
        side,
        TradeOrderType::Market(MarketQuantityType::Quantity(quantity)),
        false,
    )
    .await?;
    println!("market reponse: {:#?}", response);

    Ok(())
}
