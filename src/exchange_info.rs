use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused)]
use crate::de_string_or_number::{de_string_or_number_to_f64, de_string_or_number_to_u64};

use strum_macros::IntoStaticStr;
#[derive(Debug, Deserialize, Serialize, IntoStaticStr)]
#[serde(tag = "filterType")]
pub enum ExchangeFilters {
    #[serde(rename = "EXCHANGE_MAX_NUM_ORDERS")]
    ExchangeMaxNumOrders {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "maxNumOrders")]
        max_num_orders: u64,
    },
    #[serde(rename = "EXCHANGE_MAX_NUM_ALGO_ORDERS")]
    ExchangeMaxAlgoOrders {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "maxNumAlgoOrders")]
        max_num_algo_orders: u64,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SizeData {
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    #[serde(rename = "minQty")]
    pub min_qty: f64,

    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    #[serde(rename = "maxQty")]
    pub max_qty: f64,

    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    #[serde(rename = "stepSize")]
    pub step_size: f64,
}

// Couldn't use SymbolFilters directly as key to hash because
// Eq and Hash are not implementatble for f64. Instead I'm
// using `into()` of InfoStaticStr to convert it to a keyable item.
// See: SymbolFilters.filters
//
// Accessing this requires a match and isn't pretty, IMHO.
// Maybe [enum-as-inner](https://crates.io/crates/enum-as-inner#:~:text=named%20field%20case)
// Or [enum variants as types](https://www.reddit.com/r/rust/comments/2rdoxx/enum_variants_as_types/)
// related to [datasort refinements](https://github.com/rust-lang/rfcs/issues/754)?
#[derive(Debug, Deserialize, Serialize, IntoStaticStr)]
#[serde(tag = "filterType")]
pub enum SymbolFilters {
    #[serde(rename = "PRICE_FILTER")]
    PriceFilter {
        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "minPrice")]
        min_price: f64,

        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "maxPrice")]
        max_price: f64,

        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "tickSize")]
        tick_size: f64,
    },

    #[serde(rename = "PERCENT_PRICE")]
    PrecentPrice {
        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "multiplierUp")]
        mulitplier_up: f64,

        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "multiplierDown")]
        multiplier_down: f64,

        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: u64,
    },

    #[serde(rename = "LOT_SIZE")]
    LotSize(SizeData),

    #[serde(rename = "MIN_NOTIONAL")]
    MinNotional {
        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "minNotional")]
        min_notional: f64,

        #[serde(rename = "applyToMarket")]
        apply_to_market: bool,

        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: u64,
    },

    #[serde(rename = "ICEBERG_PARTS")]
    IcebergParts {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        limit: u64,
    },

    #[serde(rename = "MARKET_LOT_SIZE")]
    MarketLotSize {
        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "minQty")]
        min_qty: f64,

        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "maxQty")]
        max_qty: f64,

        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "stepSize")]
        step_size: f64,
    },

    #[serde(rename = "MAX_NUM_ORDERS")]
    MaxNumOrders {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "maxNumOrders")]
        max_num_orders: u64,
    },

    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    MaxNumAlgoOrders {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "maxNumAlgoOrders")]
        max_num_algo_orders: u64,
    },

    #[serde(rename = "MAX_NUM_ICEBERG_ORDERS")]
    MaxNumIcebergOrders {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "maxNumIcebergOrders")]
        max_num_iceberg_orders: u64,
    },

    #[serde(rename = "MAX_POSITION")]
    MaxPosition {
        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "maxPosition")]
        max_position: f64,
    },
}

impl SymbolFilters {
    pub fn get_lot_size(&self) -> Option<&SizeData> {
        match self {
            SymbolFilters::LotSize(sd) => Some(sd),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, IntoStaticStr)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitType {
    RawRequest,
    RequestWeight,
    Orders,
}

#[derive(Debug, Deserialize, Serialize, IntoStaticStr)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntervalType {
    Minute,
    Second,
    Day,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: RateLimitType, // Type of rate limit
    pub interval: IntervalType,         // Type of interval
    pub interval_num: u64,              // interval_num * interval is a duration
    pub limit: u64,                     // limit is the maximum rate in the duration.
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol<'a> {
    pub symbol: String,     // +enum BTCUSD?
    pub base_asset: String, // +enum BTC?
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub base_asset_precision: u64,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub base_commission_precision: u64,
    pub iceberg_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub oco_allowed: bool,
    pub quote_asset: String, // +enum USD?
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub quote_asset_precision: u64,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub quote_commission_precision: u64,
    pub quote_order_qty_market_allowed: bool,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub quote_precision: u64,
    pub status: String, // +enum TRADING?
    pub permissions: Vec<String>,
    pub order_types: Vec<String>,
    pub filters: Vec<SymbolFilters>,

    #[serde(skip)]
    filters_map: HashMap<&'a str, &'a SymbolFilters>,
}

fn create_filters_map<'s>(sym: &'s mut Symbol<'s>) -> &'s Symbol<'s> {
    sym.filters_map = HashMap::<&str, &SymbolFilters>::new();

    for item in &sym.filters {
        let key = item.into();
        sym.filters_map.insert(key, item);
    }

    //println!("create_filters_map sym={:#?}", sym);
    sym
}

impl<'s> Symbol<'s> {
    fn create_filters_map(&'s mut self) -> &Self {
        self.filters_map = HashMap::<&'s str, &'s SymbolFilters>::new();

        for item in &self.filters {
            let key = item.into();
            self.filters_map.insert(key, item);
        }

        //println!("create_filters_map sym={:#?}", self);
        self
    }

    // Not yet working mut/lifetime problems, is empty
    pub fn get_lot_size(&self) -> Option<&'s SizeData> {
        self.filters_map.get("LotSize")?.get_lot_size()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo<'e> {
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub server_time: u64,
    pub exchange_filters: Vec<ExchangeFilters>,
    pub rate_limits: Vec<RateLimit>,
    pub symbols: Vec<Symbol<'e>>,

    #[serde(skip)]
    symbols_map: HashMap<&'e str, &'e Symbol<'e>>,
}

impl<'e> ExchangeInfo<'e> {
    fn create_symbols_map(&'e mut self) -> &Self {
        self.symbols_map = HashMap::<&'e str, &'e Symbol<'e>>::new();

        for item in &self.symbols {
            //let sym = item.create_filters_map();
            //self.symbols_map.insert(&sym.symbol, &sym);
            self.symbols_map.insert(&item.symbol, item);
        }

        self
    }

    pub fn get_sym(&'e self, symbol: &str) -> Option<&'e Symbol<'e>> {
        Some(*self.symbols_map.get(symbol)?)
    }

    // Symbol::filters_map not yet initialized, got mut/lifetime problems :(
    //pub fn get_lot_size(&'e self, symbol: &str) -> Option<&'e SizeData> {
    //   self.get_sym(symbol)?.get_lot_size()
    //}
}

#[cfg(test)]
mod test {
    //extern crate test;

    //#[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_create_filters() {
        // Here I can create_filters_map using either technique.
        // But I can't create_symbols_map
        let mut mut_ei: ExchangeInfo = match serde_json::from_str(EXCHANGE_INFO_DATA) {
            Ok(info) => info,
            Err(e) => panic!("Error processing response: e={}", e),
        };

        for xyz in &mut mut_ei.symbols {
            if true {
                create_filters_map(xyz); // Works
            } else {
                xyz.create_filters_map(); // Works
            }
        }

        // If uncommented this causes the error below
        let ei = mut_ei.create_symbols_map();
        //    error[E0499]: cannot borrow `mut_ei` as mutable more than once at a time
        //       --> src/exchange_info.rs:306:18
        //        |
        //    296 |         for xyz in &mut mut_ei.symbols {
        //        |                    ------------------- first mutable borrow occurs here
        //    ...
        //    305 |         let ei = mut_ei.create_symbols_map();
        //        |                  ^^^^^^
        //        |                  |
        //        |                  second mutable borrow occurs here
        //        |                  first borrow later used here
        //
        //    error: aborting due to previous error
        //
        //    For more information about this error, try `rustc --explain E0499`.
        //    error: could not compile `binance-auto-sell`
    }

    #[test]
    fn test_exchange_info() {
        // vvv Ok. Here I create_symbols_map but I can't creat_filters_map
        let mut mut_ei: ExchangeInfo = match serde_json::from_str(EXCHANGE_INFO_DATA) {
            Ok(info) => info,
            Err(e) => panic!("Error processing response: e={}", e),
        };
        let ei = mut_ei.create_symbols_map();
        println!("ei.server_time={:#?}", ei.server_time);
        println!("ei={:#?}", ei);

        // Verify symbols_map points at symbols
        println!("&ei.symbols[0].symbol={:p}", &ei.symbols[0].symbol);
        println!(
            "&ei.symbols_map[\"BTCUSD\"].symbol={:p}",
            &ei.symbols_map["BTCUSD"].symbol
        );
        assert_eq!(&ei.symbols[0].symbol, &ei.symbols_map["BTCUSD"].symbol);

        println!(
            "&ei.symbols[0].order_types={:p}",
            &ei.symbols[0].order_types
        );
        println!(
            "&ei.symbols_map[\"BTCUSD\"].order_types={:p}",
            &ei.symbols_map["BTCUSD"].order_types
        );
        assert_eq!(
            &ei.symbols[0].order_types,
            &ei.symbols_map["BTCUSD"].order_types
        );

        // To "complex" for testing
        match &ei.exchange_filters[0] {
            ExchangeFilters::ExchangeMaxNumOrders {
                max_num_orders: num,
            } => assert_eq!(*num, 123),
            _ => assert!(false),
        };
        // This is simpler but seems to still need a `match` to access the field
        let ef0 = &ei.exchange_filters[0];
        assert!(matches!(ef0, ExchangeFilters::ExchangeMaxNumOrders { .. }));

        match ei.exchange_filters[1] {
            ExchangeFilters::ExchangeMaxAlgoOrders {
                max_num_algo_orders: num,
            } => assert_eq!(num, 456),
            _ => assert!(false),
        };

        // Using `matches!` is nice for this "homogeneous" structure with typed fields
        let rl0 = &ei.rate_limits[0];
        assert!(matches!(rl0.rate_limit_type, RateLimitType::RawRequest));
        assert!(matches!(rl0.interval, IntervalType::Minute));
        assert_eq!(rl0.interval_num, 1);
        assert_eq!(rl0.limit, 1200);

        let rl1 = &ei.rate_limits[1];
        assert!(matches!(rl1.rate_limit_type, RateLimitType::RequestWeight));
        assert!(matches!(rl1.interval, IntervalType::Second));
        assert_eq!(rl1.interval_num, 10);
        assert_eq!(rl1.limit, 100);

        let rl2 = &ei.rate_limits[2];
        assert!(matches!(rl2.rate_limit_type, RateLimitType::Orders));
        assert!(matches!(rl2.interval, IntervalType::Day));
        assert_eq!(rl2.interval_num, 1);
        assert_eq!(rl2.limit, 200000);

        // Symbols
        let s0 = &ei.symbols[0];
        assert_eq!(s0.symbol, "BTCUSD");
        assert_eq!(s0.base_asset, "BTC");
        assert_eq!(s0.quote_asset, "USD");
        assert_eq!(s0.base_asset_precision, 8);
        assert_eq!(s0.base_commission_precision, 8);
        assert_eq!(s0.iceberg_allowed, true);
        assert_eq!(s0.is_margin_trading_allowed, false);
        assert_eq!(s0.is_spot_trading_allowed, true);
        assert_eq!(s0.oco_allowed, true);
        assert_eq!(s0.quote_asset_precision, 4);
        assert_eq!(s0.quote_commission_precision, 2);
        assert_eq!(s0.quote_order_qty_market_allowed, true);
        assert_eq!(s0.quote_precision, 4);
        assert_eq!(s0.status, "TRADING");
        assert_eq!(s0.permissions, ["SPOT"]);
        assert_eq!(
            s0.order_types,
            [
                "LIMIT",
                "LIMIT_MAKER",
                "MARKET",
                "STOP_LOSS_LIMIT",
                "TAKE_PROFIT_LIMIT",
            ]
        );

        let s0f0 = &s0.filters[0];
        match *s0f0 {
            SymbolFilters::PriceFilter {
                min_price,
                max_price,
                tick_size,
            } => {
                assert_eq!(min_price, 0.01);
                assert_eq!(max_price, 100000.0);
                assert_eq!(tick_size, 0.01);
            }
            _ => assert!(false),
        }

        let btcusd_sym = ei.get_sym("BTCUSD");
        assert!(btcusd_sym.is_some());

        let xxxxxx_sym = ei.get_sym("XXXXXX");
        assert!(xxxxxx_sym.is_none());
        println!("xxxxxx_sym={:#?}", xxxxxx_sym);
        // ^^^ OK
    }

    #[allow(unused)]
    const EXCHANGE_INFO_DATA: &str = r#"{
         "serverTime": 1618003698059,
         "exchangeFilters": [
             {
                 "filterType": "EXCHANGE_MAX_NUM_ORDERS",
                 "maxNumOrders": 123
             },
             {
                 "filterType": "EXCHANGE_MAX_NUM_ALGO_ORDERS",
                 "maxNumAlgoOrders": "456"
             }
         ],
         "rateLimits": [
             {
                 "interval": "MINUTE",
                 "intervalNum": 1,
                 "limit": 1200,
                 "rateLimitType": "RAW_REQUEST"
             },
             {
                 "interval": "SECOND",
                 "intervalNum": 10,
                 "limit": 100,
                 "rateLimitType": "REQUEST_WEIGHT"
             },
             {
                 "interval": "DAY",
                 "intervalNum": 1,
                 "limit": 200000,
                 "rateLimitType": "ORDERS"
             }
         ],
         "symbols": [
             {
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
                         "avgPriceMins": 5,
                         "filterType": "PERCENT_PRICE",
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
                         "applyToMarket": true,
                         "avgPriceMins": 5,
                         "filterType": "MIN_NOTIONAL",
                         "minNotional": "10.0000"
                     },
                     {
                         "filterType": "ICEBERG_PARTS",
                         "limit": 10
                     },
                     {
                         "filterType": "MARKET_LOT_SIZE",
                         "maxQty": "3200.00000000",
                         "minQty": "0.00000000",
                         "stepSize": "0.00000000"
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
                         "filterType": "MAX_NUM_ICEBERG_ORDERS",
                         "maxNumIcebergOrders": 5
                     },
                     {
                         "filterType": "MAX_POSITION",
                         "maxPosition": 10.0
                     }
                 ]
             }
         ]
     }"#;
}
