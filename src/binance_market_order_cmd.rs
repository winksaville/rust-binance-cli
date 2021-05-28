//use std::fs;

use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use crate::{
    binance_account_info::get_account_info,
    binance_avg_price::{get_avg_price, AvgPrice},
    binance_exchange_info::{get_exchange_info, ExchangeInfo},
    binance_order_response::TradeResponse,
    binance_orders::get_open_orders,
    binance_trade::{
        self, binance_new_order_or_test, order_log_file, MarketQuantityType, TradeOrderType,
    },
    binance_verify_order::{
        adj_quantity_verify_lot_size, verify_max_position, verify_min_notional, verify_open_orders,
        verify_quanity_is_greater_than_free,
    },
    common::{InternalErrorRec, Side},
    configuration::Configuration,
    ier_new,
};
use binance_trade::log_order_response;

pub async fn market_order(
    config: &Configuration,
    ei: &ExchangeInfo,
    symbol_name: &str,
    order_type: &TradeOrderType,
    side: Side,
    test: bool,
) -> Result<TradeResponse, Box<dyn std::error::Error>> {
    trace!(
        "market_order: config={:#?} symbol_name={} order_type={} side={} test={}",
        config,
        symbol_name,
        order_type,
        side,
        test
    );
    let order_log_path = config.order_log_path.as_ref().unwrap(); // FIXME: NO UNWRAP
    let mut log_writer = order_log_file(order_log_path)?;

    let symbol = match ei.get_symbol(&symbol_name) {
        Some(s) => s,
        None => {
            let tr = TradeResponse::FailureInternal(ier_new!(
                2,
                &format!("No asset named {}", symbol_name)
            ));
            log_order_response(&mut log_writer, &tr)?;
            return Ok(tr);
        }
    };
    trace!("market_order: Got symbol");

    let adj_order_type: TradeOrderType;

    let avg_price: AvgPrice = get_avg_price(config, &symbol.symbol).await?;
    let quantity = match order_type {
        TradeOrderType::Market(MarketQuantityType::Quantity(qty)) => {
            // Adjust quantity and verify the quantity meets the LotSize criteria
            let qty = adj_quantity_verify_lot_size(symbol, *qty);

            // Could have gone zero, if so return an error
            if qty <= dec!(0) {
                let tr = TradeResponse::FailureInternal(ier_new!(
                    3,
                    &format!("adjusted quantity: {} <= 0", qty)
                ));
                log_order_response(&mut log_writer, &tr)?;
                return Ok(tr);
            }

            // We may have modified!
            adj_order_type = TradeOrderType::Market(MarketQuantityType::Quantity(qty));

            qty
        }
        TradeOrderType::Market(MarketQuantityType::QuoteOrderQty(qty)) => {
            // Unmodified
            adj_order_type = order_type.clone();

            qty / avg_price.price
        }
    };

    // Verify the quantity meets the min_notional criteria
    if let Err(e) = verify_min_notional(&avg_price, symbol, quantity) {
        let tr = TradeResponse::FailureInternal(ier_new!(4, &e.to_string()));
        log_order_response(&mut log_writer, &tr)?;
        return Ok(tr);
    }

    let ai = get_account_info(config).await?;
    trace!("market_order: Got AccountInfo");

    let open_orders = get_open_orders(config, &symbol.symbol).await?;

    // Verify the maximum number of orders isn't exceeded.
    verify_open_orders(&open_orders, symbol)?;

    match side {
        Side::SELL => {
            // Selling, be sure we have enough to sell
            verify_quanity_is_greater_than_free(&ai, symbol, quantity)?;
        }
        Side::BUY => {
            // Buying, verify we don't exceed MaxPosition
            verify_max_position(&ai, &open_orders, symbol, quantity)?;
        }
    }

    let tr = binance_new_order_or_test(
        config,
        &mut log_writer,
        ei,
        &symbol_name,
        side,
        adj_order_type,
        test,
    )
    .await?;
    trace!("market_order: trade reponse: {:#?}", tr);

    Ok(tr)
}

#[derive(Debug, Clone, Default)]
pub struct MarketCmdRec {
    /// Symbol name
    pub sym_name: String,

    /// Number of shares
    pub quantity: Decimal,
}

pub async fn buy_market_order_cmd(
    config: &Configuration,
    sym_name: &str,
    order_type: TradeOrderType,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!(
        "buy_market_order: sym_name: {} {} config:\n{:#?}",
        sym_name,
        order_type,
        config
    );

    let ei = &get_exchange_info(config).await?;
    let tr = market_order(config, ei, sym_name, &order_type, Side::BUY, config.test).await?;
    println!("{}", tr);

    Ok(())
}

pub async fn sell_market_order_cmd(
    config: &Configuration,
    sym_name: &str,
    order_type: TradeOrderType,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!(
        "sell_market_order: sym_name: {} {} config:\n{:#?}",
        sym_name,
        order_type,
        config
    );

    let ei = &get_exchange_info(config).await?;
    let tr = market_order(config, ei, sym_name, &order_type, Side::SELL, config.test).await?;
    println!("{}", tr);

    Ok(())
}
