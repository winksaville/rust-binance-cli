use chrono::{DateTime, Utc};
use log::trace;
use serde::{Deserialize, Serialize};

use rust_decimal::prelude::*;

use crate::{
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    common::{get_req_get_response, InternalErrorRec, ResponseErrorRec},
    common::{utc_now_to_time_ms, utc_to_time_ms},
    configuration::Configuration,
    de_string_or_number::{de_string_or_number_to_i32, de_string_or_number_to_i64},
    ier_new,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRec {
    pub asset: String,
    pub amount: Decimal,
    #[serde(default)]
    pub transaction_fee: Decimal,
    #[serde(default, deserialize_with = "de_string_or_number_to_i64")]
    pub insert_time: i64,
    #[serde(default, deserialize_with = "de_string_or_number_to_i64")]
    pub apply_time: i64,
    //pub creator: Option<String>,
    pub address: String,
    pub tx_id: Option<String>,
    pub id: Option<String>,
    pub withdraw_order_id: Option<String>,
    pub address_tag: Option<String>,
    #[serde(deserialize_with = "de_string_or_number_to_i32")]
    pub status: i32,
    pub network: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Histories {
    pub deposit_list: Option<Vec<HistoryRec>>,
    pub withdraw_list: Option<Vec<HistoryRec>>,
    pub success: bool,
}

/// TODO: Consider making generic or process macro as is
/// copy/paste fo orders_get_req_response
async fn history_get_req_and_response(
    config: &Configuration,
    cmd: &str,
    mut params: Vec<(&str, &str)>,
) -> Result<Histories, Box<dyn std::error::Error>> {
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
    trace!(
        "history_get_req_and_response: query_string={}",
        &query_string
    );

    let mut url = config.make_url("api", &format!("/wapi/v3/{}?", cmd));
    url.push_str(&query_string);
    trace!("history_get_req_and_response: url={}", url);

    let response = get_req_get_response(api_key, &url).await?;
    trace!("history_get_req_and_response: response={:#?}", response);
    let response_status = response.status();
    let response_body = response.text().await?;

    // Process the response
    let result = if response_status == 200 {
        trace!(
            "history_get_req_and_response: response_body={}",
            response_body
        );
        let histories: Histories = serde_json::from_str(&response_body)?;

        //let deposits = Deposits { deposit_list: deposit_rec };
        trace!("history_get_req_and_response: deposits={:?}", histories);

        Ok(histories)
    } else {
        let rer = ResponseErrorRec::new(
            false,
            response_status.as_u16(),
            &query_string,
            &response_body,
        );
        trace!(
            "{}",
            format!("history_get_req_and_response: ResponseErrRec={:#?}", &rer)
        );

        let ier: InternalErrorRec = ier_new!(8, &rer.to_string());
        Err(ier.to_string().into())
    };

    result
}

pub async fn get_history(
    config: &Configuration,
    url_page: &str,
    asset: Option<&str>,
    status: Option<u32>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
) -> Result<Histories, Box<dyn std::error::Error>> {
    let mut params: Vec<(&str, &str)> = Vec::new();

    if let Some(asset) = asset {
        params.push(("asset", asset));
    }

    let status_string: String;
    if let Some(s) = status {
        status_string = s.to_string();
        params.push(("status", &status_string));
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

    history_get_req_and_response(config, url_page, params).await
}

pub async fn get_deposit_history(
    config: &Configuration,
    asset: Option<&str>,
    status: Option<u32>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
) -> Result<Vec<HistoryRec>, Box<dyn std::error::Error>> {
    let histories = get_history(
        config,
        "depositHistory.html",
        asset,
        status,
        start_date_time,
        end_date_time,
    )
    .await?;

    if let Some(deposit_list) = histories.deposit_list {
        Ok(deposit_list)
    } else {
        let ier = ier_new!(
            7,
            "Should not happen; expected deposit_list but it was None: "
        );
        return Err(format!("{}", ier).into());
    }
}

pub async fn get_withdraw_history(
    config: &Configuration,
    asset: Option<&str>,
    status: Option<u32>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
) -> Result<Vec<HistoryRec>, Box<dyn std::error::Error>> {
    let histories = get_history(
        config,
        "withdrawHistory.html",
        asset,
        status,
        start_date_time,
        end_date_time,
    )
    .await?;

    if let Some(withdraw_list) = histories.withdraw_list {
        Ok(withdraw_list)
    } else {
        let ier = ier_new!(
            7,
            "Should not happen; expected withdraw_list but it was None: "
        );
        return Err(format!("{}", ier).into());
    }
}

#[cfg(test)]
mod test {
    //use super::*;
    //use rust_decimal_macros::dec;

    //const HISTORY_REC: &str = r#"{
    //    "symbol": "ETHUSD",
    //    "id": 7203346,
    //    "orderId": 226649654,
    //    "orderListId": -1,
    //    "price": 4260.0000,
    //    "qty": 1.85538000,
    //    "quoteQty": 7903.9188,
    //    "commission": 0.00185538,
    //    "commissionAsset": "ETH",
    //    "time": 1620832760334,
    //    "isBuyer": true,
    //    "isMaker": true,
    //    "isBestMatch": true
    //}"#;

    #[test]
    fn test_history_rec() {
        //let tr: DepositRec = match serde_json::from_str(HISTORY_REC) {
        //    Ok(info) => info,
        //    Err(e) => panic!("Error processing response: e={}", e),
        //};
        //assert_eq!("ETHUSD", tr.symbol);
        //assert_eq!(7203346, tr.id);
        //assert_eq!(226649654, tr.order_id);
        //assert_eq!(dec!(4260), tr.price);
        //assert_eq!(dec!(1.85538), tr.qty);
        //assert_eq!(dec!(7903.9188), tr.quote_qty);
        //assert_eq!(dec!(0.00185538), tr.commission);
        //assert_eq!("ETH", tr.commission_asset);
        //assert_eq!(1620832760334, tr.time);
        //assert_eq!(true, tr.is_buyer);
        //assert_eq!(true, tr.is_maker);
        //assert_eq!(true, tr.is_best_match);
    }
}
