use serde::{Deserialize, Serialize};

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use crate::de_string_or_number::{de_string_or_number_to_i64, de_string_or_number_to_u64};

use crate::common::{BinanceError, OrderType};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    pub commission_asset: String,
    pub commission: Decimal,
    pub price: Decimal,
    pub qty: Decimal,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub trade_id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AckTradeResponseRec {
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub query: String,
    symbol: String,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub order_id: u64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64,
    pub client_order_id: String,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub transact_time: u64,
}

impl Default for AckTradeResponseRec {
    fn default() -> AckTradeResponseRec {
        AckTradeResponseRec {
            test: false,
            query: "".to_string(),
            symbol: "".to_string(),
            order_id: 0,
            order_list_id: -1,
            client_order_id: "".to_string(),
            transact_time: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultTradeResponseRec {
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub query: String,
    symbol: String,
    pub order_id: u64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64,
    pub client_order_id: String,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub transact_time: u64,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: String,        // add enum
    pub time_in_force: String, // add enum TimeInForce
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: String, // add enum Side (it's currently defined in main.rs)
    #[serde(default)]
    pub value_usd: Decimal,
    #[serde(default)]
    pub commission_usd: Decimal,
}

impl Default for ResultTradeResponseRec {
    fn default() -> ResultTradeResponseRec {
        ResultTradeResponseRec {
            test: false,
            query: "".to_string(),
            symbol: "".to_string(),
            order_id: 0,
            order_list_id: -1,
            client_order_id: "".to_string(),
            transact_time: 0,
            price: dec!(0),
            orig_qty: dec!(0),
            executed_qty: dec!(0),
            cummulative_quote_qty: dec!(0),
            status: "".to_string(),
            time_in_force: "".to_string(),
            order_type: OrderType::MARKET,
            side: "".to_string(),
            value_usd: dec!(0),
            commission_usd: dec!(0),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullTradeResponseRec {
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub query: String,
    symbol: String,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub order_id: u64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64,
    pub client_order_id: String,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub transact_time: u64,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: String,        // add enum
    pub time_in_force: String, // add enum TimeInForce
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: String, // add enum Side (it's currently defined in main.rs)
    pub fills: Vec<Fill>,
    #[serde(default)]
    pub value_usd: Decimal,
    #[serde(default)]
    pub commission_usd: Decimal,
}

impl Default for FullTradeResponseRec {
    fn default() -> FullTradeResponseRec {
        FullTradeResponseRec {
            test: false,
            query: "".to_string(),
            symbol: "".to_string(),
            order_id: 0,
            order_list_id: -1,
            client_order_id: "".to_string(),
            transact_time: 0,
            price: dec!(0),
            orig_qty: dec!(0),
            executed_qty: dec!(0),
            cummulative_quote_qty: dec!(0),
            status: "".to_string(),
            time_in_force: "".to_string(),
            order_type: OrderType::MARKET,
            side: "".to_string(),
            fills: vec![],
            value_usd: dec!(0),
            commission_usd: dec!(0),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownTradeResponseRec {
    pub test: bool,
    pub query: String,
    pub response_body: String,
    pub error_internal: String,
}

impl Default for UnknownTradeResponseRec {
    fn default() -> UnknownTradeResponseRec {
        UnknownTradeResponseRec {
            test: false,
            query: "".to_string(),
            response_body: "".to_string(),
            error_internal: "".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TradeResponse {
    SuccessAck(AckTradeResponseRec),
    SuccessResult(ResultTradeResponseRec),
    SuccessFull(FullTradeResponseRec),
    SuccessUnknown(UnknownTradeResponseRec),
    Failure(BinanceError),
}

#[cfg(test)]
mod test {

    use super::*;

    const SUCCESS_ACK: &str = r#"{
         "symbol":"BNBUSD",
         "orderId":93961452,
         "orderListId":-1,
         "clientOrderId":"ekDlCDqC8WT5jOLOKgTkjo",
         "transactTime":1617910570364
    }"#;

    #[test]
    fn test_order_response_success_ack() {
        // Verify SUCCESS_FULL is ok
        let mut response = serde_json::from_str::<AckTradeResponseRec>(SUCCESS_FULL);
        assert!(response.is_ok());

        // Verify SUCCESS_RESULT is ok
        response = serde_json::from_str::<AckTradeResponseRec>(SUCCESS_RESULT);
        assert!(response.is_ok());

        // Verify SUCCESS_ACK is ok
        let mut order_response: AckTradeResponseRec = match serde_json::from_str(SUCCESS_ACK) {
            Ok(response) => response,
            Err(e) => panic!("Error processing response: e={}", e),
        };
        order_response.query = "a_query".to_owned();
        // println!("order_response={:#?}", order_response);
        assert_eq!(order_response.test, false);
        assert_eq!(order_response.query, "a_query");
        assert_eq!(order_response.symbol, "BNBUSD");
        assert_eq!(order_response.order_id, 93961452);
        assert_eq!(order_response.order_list_id, -1);
        assert_eq!(order_response.client_order_id, "ekDlCDqC8WT5jOLOKgTkjo");
        assert_eq!(order_response.transact_time, 1617910570364);
    }

    const SUCCESS_RESULT: &str = r#"{
         "symbol":"BNBUSD",
         "orderId":93961452,
         "orderListId":-1,
         "clientOrderId":"ekDlCDqC8WT5jOLOKgTkjo",
         "transactTime":1617910570364,
         "price":"0.0000",
         "origQty":"0.03000000",
         "executedQty":"0.03000000",
         "cummulativeQuoteQty":"12.5346",
         "status":"FILLED",
         "timeInForce":"GTC",
         "type":"MARKET",
         "side":"BUY"
    }"#;

    #[test]
    fn test_order_response_success_result() {
        // Verify SUCCESS_ACK is err
        let mut response = serde_json::from_str::<ResultTradeResponseRec>(SUCCESS_ACK);
        assert!(response.is_err());

        // Verify SUCCESS_FULL is ok
        response = serde_json::from_str::<ResultTradeResponseRec>(SUCCESS_FULL);
        assert!(response.is_ok());

        // Verify SUCCESS_RESULT is ok
        let mut order_response: ResultTradeResponseRec = match serde_json::from_str(SUCCESS_RESULT)
        {
            Ok(response) => response,
            Err(e) => panic!("Error processing response: e={}", e),
        };
        order_response.query = "a_query".to_owned();
        // println!("order_response={:#?}", order_response);
        assert_eq!(order_response.test, false);
        assert_eq!(order_response.query, "a_query");
        assert_eq!(order_response.symbol, "BNBUSD");
        assert_eq!(order_response.order_id, 93961452);
        assert_eq!(order_response.order_list_id, -1);
        assert_eq!(order_response.client_order_id, "ekDlCDqC8WT5jOLOKgTkjo");
        assert_eq!(order_response.transact_time, 1617910570364);
        assert_eq!(order_response.price, dec!(0.0));
        assert_eq!(order_response.orig_qty, dec!(0.03));
        assert_eq!(order_response.executed_qty, dec!(0.03));
        assert_eq!(order_response.cummulative_quote_qty, dec!(12.5346));
        assert_eq!(order_response.status, "FILLED");
        assert_eq!(order_response.time_in_force, "GTC");
        assert_eq!(order_response.order_type, OrderType::MARKET);
        assert_eq!(order_response.side, "BUY");
    }

    const SUCCESS_FULL: &str = r#"{
         "symbol":"BNBUSD",
         "orderId":93961452,
         "orderListId":-1,
         "clientOrderId":"ekDlCDqC8WT5jOLOKgTkjo",
         "transactTime":1617910570364,
         "price":"0.0000",
         "origQty":"0.03000000",
         "executedQty":"0.03000000",
         "cummulativeQuoteQty":"12.5346",
         "status":"FILLED",
         "timeInForce":"GTC",
         "type":"MARKET",
         "side":"BUY",
         "fills":[{
             "price":"417.8216",
             "qty":"0.03000000",
             "commission":"0.00002250",
             "commissionAsset":"BNB",
             "tradeId":2813236
           }
         ]
    }"#;

    #[test]
    fn test_order_response_success_full() {
        // Verify SUCCESS_ACK is err
        let mut response = serde_json::from_str::<FullTradeResponseRec>(SUCCESS_ACK);
        assert!(response.is_err());

        // Verify SUCCESS_RESULT is err
        response = serde_json::from_str::<FullTradeResponseRec>(SUCCESS_RESULT);
        assert!(response.is_err());

        // Verify SUCCESS_FULL is ok
        let mut order_response: FullTradeResponseRec = match serde_json::from_str(SUCCESS_FULL) {
            Ok(response) => response,
            Err(e) => panic!("Error processing response: e={}", e),
        };
        order_response.query = "a_query".to_owned();
        // println!("order_response={:#?}", order_response);
        assert_eq!(order_response.test, false);
        assert_eq!(order_response.query, "a_query");
        assert_eq!(order_response.symbol, "BNBUSD");
        assert_eq!(order_response.order_id, 93961452);
        assert_eq!(order_response.order_list_id, -1);
        assert_eq!(order_response.client_order_id, "ekDlCDqC8WT5jOLOKgTkjo");
        assert_eq!(order_response.transact_time, 1617910570364);
        assert_eq!(order_response.price, dec!(0.0));
        assert_eq!(order_response.orig_qty, dec!(0.03));
        assert_eq!(order_response.executed_qty, dec!(0.03));
        assert_eq!(order_response.cummulative_quote_qty, dec!(12.5346));
        assert_eq!(order_response.status, "FILLED");
        assert_eq!(order_response.time_in_force, "GTC");
        assert_eq!(order_response.order_type, OrderType::MARKET);
        assert_eq!(order_response.side, "BUY");
        assert_eq!(order_response.fills[0].price, dec!(417.8216));
        assert_eq!(order_response.fills[0].qty, dec!(0.03));
        assert_eq!(order_response.fills[0].commission, dec!(0.00002250));
        assert_eq!(order_response.fills[0].commission_asset, "BNB");
        assert_eq!(order_response.fills[0].trade_id, 2813236);
    }

    const SUCCESS_UNKNOWN: &str = r#"{
        "test":false,
        "query":"a_unknow_query",
        "responseBody":"a body with unknown contents",
        "errorInternal":"some error message"
    }"#;

    #[test]
    fn test_order_response_success_unknown() {
        // Verify SUCCESS_ACK is err
        let mut response = serde_json::from_str::<UnknownTradeResponseRec>(SUCCESS_ACK);
        assert!(response.is_err());

        // Verify SUCCESS_RESULT is err
        response = serde_json::from_str::<UnknownTradeResponseRec>(SUCCESS_RESULT);
        assert!(response.is_err());

        // Verify SUCCESS_FULL is err
        response = serde_json::from_str::<UnknownTradeResponseRec>(SUCCESS_FULL);
        assert!(response.is_err());

        // Verify SUCCESS_UNKNOW is ok
        let mut order_response: UnknownTradeResponseRec =
            match serde_json::from_str(SUCCESS_UNKNOWN) {
                Ok(response) => response,
                Err(e) => panic!("Error processing response: e={}", e),
            };
        order_response.query = "a_unknown_query".to_owned();
        // println!("order_response={:#?}", order_response);
        assert_eq!(order_response.test, false);
        assert_eq!(order_response.query, "a_unknown_query");
        assert_eq!(order_response.response_body, "a body with unknown contents");
        assert_eq!(order_response.error_internal, "some error message");
    }
}
