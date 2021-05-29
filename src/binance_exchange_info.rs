use log::trace;
use serde::{de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize};
//use serde_json::value::Value;

use rust_decimal::prelude::*;

use std::collections::HashMap;

use crate::common::get_req_get_response;
//use crate::de_string_or_number::u32_de_string_or_number;
use crate::de_string_or_number::de_string_or_number_to_u64;
use crate::de_string_or_number::de_string_or_number_to_u32;
use crate::{common::OrderType, configuration::Configuration};

use strum_macros::IntoStaticStr;
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct SizeRec {
    #[serde(rename = "minQty")]
    pub min_qty: Decimal,

    #[serde(rename = "maxQty")]
    pub max_qty: Decimal,

    #[serde(rename = "stepSize")]
    pub step_size: Decimal,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct PriceFilterRec {
    #[serde(rename = "minPrice")]
    pub min_price: Decimal,

    #[serde(rename = "maxPrice")]
    pub max_price: Decimal,

    #[serde(rename = "tickSize")]
    pub tick_size: Decimal,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct PercentPriceRec {
    #[serde(rename = "multiplierUp")]
    pub mulitplier_up: Decimal,

    #[serde(rename = "multiplierDown")]
    pub multiplier_down: Decimal,

    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    #[serde(rename = "avgPriceMins")]
    pub avg_price_mins: u64,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct MinNotionalRec {
    #[serde(rename = "minNotional")]
    pub min_notional: Decimal,

    #[serde(rename = "applyToMarket")]
    pub apply_to_market: bool,

    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    #[serde(rename = "avgPriceMins")]
    pub avg_price_mins: u64,
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
        #[serde(rename = "maxPosition")]
        max_position: Decimal,
    },
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
                // println!("item={:#?}", item);
                let key: &'static str = item.into();
                map.insert(key.to_string(), item);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
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

    pub fn get_mut_lot_size(&mut self) -> Option<&mut SizeRec> {
        match self {
            SymbolFilters::LotSize(sr) => Some(sr),
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

    pub fn get_max_position(&self) -> Option<Decimal> {
        match self {
            SymbolFilters::MaxPosition { max_position } => Some(*max_position),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
//pub struct Symbol<'a> {
pub struct Symbol {
    pub symbol: String,     // +enum BTCUSD?
    pub base_asset: String, // +enum BTC?
    #[serde(deserialize_with = "de_string_or_number_to_u32")]
    pub base_asset_precision: u32,
    #[serde(deserialize_with = "de_string_or_number_to_u32")]
    pub base_commission_precision: u32,
    pub iceberg_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub oco_allowed: bool,
    pub quote_asset: String, // +enum USD?
    #[serde(deserialize_with = "de_string_or_number_to_u32")]
    pub quote_asset_precision: u32,
    #[serde(deserialize_with = "de_string_or_number_to_u32")]
    pub quote_commission_precision: u32,
    pub quote_order_qty_market_allowed: bool,
    #[serde(deserialize_with = "de_string_or_number_to_u32")]
    pub quote_precision: u32,
    pub status: String, // +enum TRADING?
    pub permissions: Vec<String>,
    pub order_types: Vec<OrderType>, // +HashSet<OrderType>?
    #[serde(deserialize_with = "de_vec_symbol_filters_to_hashmap")]
    #[serde(rename = "filters")]
    pub filters_map: HashMap<String, SymbolFilters>,
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
                // println!("sym={:#?}", sym);
                map.insert(sym.symbol.clone(), sym);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

#[allow(unused)] // For now used in testing
impl Symbol {
    pub fn get_lot_size(&self) -> Option<&SizeRec> {
        self.filters_map.get("LotSize")?.get_lot_size()
    }

    pub fn get_market_lot_size(&self) -> Option<&SizeRec> {
        self.filters_map.get("MarketLotSize")?.get_market_lot_size()
    }

    // Used for testing only
    pub fn get_mut_lot_size(&mut self) -> Option<&mut SizeRec> {
        self.filters_map.get_mut("LotSize")?.get_mut_lot_size()
    }

    pub fn get_price_filter(&self) -> Option<&PriceFilterRec> {
        self.filters_map.get("PriceFilter")?.get_price_filter()
    }

    pub fn get_percent_price(&self) -> Option<&PercentPriceRec> {
        self.filters_map.get("PercentPrice")?.get_percent_price()
    }

    pub fn get_min_notional(&self) -> Option<&MinNotionalRec> {
        self.filters_map.get("MinNotional")?.get_min_notional()
    }

    pub fn get_iceberg_parts(&self) -> Option<u64> {
        self.filters_map.get("IcebergParts")?.get_iceberg_parts()
    }

    pub fn get_max_num_iceberg_orders(&self) -> Option<u64> {
        self.filters_map
            .get("MaxNumIcebergOrders")?
            .get_max_num_iceberg_orders()
    }

    pub fn get_max_num_orders(&self) -> Option<u64> {
        self.filters_map.get("MaxNumOrders")?.get_max_num_orders()
    }

    pub fn get_max_num_algo_orders(&self) -> Option<u64> {
        self.filters_map
            .get("MaxNumAlgoOrders")?
            .get_max_num_algo_orders()
    }

    pub fn get_max_position(&self) -> Option<Decimal> {
        self.filters_map.get("MaxPosition")?.get_max_position()
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoStaticStr)]
#[serde(tag = "filterType")]
pub enum ExchangeFilters {
    #[serde(rename = "EXCHANGE_MAX_NUM_ORDERS")]
    ExchangeMaxNumOrders {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "maxNumOrders")]
        max_num_orders: u64,
    },
    #[serde(rename = "EXCHANGE_MAX_NUM_ALGO_ORDERS")]
    ExchangeMaxNumAlgoOrders {
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        #[serde(rename = "maxNumAlgoOrders")]
        max_num_algo_orders: u64,
    },
}

impl ExchangeFilters {
    pub fn get_max_num_orders(&self) -> Option<u64> {
        match self {
            ExchangeFilters::ExchangeMaxNumOrders { max_num_orders } => Some(*max_num_orders),
            _ => None,
        }
    }

    pub fn get_max_num_algo_orders(&self) -> Option<u64> {
        match self {
            ExchangeFilters::ExchangeMaxNumAlgoOrders {
                max_num_algo_orders,
            } => Some(*max_num_algo_orders),
            _ => None,
        }
    }
}

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
pub fn de_vec_exchange_filters_to_hashmap<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, ExchangeFilters>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = HashMap<String, ExchangeFilters>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<String, ExchangeFilters>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<String, ExchangeFilters> =
                HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<ExchangeFilters>()? {
                // println!("item={:#?}", item);
                let key: &'static str = item.into();
                map.insert(key.to_string(), item);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoStaticStr)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitType {
    RawRequest,
    RequestWeight,
    Orders,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoStaticStr)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntervalType {
    Minute,
    Second,
    Day,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: RateLimitType, // Type of rate limit
    pub interval: IntervalType,         // Type of interval
    pub interval_num: u64,              // interval_num * interval is a duration
    pub limit: u64,                     // limit is the maximum rate in the duration.
}

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
pub fn de_vec_rate_limit_to_hashmap<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, RateLimit>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = HashMap<String, RateLimit>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<String, RateLimit>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<String, RateLimit> =
                HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<RateLimit>()? {
                // println!("item={:#?}", item);
                let key: &'static str = item.rate_limit_type.into();
                map.insert(key.to_string(), item);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

impl RateLimit {
    pub fn get_raw_request(&self) -> Option<&RateLimit> {
        match self.rate_limit_type {
            RateLimitType::RawRequest => Some(self),
            _ => None,
        }
    }

    pub fn get_request_weight(&self) -> Option<&RateLimit> {
        match self.rate_limit_type {
            RateLimitType::RequestWeight => Some(self),
            _ => None,
        }
    }

    pub fn get_orders(&self) -> Option<&RateLimit> {
        match self.rate_limit_type {
            RateLimitType::Orders => Some(self),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub server_time: u64,
    #[serde(deserialize_with = "de_vec_exchange_filters_to_hashmap")]
    pub exchange_filters: HashMap<String, ExchangeFilters>,
    #[serde(deserialize_with = "de_vec_rate_limit_to_hashmap")]
    pub rate_limits: HashMap<String, RateLimit>,
    #[serde(deserialize_with = "de_vec_symbols_to_hashmap")]
    #[serde(rename = "symbols")]
    pub symbols_map: HashMap<String, Symbol>,
}

#[allow(unused)]
impl ExchangeInfo {
    pub fn get_max_num_orders(&self) -> Option<u64> {
        self.exchange_filters
            .get("ExchangeMaxNumOrders")?
            .get_max_num_orders()
    }

    pub fn get_max_num_algo_orders(&self) -> Option<u64> {
        self.exchange_filters
            .get("ExchangeMaxNumAlgoOrders")?
            .get_max_num_algo_orders()
    }

    pub fn get_raw_request_rate_limit(&self) -> Option<&RateLimit> {
        self.rate_limits.get("RawRequest")?.get_raw_request()
    }

    pub fn get_request_weight_rate_limit(&self) -> Option<&RateLimit> {
        self.rate_limits.get("RequestWeight")?.get_request_weight()
    }

    pub fn get_orders_rate_limit(&self) -> Option<&RateLimit> {
        self.rate_limits.get("Orders")?.get_orders()
    }

    pub fn get_symbol(&self, symbol: &str) -> Option<&Symbol> {
        self.symbols_map.get(symbol)
    }
}

pub async fn get_exchange_info<'e>(
    config: &Configuration,
) -> Result<ExchangeInfo, Box<dyn std::error::Error>> {
    trace!("get_exchange_info: +");

    let url = config.make_url("api", "/api/v3/exchangeInfo");
    trace!("get_exchange_info: url={}", url);

    let response = get_req_get_response(&config.api_key, &url).await?;
    trace!("response={:#?}", response);

    let response_status = response.status();
    let response_body = response.text().await?;
    if response_status != 200 {
        let err = format!("error  status={} body={}", response_status, response_body);
        trace!("get_account_info: err: {}", err);
        return Err(err.into());
    }

    let exchange_info: ExchangeInfo = serde_json::from_str(&response_body)?;

    trace!("get_exchange_info: -");
    Ok(exchange_info)
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_exchange_info() {
        let ei: ExchangeInfo = match serde_json::from_str(EXCHANGE_INFO_DATA) {
            Ok(info) => info,
            Err(e) => panic!("Error processing response: e={}", e),
        };
        // println!("ei.server_time={:#?}", ei.server_time);
        assert_eq!(ei.server_time, 1618003698059);

        // Verify we can get "the" symbol
        let btcusd = ei.get_symbol("BTCUSD");
        assert!(btcusd.is_some(), "BTCUSD should have been found");
        let btcusd = btcusd.unwrap();
        // println!("btcusd={:#?}", btcusd);
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

        // Test order_types in bulk
        assert_eq!(
            btcusd.order_types,
            [
                OrderType::LIMIT,
                OrderType::LIMIT_MAKER,
                OrderType::MARKET,
                OrderType::STOP_LOSS_LIMIT,
                OrderType::TAKE_PROFIT_LIMIT,
            ]
        );

        // Test order_types one at a time
        assert!(btcusd.order_types.contains(&OrderType::LIMIT));
        assert!(btcusd.order_types.contains(&OrderType::LIMIT_MAKER));
        assert!(btcusd.order_types.contains(&OrderType::MARKET));
        assert!(btcusd.order_types.contains(&OrderType::STOP_LOSS_LIMIT));
        assert!(btcusd.order_types.contains(&OrderType::TAKE_PROFIT_LIMIT));
        assert!(!btcusd.order_types.contains(&OrderType::STOP_LOSS));
        assert!(!btcusd.order_types.contains(&OrderType::TAKE_PROFIT));

        // Verify we get None when a symbol isn't found
        assert!(ei.symbols_map.get("NOT-A-SYMBOL").is_none());

        let ei_mno = ei.get_max_num_orders();
        assert!(ei_mno.is_some());
        let ei_mno = ei_mno.unwrap();
        assert_eq!(ei_mno, 123);

        let ei_mnao = ei.get_max_num_algo_orders();
        assert!(ei_mnao.is_some());
        let ei_mnao = ei_mnao.unwrap();
        assert_eq!(ei_mnao, 456);

        let ei_rr_rl = ei.get_raw_request_rate_limit();
        assert!(ei_rr_rl.is_some(), "Should always succeed");
        let ei_rr_rl = ei_rr_rl.unwrap();
        assert!(matches!(
            ei_rr_rl.rate_limit_type,
            RateLimitType::RawRequest
        ));
        assert!(matches!(ei_rr_rl.interval, IntervalType::Minute));
        assert_eq!(ei_rr_rl.interval_num, 1);
        assert_eq!(ei_rr_rl.limit, 1200);

        let ei_rw_rl = ei.get_request_weight_rate_limit();
        assert!(ei_rw_rl.is_some(), "Should always succeed");
        let ei_rw_rl = ei_rw_rl.unwrap();
        assert!(matches!(
            ei_rw_rl.rate_limit_type,
            RateLimitType::RequestWeight
        ));
        assert!(matches!(ei_rw_rl.interval, IntervalType::Second));
        assert_eq!(ei_rw_rl.interval_num, 10);
        assert_eq!(ei_rw_rl.limit, 100);

        let ei_orders_rl = ei.get_orders_rate_limit();
        assert!(ei_orders_rl.is_some(), "Should always succeed");
        let ei_orders_rl = ei_orders_rl.unwrap();
        assert!(matches!(
            ei_orders_rl.rate_limit_type,
            RateLimitType::Orders
        ));
        assert!(matches!(ei_orders_rl.interval, IntervalType::Day));
        assert_eq!(ei_orders_rl.interval_num, 1);
        assert_eq!(ei_orders_rl.limit, 200000);

        let pfr = btcusd.get_price_filter();
        assert!(pfr.is_some(), "Should always succeed");
        let pfr = pfr.unwrap();
        assert_eq!(pfr.min_price, dec!(0.01));
        assert_eq!(pfr.max_price, dec!(100000.0));
        assert_eq!(pfr.tick_size, dec!(0.01));

        let ppr = btcusd.get_percent_price();
        assert!(ppr.is_some(), "Should always succeed");
        let ppr = ppr.unwrap();
        assert_eq!(ppr.multiplier_down, dec!(0.2));
        assert_eq!(ppr.mulitplier_up, dec!(5.0));

        let btcusd_ls = btcusd.get_lot_size().unwrap();
        assert_eq!(dec!(0.000001), btcusd_ls.min_qty);
        assert_eq!(dec!(9000.0), btcusd_ls.max_qty);
        assert_eq!(dec!(0.000001), btcusd_ls.step_size);

        let btcusd_ls = btcusd.get_market_lot_size().unwrap();
        assert_eq!(dec!(0.1), btcusd_ls.min_qty);
        assert_eq!(dec!(3200.0), btcusd_ls.max_qty);
        assert_eq!(dec!(0.01), btcusd_ls.step_size);

        let mnr = btcusd.get_min_notional();
        assert!(mnr.is_some(), "Should always succeed");
        let mnr = mnr.unwrap();
        assert_eq!(mnr.min_notional, dec!(0.001));
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
        assert_eq!(mp, dec!(10));
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
