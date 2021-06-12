use std::{error::Error, fmt};

use log::trace;
use serde::{Deserialize, Serialize};

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use semver::Version;

use crate::{
    binance_withdraw_cmd::WithdrawParams,
    common::{dec_to_money_string, time_ms_to_utc, InternalErrorRec, ResponseErrorRec, Side},
    de_string_or_number::{de_string_or_number_to_i64, de_string_or_number_to_u64},
};

use crate::common::OrderType;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderRec {
    pub app_version: Version,
    pub rec_version: Option<Version>,
    #[serde(default)]
    pub test: Option<bool>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub value_usd: Option<Decimal>,
    #[serde(default)]
    pub commission_usd: Option<Decimal>,
    #[serde(default)]
    pub internal_errors: Vec<String>,
}

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
    pub symbol: String,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub order_id: u64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64,
    pub client_order_id: String,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub transact_time: i64,
}

impl fmt::Display for AckTradeResponseRec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        trace!("Display::atrr: {:#?}", self);
        write!(
            f,
            "Trade Ack for {} at {}",
            self.symbol,
            time_ms_to_utc(self.transact_time),
        )
    }
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
    pub symbol: String,
    pub order_id: u64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64,
    pub client_order_id: String,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub transact_time: i64,
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

impl fmt::Display for ResultTradeResponseRec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        trace!("Display::rtrr: {:#?}", self);
        let price = if self.price > dec!(0) {
            self.price
        } else {
            self.cummulative_quote_qty / self.executed_qty
        };
        write!(
            f,
            "{:8} {:14.6} at {:.4}/per of {:10} valued at {}",
            self.side,
            self.executed_qty,
            price.round_dp(4),
            self.symbol,
            dec_to_money_string(self.cummulative_quote_qty),
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullTradeResponseRec {
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub query: String,
    pub symbol: String,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub order_id: u64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64,
    pub client_order_id: String,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub transact_time: i64,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: String,        // add enum
    pub time_in_force: String, // add enum TimeInForce
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: Side,
    pub fills: Vec<Fill>,
    #[serde(default)]
    pub value_usd: Decimal,
    #[serde(default)]
    pub commission_usd: Decimal,
}

impl fmt::Display for FullTradeResponseRec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        trace!("Display::fttr: {:#?}", self);
        let price = if self.price > dec!(0) {
            self.price
        } else {
            self.cummulative_quote_qty / self.executed_qty
        };
        write!(
            f,
            "{:8} {:14.6} at {:.4}/per of {:10} valued at {}",
            self.side,
            self.executed_qty,
            price.round_dp(4),
            self.symbol,
            dec_to_money_string(self.cummulative_quote_qty),
        )
    }
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
            side: Side::BUY,
            fills: vec![],
            value_usd: dec!(0),
            commission_usd: dec!(0),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawResponseRec {
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub params: WithdrawParams,
    #[serde(default)]
    pub response_body: String,
    #[serde(default)]
    pub msg: String,
    #[serde(default)]
    pub id: String,

    pub success: bool,
}

impl fmt::Display for WithdrawResponseRec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        trace!("Display::wrr: {:#?}", self);
        write!(
            f,
            "{} of {} {} to addr: {}{}{}{}{}",
            if self.success {
                "Successful withdraw"
            } else {
                "Failure withdrawing"
            },
            self.params.quantity,
            self.params.sym_name,
            if let Some(l) = &self.params.label {
                format!("{}:", l)
            } else {
                "".to_string()
            },
            self.params.address,
            if let Some(sa) = &self.params.secondary_address {
                format!(":{}", sa)
            } else {
                "".to_string()
            },
            if self.msg.is_empty() {
                "".to_string()
            } else {
                format!(" msg: {}", self.msg)
            },
            if self.id.is_empty() {
                "".to_string()
            } else {
                format!(" id: {}", self.id)
            },
        )
    }
}

impl Default for WithdrawResponseRec {
    fn default() -> WithdrawResponseRec {
        WithdrawResponseRec {
            test: false,
            query: "".to_string(),
            params: WithdrawParams::default(),
            response_body: "".to_string(),
            msg: "".to_string(),
            success: false,
            id: "".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestTradeResponseRec {
    pub test: bool,
    pub query: String,
    pub response_body: String,
}

impl fmt::Display for TestTradeResponseRec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        trace!("Display::tttr: {:#?}", self);
        write!(f, "Successful test, response body: {}", self.response_body)
    }
}

impl Default for TestTradeResponseRec {
    fn default() -> TestTradeResponseRec {
        TestTradeResponseRec {
            test: false,
            query: "".to_string(),
            response_body: "".to_string(),
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

impl fmt::Display for UnknownTradeResponseRec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        trace!("Display::uttr: {:#?}", self);
        write!(
            f,
            "Successful but unknown response, body: {}",
            self.response_body
        )
    }
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
    SuccessTest(TestTradeResponseRec),
    SuccessWithdraw(WithdrawResponseRec),
    FailureWithdraw(WithdrawResponseRec),
    SuccessTestWithdraw(WithdrawResponseRec),
    SuccessUnknown(UnknownTradeResponseRec),
    FailureResponse(ResponseErrorRec),
    FailureInternal(InternalErrorRec),
}

impl Error for TradeResponse {}

impl fmt::Display for TradeResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TradeResponse::SuccessAck(tr) => write!(f, "{}", tr),
            TradeResponse::SuccessResult(tr) => write!(f, "{}", tr),
            TradeResponse::SuccessFull(tr) => write!(f, "{}", tr),
            TradeResponse::SuccessTest(tr) => write!(f, "{}", tr),
            TradeResponse::SuccessWithdraw(tr) => write!(f, "{}", tr),
            TradeResponse::FailureWithdraw(tr) => write!(f, "{}", tr),
            TradeResponse::SuccessTestWithdraw(tr) => write!(f, "{}", tr),
            TradeResponse::SuccessUnknown(tr) => write!(f, "{}", tr),
            TradeResponse::FailureResponse(ber) => write!(f, "{}", ber),
            TradeResponse::FailureInternal(ier) => write!(f, "{}", ier),
        }
    }
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
        assert_eq!(order_response.side, Side::BUY);
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

    #[test]
    fn test_order_response_semver() {
        let ver = Version::parse("1.2.3-alpha1+1234").unwrap();
        println!("ver:         {:?}", ver);
        let ver_json = serde_json::to_string(&ver).unwrap();
        println!("ver_json:    {:?}", ver_json);
        let ver_json_de: Version = serde_json::from_str(&ver_json).unwrap();
        println!("ver_json_de: {:?}", ver_json_de);
        assert_eq!(ver, ver_json_de);
    }

    const HEADER_REC_MIN: &str = r#"{
        "appVersion":"1.2.3-alpha1+1234"
    }"#;
    //    "Version":"1.2.3-alpha1+1234",
    //    pub rec_version: Option<Version>,
    //    #[serde(default)]
    //    pub test: Option<bool>,
    //    #[serde(default)]
    //    pub query: Option<String>,
    //    #[serde(default)]
    //    pub value_usd: Option<Decimal>,
    //    #[serde(default)]
    //    pub commission_usd: Option<Decimal>,
    //    #[serde(default)]
    //    pub internal_errors: Vec<String>,
    //"#;

    #[test]
    fn test_order_response_header_rec_min() {
        let hr: HeaderRec = serde_json::from_str(HEADER_REC_MIN).unwrap();
        println!("hr: {:?}", hr);
        println!("app_version: {:?}", hr.app_version);
        let expected = Version::parse("1.2.3-alpha1+1234").unwrap();
        println!("expected:    {:?}", expected);
        assert!(expected == hr.app_version);
        assert!(None == hr.rec_version);
        assert!(None == hr.test);
        assert!(None == hr.query);
        assert!(None == hr.value_usd);
        assert!(None == hr.commission_usd);
        assert!(hr.internal_errors.is_empty());
    }

    const HEADER_REC_APP_REC_VERSIONS: &str = r#"{
        "appVersion":"1.2.3-alpha1+1234",
        "recVersion":"3.2.1-beta2+9e0cec6"
    }"#;
    //    pub rec_version: Option<Version>,
    //    #[serde(default)]
    //    pub test: Option<bool>,
    //    #[serde(default)]
    //    pub query: Option<String>,
    //    #[serde(default)]
    //    pub value_usd: Option<Decimal>,
    //    #[serde(default)]
    //    pub commission_usd: Option<Decimal>,
    //    #[serde(default)]
    //    pub internal_errors: Vec<String>,
    //}"#;

    #[test]
    fn test_order_response_header_rec_app_rec_versions() {
        let hr: HeaderRec = serde_json::from_str(HEADER_REC_APP_REC_VERSIONS).unwrap();
        println!("hr: {:?}", hr);

        println!("app_version: {:?}", hr.app_version);
        let expected = Version::parse("1.2.3-alpha1+1234").unwrap();
        println!("expected:    {:?}", expected);
        assert!(expected == hr.app_version);

        println!("rec_version: {:?}", hr.rec_version);
        let expected = Some(Version::parse("3.2.1-beta2+9e0cec6").unwrap());
        println!("expected:    {:?}", expected);
        assert!(expected == hr.rec_version);
        assert!(None == hr.test);
        assert!(None == hr.query);
        assert!(None == hr.value_usd);
        assert!(None == hr.commission_usd);
        assert!(hr.internal_errors.is_empty());
    }

    const HEADER_REC_MAX: &str = r#"{
        "appVersion":"1.2.3-alpha1+1234",
        "recVersion":"3.2.1-beta2+9e0cec6",
        "test":false,
        "query":"A query string",
        "valueUsd":"123.456",
        "commissionUsd":"1",
        "internalErrors": [
            "abc",
            "def"
        ]
    }"#;

    #[test]
    fn test_order_response_header_rec_max() {
        let hr: HeaderRec = serde_json::from_str(HEADER_REC_MAX).unwrap();
        println!("hr: {:?}", hr);

        let expected = Version::parse("1.2.3-alpha1+1234").unwrap();
        assert!(expected == hr.app_version);

        let expected = Some(Version::parse("3.2.1-beta2+9e0cec6").unwrap());
        assert!(expected == hr.rec_version);
        let expected = Some(false);
        assert!(expected == hr.test);
        let expected = Some("A query string".to_string());
        assert!(expected == hr.query);
        let expected = Some(dec!(123.456));
        assert!(expected == hr.value_usd);
        let expected = Some(dec!(1));
        assert!(expected == hr.commission_usd);
        let expected = vec!["abc", "def"];
        assert!(expected == hr.internal_errors);
    }
}
