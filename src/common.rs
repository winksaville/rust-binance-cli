use chrono::prelude::{DateTime, NaiveDateTime, Utc};
use log::trace;

use reqwest::{
    self,
    header::{HeaderMap, HeaderValue},
    Response,
};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use strum_macros::IntoStaticStr;

use serde::{Deserialize, Serialize};

use crate::de_string_or_number::de_string_or_number_to_i64;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, IntoStaticStr)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum OrderType {
    MARKET,
    LIMIT,
    STOP_LOSS,
    STOP_LOSS_LIMIT,
    TAKE_PROFIT,
    TAKE_PROFIT_LIMIT,
    LIMIT_MAKER,
}

#[derive(Clone, Debug, Deserialize, Serialize, IntoStaticStr)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResponseErrorRec {
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub status: u16,
    #[serde(default)]
    pub query: String,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub code: i64,
    pub msg: String,
}

impl ResponseErrorRec {
    pub fn new(
        test: bool,
        status: u16,
        query: &str,
        body: &str, // Assumeed to be json object: "{ \"code\": -1121, \"msg\": \"string message\" }"
    ) -> Self {
        #[derive(Deserialize, Serialize)]
        struct CodeMsg {
            code: i64,
            msg: String,
        }
        let code_msg: CodeMsg = match serde_json::from_str(body) {
            Ok(cm) => cm,
            Err(_) => CodeMsg {
                code: 0,
                msg: body.to_string(),
            },
        };

        Self {
            test,
            status,
            query: query.to_string(),
            code: code_msg.code,
            msg: code_msg.msg,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BinanceError {
    Response(ResponseErrorRec),
}

impl Error for BinanceError {}

impl Display for BinanceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            BinanceError::Response(rec) => {
                write!(
                    f,
                    "BinanceError::Response: test={} status={} code={} msg={} query={}",
                    rec.test, rec.status, rec.code, rec.msg, rec.query
                )
            }
        }
    }
}

/// A Low Level post req and get response
pub async fn post_req_get_response_ll(
    url: &str,
    headers_map: HeaderMap,
    body: &str,
) -> Result<Response, Box<dyn std::error::Error>> {
    let mut req_builder = reqwest::Client::builder()
        //.proxy(reqwest::Proxy::https("http://localhost:8080")?)
        .build()?
        .post(url);
    if !headers_map.is_empty() {
        req_builder = req_builder.headers(headers_map);
    }
    req_builder = req_builder.body(body.to_owned());
    trace!("req_builder={:#?}", req_builder);

    let response = req_builder.send().await?;
    trace!("response={:#?}", response);

    Ok(response)
}

/// A Low Level get req and get response
pub async fn get_req_get_response_ll(
    url: &str,
    headers_map: HeaderMap,
) -> Result<Response, Box<dyn std::error::Error>> {
    let mut req_builder = reqwest::Client::builder()
        //.proxy(reqwest::Proxy::https("http://localhost:8080")?)
        .build()?
        .get(url);
    if !headers_map.is_empty() {
        req_builder = req_builder.headers(headers_map);
    }
    trace!("req_builder={:#?}", &req_builder);

    let response = req_builder.send().await?;
    trace!("response={:#?}", response);

    Ok(response)
}

/// Binance post_req_get_response
pub async fn post_req_get_response(
    api_key: &str,
    url: &str,
    body: &str,
) -> Result<Response, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(api_key)?);

    let response = post_req_get_response_ll(url, headers, body).await?;
    Ok(response)
}

/// Binance get_req_get_response
pub async fn get_req_get_response(
    api_key: &str,
    url: &str,
) -> Result<Response, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(api_key)?);

    let response = get_req_get_response_ll(url, headers).await?;
    Ok(response)
}

fn timestamp_ms_to_secs_nsecs(timestamp_ms: i64) -> (i64, u32) {
    // println!("time_ms_to_utc: + timestamp_ms={}", timestamp_ms);
    let mut secs = timestamp_ms / 1000;
    let ms: u32 = if timestamp_ms < 0 {
        // When time is less than zero the it's only negative
        // to the "epoch" thus seconds are "negative" but the
        // milli-seconds are positive. Thus -1ms is represented
        // in time as -1sec + 0.999ms. Sooooooo

        // First negate then modulo 1000 to get millis as a u32
        let mut millis = (-timestamp_ms % 1_000) as u32;

        // This is very "likely" and it would be nice to be able
        // to tell the compiler with `if likely(millis > 0) {...}
        if millis > 0 {
            // We need to reduce secs by 1
            secs -= 1;

            // And map ms 1..999 to 999..1
            millis = 1_000 - millis;
            // println!("time_ms_to_utc: adjusted   timestamp_ms={} secs={} millis={}", timestamp_ms, secs, millis);
        } else {
            // millis is 0 and secs is correct as is.
            // println!("time_ms_to_utc: unadjusted timestamp_ms={} secs={} millis={}", timestamp_ms, secs, millis);
        }

        millis
    } else {
        // This actually caused clippy to output "unnecessarary `let` binding"
        // but for I want to be able to have the pritnln and I've found that
        // allowing unnecessary_cast suppresses the warning.
        #[allow(clippy::unnecessary_cast)]
        let millis = (timestamp_ms % 1000) as u32;
        //println!("time_ms_to_utc: unadjusted timestamp_ms={} secs={} millis={}", timestamp_ms, secs, millis);

        millis
    };

    let nsecs = ms * 1_000_000u32;

    // println!("time_ms_to_utc: - timestamp_ms={} secs={} nsecs={}", timestamp_ms, secs, nsecs);
    (secs, nsecs)
}

pub fn time_ms_to_utc(timestamp_ms: i64) -> DateTime<Utc> {
    let (secs, nsecs) = timestamp_ms_to_secs_nsecs(timestamp_ms);
    let naive_datetime = NaiveDateTime::from_timestamp(secs, nsecs);
    DateTime::from_utc(naive_datetime, Utc)
}

pub fn utc_now_to_time_ms() -> i64 {
    (Utc::now().timestamp_nanos() + 500_000) / 1_000_000
}

pub fn utc_to_time_ms(date_time: &DateTime<Utc>) -> i64 {
    (date_time.timestamp_nanos() + 500_000) / 1_000_000
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_binance_response_error_rec() {
        const RESPONSE_FAILURE_BODY: &str = r#"{"code":-1121,"msg":"Invalid symbol."}"#;

        let response = ResponseErrorRec::new(false, 400, "a_query", RESPONSE_FAILURE_BODY);

        assert_eq!(response.test, false);
        assert_eq!(response.query, "a_query");
        assert_eq!(response.status, 400);
        assert_eq!(response.code, -1121);
        assert_eq!(response.msg, "Invalid symbol.");
    }

    #[test]
    fn test_binance_response_error_rec_bad_body() {
        const RESPONSE_FAILURE_BODY: &str = "An unexpected error";

        let response = ResponseErrorRec::new(false, 505, "a_query", RESPONSE_FAILURE_BODY);

        assert_eq!(response.test, false);
        assert_eq!(response.query, "a_query");
        assert_eq!(response.status, 505);
        assert_eq!(response.code, 0);
        assert_eq!(response.msg, "An unexpected error");
    }

    //fn test_binance_response_failure_as_Error() {
    //    assert!(get_binance_response_failure().is_err());
    //}

    #[test]
    fn test_timestamp_ms_to_secs_nsecs() {
        assert_eq!(timestamp_ms_to_secs_nsecs(-2001), (-3i64, 999_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(-2000), (-2i64, 0u32));
        //assert_eq!(timestamp_ms_to_secs_nsecs(-2000), (-3i64, 1_000_000_000u32)); // No Adjustment
        assert_eq!(timestamp_ms_to_secs_nsecs(-1999), (-2i64, 1_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(-1001), (-2i64, 999_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(-1000), (-1i64, 0u32));
        //assert_eq!(timestamp_ms_to_secs_nsecs(-1000), (0i64, 1_000_000_000u32)); // No adjustment
        assert_eq!(timestamp_ms_to_secs_nsecs(-999), (-1i64, 1_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(-1), (-1i64, 999_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(0), (0i64, 0u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(1), (0i64, 1_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(999), (0i64, 999_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(1000), (1i64, 0u32));
    }

    #[test]
    fn test_utc_now_to_time_ms() {
        let start = Instant::now();

        // Because we use integer arithmetic we must
        // see 2 milli-second time ticks to see a minimum
        // duration of > 1ms.
        let tms1 = utc_now_to_time_ms();
        let mut tms2 = tms1;
        while tms2 < (tms1 + 2) {
            tms2 = utc_now_to_time_ms();
        }
        let done = Instant::now();
        let duration = done.duration_since(start);

        println!(
            "tms1: {} tms2: {} done: {:?} - start {:?} = {}ns or {}ms",
            tms1,
            tms2,
            done,
            start,
            duration.as_nanos(),
            duration.as_millis()
        );

        assert!(tms2 >= (tms1 + 2));
        assert!(duration.as_millis() >= 1);

        // The duration.as_millis should be < 2ms. But with Tarpaulin
        // I've seen durations over 4ms so we skip this test.
        // assert!(duration.as_millis() < 2);
    }
}
