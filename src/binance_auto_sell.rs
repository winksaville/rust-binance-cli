use log::trace;
use rust_decimal_macros::dec;
use std::{collections::HashMap, path::PathBuf};
use structopt::StructOpt;
use tokio::fs;

use rust_decimal::prelude::*;
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::{
    binance_account_info::get_account_info,
    binance_context::BinanceContext,
    binance_exchange_info::{get_exchange_info, ExchangeInfo},
    binance_market_order_cmd::market_order,
    common::{are_you_sure_stdout_stdin, Side},
};

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

    pub order_log_path: Option<PathBuf>,

    #[serde(default = "default_sell_to_asset")]
    pub default_sell_to_asset: String,

    #[serde(deserialize_with = "de_vec_keep_rec_to_hashmap")]
    #[serde(default)]
    pub keep: HashMap<String, KeepRec>,
}

fn default_sell_to_asset() -> String {
    "USD".to_string()
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
    ei: &ExchangeInfo,
    config_file: &str,
    test: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("auto_sell:+ test: {} config_file: {}", test, config_file);

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
    if let Some(olp) = config.order_log_path {
        ctx.opts.order_log_path = olp;
    }
    let ctx = &ctx;

    let mut ai = get_account_info(ctx).await?;
    ai.update_values_in_usd(&ctx, true).await;
    //ai.print().await;

    #[derive(Default)]
    struct ProcessRec {
        asset: String,
        price_in_usd: Decimal,
        sell_to_asset: String,
        owned_qty: Decimal,
        sell_value_in_usd: Decimal,
        sell_qty: Decimal,
        keep_value_in_usd: Decimal,
        keep_qty: Decimal,
    }

    let mut vec_process_rec = Vec::new();
    for balance in ai.balances_map.values() {
        let keep_qty: Decimal;
        let sell_qty: Decimal;
        let sell_to_asset: &str;

        assert!(!config.default_sell_to_asset.is_empty());

        let owned_qty = balance.free + balance.locked;
        if owned_qty > dec!(0) {
            if let Some(keeping) = config.keep.get(&balance.asset) {
                keep_qty = if keeping.min < Decimal::MAX && keeping.min < owned_qty {
                    keeping.min
                } else {
                    owned_qty
                };
                sell_qty = owned_qty - keep_qty;

                sell_to_asset = if keeping.sell_to_asset.is_empty() {
                    &config.default_sell_to_asset
                } else {
                    &keeping.sell_to_asset
                };
            } else {
                // Selling all
                keep_qty = dec!(0);
                sell_qty = owned_qty;
                sell_to_asset = &config.default_sell_to_asset;
            }

            vec_process_rec.push(ProcessRec {
                asset: balance.asset.clone(),
                price_in_usd: balance.price_in_usd,
                sell_to_asset: sell_to_asset.to_string(),
                owned_qty,
                sell_value_in_usd: (sell_qty / owned_qty) * balance.value_in_usd,
                sell_qty,
                keep_value_in_usd: (keep_qty / owned_qty) * balance.value_in_usd,
                keep_qty,
            });
        }
    }

    // Print assets being kept
    let mut kept_cnt: i64 = 0;
    for kr in &vec_process_rec {
        if kr.sell_qty <= dec!(0) {
            kept_cnt += 1;
            println!(
                "Keeping {:14.6} of {:10} at about ${:10.2}/per worth ${:10.2} selling NONE",
                kr.owned_qty,
                kr.asset,
                kr.price_in_usd,
                kr.keep_value_in_usd.round_dp(2)
            );
        }
    }
    if kept_cnt > 0 {
        println!();
    }

    // Print assets being sold
    let mut total_sell_in_usd = dec!(0);
    let mut total_assets_selling_some_or_all = 0i64;
    for kr in &vec_process_rec {
        if kr.sell_qty > dec!(0) {
            print!(
                "SELLING {:14.6} of {:10} at about ${:10.2}/per worth ${:10.2} keeping ",
                kr.sell_qty,
                kr.asset,
                kr.price_in_usd,
                kr.sell_value_in_usd.round_dp(2),
            );
            if kr.keep_qty > dec!(0) {
                println!(
                    "{:6} worth ${:10.2}",
                    kr.keep_qty,
                    kr.keep_value_in_usd.round_dp(2),
                );
            } else {
                println!("NONE");
            }
            total_sell_in_usd += kr.sell_value_in_usd;
            total_assets_selling_some_or_all += 1;
        }
    }

    if total_assets_selling_some_or_all > 0 {
        println!(
            "\nSELLING {} assets for ${:10.2}",
            total_assets_selling_some_or_all,
            total_sell_in_usd.round_dp(2)
        );
        if test || are_you_sure_stdout_stdin() {
            for kr in &vec_process_rec {
                if kr.sell_qty > dec!(0) {
                    let symbol_name: String = kr.asset.clone() + &kr.sell_to_asset;
                    println!(
                        "\nSELLING {:14.6} of {:10} at about ${:10.2}/per worth ${:10.2}",
                        kr.sell_qty,
                        symbol_name,
                        kr.price_in_usd,
                        kr.sell_value_in_usd.round_dp(2)
                    );
                    match market_order(ctx, ei, &symbol_name, kr.sell_qty, Side::SELL, test).await {
                        Ok(tr) => println!("{}", tr),
                        Err(e) => println!("SKIPPING {}, {}", symbol_name, e),
                    }
                }
            }
        } else {
            println!("\n ** Aborted **");
        }
    } else {
        println!("\n ** NOTHING to sell **");
    }
    println!();

    trace!("auto_sell:- test: {} config_file: {}", test, config_file);
    Ok(())
}

#[derive(Debug, Clone, Default, StructOpt)]
#[structopt(
    about = "Auto sell keeping some assets as defined in the keep section of the config file"
)]
pub struct AutoSellCmdRec {
    /// full path to auto-sell configuration toml file, example: data/config-auto-cell.toml
    config_file: String,

    /// Enable test mode
    #[structopt(short = "t", long)]
    test: bool,
}

pub async fn auto_sell_cmd(
    ctx: &BinanceContext,
    rec: &AutoSellCmdRec,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("auto_sell_cmd: rec: {:#?}", rec);

    let ei = get_exchange_info(ctx).await?;
    auto_sell(ctx, &ei, &rec.config_file, rec.test).await?;

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
