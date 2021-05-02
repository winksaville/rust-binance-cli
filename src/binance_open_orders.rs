use log::trace;
use serde::{Deserialize, Serialize};

use crate::{
    binance_context::BinanceContext,
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    common::utc_now_to_time_ms,
    common::{BinanceError, ResponseErrorRec},
    de_string_or_number::{de_string_or_number_to_f64, de_string_or_number_to_i64},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrderRec {
    pub symbol: String,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_id: i64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64, //Unless OCO, the value will always be -1
    pub client_order_id: String,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub orig_qty: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub executed_qty: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub cummulative_quote_qty: f64,
    pub status: String,        // "NEW" change to enum
    pub time_in_force: String, // "GTC", change to enum
    #[serde(rename = "type")]
    pub order_type: String, // "LIMIT", enum
    pub side: String,          // "BUY", enum
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub stop_price: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub iceberg_qty: f64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub time: i64, // Consider being chrono::Utc or creating a Utc
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub update_time: i64, // Utc
    is_working: bool,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub orig_quote_order_qty: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrders {
    orders: Vec<OpenOrderRec>,
}

pub async fn get_open_orders(
    ctx: &BinanceContext,
    symbol: &str,
) -> Result<OpenOrders, Box<dyn std::error::Error>> {
    let sig_key = ctx.opts.secret_key.as_bytes();
    let api_key = ctx.opts.api_key.as_bytes();

    let mut params = vec![("recvWindow", "5000")];
    if !symbol.is_empty() {
        params.push(("symbol", symbol));
    }

    let ts_string: String = format!("{}", utc_now_to_time_ms());
    params.push(("timestamp", ts_string.as_str()));

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data is qs and query as body
    let signature = binance_signature(&sig_key, &query, &[]);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("query_string={}", &query_string);

    let mut url = ctx.make_url("api", "/api/v3/openOrders?");
    url.push_str(&query_string);
    trace!("get_open_orders: url={}", url);

    // Build request
    let client = reqwest::Client::builder();
    let req_builder = client
        //.proxy(reqwest::Proxy::https("http://localhost:8080")?)
        .build()?
        .get(url)
        .header("X-MBX-APIKEY", api_key);
    trace!("req_builder={:#?}", req_builder);

    // Send and get response
    let response = req_builder.send().await?;
    trace!("response={:#?}", &response);

    let response_status = response.status();
    let response_body = response.text().await?;

    // Log the response
    let result = if response_status == 200 {
        trace!("response_body={}", response_body);
        let orders: Vec<OpenOrderRec> = serde_json::from_str(&response_body)?;

        let open_orders = OpenOrders { orders };
        trace!("open_orders={:?}", open_orders);

        Ok(open_orders)
    } else {
        let rer = ResponseErrorRec::new(
            false,
            response_status.as_u16(),
            &query_string,
            &response_body,
        );
        let binance_error_response = BinanceError::Response(rer);

        trace!(
            "{}",
            format!(
                "binance_market_order_or_test: symbol={} order_resp={:#?}",
                symbol, &binance_error_response
            )
        );

        Err(binance_error_response.into())
    };

    result
}

// TODO: Add some tests
