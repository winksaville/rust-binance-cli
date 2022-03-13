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

pub fn dec_to_string_or_empty(d: Option<Decimal>) -> String {
    if let Some(q) = d {
        format!("{}", q)
    } else {
        "".to_owned()
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
    use reqwest::header::HeaderName;
    use rust_decimal_macros::dec;

    use super::*;

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
}
