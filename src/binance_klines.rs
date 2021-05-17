use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    binance_context::BinanceContext,
    binance_signature::query_vec_u8,
    common::{get_req_get_response, time_ms_to_utc, utc_now_to_time_ms},
    common::{BinanceError, ResponseErrorRec},
};

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename = "camelCase")]
pub struct KlineRec {
    pub open_time: i64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub close_time: i64,
    pub quote_asset_volume: Decimal,
    pub number_of_trades: u64,
    pub taker_buy_base_asset_volume: Decimal,
    pub taker_buy_quote_asset_volume: Decimal,
    pub ignore: Decimal,
}

impl Default for KlineRec {
    fn default() -> KlineRec {
        KlineRec {
            open_time: 0,
            open: dec!(0),
            high: dec!(0),
            low: dec!(0),
            close: dec!(0),
            volume: dec!(0),
            close_time: 0,
            quote_asset_volume: dec!(0),
            number_of_trades: 0,
            taker_buy_base_asset_volume: dec!(0),
            taker_buy_quote_asset_volume: dec!(0),
            ignore: dec!(0),
        }
    }
}

#[allow(unused)]
pub enum KlineInterval {
    Mins1,
    Mins3,
    Mins5,
    Mins15,
    Mins30,
    Hrs1,
    Hrs2,
    Hrs4,
    Hrs6,
    Hrs8,
    Hrs12,
    Days1,
    Days3,
    Weeks,
    Months,
}

impl KlineInterval {
    #[allow(unused)]
    fn to_string(&self) -> &str {
        let interval_string: &str = match self {
            KlineInterval::Mins1 => "1m",
            KlineInterval::Mins3 => "3m",
            KlineInterval::Mins5 => "5m",
            KlineInterval::Mins15 => "15m",
            KlineInterval::Mins30 => "30m",
            KlineInterval::Hrs1 => "1h",
            KlineInterval::Hrs2 => "2h",
            KlineInterval::Hrs4 => "4h",
            KlineInterval::Hrs6 => "6h",
            KlineInterval::Hrs8 => "8h",
            KlineInterval::Hrs12 => "12h",
            KlineInterval::Days1 => "1d",
            KlineInterval::Days3 => "3d",
            KlineInterval::Weeks => "1w",
            KlineInterval::Months => "1M",
        };

        interval_string //.to_string()
    }
}

pub async fn get_klines(
    ctx: &BinanceContext,
    symbol: &str,
    interval: KlineInterval,
    start_time_ms: Option<i64>,
    end_time_ms: Option<i64>,
    limit: Option<u16>,
) -> Result<Vec<KlineRec>, Box<dyn std::error::Error>> {
    let mut params = vec![("symbol", symbol), ("interval", interval.to_string())];

    let st_ms_string: String;
    if let Some(st_ms) = start_time_ms {
        st_ms_string = st_ms.to_string();
        params.push(("startTime", &st_ms_string));
    }

    let et_ms_string: String;
    if let Some(et_ms) = end_time_ms {
        et_ms_string = et_ms.to_string();
        params.push(("endTime", &et_ms_string));
    }

    let limit_string: String;
    if let Some(l) = limit {
        limit_string = l.to_string();
        params.push(("limit", &limit_string));
    }

    trace!("get_klines: params={:#?}", params);

    let query = query_vec_u8(&params);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("get_klines: uery_string={}", &query_string);

    let url = ctx.make_url("api", &format!("/api/v3/klines?{}", query_string));
    trace!("get_klines: url={}", url);

    let response = get_req_get_response(&ctx.opts.api_key, &url).await?;
    let response_status = response.status();
    let response_body = response.text().await?;

    // Log the response
    let result = if response_status == 200 {
        // Convert the array of array of klinerec to serde_json::Value
        let values: Value = serde_json::from_str(&response_body)?;
        let klines: Vec<KlineRec> = serde_json::from_value(values)?;

        trace!("get_klines: klines={:?}", klines);
        Ok(klines)
    } else {
        let response_error_rec =
            ResponseErrorRec::new(false, response_status.as_u16(), &url, &response_body);
        let binance_error_response = BinanceError::Response(response_error_rec);

        trace!(
            "get_klines: error symbol={} resp_failure={:?}",
            symbol,
            binance_error_response,
        );

        Err(binance_error_response.into())
    };

    result
}

/// Get kline for sym_name at start_time_ms
pub async fn get_kline(
    ctx: &BinanceContext,
    sym_name: &str,
    start_time_ms: i64,
) -> Result<KlineRec, Box<dyn std::error::Error>> {
    // Some constants
    const SEC: i64 = 1000;
    const MIN: i64 = 60 * SEC;
    const INTERVAL_MIN: i64 = 1;
    const MINIMUM_INTERVAL_ELAPSED_SECS: i64 = 10 * SEC;

    // Truncate st to beginning of the current INTERVAL_MIN
    let mut st = start_time_ms;
    st = st - (st % (INTERVAL_MIN * MIN));

    // If st is within the first few seconds of now change
    // st to the previous minute, otherwise there may be
    // nothing returned as so little time has elapsed.
    // In my short empherical investigation this "dead"
    // interval was about 3 or 4 seconds.
    let now = utc_now_to_time_ms();
    if (st + MINIMUM_INTERVAL_ELAPSED_SECS) >= now {
        st -= MIN;
        trace!("backup to previous minute");
    }
    let krs: Vec<KlineRec> =
        get_klines(ctx, sym_name, KlineInterval::Mins1, Some(st), None, Some(1)).await?;

    if krs.is_empty() {
        Err(format!("No KlineRec available for {}", sym_name).into())
    } else {
        let kr = krs[0];
        trace!(
            "Open time: {} Close time: {} diff: {}",
            time_ms_to_utc(kr.open_time),
            time_ms_to_utc(kr.close_time),
            (kr.close_time - kr.open_time) as f64 / MIN as f64
        );
        trace!("{:#?}", kr);
        Ok(kr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const KLINE_REC: &str = r#"[
        [
            1499040000000,
            "0.01634790",
            "0.80000000",
            "0.01575800",
            "0.01577100",
            "148976.11427815",
            1499644799999,
            "2434.19055334",
            308,
            "1756.87402397",
            "28.46694368",
            "17928899.62484339"
        ]
    ]"#;

    #[test]
    fn test_kline_rec() {
        let v: Value = serde_json::from_str(KLINE_REC).unwrap();
        let krs: Vec<KlineRec> = serde_json::from_value(v).unwrap();
        //println!("krs: {:#?}", krs);
        assert_eq!(krs.len(), 1);
        assert_eq!(krs[0].open_time, 1499040000000);
        assert_eq!(krs[0].open, dec!(0.01634790));
        assert_eq!(krs[0].high, dec!(0.80000000));
        assert_eq!(krs[0].low, dec!(0.01575800));
        assert_eq!(krs[0].close, dec!(0.01577100));
        assert_eq!(krs[0].volume, dec!(148976.11427815));
        assert_eq!(krs[0].close_time, 1499644799999);
        assert_eq!(krs[0].quote_asset_volume, dec!(2434.19055334));
        assert_eq!(krs[0].number_of_trades, 308);
        assert_eq!(krs[0].taker_buy_base_asset_volume, dec!(1756.87402397));
        assert_eq!(krs[0].taker_buy_quote_asset_volume, dec!(28.46694368));
        assert_eq!(krs[0].ignore, dec!(17928899.62484339));
    }
}
