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

// Example response_body
//{
//    "insertTime":1620402126963,
//    "amount":0.14496705,
//    "creator":null,
//    "address":"1AALZKmCQzLrmfgRLu5ENu99ierxnnYfEs",
//    "addressTag":"",
//    "txId":"44893c41090adb75053221f69c7dbd0e8f09b7a9c1936cc108b72f8ff830fdd4",
//    "asset":"BTC",
//    "status":1},
//}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositRec {
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub insert_time: i64,
    pub amount: Decimal,
    pub address: String,
    pub asset: String,
    #[serde(deserialize_with = "de_string_or_number_to_i32")]
    pub status: i32,

    // Always Some
    pub tx_id: Option<String>,

    // Always Some but an empty string
    pub address_tag: Option<String>,

    // Always None
    pub creator: Option<String>,
}

// Example response_body
//{
//    "amount":15.98520052,
//    "transactionFee":0.013,
//    "address":"0xd989942D49De163A54273af6De971Ba80308D5cD",
//    "withdrawOrderId":null,
//    "addressTag":null,
//    "txId":"0x7972ee7e7e83d3fd356c85f0ac234c2bb95b5a668fb95bd6023c429b77516915",
//    "id":"5f28c73fce8b40f784c07d8655bab732",
//    "asset":"ETH",
//    "applyTime":1620970959336,
//    "status":6,
//    "network":"ETH"
//}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawRec {
    pub amount: Decimal,
    pub transaction_fee: Decimal,
    pub address: String,
    pub id: String,
    pub asset: String,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub apply_time: i64,
    #[serde(deserialize_with = "de_string_or_number_to_i32")]
    pub status: i32,
    pub network: String,

    // One None the rest were Some
    pub tx_id: Option<String>,

    // Always None
    pub address_tag: Option<String>,

    // Always None
    pub withdraw_order_id: Option<String>,
}

// {
//     "orderId":"6c2ff984890145fdac2b7160299062f0",
//     "paymentAccount": "4a992541-c12d-4cca-bbd6-df637f801526",
//     "paymentChannel": "PRIMETRUST",
//     "paymentMethod": "WIRE_INTERNATIONAL",
//     "orderStatus": "Processing",
//     "amount": "65",
//     "transactionFee": "20",
//     "platformFee": "0"
// }
// {
//     "orderId":"a67cf36288c5408e94ec4436cb8357b7",
//     "paymentChannel":"PRIMETRUST",
//     "paymentMethod":"WIRE",
//     "orderStatus":"Successful",
//     "amount":"450",
//     "transactionFee":"0",
//     "platformFee":"0"
// }

// {
//     "orderId":"70c95675dfd645ae93cb5951b281de20",
//     "paymentAccount":"24c58fbc-c203-400b-8885-8162c1c7f11b",
//     "paymentChannel":"PRIMETRUST",
//     "paymentMethod":"CREDIT_CARD",
//     "orderStatus":"Successful",
//     "amount":"95.5",
//     "transactionFee":"4.5",
//     "platformFee":"0"
// }

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetLogRec {
    order_id: String,
    payment_account: Option<String>,
    payment_method: String,
    order_status: String,
    amount: Decimal,
    transaction_fee: Decimal,
    platform_fee: Decimal,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Histories {
    pub deposit_list: Option<Vec<DepositRec>>,
    pub withdraw_list: Option<Vec<WithdrawRec>>,
    pub asset_log_record_list: Option<Vec<AssetLogRec>>,
    pub success: Option<bool>,
}

/// TODO: Consider making generic or process macro as is
/// copy/paste fo orders_get_req_response
async fn history_get_req_and_response(
    config: &Configuration,
    full_path: &str,
    mut params: Vec<(&str, &str)>,
) -> Result<Histories, Box<dyn std::error::Error>> {
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
    trace!(
        "history_get_req_and_response: query_string={}",
        &query_string
    );

    let mut url = config.make_url("api", &format!("{}?", full_path));
    url.push_str(&query_string);
    trace!("history_get_req_and_response: url={}", url);

    let response = get_req_get_response(&api_key, &url).await?;
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

        Err(ier_new!(8, &rer.to_string()).into())
    };

    result
}

pub async fn get_history(
    config: &Configuration,
    full_path: &str,
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

    history_get_req_and_response(config, full_path, params).await
}

pub async fn get_deposit_history(
    config: &Configuration,
    asset: Option<&str>,
    status: Option<u32>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
) -> Result<Vec<DepositRec>, Box<dyn std::error::Error>> {
    let histories = get_history(
        config,
        "/wapi/v3/depositHistory.html",
        asset,
        status,
        start_date_time,
        end_date_time,
    )
    .await?;

    if let Some(deposit_list) = histories.deposit_list {
        Ok(deposit_list)
    } else {
        Err(ier_new!(
            7,
            "Should not happen; expected depositList, but there was None"
        )
        .into())
    }
}

pub async fn get_withdraw_history(
    config: &Configuration,
    asset: Option<&str>,
    status: Option<u32>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
) -> Result<Vec<WithdrawRec>, Box<dyn std::error::Error>> {
    let histories = get_history(
        config,
        "/wapi/v3/withdrawHistory.html",
        asset,
        status,
        start_date_time,
        end_date_time,
    )
    .await?;

    if let Some(withdraw_list) = histories.withdraw_list {
        Ok(withdraw_list)
    } else {
        Err(ier_new!(
            7,
            "Should not happen; expected withdrawList, but there was None"
        )
        .into())
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn get_fiat_currency_history(
    config: &Configuration,
    full_path: &str,
    fiat_currency: Option<&str>,
    order_id: Option<&str>,
    offset: Option<i64>,
    payment_channel: Option<&str>,
    payment_method: Option<&str>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
) -> Result<Vec<AssetLogRec>, Box<dyn std::error::Error>> {
    let mut params: Vec<(&str, &str)> = Vec::new();

    if let Some(fc) = fiat_currency {
        params.push(("fiatCurrency", fc));
    }

    if let Some(ois) = order_id {
        params.push(("orderId", ois));
    }

    let offset_string: String;
    if let Some(o) = offset {
        offset_string = o.to_string();
        params.push(("offset", &offset_string));
    }

    if let Some(pcs) = payment_channel {
        params.push(("paymentChannel", pcs));
    }

    if let Some(pm) = payment_method {
        params.push(("paymentMethod", pm));
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

    let histories = history_get_req_and_response(config, full_path, params).await?;

    if let Some(alrs) = histories.asset_log_record_list {
        Ok(alrs)
    } else {
        Err(ier_new!(
            7,
            "Should not happen; expected assetLogRecordList, but there was None"
        )
        .into())
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn get_fiat_currency_deposit_history(
    config: &Configuration,
    fiat_currency: Option<&str>,
    order_id: Option<&str>,
    offset: Option<i64>,
    payment_channel: Option<&str>,
    payment_method: Option<&str>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
) -> Result<Vec<AssetLogRec>, Box<dyn std::error::Error>> {
    Ok(get_fiat_currency_history(
        config,
        "/sapi/v1/fiatpayment/query/deposit/history",
        fiat_currency,
        order_id,
        offset,
        payment_channel,
        payment_method,
        start_date_time,
        end_date_time,
    )
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn get_fiat_currency_withdraw_history(
    config: &Configuration,
    fiat_currency: Option<&str>,
    order_id: Option<&str>,
    offset: Option<i64>,
    payment_channel: Option<&str>,
    payment_method: Option<&str>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
) -> Result<Vec<AssetLogRec>, Box<dyn std::error::Error>> {
    Ok(get_fiat_currency_history(
        config,
        "/sapi/v1/fiatpayment/query/withdraw/history",
        fiat_currency,
        order_id,
        offset,
        payment_channel,
        payment_method,
        start_date_time,
        end_date_time,
    )
    .await?)
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
