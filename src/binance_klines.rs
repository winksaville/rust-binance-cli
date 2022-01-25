use std::fmt::{self, Display};

use chrono::Local;
use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use strum_macros::IntoStaticStr;

use crate::{
    binance_order_response::TradeResponse,
    binance_signature::query_vec_u8,
    common::{get_req_get_response, time_ms_to_utc, utc_now_to_time_ms, ResponseErrorRec},
    configuration::Configuration,
};

// Seconds and minutes in milli-seconds
const SEC: i64 = 1000;
const MIN: i64 = 60 * SEC;

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
#[derive(PartialEq, IntoStaticStr)]
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

impl Display for KlineRec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let local = Local::now().timezone();
        trace!("Display::InternalErrorRec: {:#?}", self);
        write!(
            f,
            r#"        openTime: {}
       closeTime: {}
            open: {:10.4}
            high: {:10.4}
             low: {:10.4}
           close: {:10.4}
          volume: {:10.4}
  numberOfTrades: {}
 BaseAssetVolume: {}
QuoteAssetVolume: {}"#,
            time_ms_to_utc(self.open_time).with_timezone(&local),
            time_ms_to_utc(self.close_time).with_timezone(&local),
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
            self.number_of_trades,
            self.taker_buy_base_asset_volume,
            self.taker_buy_quote_asset_volume,
        )
    }
}

impl KlineInterval {
    pub fn from_string(s: &str) -> Result<KlineInterval, Box<dyn std::error::Error>> {
        let interval: Self = match s {
            "1m" => KlineInterval::Mins1,
            "3m" => KlineInterval::Mins3,
            "5m" => KlineInterval::Mins5,
            "15m" => KlineInterval::Mins15,
            "30m" => KlineInterval::Mins30,
            "1h" => KlineInterval::Hrs1,
            "2h" => KlineInterval::Hrs2,
            "4h" => KlineInterval::Hrs4,
            "6h" => KlineInterval::Hrs6,
            "8h" => KlineInterval::Hrs8,
            "12h" => KlineInterval::Hrs12,
            "1d" => KlineInterval::Days1,
            "3d" => KlineInterval::Days3,
            "1w" => KlineInterval::Weeks,
            "1M" => KlineInterval::Months,
            _ => return Err(format!("Unknown kline interval, {}, expecting: 1m 3m 5m 15m 30m 1h 2h 4h 6h 8h 12h 1d 3d 1w 1M", s).into()),
        };

        Ok(interval)
    }

    pub fn to_string(&self) -> &str {
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

        interval_string
    }
}

/// Get zero or more klines for the symbol at the specified KlineInterval.
/// A maximum of 1000 records is returned.
pub async fn get_klines(
    config: &Configuration,
    symbol: &str,
    interval: KlineInterval,
    start_time_ms: Option<i64>,
    end_time_ms: Option<i64>,
    limit: Option<u16>,
) -> Result<Vec<KlineRec>, Box<dyn std::error::Error>> {
    trace!("get_klines:");

    let mut params = vec![("symbol", symbol), ("interval", interval.to_string())];

    let st_ms_string: String;
    if let Some(st_ms) = start_time_ms {
        // If st_ms is within the first minute of "now" or it is
        // in the future do not send the startTime.
        // Alhough my simple emperical testing indicated the window
        // where no data is sent is only 2 or 3 seconds, I've chosen
        // to not send the startTime if the time is within a minute,
        // which is the smallest interval. This causes the most current
        // N records to allways be returned.
        let now = utc_now_to_time_ms();
        if now - st_ms > MIN {
            st_ms_string = st_ms.to_string();
            params.push(("startTime", &st_ms_string));
        }
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
    trace!("get_klines: query_string: {}", &query_string);

    let url = config.make_url("api", &format!("/api/v3/klines?{}", query_string));
    trace!("get_klines: url={}", url);

    let response = get_req_get_response(config.keys.get_ak_or_err()?, &url).await?;
    let response_headers = response.headers().clone();
    //println!("{response_headers:#?}");
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
        let rer = ResponseErrorRec::new(
            false,
            response_status.as_u16(),
            &url,
            response_headers,
            &response_body,
        );
        let binance_error_response = TradeResponse::FailureResponse(rer);

        //trace!(
        println!(
            "get_klines: error symbol={} resp_failure={:?}",
            symbol, binance_error_response,
        );

        Err(binance_error_response.into())
    };

    result
}

/// Get kline using KlineInterval Mins1 for sym_name at start_time_ms.
/// The start_time_ms is UTC.
#[allow(unused)]
pub async fn get_kline(
    config: &Configuration,
    sym_name: &str,
    start_time_ms: i64,
) -> Result<KlineRec, Box<dyn std::error::Error>> {
    //println!("get_kline: {sym_name} {start_time_ms} {}", time_ms_to_utc(start_time_ms));

    let krs: Vec<KlineRec> = get_klines(
        config,
        sym_name,
        KlineInterval::Mins1,
        Some(start_time_ms),
        None,
        Some(1),
    )
    .await?;

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
        Ok(kr)
    }
}

/// Get the approximate value of a base_asset in one of
/// the provided value_assets for the specified time.
pub async fn get_kline_of_primary_asset_for_value_asset(
    config: &Configuration,
    time: i64,
    primary_asset: &str,
    value_assets: &[&str],
) -> Option<(String, KlineRec)> {
    for value_asset in value_assets {
        let sym_name = primary_asset.to_owned() + value_asset;
        match get_kline(config, &sym_name, time).await {
            Ok(kr) => {
                trace!("Ok: {sym_name} time: {} kr: {kr:?}", time_ms_to_utc(time));
                return Some((sym_name, kr));
            }
            Err(e) => {
                trace!("Error: {sym_name} time: {} e: {e}", time_ms_to_utc(time));
            }
        }
    }
    None
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
