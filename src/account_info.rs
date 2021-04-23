use serde::{Deserialize, Serialize};

use crate::de_string_or_number::{de_string_or_number_to_f64, de_string_or_number_to_u64};

#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    asset: String,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    free: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    locked: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    account_type: String,
    can_deposit: bool,
    can_trade: bool,
    can_withdraw: bool,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    buyer_commission: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    maker_commission: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    seller_commission: f64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    taker_commission: f64,
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    update_time: u64,
    permissions: Vec<String>,
    balances: Vec<Balance>,
}

#[cfg(test)]
mod test {
    // extern crate test;

    use super::*;

    const ACCOUNT_INFO_DATA: &str = r#"{
        "makerCommission":10,
        "takerCommission":10,
        "buyerCommission":0,
        "sellerCommission":0,
        "canTrade":true,
        "canWithdraw":true,
        "canDeposit":true,
        "updateTime":1616461066366,
        "accountType":"SPOT",
        "permissions":["SPOT"],
        "balances":[
            {"asset":"BTC","free":"0.00000000","locked":"0.00000000"},
            {"asset":"ETH","free":"0.00000000","locked":"0.00000000"}
        ]
    }"#;

    #[test]
    fn test_account_info() {
        let account_info: AccountInfo = match serde_json::from_str(ACCOUNT_INFO_DATA) {
            Ok(info) => info,
            Err(e) => panic!("Error processing response: e={}", e),
        };
        // println!("account_info={:#?}", account_info);
        assert_eq!(10f64, account_info.maker_commission);
        assert_eq!(10f64, account_info.taker_commission);
        assert_eq!(0f64, account_info.buyer_commission);
        assert_eq!(0f64, account_info.seller_commission);
        assert_eq!(true, account_info.can_trade);
        assert_eq!(true, account_info.can_deposit);
        assert_eq!(1616461066366, account_info.update_time);
        assert_eq!("SPOT", account_info.account_type);
        assert_eq!("SPOT", account_info.permissions[0]);
        assert_eq!("BTC", account_info.balances[0].asset);
        assert_eq!(0f64, account_info.balances[0].free);
        assert_eq!(0f64, account_info.balances[0].locked);
        assert_eq!("ETH", account_info.balances[1].asset);
        assert_eq!(0f64, account_info.balances[1].free);
        assert_eq!(0f64, account_info.balances[1].locked);
    }
}
