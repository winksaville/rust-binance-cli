use serde::{de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize};
//use serde_json::value::Value;

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

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct SizeRec {
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

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct PriceFilterRec {
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    #[serde(rename = "minPrice")]
    min_price: f64,

    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    #[serde(rename = "maxPrice")]
    max_price: f64,

    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    #[serde(rename = "tickSize")]
    tick_size: f64,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct PercentPriceRec {
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    #[serde(rename = "multiplierUp")]
    mulitplier_up: f64,

    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    #[serde(rename = "multiplierDown")]
    multiplier_down: f64,

    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    #[serde(rename = "avgPriceMins")]
    avg_price_mins: u64,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct MinNotionalRec {
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    #[serde(rename = "minNotional")]
    min_notional: f64,

    #[serde(rename = "applyToMarket")]
    apply_to_market: bool,

    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    #[serde(rename = "avgPriceMins")]
    avg_price_mins: u64,
}

// Accessing this requires a match and isn't pretty, IMHO.
// Maybe [enum-as-inner](https://crates.io/crates/enum-as-inner#:~:text=named%20field%20case)
// Or [enum variants as types](https://www.reddit.com/r/rust/comments/2rdoxx/enum_variants_as_types/)
// related to [datasort refinements](https://github.com/rust-lang/rfcs/issues/754)?
#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoStaticStr)]
#[serde(tag = "filterType")]
pub enum SymbolFilters {
    #[serde(rename = "PRICE_FILTER")]
    PriceFilter(PriceFilterRec),

    #[serde(rename = "PERCENT_PRICE")]
    PercentPrice(PercentPriceRec),

    #[serde(rename = "LOT_SIZE")]
    LotSize(SizeRec),

    #[serde(rename = "MARKET_LOT_SIZE")]
    MarketLotSize(SizeRec),

    #[serde(rename = "MIN_NOTIONAL")]
    MinNotional(MinNotionalRec),

    #[serde(rename = "ICEBERG_PARTS")]
    IcebergParts {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        limit: u64,
    },

    #[serde(rename = "MAX_NUM_ICEBERG_ORDERS")]
    MaxNumIcebergOrders {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "maxNumIcebergOrders")]
        max_num_iceberg_orders: u64,
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

    #[serde(rename = "MAX_POSITION")]
    MaxPosition {
        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        #[serde(rename = "maxPosition")]
        max_position: f64,
    },
}

impl SymbolFilters {
    pub fn get_lot_size(&self) -> Option<&SizeRec> {
        match self {
            SymbolFilters::LotSize(sr) => Some(sr),
            _ => None,
        }
    }

    pub fn get_market_lot_size(&self) -> Option<&SizeRec> {
        match self {
            SymbolFilters::MarketLotSize(sr) => Some(sr),
            _ => None,
        }
    }

    pub fn get_price_filter(&self) -> Option<&PriceFilterRec> {
        match self {
            SymbolFilters::PriceFilter(pfr) => Some(pfr),
            _ => None,
        }
    }

    pub fn get_percent_price(&self) -> Option<&PercentPriceRec> {
        match self {
            SymbolFilters::PercentPrice(ppr) => Some(ppr),
            _ => None,
        }
    }

    pub fn get_min_notional(&self) -> Option<&MinNotionalRec> {
        match self {
            SymbolFilters::MinNotional(mnr) => Some(mnr),
            _ => None,
        }
    }

    pub fn get_iceberg_parts(&self) -> Option<u64> {
        match self {
            SymbolFilters::IcebergParts { limit } => Some(*limit),
            _ => None,
        }
    }

    pub fn get_max_num_iceberg_orders(&self) -> Option<u64> {
        match self {
            SymbolFilters::MaxNumIcebergOrders {
                max_num_iceberg_orders,
            } => Some(*max_num_iceberg_orders),
            _ => None,
        }
    }

    pub fn get_max_num_orders(&self) -> Option<u64> {
        match self {
            SymbolFilters::MaxNumOrders { max_num_orders } => Some(*max_num_orders),
            _ => None,
        }
    }

    pub fn get_max_num_algo_orders(&self) -> Option<u64> {
        match self {
            SymbolFilters::MaxNumAlgoOrders {
                max_num_algo_orders,
            } => Some(*max_num_algo_orders),
            _ => None,
        }
    }

    pub fn get_max_position(&self) -> Option<f64> {
        match self {
            SymbolFilters::MaxPosition { max_position } => Some(*max_position),
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
//pub struct Symbol<'a> {
pub struct Symbol {
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
    #[serde(deserialize_with = "de_vec_symbol_filters_to_hashmap")]
    #[serde(rename = "filters")]
    pub filters_map: HashMap<String, SymbolFilters>,
}

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
pub fn de_vec_symbol_filters_to_hashmap<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, SymbolFilters>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = HashMap<String, SymbolFilters>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<String, SymbolFilters>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<String, SymbolFilters> =
                HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<SymbolFilters>()? {
                println!("item={:#?}", item);
                let key: &'static str = item.into();
                map.insert(key.to_string(), item);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
pub fn de_vec_symbols_to_hashmap<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Symbol>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = HashMap<String, Symbol>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<String, Symbol>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<String, Symbol> =
                HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(sym) = seq.next_element::<Symbol>()? {
                println!("sym={:#?}", sym);
                map.insert(sym.symbol.clone(), sym);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

impl Symbol {
    #[allow(unused)] // For now used in testing
    pub fn get_lot_size(&self) -> Option<&SizeRec> {
        self.filters_map.get("LotSize")?.get_lot_size()
    }

    #[allow(unused)] // For now used in testing
    pub fn get_market_lot_size(&self) -> Option<&SizeRec> {
        self.filters_map.get("MarketLotSize")?.get_market_lot_size()
    }

    #[allow(unused)] // For now used in testing
    pub fn get_price_filter(&self) -> Option<&PriceFilterRec> {
        self.filters_map.get("PriceFilter")?.get_price_filter()
    }

    #[allow(unused)] // For now used in testing
    pub fn get_percent_price(&self) -> Option<&PercentPriceRec> {
        self.filters_map.get("PercentPrice")?.get_percent_price()
    }

    #[allow(unused)] // For now used in testing
    pub fn get_min_notional(&self) -> Option<&MinNotionalRec> {
        self.filters_map.get("MinNotional")?.get_min_notional()
    }

    #[allow(unused)] // For now used in testing
    pub fn get_iceberg_parts(&self) -> Option<u64> {
        self.filters_map.get("IcebergParts")?.get_iceberg_parts()
    }

    #[allow(unused)] // For now used in testing
    pub fn get_max_num_iceberg_orders(&self) -> Option<u64> {
        self.filters_map
            .get("MaxNumIcebergOrders")?
            .get_max_num_iceberg_orders()
    }

    #[allow(unused)] // For now used in testing
    pub fn get_max_num_orders(&self) -> Option<u64> {
        self.filters_map.get("MaxNumOrders")?.get_max_num_orders()
    }

    #[allow(unused)] // For now used in testing
    pub fn get_max_num_algo_orders(&self) -> Option<u64> {
        self.filters_map
            .get("MaxNumAlgoOrders")?
            .get_max_num_algo_orders()
    }

    #[allow(unused)] // For now used in testing
    pub fn get_max_position(&self) -> Option<f64> {
        self.filters_map.get("MaxPosition")?.get_max_position()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub server_time: u64,
    pub exchange_filters: Vec<ExchangeFilters>,
    pub rate_limits: Vec<RateLimit>,
    #[serde(deserialize_with = "de_vec_symbols_to_hashmap")]
    #[serde(rename = "symbols")]
    symbols_map: HashMap<String, Symbol>,
}

#[allow(unused)]
impl<'e> ExchangeInfo {
    pub fn get_sym(&self, symbol: &str) -> Option<&Symbol> {
        self.symbols_map.get(symbol)
    }
}

#[cfg(test)]
mod test {
    //extern crate test;

    //#[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_exchange_info() {
        let ei: ExchangeInfo = match serde_json::from_str(EXCHANGE_INFO_DATA) {
            Ok(info) => info,
            Err(e) => panic!("Error processing response: e={}", e),
        };
        //ei.symbols_map = create_symbols_map(&ei.symbols);
        println!("ei.server_time={:#?}", ei.server_time);
        //println!("ei={:#?}", ei);

        // Verify we can get "the" symbol
        let btcusd = ei.symbols_map.get("BTCUSD");
        assert!(btcusd.is_some(), "BTCUSD should have been found");
        let btcusd = btcusd.unwrap();
        println!("btcusd={:#?}", btcusd);
        assert_eq!(btcusd.symbol, "BTCUSD");
        assert_eq!(btcusd.base_asset, "BTC");
        assert_eq!(btcusd.quote_asset, "USD");
        assert_eq!(btcusd.base_asset_precision, 8);
        assert_eq!(btcusd.base_commission_precision, 8);
        assert_eq!(btcusd.iceberg_allowed, true);
        assert_eq!(btcusd.is_margin_trading_allowed, false);
        assert_eq!(btcusd.is_spot_trading_allowed, true);
        assert_eq!(btcusd.oco_allowed, true);
        assert_eq!(btcusd.quote_asset_precision, 4);
        assert_eq!(btcusd.quote_commission_precision, 2);
        assert_eq!(btcusd.quote_order_qty_market_allowed, true);
        assert_eq!(btcusd.quote_precision, 4);
        assert_eq!(btcusd.status, "TRADING");
        assert_eq!(btcusd.permissions, ["SPOT"]);
        assert_eq!(
            btcusd.order_types,
            [
                "LIMIT",
                "LIMIT_MAKER",
                "MARKET",
                "STOP_LOSS_LIMIT",
                "TAKE_PROFIT_LIMIT",
            ]
        );

        // Verify we get None when a symbol isn't found
        assert!(ei.symbols_map.get("NOT-A-SYMBOL").is_none());

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

        let pfr = btcusd.get_price_filter();
        assert!(pfr.is_some(), "Should always succeed");
        let pfr = pfr.unwrap();
        assert_eq!(pfr.min_price, 0.01);
        assert_eq!(pfr.max_price, 100000.0);
        assert_eq!(pfr.tick_size, 0.01);

        let ppr = btcusd.get_percent_price();
        assert!(ppr.is_some(), "Should always succeed");
        let ppr = ppr.unwrap();
        assert_eq!(ppr.multiplier_down, 0.2);
        assert_eq!(ppr.mulitplier_up, 5.0);

        let btcusd_ls = btcusd.get_lot_size().unwrap();
        assert_eq!(0.000001, btcusd_ls.min_qty);
        assert_eq!(9000.0, btcusd_ls.max_qty);
        assert_eq!(0.000001, btcusd_ls.step_size);

        let btcusd_ls = btcusd.get_market_lot_size().unwrap();
        assert_eq!(0.1, btcusd_ls.min_qty);
        assert_eq!(3200.0, btcusd_ls.max_qty);
        assert_eq!(0.01, btcusd_ls.step_size);

        let mnr = btcusd.get_min_notional();
        assert!(mnr.is_some(), "Should always succeed");
        let mnr = mnr.unwrap();
        assert_eq!(mnr.min_notional, 0.001);
        assert_eq!(mnr.apply_to_market, true);
        assert_eq!(mnr.avg_price_mins, 5);

        let ibp = btcusd.get_iceberg_parts();
        assert!(ibp.is_some(), "Should always succeed");
        let ibp = ibp.unwrap();
        assert_eq!(ibp, 10);

        let mnibo = btcusd.get_max_num_iceberg_orders();
        assert!(mnibo.is_some(), "Should always succeed");
        let mnibo = mnibo.unwrap();
        assert_eq!(mnibo, 5);

        let mno = btcusd.get_max_num_orders();
        assert!(mno.is_some(), "Should always succeed");
        let mno = mno.unwrap();
        assert_eq!(mno, 200);

        let mnao = btcusd.get_max_num_algo_orders();
        assert!(mnao.is_some(), "Should always succeed");
        let mno = mnao.unwrap();
        assert_eq!(mno, 5);

        let mp = btcusd.get_max_position();
        assert!(mp.is_some(), "Should always succeed");
        let mp = mp.unwrap();
        assert_eq!(mp, 10.0);
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
                         "minQty": "0.10000000",
                         "stepSize": "0.01000000"
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
             }
         ]
     }"#;
}
