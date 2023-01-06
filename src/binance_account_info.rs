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
    binance_klines::get_kline_of_primary_asset_for_value_asset,
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    common::{get_req_get_response, VALUE_ASSETS},
    de_string_or_number::de_string_or_number_to_i64,
    Configuration,
};

use dec_utils::{dec_to_separated_string, dec_to_usd_string};
use time_ms_conversions::time_ms_to_utc;

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
    pub price_in_usd: Decimal,
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
    pub async fn update_values_in_usd(
        &mut self,
        config: &Configuration,
        verbose: bool,
        time_ms: i64,
    ) -> Decimal {
        let mut total_value = dec!(0);
        for mut balance in self.balances_map.values_mut() {
            // Print all assets with a free or locked balance
            if balance.free != dec!(0) || balance.locked != dec!(0) {
                let price_in_usd = if balance.asset != "USD" {
                    let r = get_kline_of_primary_asset_for_value_asset(
                        config,
                        time_ms,
                        &balance.asset,
                        &VALUE_ASSETS,
                    )
                    .await;

                    match r {
                        Some((sym, kr)) => {
                            if verbose {
                                print!("{:-10} {:+10} {:+20}\r", "Updated ", sym, " ");
                                let _ = stdout().flush();
                            }

                            kr.close
                        }
                        None => dec!(0),
                    }
                } else {
                    dec!(1)
                };
                balance.price_in_usd = price_in_usd;
                balance.value_in_usd = price_in_usd * (balance.free + balance.locked);
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
        println!();

        let col_1 = 6;
        let col_2 = 12;
        let col_3 = 12;
        let col_4 = 20;
        let col_5 = 20;
        let col_6 = 20;

        let mut total_value = dec!(0);
        println!(
            "{:<col_1$} {:>col_2$} {:>col_3$} {:>col_4$} {:>col_5$} {:>col_6$}",
            "Asset", "USD value", "USD/coin", "Total Coins", "Free", "locked"
        );
        for balance in self.balances_map.values() {
            if balance.value_in_usd > dec!(0) {
                total_value += balance.value_in_usd;
                println!(
                    "{:<col_1$} {:>col_2$} {:>col_3$} {:>col_4$} {:>col_5$} {:>col_6$}",
                    balance.asset,
                    dec_to_usd_string(balance.value_in_usd),
                    dec_to_usd_string(balance.price_in_usd),
                    dec_to_separated_string(balance.free + balance.locked, 8),
                    dec_to_separated_string(balance.free, 8),
                    dec_to_separated_string(balance.locked, 8),
                );
            }
        }
        println!("total: {}", dec_to_usd_string(total_value));
    }

    pub async fn update_and_print(&mut self, config: &Configuration, time_ms: i64) {
        self.update_values_in_usd(config, config.verbose, time_ms)
            .await;
        self.print().await;
    }
}

pub async fn get_account_info<'e>(
    config: &Configuration,
    time_ms: i64,
) -> Result<AccountInfo, Box<dyn std::error::Error>> {
    trace!("get_account_info: +");

    let api_key = config.keys.get_ak_or_err()?;
    let secret_key = &config.keys.get_sk_vec_u8_or_err()?;

    let mut params = vec![];
    let ts_string: String = format!("{time_ms}");
    params.append(&mut vec![("timestamp", ts_string.as_str())]);

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data in qs and query as body
    let signature = binance_signature(secret_key, &[], &query);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("query_string={}", &query_string);

    let url = config.make_url("api", &format!("/api/v3/account?{}", &query_string));
    trace!("get_account_info: url={}", url);

    let response = get_req_get_response(api_key, &url).await?;
    trace!("response={:#?}", response);
    let response_status = response.status();
    let response_body = response.text().await?;

    let account_info: AccountInfo = if response_status == 200 {
        let ai: AccountInfo = match serde_json::from_str(&response_body) {
            Ok(info) => info,
            Err(e) => {
                let err =
                    format!("Error converting body to AccountInfo: e={e} body={response_body}");
                trace!("get_account_info: err: {}", err);
                return Err(err.into());
            }
        };
        ai
    } else {
        let err = format!("response status={response_status} body={response_body}");
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
            Err(e) => panic!("Error processing response: e={e}"),
        };
        // println!("account_info={:#?}", account_info);
        assert_eq!(dec!(10), account_info.maker_commission);
        assert_eq!(dec!(10), account_info.taker_commission);
        assert_eq!(dec!(0), account_info.buyer_commission);
        assert_eq!(dec!(0), account_info.seller_commission);
        assert!(account_info.can_trade);
        assert!(account_info.can_deposit);
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
