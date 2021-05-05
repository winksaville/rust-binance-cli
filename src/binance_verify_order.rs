use log::trace;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use crate::{
    binance_account_info::AccountInfo, binance_avg_price::AvgPrice, binance_exchange_info::Symbol,
    binance_open_orders::OpenOrders,
};

pub fn verify_open_orders(
    open_orders: &OpenOrders,
    symbol: &Symbol,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("verify_open_orders");

    // Verify the maximum number of orders isn't exceeded.
    let current_orders: u64 = open_orders.orders.len() as u64;
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
        trace!("There was no get_max_num_orders for {}", symbol.symbol);
    }

    Ok(())
}

pub fn verify_min_notional(
    avg_price: &AvgPrice,
    symbol: &Symbol,
    quantity: Decimal,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("verify_min_notional");
    // Verify the quantity meets the min_notional criteria
    match symbol.get_min_notional() {
        Some(mnr) => {
            let min_notional_quantity = mnr.min_notional / avg_price.price;
            if quantity < min_notional_quantity {
                return Err(format!(
                    "quantity: {} must be >= {} so value is >= {}",
                    quantity,
                    min_notional_quantity,
                    min_notional_quantity * avg_price.price
                )
                .into());
            }
            trace!(
                "quantity: {} >= min_notional_quantity: {}",
                quantity,
                min_notional_quantity
            );
            Ok(())
        }
        None => {
            trace!("No min_notional for {}", symbol.symbol);
            Ok(())
        }
    }
}

pub fn verify_max_position(
    ai: &AccountInfo,
    open_orders: &OpenOrders,
    symbol: &Symbol,
    quantity: Decimal,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("verify_max_position");

    let balance = match ai.balances_map.get(&symbol.base_asset) {
        Some(b) => b,
        None => {
            return Err(format!("No Balance for {}", symbol.base_asset).into());
        }
    };

    if let Some(max_position) = symbol.get_max_position() {
        let sum_buy_orders: Decimal = open_orders.sum_buy_orders();
        trace!("sum_buy_orders: {}", sum_buy_orders);

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
        trace!("There is no max_position for {}", symbol.symbol);
    }

    Ok(())
}

pub fn verify_quanity_is_greater_than_free(
    ai: &AccountInfo,
    symbol: &Symbol,
    quantity: Decimal,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("verify_max_position");

    let balance = match ai.balances_map.get(&symbol.base_asset) {
        Some(b) => b,
        None => {
            return Err(format!("No Balance for {}", symbol.base_asset).into());
        }
    };

    // Verify balance.free is ok
    if quantity > balance.free {
        return Err(format!("quantity: {} is > balance.free: {}", quantity, balance.free).into());
    }

    Ok(())
}

/// Adjust the quantity to adhere to MARKET_LOT_SIZE.step_size and then
/// verify the adjusted quantity meets the MARKET_LOT_SIZE min and max.
pub fn adj_quantity_verify_market_lot_size(
    symbol: &Symbol,
    quantity: Decimal,
) -> Result<Decimal, Box<dyn std::error::Error>> {
    trace!("adj_quantity_verify_market_lot_size");
    match symbol.get_market_lot_size() {
        Some(mls) => {
            trace!("mls: {:?}", mls);
            let adj_qty = if mls.step_size > dec!(0) {
                let rq = quantity + (mls.step_size / dec!(2));
                let rq_mod_step_size = rq % mls.step_size;
                let aq = rq - rq_mod_step_size;
                trace!(
                    "quantity: {} aq: {} rq: {} rq_mod_step_size: {}",
                    quantity,
                    aq,
                    rq,
                    rq_mod_step_size
                );
                aq
            } else {
                trace!(
                    "quantity: {} ok as mls.step_size: {} <= 0.0",
                    quantity,
                    mls.step_size
                );
                quantity
            };
            trace!("adj_qty: {}", adj_qty);

            if adj_qty < mls.min_qty {
                return Err(format!(
                    "adj_qty: {} must be >= {} MarketLotSize minimum quantity",
                    adj_qty, mls.min_qty,
                )
                .into());
            }
            trace!("adj_qty: {} >= mls.min_qty: {}", adj_qty, mls.min_qty);

            if adj_qty > mls.max_qty {
                return Err(format!(
                    "adj_qty: {} must be <= {} MarketLotSize maximum quantity",
                    adj_qty, mls.max_qty,
                )
                .into());
            }
            trace!("adj_qty: {} <= mls.max_qty: {}", adj_qty, mls.max_qty);

            //let diff = adj_qty - mls.min_qty;

            if mls.step_size > dec!(0) {
                assert_eq!((adj_qty - mls.min_qty) % mls.step_size, dec!(0));
            }

            Ok(adj_qty)
        }
        None => {
            trace!("quantity ok, No market_lot_size for {}", symbol.base_asset);
            Ok(quantity)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use test_env_log::test;

    /// More testing needed on the bounds of min_qty, max_qty and step_size
    #[test]
    fn test_adj_quantity_verify_market_lot_size() {
        let symbol: Symbol = serde_json::from_str(SYMBOL_DATA).unwrap();
        //println!("symbols: {:#?}", symbol);
        let mut quantity = dec!(1.0000009);
        let mut adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.000001);
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.0000014);
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.00000149);
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.00000149999);
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.0000014999999);
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.00000149999999);
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.000001499999999);
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.0000014999999999);
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.0000014999999999999999999999); // OK
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000001));

        quantity = dec!(1.00000149999999999999999999999); // Fails
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000002)); // "unexpected" rounds up, but not unreasonable

        quantity = dec!(1.0000015);
        adj_quantity = adj_quantity_verify_market_lot_size(&symbol, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(1.000002));

        // Test max_qty
        fn set_market_lot_size_max_qty(symbol: &mut Symbol, max_qty: Decimal) {
            match symbol.get_mut_market_lot_size() {
                Some(sr) => {
                    sr.max_qty = max_qty;
                }
                None => {}
            }
        }

        let mut s = symbol.clone();
        set_market_lot_size_max_qty(&mut s, dec!(999999999999999999999999999));
        //quantity = dec!(9999999999999999999999.000001); //no ..999.000002
        quantity = dec!(999999999999999999999.000001); // ..999.000001
        adj_quantity = adj_quantity_verify_market_lot_size(&s, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(999999999999999999999.000001));

        quantity = dec!(999999999999999999999.0000014);
        adj_quantity = adj_quantity_verify_market_lot_size(&s, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(999999999999999999999.000001));

        quantity = dec!(999999999999999999999.00000149); // "fails"
        adj_quantity = adj_quantity_verify_market_lot_size(&s, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(999999999999999999999.000002)); // "unexpected" rounds up, but not unreasonable!

        quantity = dec!(999999999999999999999.0000015);
        adj_quantity = adj_quantity_verify_market_lot_size(&s, quantity).unwrap();
        println!("quantity: {} adj_quantity: {}", quantity, adj_quantity);
        assert_eq!(adj_quantity, dec!(999999999999999999999.000002));
    }

    const SYMBOL_DATA: &str = r#"{
        "symbol": "BTCUSD",
        "baseAsset": "BTC",
        "quoteAsset": "USD",
        "baseAssetPrecision": 8,
        "baseCommissionPrecision": 8,
        "icebergAllowed": true,
        "isMarginTradingAllowed": false,
        "isSpotTradingAllowed": true,
        "ocoAllowed": true,
        "quoteAssetPrecision": 4,
        "quoteCommissionPrecision": 2,
        "quoteOrderQtyMarketAllowed": true,
        "quotePrecision": 4,
        "status": "TRADING",
        "permissions": [
            "SPOT"
        ],
        "orderTypes": [
            "LIMIT",
            "LIMIT_MAKER",
            "MARKET",
            "STOP_LOSS_LIMIT",
            "TAKE_PROFIT_LIMIT"
        ],
        "filters": [
            {
                "filterType": "PRICE_FILTER",
                "maxPrice": "100000.0000",
                "minPrice": "0.0100",
                "tickSize": "0.0100"
            },
            {
                "filterType": "PERCENT_PRICE",
                "avgPriceMins": 5,
                "multiplierDown": "0.2",
                "multiplierUp": "5"
            },
            {
                "filterType": "LOT_SIZE",
                "maxQty": "9000.00000000",
                "minQty": "0.00000100",
                "stepSize": "0.00000100"
            },
            {
                "filterType": "MARKET_LOT_SIZE",
                "maxQty": "3200.00000000",
                "minQty": "0.00000100",
                "stepSize": "0.00000100"
            },
            {
                "filterType": "MIN_NOTIONAL",
                "applyToMarket": true,
                "avgPriceMins": 5,
                "minNotional": "0.001"
            },
            {
                "filterType": "ICEBERG_PARTS",
                "limit": 10
            },
            {
                "filterType": "MAX_NUM_ICEBERG_ORDERS",
                "maxNumIcebergOrders": 5
            },
            {
                "filterType": "MAX_NUM_ORDERS",
                "maxNumOrders": 200
            },
            {
                "filterType": "MAX_NUM_ALGO_ORDERS",
                "maxNumAlgoOrders": 5
            },
            {
                "filterType": "MAX_POSITION",
                "maxPosition": 10.0
            }
        ]
    }"#;
}