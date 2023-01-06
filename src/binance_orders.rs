use chrono::{DateTime, Utc};
use log::trace;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use rust_decimal::prelude::*;
use time_ms_conversions::{utc_now_to_time_ms, utc_to_time_ms};

use crate::{
    binance_order_response::TradeResponse,
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    common::{get_req_get_response, ResponseErrorRec, Side},
    configuration::Configuration,
    de_string_or_number::de_string_or_number_to_i64,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRec {
    pub symbol: String,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_id: i64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64, //Unless OCO, the value will always be -1
    pub client_order_id: String,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: String,        // "NEW" change to enum
    pub time_in_force: String, // "GTC", change to enum
    #[serde(rename = "type")]
    pub order_type: String, // "LIMIT", enum
    pub side: String,          // "BUY", enum
    pub stop_price: Decimal,
    pub iceberg_qty: Decimal,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub time: i64, // Consider being chrono::Utc or creating a Utc
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub update_time: i64, // Utc
    is_working: bool,
    pub orig_quote_order_qty: Decimal,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Orders {
    pub orders: Vec<OrderRec>,
}

impl Orders {
    pub fn sum_buy_orders(&self) -> Decimal {
        let sum_buy_orders: Decimal = self
            .orders
            .iter()
            .map(|x| {
                if x.side.eq(Side::BUY.into()) {
                    trace!("x.orig_qty: {}", x.orig_qty);
                    x.orig_qty
                } else {
                    dec!(0)
                }
            })
            .sum();

        sum_buy_orders
    }
}

async fn orders_get_req_and_response(
    config: &Configuration,
    cmd: &str,
    mut params: Vec<(&str, &str)>,
) -> Result<Orders, Box<dyn std::error::Error>> {
    let api_key = config.keys.get_ak_or_err()?;
    let secret_key = &config.keys.get_sk_vec_u8_or_err()?;

    params.push(("recvWindow", "5000"));

    let ts_string: String = format!("{}", utc_now_to_time_ms());
    params.push(("timestamp", ts_string.as_str()));

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data is qs and query as body
    let signature = binance_signature(secret_key, &query, &[]);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("query_string={}", &query_string);

    let mut url = config.make_url("api", &format!("/api/v3/{cmd}?"));
    url.push_str(&query_string);
    trace!("get_open_orders: url={}", url);

    let response = get_req_get_response(api_key, &url).await?;
    trace!("response={:#?}", response);
    let response_headers = response.headers().clone();
    let response_status = response.status();
    let response_body = response.text().await?;

    // Log the response
    #[allow(clippy::let_and_return)]
    let result = if response_status == 200 {
        trace!("response_body={}", response_body);
        let orders: Vec<OrderRec> = serde_json::from_str(&response_body)?;

        let open_orders = Orders { orders };
        trace!("open_orders={:?}", open_orders);

        Ok(open_orders)
    } else {
        let rer = ResponseErrorRec::new(
            false,
            response_status.as_u16(),
            &query_string,
            response_headers,
            &response_body,
        );
        let binance_error_response = TradeResponse::FailureResponse(rer);

        trace!(
            "{}",
            format!(
                "binance_market_order_or_test: order_resp={:#?}",
                &binance_error_response
            )
        );

        Err(binance_error_response.into())
    };

    result
}

pub async fn get_all_orders(
    config: &Configuration,
    symbol: &str,
    order_id: Option<u64>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
    limit: Option<i64>,
) -> Result<Orders, Box<dyn std::error::Error>> {
    let mut params: Vec<(&str, &str)> = Vec::new();

    if !symbol.is_empty() {
        params.push(("symbol", symbol));
    }

    let id_string: String;
    if let Some(id) = order_id {
        id_string = id.to_string();
        params.push(("orderId", &id_string));
    }

    let stms_string: String;
    if let Some(sdt) = start_date_time {
        stms_string = utc_to_time_ms(&sdt).to_string();
        params.push(("startTime", &stms_string));
    }

    let etms_string: String;
    if let Some(edt) = end_date_time {
        etms_string = utc_to_time_ms(&edt).to_string();
        params.push(("endTime", &etms_string));
    }

    let limit_string: String;
    if let Some(l) = limit {
        limit_string = l.to_string();
        params.push(("limit", &limit_string));
    }

    orders_get_req_and_response(config, "allOrders", params).await
}

pub async fn get_open_orders(
    config: &Configuration,
    symbol: &str,
) -> Result<Orders, Box<dyn std::error::Error>> {
    let mut params: Vec<(&str, &str)> = Vec::new();

    if !symbol.is_empty() {
        params.push(("symbol", symbol));
    }

    orders_get_req_and_response(config, "openOrders", params).await
}

// TODO: Add some tests
