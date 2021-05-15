use log::trace;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use tokio::fs;

use rust_decimal::prelude::*;
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::{binance_account_info::get_account_info, binance_context::BinanceContext};

fn default_min() -> Decimal {
    Decimal::MAX
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct KeepRec {
    name: String,

    #[serde(default = "default_min")]
    min: Decimal,

    #[serde(default)]
    sell_to_asset: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigAutoSell {
    #[serde(rename = "SECRET_KEY")]
    #[serde(default)]
    pub secret_key: String,

    #[serde(rename = "API_KEY")]
    #[serde(default)]
    pub api_key: String,

    #[serde(default)]
    pub default_sell_to_asset: String,

    #[serde(deserialize_with = "de_vec_keep_rec_to_hashmap")]
    #[serde(default)]
    pub keep: HashMap<String, KeepRec>,
}

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
// TODO: Maybe a process macro can be created that generates de_vec_xxx_to_hashmap?
pub fn de_vec_keep_rec_to_hashmap<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, KeepRec>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = HashMap<String, KeepRec>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<String, KeepRec>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<String, KeepRec> =
                HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<KeepRec>()? {
                // println!("item={:#?}", item);
                map.insert(item.name.clone(), item);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

pub async fn auto_sell(
    ctx: &BinanceContext,
    config_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("auto_sell:+ config_file: {}", config_file);

    // Get the file contents and deserialize to ConfigAutoSell
    let config_string: String = fs::read_to_string(config_file).await?;
    let config: ConfigAutoSell = toml::from_str(&config_string)?;
    // println!("auto_sell: config:\n{:#?}", config);

    // Create a mutable clone so we can change the keys
    // and then change it back to immutable
    // TODO: Consider adding BinanceContext::set_keys?
    let mut ctx: BinanceContext = (*ctx).clone();
    ctx.keys.api_key = config.api_key.clone();
    ctx.keys.secret_key = config.secret_key.clone();
    let ctx = &ctx;

    let mut ai = get_account_info(ctx).await?;
    ai.update_values(&ctx).await;
    //ai.print().await;

    #[derive(Default)]
    struct KeepRec {
        asset: String,
        owned_qty: Decimal,
        sell_value: Decimal,
        sell_qty: Decimal,
        keep_value: Decimal,
        keep_qty: Decimal,
    }

    let mut vec_keep_rec = Vec::new();
    for balance in ai.balances_map.values() {
        let owned_qty = balance.free + balance.locked;
        if owned_qty > dec!(0) {
            if let Some(keeping) = config.keep.get(&balance.asset) {
                let keep_qty = if keeping.min < Decimal::MAX {
                    keeping.min
                } else {
                    owned_qty
                };
                let sell_qty = owned_qty - keep_qty;
                vec_keep_rec.push(KeepRec {
                    asset: balance.asset.clone(),
                    owned_qty,
                    sell_value: (sell_qty / owned_qty) * balance.value,
                    sell_qty,
                    keep_value: (keep_qty / owned_qty) * balance.value,
                    keep_qty,
                });
            } else {
                println!(
                    "Selling {:18.6} of {:6} worth ${:10.2} keeping none",
                    owned_qty, balance.asset, balance.value
                );
            }
        }
    }
    for kr in vec_keep_rec {
        if kr.sell_qty > dec!(0) {
            println!(
                "Keeping {:18.6} of {:6} worth ${:10.2} selling {} worth ${:10.2}",
                kr.keep_qty, kr.asset, kr.keep_value, kr.sell_qty, kr.sell_value
            );
        } else {
            println!(
                "Keeping {:18.6} of {:6} worth ${:10.2} selling none",
                kr.owned_qty, kr.asset, kr.keep_value
            );
        }
    }

    trace!("auto_sell:- config_file: {}", config_file);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use rust_decimal_macros::dec;
    use toml;

    const TOML_DATA: &str = r#"
        API_KEY = "api key"
        SECRET_KEY = "secret key"
        default_sell_to_asset="USD"

        keep = [
            { name = "USD" },
            { name = "USDT" },
            { name = "USDC" },
            { name = "BNB", min = 500 },
            { name = "ABC", min = 0, sell_to_asset = "BTC" },
            { name = "XYZ", sell_to_asset = "BNB" },
        ]
    "#;

    #[test]
    fn test_config_auto_sell_all() {
        let config: ConfigAutoSell = toml::from_str(TOML_DATA).unwrap();
        // println!("{:#?}", config);
        assert_eq!(config.api_key, "api key");
        assert_eq!(config.secret_key, "secret key");
        assert_eq!(config.default_sell_to_asset, "USD");
        assert_eq!(
            config.keep.get("USD").unwrap(),
            &KeepRec {
                name: "USD".to_string(),
                min: Decimal::MAX,
                sell_to_asset: "".to_string()
            }
        );
        assert_eq!(
            config.keep.get("USDT").unwrap(),
            &KeepRec {
                name: "USDT".to_string(),
                min: Decimal::MAX,
                sell_to_asset: "".to_string()
            }
        );
        assert_eq!(
            config.keep.get("USDC").unwrap(),
            &KeepRec {
                name: "USDC".to_string(),
                min: Decimal::MAX,
                sell_to_asset: "".to_string()
            }
        );
        assert_eq!(
            config.keep.get("BNB").unwrap(),
            &KeepRec {
                name: "BNB".to_string(),
                min: dec!(500),
                sell_to_asset: "".to_string()
            }
        );

        // ABC says sell everything to BTC
        assert_eq!(
            config.keep.get("ABC").unwrap(),
            &KeepRec {
                name: "ABC".to_string(),
                min: dec!(0),
                sell_to_asset: "BTC".to_string()
            }
        );

        // XYZ is odd as nothing will be sold since KeepRec.min default is MAX so sell_to_asset is ignored
        assert_eq!(
            config.keep.get("XYZ").unwrap(),
            &KeepRec {
                name: "XYZ".to_string(),
                min: Decimal::MAX,
                sell_to_asset: "BNB".to_string()
            }
        );
    }
}
