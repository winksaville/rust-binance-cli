use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize};
use std::{
    collections::BTreeMap,
    fmt,
    io::{stdout, Write},
};

use crate::{
    binance_klines::get_kline,
    common::{get_req_get_response, time_ms_to_utc, utc_now_to_time_ms},
    de_string_or_number::de_string_or_number_to_i64,
};

use crate::binance_signature::{append_signature, binance_signature, query_vec_u8};

use crate::binance_context::BinanceContext;

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
// TODO: Maybe a process macro can be created that generates de_vec_xxx_to_hashmap?
pub fn de_vec_balances_to_hashmap<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, Balance>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = BTreeMap<String, Balance>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<BTreeMap<String, Balance>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: BTreeMap<String, Balance> = BTreeMap::new();
            //BTreeMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<Balance>()? {
                // println!("item={:#?}", item);
                map.insert(item.asset.clone(), item);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

#[derive(Debug, Deserialize, Clone, Ord, Eq, PartialEq, PartialOrd, Serialize)]
pub struct Balance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    #[serde(skip)]
    pub value_in_usd: Decimal,
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
    pub balances_map: BTreeMap<String, Balance>,
}

impl AccountInfo {
    pub async fn update_values_in_usd(&mut self, ctx: &BinanceContext, verbose: bool) -> Decimal {
        let mut total_value = dec!(0);
        for mut balance in self.balances_map.values_mut() {
            if balance.free > dec!(0) || balance.locked > dec!(0) {
                let price = if balance.asset != "USD" {
                    let sym = balance.asset.clone() + "USD";
                    if verbose {
                        print!("{:-10} {:+10}\r", "Updating", sym);
                        let _ = stdout().flush();
                    }
                    let price = match get_kline(ctx, &sym, utc_now_to_time_ms()).await {
                        Ok(kr) => kr.close,
                        Err(_) => {
                            dec!(0)
                        }
                    };
                    price
                } else {
                    dec!(1)
                };
                balance.value_in_usd = price * (balance.free + balance.locked);
                total_value += balance.value_in_usd;
            }
        }
        if verbose {
            print!("{:-10} {:+10}\r", " ", " ");
            let _ = stdout().flush();
        }

        total_value
    }

    pub fn print_header_fields(&mut self) {
        println!("     account_type: {}", self.account_type);
        println!("      can_deposit: {}", self.can_deposit);
        println!("        can_trade: {}", self.can_trade);
        println!("     can_withdraw: {}", self.can_withdraw);
        println!(" buyer_commission: {}", self.buyer_commission);
        println!(" maker_commission: {}", self.maker_commission);
        println!("seller_commission: {}", self.seller_commission);
        println!(" taker_commission: {}", self.taker_commission);
        println!("      update_time: {}", time_ms_to_utc(self.update_time));
        println!("      permissions: {:?}", self.permissions);
    }

    pub async fn print(&mut self) {
        self.print_header_fields();
        let mut total_value = dec!(0);
        for balance in self.balances_map.values() {
            if balance.value_in_usd > dec!(0) {
                total_value += balance.value_in_usd;
                println!(
                    "  {:6}: value: ${:10.2} free: {:15.8} locked: {}",
                    balance.asset, balance.value_in_usd, balance.free, balance.locked
                );
            }
        }
        println!("total: ${:.2}", total_value);
    }

    pub async fn update_and_print(&mut self, ctx: &BinanceContext) {
        self.update_values_in_usd(ctx, true).await;
        self.print().await;
    }
}

pub async fn get_account_info<'e>(
    ctx: &BinanceContext,
) -> Result<AccountInfo, Box<dyn std::error::Error>> {
    trace!("get_account_info: +");

    let secret_key = ctx.keys.secret_key.as_bytes();
    let api_key = &ctx.keys.api_key;

    let mut params = vec![];
    let ts_string: String = format!("{}", utc_now_to_time_ms());
    params.append(&mut vec![("timestamp", ts_string.as_str())]);

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data in qs and query as body
    let signature = binance_signature(&secret_key, &[], &query);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("query_string={}", &query_string);

    let url = ctx.make_url("api", &format!("/api/v3/account?{}", &query_string));
    trace!("get_account_info: url={}", url);

    let response = get_req_get_response(api_key, &url).await?;
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
