use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, SecondsFormat, TimeZone, Utc};
use lazy_static::lazy_static;
use log::trace;
use rust_decimal::Decimal;
use rusty_money::{iso, Money};
use separator::FixedPlaceSeparatable;

use reqwest::{
    self,
    header::{HeaderMap, HeaderValue},
    Response,
};
use std::{
    fmt::{self, Debug, Display},
    fs::File,
    io::stdout,
    io::{stdin, Write},
    io::{BufReader, BufWriter},
    path::Path,
};
use strum_macros::IntoStaticStr;

use serde::{Deserialize, Serialize};

use crate::de_string_or_number::de_string_or_number_to_i64;
use crate::serde_header_map::{de_header_map, se_header_map};

const PKG_VER: &str = env!("CARGO_PKG_VERSION");
const GIT_SHORT_SHA: &str = env!("VERGEN_GIT_SHA_SHORT");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

lazy_static! {
    // I'm not sure this is the right approach but
    // having a static String seems to be reasonable
    // so it's computed only once.
    pub static ref APP_VERSION: String = format!("{}-{}", PKG_VER, GIT_SHORT_SHA);
    pub static ref APP_NAME: String = PKG_NAME.to_string();

    #[derive(Debug)]
    pub static ref VALUE_ASSETS: Vec<String>  = vec!("USD".to_owned(), "USDT".to_owned(), "BUSD".to_owned());
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalErrorRec {
    pub code: u32,
    pub line: u32,
    pub fn_name: String,
    pub file: String,
    pub msg: String,
}

impl InternalErrorRec {
    #[allow(unused)]
    pub fn new(code: u32, file: &str, fn_name: &str, line: u32, message: &str) -> Self {
        InternalErrorRec {
            code,
            file: String::from(file),
            fn_name: String::from(fn_name),
            line,
            msg: String::from(message),
        }
    }
}

impl Display for InternalErrorRec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ver = &APP_VERSION;
        trace!("Display::InternalErrorRec: {:#?}", self);
        if !self.fn_name.is_empty() {
            write!(
                f,
                "InternalErrorRec app-ver: {} file: {} fn: {} line: {} code: {} msg: {}",
                ver.as_str(),
                self.file,
                self.fn_name,
                self.line,
                self.code,
                self.msg,
            )
        } else {
            write!(
                f,
                "InternalErrorRec: app-ver: {} file: {} line:{} code: {} msg: {}",
                ver.as_str(),
                self.file,
                self.line,
                self.code,
                self.msg,
            )
        }
    }
}

impl std::error::Error for InternalErrorRec {}

#[macro_export]
macro_rules! ier_new {
    ( $c:expr, $m:expr ) => {
        //InternalErrorRec::new($c, std::file!(), function_name!(), std::line!(), $m);
        InternalErrorRec::new($c, std::file!(), "", std::line!(), $m)
    };
}

#[macro_export]
macro_rules! ie_msg {
    ( $c:expr, $m:expr ) => {
        //InternalErrorRec::new($c, std::file!(), function_name!(), std::line!(), $m);
        &format!(
            "InternalErrorRec: {}:{} code: {} msg: {}",
            std::file!(),
            std::line!(),
            $c,
            $m
        );
    };
}

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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, IntoStaticStr)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Side {
    BUY,
    SELL,
}

impl Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        trace!("Display::Side: {:#?}", self);
        let side_str = match self {
            Side::SELL => "Sell",
            Side::BUY => "Buy",
        };

        write!(f, "{}", side_str)
    }
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

    // Maybe headers don't need to be optional, as it's most
    // likely there are never empty on a response and they are
    // needed to eventually handle binance errors 429 and 418
    // to fetch the Retry-After.
    //
    // So I need it but there maybe no need to ever serde it!
    //
    // But if I do want to be able to serde it there appears
    // to be no reason to make it Optional. The original thought
    // was I needed to be able to set it to None when deserializing
    // "old" order logs that wouldn't have it. But there it can
    // be handled as an empty HeaderMap.
    //
    // So for the moment leave it as is, but Optional probably
    // isn't necessary any maybe there is no need to support
    // serde at all!

    // But using "skip_serializing_if" is the way to have headers
    // never be serialized when None!!!!!!!!
    // I figured this out using this search: https://www.google.com/search?q=rust+serializer+generate+nothing+if+None
    // And findin this https://stackoverflow.com/a/53900684/4812090 on this
    // stackoverflow question: https://stackoverflow.com/questions/53900612/how-do-i-avoid-generating-json-when-serializing-a-value-that-is-null-or-a-defaul
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "de_header_map")]
    #[serde(serialize_with = "se_header_map")]
    pub headers: Option<HeaderMap>,
}

impl ResponseErrorRec {
    pub fn new(
        test: bool,
        status: u16,
        query: &str,
        headers: HeaderMap,
        body: &str, // Assumed to be json object: "{ \"code\": -1121, \"msg\": \"string message\" }"
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

        let h = if headers.is_empty() {
            None
        } else {
            Some(headers)
        };
        Self {
            test,
            status,
            query: query.to_string(),
            code: code_msg.code,
            msg: code_msg.msg,
            headers: h,
        }
    }
}

impl Display for ResponseErrorRec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        trace!("Display::rer: {:#?}", self);
        write!(
            f,
            "ResponseErrorRec: test={} status={} code={} msg={} query={}",
            self.test, self.status, self.code, self.msg, self.query
        )
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
    // println!("timestamp_ms_to_secs_nsecs: + timestamp_ms={}", timestamp_ms);
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
            // println!("timestamp_ms_to_secs_nsecs: adjusted   timestamp_ms={} secs={} millis={}", timestamp_ms, secs, millis);
        } else {
            // millis is 0 and secs is correct as is.
            // println!("timestamp_ms_to_secs_nsecs: unadjusted timestamp_ms={} secs={} millis={}", timestamp_ms, secs, millis);
        }

        millis
    } else {
        // This actually caused clippy to output "unnecessarary `let` binding"
        // but for I want to be able to have the pritnln and I've found that
        // allowing unnecessary_cast suppresses the warning.
        #[allow(clippy::unnecessary_cast)]
        let millis = (timestamp_ms % 1000) as u32;
        //println!("timestamp_ms_to_secs_nsecs: unadjusted timestamp_ms={} secs={} millis={}", timestamp_ms, secs, millis);

        millis
    };

    let nsecs = ms * 1_000_000u32;

    // println!("timestamp_ms_to_secs_nsecs: - timestamp_ms={} secs={} nsecs={}", timestamp_ms, secs, nsecs);
    (secs, nsecs)
}

pub fn time_ms_to_utc(timestamp_ms: i64) -> DateTime<Utc> {
    let (secs, nsecs) = timestamp_ms_to_secs_nsecs(timestamp_ms);
    let naive_datetime = NaiveDateTime::from_timestamp(secs, nsecs);
    DateTime::from_utc(naive_datetime, Utc)
}

#[allow(unused)]
// Untested and not used atm.
pub fn time_ms_utc_to_naive_local(timestamp_ms: i64) -> NaiveDateTime {
    let (secs, nsecs) = timestamp_ms_to_secs_nsecs(timestamp_ms);
    NaiveDateTime::from_timestamp(secs, nsecs)
}

pub fn time_ms_to_utc_string(time_ms: i64) -> String {
    time_ms_to_utc(time_ms).to_rfc3339_opts(SecondsFormat::Millis, false)
}

pub fn utc_now_to_time_ms() -> i64 {
    (Utc::now().timestamp_nanos() + 500_000) / 1_000_000
}

pub fn utc_to_time_ms(date_time: &DateTime<Utc>) -> i64 {
    (date_time.timestamp_nanos() + 500_000) / 1_000_000
}

pub fn fo_to_time_ms(date_time: &DateTime<FixedOffset>) -> i64 {
    (date_time.timestamp_nanos() + 500_000) / 1_000_000
}

pub enum TzMassaging {
    CondAddTzUtc,
    HasTz,
    LocalTz,
}

///! DateTime string converted to utc time_ms with either T or Space seperator
pub fn dt_str_to_utc_time_ms(
    dt_str: &str,
    tz_massaging: TzMassaging,
) -> Result<i64, Box<dyn std::error::Error>> {
    pub fn dt_str_with_fmt_str_to_utc_time_ms(
        dt_str: &str,
        fmt_str: &str,
        tz_massaging: TzMassaging,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let dt_str = dt_str.trim();
        match tz_massaging {
            TzMassaging::HasTz => {
                let fs = format!("{fmt_str}%#z");
                let dtfo = DateTime::parse_from_str(dt_str, &fs)?;
                Ok(fo_to_time_ms(&dtfo))
            }
            TzMassaging::CondAddTzUtc => {
                let fs = format!("{fmt_str}%#z");

                // If there is a '+' then there "must be" a time zone
                let has_pos_tz = dt_str.matches('+').count() > 0;

                // If there is a '-' after the "year" then there must be a time zone
                let mut rmtchr = dt_str.rmatch_indices('-');
                let first_rmatch = rmtchr.next();
                let has_neg_tz = if let Some((idx, _s)) = first_rmatch {
                    // If there is a '-' after index 7 then assume there is a negative time zone
                    //     2020-01-01T...
                    //     01234567
                    idx > 7
                } else {
                    // No numeric timezone
                    false
                };

                let s = if !has_pos_tz && !has_neg_tz {
                    // Add numeric timezone for UTC
                    format!("{dt_str}+0000")
                } else {
                    // Else there is one so just convert dt_str to String
                    dt_str.to_string()
                };
                let dtfo = DateTime::parse_from_str(&s, &fs)?;
                Ok(fo_to_time_ms(&dtfo))
            }
            TzMassaging::LocalTz => {
                // Convert datetime string to DateTime<Local>
                // from: https://stackoverflow.com/questions/65820170/parsing-a-datetime-string-to-local-time-in-rust-chrono?rq=1
                let ndt = NaiveDateTime::parse_from_str(dt_str, fmt_str)?;
                let ldt = match Local.from_local_datetime(&ndt) {
                    chrono::LocalResult::None => {
                        return Err("No result".into());
                    }
                    chrono::LocalResult::Single(dt) => dt,
                    chrono::LocalResult::Ambiguous(_, _) => {
                        return Err("Ambigious result".into());
                    }
                };

                // Convert from DateTime<Local> to DateTime<Utc> with timezone information
                // from: https://stackoverflow.com/questions/56887881/how-do-i-convert-a-chrono-datetimelocal-instance-to-datetimeutc
                let dt_utc = ldt.with_timezone(&Utc);

                Ok(utc_to_time_ms(&dt_utc))
            }
        }
    }

    let tms = if dt_str.matches('T').count() == 1 {
        dt_str_with_fmt_str_to_utc_time_ms(dt_str, "%Y-%m-%dT%H:%M:%S%.f", tz_massaging)?
    } else {
        dt_str_with_fmt_str_to_utc_time_ms(dt_str, "%Y-%m-%d %H:%M:%S%.f", tz_massaging)?
    };

    Ok(tms)
}

pub fn are_you_sure_stdout_stdin() -> bool {
    print!("Are you sure, type Yes: ");
    if stdout().flush().is_err() {
        return false;
    }

    // Read line include '\n' and check for "Yes\n"
    let mut line = String::new();
    match stdin().read_line(&mut line) {
        Ok(_) => {
            trace!("line: {}", line.trim());
            line.trim().eq("Yes")
        }
        Err(_) => false,
    }
}

pub fn dec_to_money_string(v: Decimal) -> String {
    let v_string = v.round_dp(2).to_string();
    let money_string: String = match Money::from_str(&v_string, iso::USD) {
        Ok(v) => format!("{}", v),
        Err(e) => format!("({} {})", v_string, e),
    };

    money_string
}

pub fn dec_to_separated_string(v: Decimal, dp: u32) -> String {
    let v_string = v.round_dp(dp).to_string();
    let v_f64: f64 = v_string.parse().unwrap();
    v_f64.separated_string_with_fixed_place(dp as usize)
}

pub fn verify_input_files_exist(in_file_paths: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    for f in &*in_file_paths {
        if !Path::new(*f).exists() {
            return Err(format!("{} does not exist", *f).into());
        }
    }

    Ok(())
}

pub fn create_buf_writer(
    out_file_path: &str,
) -> Result<BufWriter<File>, Box<dyn std::error::Error>> {
    let out_file = if let Ok(out_f) = File::create(out_file_path) {
        out_f
    } else {
        return Err(format!("Unable to create {out_file_path}").into());
    };

    Ok(BufWriter::new(out_file))
}

pub fn create_buf_writer_from_path(
    out_file_path: &Path,
) -> Result<BufWriter<File>, Box<dyn std::error::Error>> {
    let out_file = if let Ok(out_f) = File::create(out_file_path) {
        out_f
    } else {
        let fname = out_file_path.as_os_str().to_string_lossy();
        return Err(format!("Unable to create {fname}").into());
    };

    Ok(BufWriter::new(out_file))
}

pub fn create_buf_reader(
    in_file_path: &str,
) -> Result<BufReader<File>, Box<dyn std::error::Error>> {
    let in_file = if let Ok(in_f) = File::open(in_file_path) {
        in_f
    } else {
        return Err(format!("Unable to open {in_file_path}").into());
    };
    Ok(BufReader::new(in_file))
}

#[allow(unused)]
pub fn create_buf_reader_from_path(
    in_file_path: &Path,
) -> Result<BufReader<File>, Box<dyn std::error::Error>> {
    let in_file = if let Ok(in_f) = File::open(in_file_path) {
        in_f
    } else {
        let fname = in_file_path.as_os_str().to_string_lossy();
        return Err(format!("Unable to open {fname}").into());
    };
    Ok(BufReader::new(in_file))
}

#[cfg(test)]
mod test {
    use chrono::SecondsFormat;
    use reqwest::header::HeaderName;
    use rust_decimal_macros::dec;

    use super::*;
    use std::time::Instant;

    #[test]
    fn test_binance_response_error_rec() {
        const RESPONSE_FAILURE_BODY: &str = r#"{"code":-1121,"msg":"Invalid symbol."}"#;

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("yo"),
            HeaderValue::from_static("dude"),
        );

        let response = ResponseErrorRec::new(false, 400, "a_query", headers, RESPONSE_FAILURE_BODY);

        assert_eq!(response.test, false);
        assert_eq!(response.query, "a_query");
        assert_eq!(response.status, 400);
        assert_eq!(response.code, -1121);
        if let Some(headers) = response.headers {
            let val_as_str = headers.get("yo").unwrap().to_str().unwrap();
            assert_eq!("dude", val_as_str);
        }
        assert_eq!(response.msg, "Invalid symbol.");
    }

    #[test]
    fn test_binance_response_error_rec_serialization() {
        const RESPONSE_FAILURE_BODY: &str = r#"{"code":-1121,"msg":"Invalid symbol."}"#;

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("yo"),
            HeaderValue::from_static("dude"),
        );

        let response = ResponseErrorRec::new(false, 400, "a_query", headers, RESPONSE_FAILURE_BODY);
        let response_json = serde_json::to_string(&response).unwrap();
        dbg!(&response_json);
        assert_eq!(&response_json, "{\"test\":false,\"status\":400,\"query\":\"a_query\",\"code\":-1121,\"msg\":\"Invalid symbol.\",\"headers\":{\"yo\":\"dude\"}}");
    }

    #[test]
    fn test_binance_response_error_rec_serialization_empty() {
        const RESPONSE_FAILURE_BODY: &str = r#"{"code":-1121,"msg":"Invalid symbol."}"#;

        let headers = HeaderMap::new();
        let response = ResponseErrorRec::new(false, 400, "a_query", headers, RESPONSE_FAILURE_BODY);
        let response_json = serde_json::to_string(&response).unwrap();
        dbg!(&response_json);
        assert_eq!(&response_json, "{\"test\":false,\"status\":400,\"query\":\"a_query\",\"code\":-1121,\"msg\":\"Invalid symbol.\"}");
    }

    #[test]
    fn test_binance_response_error_rec_bad_body() {
        const RESPONSE_FAILURE_BODY: &str = "An unexpected error";
        let headers = HeaderMap::new();

        let response = ResponseErrorRec::new(false, 505, "a_query", headers, RESPONSE_FAILURE_BODY);

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

    #[test]
    fn test_internal_error() {
        let ie1 = ier_new!(1, "err 1");
        println!("{:#?}", ie1);
        assert_eq!(ie1.code, 1);
        assert_eq!(ie1.line, line!() - 3);
        assert_eq!(ie1.file, file!());
    }

    #[test]
    fn test_dec_to_money_string() {
        assert_eq!(dec_to_money_string(dec!(1.024)), "$1.02");
        assert_eq!(dec_to_money_string(dec!(1.026)), "$1.03");
        assert_eq!(dec_to_money_string(dec!(1000.026)), "$1,000.03");
    }

    #[test]
    fn test_dec_to_separated_string() {
        assert_eq!(dec_to_separated_string(dec!(1.024), 2), "1.02");
        assert_eq!(dec_to_separated_string(dec!(1.026), 2), "1.03");
        assert_eq!(dec_to_separated_string(dec!(1000.026), 2), "1,000.03");
    }

    #[test]
    fn test_dt_str_with_tee_to_utc_time_ms() {
        let str_time_no_ms = "1970-01-01T00:00:00";
        let ts = dt_str_to_utc_time_ms(str_time_no_ms, TzMassaging::CondAddTzUtc)
            .expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 0);

        let str_time_with_ms = "1970-01-01T00:00:00.123";
        let tms = dt_str_to_utc_time_ms(str_time_with_ms, TzMassaging::CondAddTzUtc)
            .expect("Bad time format with milliseconds");
        dbg!(tms);
        assert_eq!(tms, 123);
    }

    #[test]
    fn test_dt_str_with_space_to_utc_time_ms() {
        let str_time_no_ms = "1970-01-01 00:00:00";
        let ts = dt_str_to_utc_time_ms(str_time_no_ms, TzMassaging::CondAddTzUtc)
            .expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 0);

        let str_time_with_ms = "1970-01-01 00:00:00.123";
        let tms = dt_str_to_utc_time_ms(str_time_with_ms, TzMassaging::CondAddTzUtc)
            .expect("Bad time format with milliseconds");
        dbg!(tms);
        assert_eq!(tms, 123);
    }

    #[test]
    fn test_dt_str_with_leading_trailing_spaces_to_utc_time_ms() {
        let str_time_no_ms = " 1970-01-01 00:00:00";
        let ts = dt_str_to_utc_time_ms(str_time_no_ms, TzMassaging::CondAddTzUtc)
            .expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 0);

        let str_time_with_ms = "1970-01-01 00:00:00.123 ";
        let tms = dt_str_to_utc_time_ms(str_time_with_ms, TzMassaging::CondAddTzUtc)
            .expect("Bad time format with milliseconds");
        dbg!(tms);
        assert_eq!(tms, 123);
        let str_time_no_ms = " 1970-01-01T00:00:00  ";
        let ts = dt_str_to_utc_time_ms(str_time_no_ms, TzMassaging::CondAddTzUtc)
            .expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 0);

        let str_time_with_ms = "  1970-01-01T00:00:00.123  ";
        let tms = dt_str_to_utc_time_ms(str_time_with_ms, TzMassaging::CondAddTzUtc)
            .expect("Bad time format with milliseconds");
        dbg!(tms);
        assert_eq!(tms, 123);
    }

    #[test]
    fn test_dt_str_addtzutc_with_utc() {
        let str_time_tz = "1970-01-01 00:00:00+00";
        let ts =
            dt_str_to_utc_time_ms(str_time_tz, TzMassaging::CondAddTzUtc).expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 0);
        let str_time_tz = "1970-01-01T00:00:00.1+00";
        let ts =
            dt_str_to_utc_time_ms(str_time_tz, TzMassaging::CondAddTzUtc).expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 100);

        let str_time_tz = "1970-01-01T00:00:00.123+0000";
        let ts =
            dt_str_to_utc_time_ms(str_time_tz, TzMassaging::CondAddTzUtc).expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 123);

        let str_time_tz = "1970-01-01 00:00:00+00:00";
        let ts =
            dt_str_to_utc_time_ms(str_time_tz, TzMassaging::CondAddTzUtc).expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 0);

        let str_time_tz = "1970-01-01 00:00:00.456+00:00";
        let ts =
            dt_str_to_utc_time_ms(str_time_tz, TzMassaging::CondAddTzUtc).expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 456);
    }

    #[test]
    fn test_dt_str_with_tz_to_utc_time_ms() {
        let str_time_no_ms = "1970-01-01T00:00:00+0000";
        let ts =
            dt_str_to_utc_time_ms(str_time_no_ms, TzMassaging::HasTz).expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 0);

        let str_time_with_ms = "1970-01-01T00:00:00.123+00:00";
        let tms = dt_str_to_utc_time_ms(str_time_with_ms, TzMassaging::HasTz)
            .expect("Bad time format with milliseconds");
        dbg!(tms);
        assert_eq!(tms, 123);
    }

    #[test]
    fn test_dt_str_both_hastz() {
        let str_time_tz = "1970-01-01T00:00:00+0000";
        let ts = dt_str_to_utc_time_ms(str_time_tz, TzMassaging::HasTz).expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 0);

        let str_time_pst = "1969-12-31T16:00:00-0800";
        let ts_pst = dt_str_to_utc_time_ms(str_time_pst, TzMassaging::HasTz)
            .expect("Bad time format with milliseconds");
        dbg!(ts_pst);
        assert_eq!(ts, ts_pst);
    }

    #[test]
    fn test_dt_str_addtzutc_hastz() {
        let str_time_tz = "1970-01-01T00:00:00";
        let ts =
            dt_str_to_utc_time_ms(str_time_tz, TzMassaging::CondAddTzUtc).expect("Bad time format");
        dbg!(ts);
        assert_eq!(ts, 0);

        let str_time_pst = "1969-12-31T16:00:00-0800";
        let ts_pst = dt_str_to_utc_time_ms(str_time_pst, TzMassaging::HasTz)
            .expect("Bad time format with milliseconds");
        dbg!(ts_pst);
        assert_eq!(ts, ts_pst);
    }

    #[test]
    fn test_dt_str_to_utc_time_ms_using_localtz() {
        let lt_str = "1970-01-01T00:00:00";
        let ndt: NaiveDateTime = lt_str.parse().unwrap();
        dbg!(ndt);
        let ldt = Local.from_local_datetime(&ndt).unwrap();
        dbg!(ldt);
        let ldt_offset = ldt.offset();
        dbg!(ldt_offset);
        let ldt_offset_secs = ldt_offset.local_minus_utc();
        dbg!(ldt_offset_secs);

        let tms = dt_str_to_utc_time_ms(lt_str, TzMassaging::LocalTz).expect("Bad time format");
        dbg!(tms);

        assert_eq!(tms, ldt_offset_secs as i64 * -1000);
    }

    #[test]
    fn test_time_ms_to_utc() {
        let dt = time_ms_to_utc(0i64);
        assert_eq!(
            dt.to_rfc3339_opts(SecondsFormat::Millis, true),
            "1970-01-01T00:00:00.000Z"
        );
        assert_eq!(
            dt.to_rfc3339_opts(SecondsFormat::Millis, false),
            "1970-01-01T00:00:00.000+00:00"
        );
    }

    #[test]
    fn test_time_ms_to_utc_string() {
        let dt = time_ms_to_utc_string(0i64);
        assert_eq!(dt, "1970-01-01T00:00:00.000+00:00");
    }

    #[test]
    fn test_parse_from_str() {
        let s = format!("1970-01-01T00:00:00.000{}", "Z");
        let dt = match DateTime::parse_from_rfc3339(&s) {
            Ok(v) => v,
            Err(e) => panic!("shit {e}"),
        };
        println!("test_parse_from_str: {dt}");
    }
}
