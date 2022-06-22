use chrono::{DateTime, Utc};
use log::trace;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use rust_decimal::prelude::*;
use time_ms_conversions::{utc_now_to_time_ms, utc_to_time_ms};

use crate::{
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    common::{get_req_get_response, InternalErrorRec, ResponseErrorRec},
    configuration::Configuration,
    de_string_or_number::de_string_or_number_to_i64,
    ier_new,
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
    pub is_buyer: bool,
    pub is_maker: bool,
    pub is_best_match: bool,
}

#[allow(unused)]
impl TradeRec {
    pub fn is_buyer_factor(&self) -> Decimal {
        if self.is_buyer {
            dec!(1)
        } else {
            dec!(-1)
        }
    }
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

    let mut url = config.make_url("api", &format!("/api/v3/{}?", cmd));
    url.push_str(&query_string);
    trace!("get_open_orders: url={}", url);

    let response = get_req_get_response(api_key, &url).await?;
    trace!("response={:#?}", response);
    let response_headers = response.headers().clone();
    let response_status = response.status();
    let response_body = response.text().await?;

    // Process the response
    #[allow(clippy::let_and_return)]
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
            response_headers,
            &response_body,
        );
        trace!(
            "{}",
            format!("trades_get_req_and_response: ResponseErrRec={:#?}", &rer)
        );

        Err(ier_new!(8, &rer.to_string()).into())
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

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

    const TRADE_REC_1: &str = r#"{
        "symbol": "ETHUSD",
        "id": 7203346,
        "orderId": 226649654,
        "orderListId": -1,
        "price": 4260.0000,
        "qty": 1.85538000,
        "quoteQty": 7903.9188,
        "commission": 0.00185538,
        "commissionAsset": "ETH",
        "time": 1620832760334,
        "isBuyer": true,
        "isMaker": true,
        "isBestMatch": true
    }"#;

    #[test]
    fn test_trade_rec() {
        let tr: TradeRec = match serde_json::from_str(TRADE_REC_1) {
            Ok(info) => info,
            Err(e) => panic!("Error processing response: e={}", e),
        };
        assert_eq!("ETHUSD", tr.symbol);
        assert_eq!(7203346, tr.id);
        assert_eq!(226649654, tr.order_id);
        assert_eq!(dec!(4260), tr.price);
        assert_eq!(dec!(1.85538), tr.qty);
        assert_eq!(dec!(7903.9188), tr.quote_qty);
        assert_eq!(dec!(0.00185538), tr.commission);
        assert_eq!("ETH", tr.commission_asset);
        assert_eq!(1620832760334, tr.time);
        assert_eq!(true, tr.is_buyer);
        assert_eq!(true, tr.is_maker);
        assert_eq!(true, tr.is_best_match);
    }

    #[test]
    fn test_trade_rec_is_buyer_factor() {
        let mut tr: TradeRec = match serde_json::from_str(TRADE_REC_1) {
            Ok(info) => info,
            Err(e) => panic!("Error processing response: e={}", e),
        };
        assert_eq!(dec!(1), tr.is_buyer_factor());
        tr.is_buyer = false;
        assert_eq!(dec!(-1), tr.is_buyer_factor());
    }
}
