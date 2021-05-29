// Based on https://stackoverflow.com/a/55134333/4812090
use clap::ArgMatches;
use log::trace;
use rust_decimal::Decimal;
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
// TODO: Maybe a process macro can be created that generates de_vec_xxx_to_hashmap?
fn de_vec_keep_rec_to_hashmap<'de, D>(deserializer: D) -> Result<HashMap<String, KeepRec>, D::Error>
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct KeepRec {
    pub name: String,

    #[serde(default = "default_min")]
    pub min: Decimal,

    #[serde(default)]
    pub quote_asset: String,
}

fn default_min() -> Decimal {
    Decimal::MAX
}

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
// TODO: Maybe a process macro can be created that generates de_vec_xxx_to_hashmap?
fn de_vec_buy_rec_to_hashmap<'de, D>(deserializer: D) -> Result<HashMap<String, BuyRec>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = HashMap<String, BuyRec>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<String, BuyRec>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<String, BuyRec> =
                HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<BuyRec>()? {
                // println!("item={:#?}", item);
                map.insert(item.name.clone(), item);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BuyRec {
    pub name: String,

    #[serde(default)]
    pub percent: Decimal,

    #[serde(default)]
    pub quote_asset: String,

    #[serde(default, skip)]
    pub buy_qty: Decimal,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    #[serde(rename = "SECRET_KEY")]
    #[serde(default)]
    pub secret_key: String,

    #[serde(rename = "API_KEY")]
    #[serde(default)]
    pub api_key: String,

    #[serde(default)]
    pub order_log_path: Option<PathBuf>,

    #[serde(default = "default_quote_asset")]
    pub default_quote_asset: String,

    #[serde(default)]
    pub test: bool,

    #[serde(deserialize_with = "de_vec_keep_rec_to_hashmap")]
    #[serde(default)]
    pub keep: HashMap<String, KeepRec>,

    #[serde(deserialize_with = "de_vec_buy_rec_to_hashmap")]
    #[serde(default)]
    pub buy: HashMap<String, BuyRec>,

    #[serde(default = "default_scheme")]
    pub scheme: String,

    #[serde(default = "default_domain")]
    pub domain: String,
}

fn default_quote_asset() -> String {
    "USD".to_string()
}

fn default_scheme() -> String {
    "https".to_string()
}

fn default_domain() -> String {
    "binance.us".to_string()
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            api_key: "".to_string(),
            secret_key: "".to_string(),
            order_log_path: None,
            default_quote_asset: default_quote_asset(),
            test: false,
            scheme: default_scheme(),
            domain: default_domain(),
            keep: HashMap::<String, KeepRec>::new(),
            buy: HashMap::<String, BuyRec>::new(),
        }
    }
}

impl Configuration {
    pub fn new(matches: &ArgMatches) -> Result<Self, Box<dyn std::error::Error>> {
        let opt_config = matches.value_of("config");
        trace!("Configuration::new: opt_config={:#?}", opt_config);
        let mut config = if let Some(path_str) = opt_config {
            let config_file_path = PathBuf::from(path_str.to_string());
            let config: Configuration = match read_to_string(config_file_path) {
                Ok(str) => match toml::from_str(&str) {
                    Ok(cfg) => {
                        trace!("config from file:\n{:#?}", cfg);
                        cfg
                    }
                    Err(e) => return Err(format!("Error processing {}: {}", path_str, e).into()),
                },
                Err(e) => return Err(format!("Error reading {}: {}", path_str, e).into()),
            };
            config
        } else {
            Configuration::default()
        };

        config.update_config(&matches);
        trace!("config after update_config:\n{:#?}", config);

        Ok(config)
    }

    pub fn make_url(&self, subdomain: &str, full_path: &str) -> String {
        let sd = if !subdomain.is_empty() {
            format!("{}.", subdomain)
        } else {
            "".to_string()
        };

        format!("{}://{}{}{}", self.scheme, sd, self.domain, full_path)
    }

    /// This updates configuration only from global options.
    // For instance I looked at cloning them into each subcommand on an
    // as-needed-basis but then this function doesn't find any of them
    // and the configuration is not updated.
    fn update_config(&mut self, matches: &ArgMatches) {
        if let Some(value) = matches.value_of("api-key") {
            self.api_key = value.to_string();
        }

        if let Some(value) = matches.value_of("secret-key") {
            self.secret_key = value.to_string();
        }

        if let Some(value) = matches.value_of("order-log-path") {
            let path_buf = PathBuf::from(value.to_string());
            self.order_log_path = Some(path_buf);
        }

        if let Some(value) = matches.value_of("default-quote-asset") {
            self.default_quote_asset = value.to_string();
        }

        if matches.is_present("test") {
            self.test = true;
        }

        if matches.is_present("no-test") {
            self.test = false;
        }

        if let Some(value) = matches.value_of("scheme") {
            self.scheme = value.to_string();
        }

        if let Some(value) = matches.value_of("domain") {
            self.domain = value.to_string();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::configuration::BuyRec;

    use super::*;

    use rust_decimal_macros::dec;
    use toml;

    #[test]
    fn test_config_empty() {
        let config: Configuration = toml::from_str("").unwrap();
        // println!("{:#?}", config);
        assert_eq!(config.api_key, "");
        assert_eq!(config.secret_key, "");
        assert!(config.order_log_path.is_none());
        assert_eq!(config.default_quote_asset, "USD");
        assert_eq!(config.scheme, "https");
        assert_eq!(config.domain, "binance.us");
        assert_eq!(config.test, false);
        assert!(config.keep.is_empty());
        assert!(config.buy.is_empty());
    }

    const TOML_DATA: &str = r#"
        API_KEY = "api key"
        SECRET_KEY = "secret key"

        buy = [
            { name = "ABC", percent = 20 },
            { name = "DEF", percent = 23.5, quote_asset = "USD" },
        ]
    "#;

    #[test]
    fn test_config_buy() {
        let config: Configuration = toml::from_str(TOML_DATA).unwrap();
        // println!("{:#?}", config);
        assert_eq!(config.api_key, "api key");
        assert_eq!(config.secret_key, "secret key");
        assert!(config.order_log_path.is_none()); // The default
        assert_eq!(config.default_quote_asset, "USD"); // The default
        assert_eq!(config.scheme, "https"); // The default
        assert_eq!(config.domain, "binance.us"); // The default
        assert_eq!(config.test, false); // The default
        assert_eq!(
            config.buy.get("ABC").unwrap(),
            &BuyRec {
                name: "ABC".to_string(),
                percent: dec!(20),
                quote_asset: "".to_string(),
                buy_qty: dec!(0),
            }
        );
        assert_eq!(
            config.buy.get("DEF").unwrap(),
            &BuyRec {
                name: "DEF".to_string(),
                percent: dec!(23.5),
                quote_asset: "USD".to_string(),
                buy_qty: dec!(0),
            }
        );
    }

    const TOML_DATA_KEEP: &str = r#"
        API_KEY = "api key"
        SECRET_KEY = "secret key"
        order_log_path = "data/xyz.txt"
        default_quote_asset="BTC"
        test = true
        scheme = "http"
        domain = "binance.com"

        keep = [
            { name = "USD" },
            { name = "USDT" },
            { name = "USDC" },
            { name = "BNB", min = 500 },
            { name = "ABC", min = 0, quote_asset = "BTC" },
            { name = "XYZ", quote_asset = "BNB" },
        ]
    "#;

    #[test]
    fn test_config_keep() {
        let config: Configuration = toml::from_str(TOML_DATA_KEEP).unwrap();
        // println!("{:#?}", config);
        assert_eq!(config.api_key, "api key");
        assert_eq!(config.secret_key, "secret key");
        assert_eq!(
            config.order_log_path,
            Some(PathBuf::from("data/xyz.txt".to_string()))
        );
        assert_eq!(config.default_quote_asset, "BTC");
        assert_eq!(config.scheme, "http");
        assert_eq!(config.domain, "binance.com");
        assert_eq!(config.test, true);
        assert_eq!(
            config.keep.get("USD").unwrap(),
            &KeepRec {
                name: "USD".to_string(),
                min: Decimal::MAX,
                quote_asset: "".to_string()
            }
        );
        assert_eq!(
            config.keep.get("USDT").unwrap(),
            &KeepRec {
                name: "USDT".to_string(),
                min: Decimal::MAX,
                quote_asset: "".to_string()
            }
        );
        assert_eq!(
            config.keep.get("USDC").unwrap(),
            &KeepRec {
                name: "USDC".to_string(),
                min: Decimal::MAX,
                quote_asset: "".to_string()
            }
        );
        assert_eq!(
            config.keep.get("BNB").unwrap(),
            &KeepRec {
                name: "BNB".to_string(),
                min: dec!(500),
                quote_asset: "".to_string()
            }
        );

        // ABC says sell everything to BTC
        assert_eq!(
            config.keep.get("ABC").unwrap(),
            &KeepRec {
                name: "ABC".to_string(),
                min: dec!(0),
                quote_asset: "BTC".to_string()
            }
        );

        // XYZ is odd as nothing will be sold since KeepRec.min default is MAX so quote_asset is ignored
        assert_eq!(
            config.keep.get("XYZ").unwrap(),
            &KeepRec {
                name: "XYZ".to_string(),
                min: Decimal::MAX,
                quote_asset: "BNB".to_string()
            }
        );
    }
}
