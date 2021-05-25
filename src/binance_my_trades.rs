use chrono::{DateTime, Utc};
use log::trace;
use serde::{Deserialize, Serialize};

use rust_decimal::prelude::*;

use crate::{
    binance_order_response::TradeResponse,
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    common::{get_req_get_response, ResponseErrorRec},
    common::{utc_now_to_time_ms, utc_to_time_ms},
    configuration::Configuration,
    de_string_or_number::de_string_or_number_to_i64,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeRec {
    pub symbol: String,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub id: i64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_id: i64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64, //Unless OCO, the value will always be -1
    pub price: Decimal,
    pub qty: Decimal,
    pub quote_qty: Decimal,
    pub commission: Decimal,
    pub commission_asset: String,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub time: i64, // Consider being chrono::Utc or creating a Utc
    is_buyer: bool,
    is_maker: bool,
    is_best_match: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trades {
    pub trades: Vec<TradeRec>,
}

/// TODO: Consider making generic or process macro as is
/// copy/paste fo orders_get_req_response
async fn trades_get_req_and_response(
    config: &Configuration,
    cmd: &str,
    mut params: Vec<(&str, &str)>,
) -> Result<Trades, Box<dyn std::error::Error>> {
    let secret_key = config.secret_key.as_bytes();
    let api_key = &config.api_key;

    params.push(("recvWindow", "5000"));

    let ts_string: String = format!("{}", utc_now_to_time_ms());
    params.push(("timestamp", ts_string.as_str()));

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data is qs and query as body
    let signature = binance_signature(&secret_key, &query, &[]);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("query_string={}", &query_string);

    let mut url = config.make_url("api", &format!("/api/v3/{}?", cmd));
    url.push_str(&query_string);
    trace!("get_open_orders: url={}", url);

    let response = get_req_get_response(api_key, &url).await?;
    trace!("response={:#?}", response);
    let response_status = response.status();
    let response_body = response.text().await?;

    // Log the response
    let result = if response_status == 200 {
        trace!("response_body={}", response_body);
        let trade_rec: Vec<TradeRec> = serde_json::from_str(&response_body)?;

        let trades = Trades { trades: trade_rec };
        trace!("trades={:?}", trades);

        Ok(trades)
    } else {
        let rer = ResponseErrorRec::new(
            false,
            response_status.as_u16(),
            &query_string,
            &response_body,
        );
        let binance_error_response = TradeResponse::FailureResponse(rer);

        trace!(
            "{}",
            format!(
                "my_trades: binance_error_response={:#?}",
                &binance_error_response
            )
        );

        Err(binance_error_response.into())
    };

    result
}

pub async fn get_my_trades(
    config: &Configuration,
    symbol: &str,
    from_id: Option<u64>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
    limit: Option<i64>,
) -> Result<Trades, Box<dyn std::error::Error>> {
    let mut params: Vec<(&str, &str)> = Vec::new();

    if !symbol.is_empty() {
        params.push(("symbol", symbol));
    }

    let fid_string: String;
    if let Some(fid) = from_id {
        fid_string = fid.to_string();
        params.push(("fromId", &fid_string));
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

    trades_get_req_and_response(config, "myTrades", params).await
}

// TODO: Add some tests
