use serde::{Deserialize, Serialize};

use crate::de_string_or_number::{
    de_string_or_number_to_f64, de_string_or_number_to_i64, de_string_or_number_to_u64,
};

use crate::common::BinanceResponseError;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    pub commission_asset: String,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub commission: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub qty: f64,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub trade_id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponseSuccess {
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub query: String,
    symbol: String,
    pub client_order_id: String,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub order_id: u64,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub order_list_id: i64,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub transact_time: u64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub orig_qty: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub executed_qty: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub cummulative_quote_qty: f64,
    pub status: String,        // add enum
    pub time_in_force: String, // add enum TimeInForce
    #[serde(rename = "type")]
    pub order_type: String, // add enum OrderType
    pub side: String,          // add enum Side (it's currently defined in main.rs)
    pub fills: Vec<Fill>,
}

impl Default for OrderResponseSuccess {
    fn default() -> OrderResponseSuccess {
        OrderResponseSuccess {
            test: false,
            query: "".to_string(),
            symbol: "".to_string(),
            client_order_id: "".to_string(),
            order_id: 0,
            order_list_id: -1,
            transact_time: 0,
            price: 0f64,
            orig_qty: 0f64,
            executed_qty: 0f64,
            cummulative_quote_qty: 0f64,
            status: "".to_string(),
            time_in_force: "".to_string(),
            order_type: "".to_string(),
            side: "".to_string(),
            fills: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum OrderResponse {
    Success(OrderResponseSuccess),
    Failure(BinanceResponseError),
}

#[cfg(test)]
mod test {
    use super::*;

    const ORDER_RESPONSE_SUCCESS: &str = r#"{
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
    fn test_order_response_success() {
        let mut order_response: OrderResponseSuccess =
            match serde_json::from_str(ORDER_RESPONSE_SUCCESS) {
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
        assert_eq!(order_response.price, 0.0);
        assert_eq!(order_response.orig_qty, 0.03);
        assert_eq!(order_response.executed_qty, 0.03);
        assert_eq!(order_response.cummulative_quote_qty, 12.5346);
        assert_eq!(order_response.status, "FILLED");
        assert_eq!(order_response.time_in_force, "GTC");
        assert_eq!(order_response.order_type, "MARKET");
        assert_eq!(order_response.side, "BUY");
        assert_eq!(order_response.fills[0].price, 417.8216);
        assert_eq!(order_response.fills[0].qty, 0.03);
        assert_eq!(order_response.fills[0].commission, 0.00002250);
        assert_eq!(order_response.fills[0].commission_asset, "BNB");
        assert_eq!(order_response.fills[0].trade_id, 2813236);
    }
}
