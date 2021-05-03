use log::trace;
use rust_decimal::prelude::*;
use serde::{de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use crate::{common::utc_now_to_time_ms, de_string_or_number::de_string_or_number_to_i64};

use crate::binance_signature::{append_signature, binance_signature, query_vec_u8};

use crate::binance_context::BinanceContext;

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
// TODO: Maybe a process macro can be created that generates de_vec_xxx_to_hashmap?
pub fn de_vec_balances_to_hashmap<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Balance>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = HashMap<String, Balance>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<String, Balance>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<String, Balance> =
                HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<Balance>()? {
                // println!("item={:#?}", item);
                map.insert(item.asset.clone(), item);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub account_type: String,
    pub can_deposit: bool,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub buyer_commission: Decimal,
    pub maker_commission: Decimal,
    pub seller_commission: Decimal,
    pub taker_commission: Decimal,
    #[serde(deserialize_with = "de_string_or_number_to_i64")]
    pub update_time: i64,
    pub permissions: Vec<String>,
    #[serde(deserialize_with = "de_vec_balances_to_hashmap")]
    #[serde(rename = "balances")]
    pub balances_map: HashMap<String, Balance>,
}

pub async fn get_account_info<'e>(
    ctx: &BinanceContext,
) -> Result<AccountInfo, Box<dyn std::error::Error>> {
    trace!("get_account_info: +");

    let sig_key = ctx.opts.secret_key.as_bytes();
    let api_key = ctx.opts.api_key.as_bytes();

    let mut params = vec![];
    let ts_string: String = format!("{}", utc_now_to_time_ms());
    params.append(&mut vec![("timestamp", ts_string.as_str())]);

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data in qs and query as body
    let signature = binance_signature(&sig_key, &[], &query);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("query_string={}", &query_string);

    let url = ctx.make_url("api", &format!("/api/v3/account?{}", &query_string));
    trace!("get_account_info: url={}", url);

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
    trace!("response={:#?}", response);
    let response_status = response.status();
    let response_body = response.text().await?;
    let account_info: AccountInfo = if response_status == 200 {
        let ai: AccountInfo = match serde_json::from_str(&response_body) {
            Ok(info) => info,
            Err(e) => {
                let err = format!(
                    "Error converting body to AccountInfo: e={} body={}",
                    e, response_body
                );
                trace!("get_account_info: err: {}", err);
                return Err(err.into());
            }
        };
        ai
    } else {
        let err = format!("response status={} body={}", response_status, response_body);
        trace!("get_account_info: err: {}", err);
        return Err(err.into());
    };

    trace!("get_account_info: -");
    Ok(account_info)
}
#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

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
        assert_eq!(dec!(10), account_info.maker_commission);
        assert_eq!(dec!(10), account_info.taker_commission);
        assert_eq!(dec!(0), account_info.buyer_commission);
        assert_eq!(dec!(0), account_info.seller_commission);
        assert_eq!(true, account_info.can_trade);
        assert_eq!(true, account_info.can_deposit);
        assert_eq!(1616461066366, account_info.update_time);
        assert_eq!("SPOT", account_info.account_type);
        assert_eq!("SPOT", account_info.permissions[0]);
        let balance = account_info.balances_map.get("BTC").unwrap();
        assert_eq!("BTC", balance.asset);
        assert_eq!(dec!(0), balance.free);
        assert_eq!(dec!(0), balance.locked);
        let balance = account_info.balances_map.get("ETH").unwrap();
        assert_eq!("ETH", balance.asset);
        assert_eq!(dec!(0), balance.free);
        assert_eq!(dec!(0), balance.locked);
    }
}
