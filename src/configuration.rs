// Based on https://stackoverflow.com/a/55134333/4812090
use crate::common::InternalErrorRec;
use crate::ier_new;
use clap::ArgMatches;
use core::mem::size_of;
use log::trace;
use rust_decimal::Decimal;
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::error::Error;
use std::{collections::HashMap, fmt, fs::read_to_string, path::PathBuf};

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
// TODO: Maybe a process macro can be created that generates de_vec_xxx_to_hashmap?
fn de_vec_keep_rec_to_hashmap<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, KeepRec>>, D::Error>
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

    // println!("dev_vec_keep_rec_to_hashmap: in Visitor");
    let result = deserializer.deserialize_seq(ItemsVisitor);

    match result {
        Ok(v) => Ok(Some(v)),
        Err(e) => Err(e),
    }
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
fn de_vec_buy_rec_to_hashmap<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, BuyRec>>, D::Error>
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

    // println!("dev_vec_buy_rec_to_hashmap: in Visitor");
    let result = deserializer.deserialize_seq(ItemsVisitor);

    match result {
        Ok(v) => Ok(Some(v)),
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BuyRec {
    pub name: String,

    pub percent: Decimal,

    #[serde(default)]
    pub quote_asset: String,
}

#[derive(Clone, Default, Deserialize, PartialEq)]
pub struct Keys {
    #[serde(rename = "SECRET_KEY")]
    pub secret_key: Option<String>,

    #[serde(rename = "API_KEY")]
    pub api_key: Option<String>,
}

impl Keys {
    //#[inline(always)]
    //#[inline(never)]
    pub fn get_ak_or_err(&self) -> Result<&str, Box<dyn Error>> {
        match &self.api_key {
            Some(ak) => Ok(ak.as_str()),
            None => Err("No api-key".into()),
        }
    }

    #[allow(unused)]
    //#[inline(always)]
    //#[inline(never)]
    pub fn get_ak_vec_u8_or_err(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self.get_ak_or_err()?.to_string().into_bytes())
    }

    //#[inline(always)]
    //#[inline(never)]
    pub fn get_sk_or_err(&self) -> Result<&str, Box<dyn Error>> {
        match &self.secret_key {
            Some(sk) => Ok(sk.as_str()),
            None => Err("No secret-key".into()),
        }
    }

    //#[inline(always)]
    //#[inline(never)]
    pub fn get_sk_vec_u8_or_err(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self.get_sk_or_err()?.to_string().into_bytes())
    }
}

/// Never accidentally output the secret_key when doing debug output
impl fmt::Debug for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const BEG_LEN: usize = 6;
        let mut beg_api_key: String = String::with_capacity(size_of::<char>() * BEG_LEN);
        // TODO: Why do I have to clone the api_key String?
        if let Some(api_key) = self.api_key.clone() {
            for (i, ch) in api_key.chars().enumerate() {
                if i >= BEG_LEN {
                    break;
                }
                beg_api_key.push(ch);
            }
        } else {
            beg_api_key = "None".to_string();
        }
        f.debug_struct("Keys")
            .field("secret_key", &"******".to_string())
            .field("api_key", &beg_api_key)
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    #[serde(flatten)]
    pub keys: Keys,

    #[serde(default)]
    pub order_log_path: Option<PathBuf>,

    #[serde(default = "default_quote_asset")]
    pub default_quote_asset: String,

    #[serde(default = "default_test")]
    pub test: bool,

    // Used to enable/disable verbose output, currently
    #[serde(default = "default_verbose")]
    pub verbose: bool,

    // Used to enable/disable progress_info output
    #[serde(default = "default_progress_info")]
    pub progress_info: bool,

    #[serde(default = "default_throttle_rate_ms")]
    pub throttle_rate_ms: u64,

    #[serde(default = "default_confirmation_required")]
    pub confirmation_required: bool,

    #[serde(default)]
    #[serde(deserialize_with = "de_vec_keep_rec_to_hashmap")]
    pub keep: Option<HashMap<String, KeepRec>>,

    #[serde(default)]
    #[serde(deserialize_with = "de_vec_buy_rec_to_hashmap")]
    pub buy: Option<HashMap<String, BuyRec>>,

    #[serde(default = "default_scheme")]
    pub scheme: String,

    #[serde(default = "default_domain")]
    pub domain: String,

    pub xyz: Option<String>,
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

fn default_test() -> bool {
    true
}

fn default_verbose() -> bool {
    true
}

fn default_progress_info() -> bool {
    true
}

fn default_throttle_rate_ms() -> u64 {
    500
}

fn default_confirmation_required() -> bool {
    true
}

fn default_keys() -> Keys {
    Keys {
        secret_key: Some("secret-key".to_owned()),
        api_key: Some("api-key".to_owned()),
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            keys: default_keys(),
            order_log_path: None,
            default_quote_asset: default_quote_asset(),
            test: default_test(),
            verbose: default_verbose(),
            progress_info: default_progress_info(),
            throttle_rate_ms: default_throttle_rate_ms(),
            confirmation_required: default_confirmation_required(),
            scheme: default_scheme(),
            domain: default_domain(),
            keep: None,
            buy: None,
            xyz: None,
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
                    Err(e) => {
                        return Err(
                            ier_new!(9, &format!("Error processing {}: {}", path_str, e)).into(),
                        )
                    }
                },
                Err(e) => match e.kind() {
                    std::io::ErrorKind::NotFound => Configuration::default(),
                    _ => {
                        return Err(
                            ier_new!(9, &format!("Error reading {}: {}", path_str, e)).into()
                        );
                    }
                },
            };
            config
        } else {
            Configuration::default()
        };

        config.update_config(matches);
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
            self.keys.api_key = Some(value.to_string());
        }

        if let Some(value) = matches.value_of("secret-key") {
            self.keys.secret_key = Some(value.to_string());
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

        if matches.is_present("verbose") {
            self.verbose = true;
        }

        if matches.is_present("no-verbose") {
            self.verbose = false;
        }

        if matches.is_present("progress-info") {
            self.progress_info = true;
        }

        if matches.is_present("no-progress-info") {
            self.progress_info = false;
        }

        if let Some(value) = matches.value_of("throttle-rate-ms") {
            self.throttle_rate_ms = value.parse().unwrap();
        }

        if matches.is_present("confirmation-required") {
            self.confirmation_required = true;
        }
        if matches.is_present("no-confirmation-required") {
            self.confirmation_required = false;
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
        assert_eq!(config.keys.api_key, None);
        assert_eq!(config.keys.secret_key, None);
        assert!(config.order_log_path.is_none());
        assert_eq!(config.default_quote_asset, "USD");
        assert_eq!(config.scheme, "https");
        assert_eq!(config.domain, "binance.us");
        assert_eq!(config.test, true);
        assert_eq!(config.verbose, true);
        assert_eq!(config.confirmation_required, true);
        assert!(config.keep.is_none());
        assert!(config.buy.is_none());
    }

    #[test]
    fn test_config_default() {
        let config = Configuration::default();
        // println!("{:#?}", config);
        assert_eq!(config.keys.api_key, Some("api-key".to_owned()));
        assert_eq!(config.keys.secret_key, Some("secret-key".to_owned()));
        assert!(config.order_log_path.is_none());
        assert_eq!(config.default_quote_asset, "USD");
        assert_eq!(config.scheme, "https");
        assert_eq!(config.domain, "binance.us");
        assert_eq!(config.test, true);
        assert_eq!(config.verbose, true);
        assert_eq!(config.confirmation_required, true);
        assert!(config.keep.is_none());
        assert!(config.buy.is_none());
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
        assert_eq!(config.keys.api_key, Some("api key".to_string()));
        assert_eq!(config.keys.secret_key, Some("secret key".to_string()));
        assert!(config.order_log_path.is_none()); // The default
        assert_eq!(config.default_quote_asset, "USD"); // The default
        assert_eq!(config.scheme, "https"); // The default
        assert_eq!(config.domain, "binance.us"); // The default
        assert_eq!(config.test, true); // The default
        assert_eq!(config.verbose, true); // The default
        assert_eq!(config.confirmation_required, true); // The default
        let brs = &config.buy.unwrap();
        assert_eq!(
            brs.get("ABC").unwrap(),
            &BuyRec {
                name: "ABC".to_string(),
                percent: dec!(20),
                quote_asset: "".to_string(),
            }
        );
        assert_eq!(
            brs.get("DEF").unwrap(),
            &BuyRec {
                name: "DEF".to_string(),
                percent: dec!(23.5),
                quote_asset: "USD".to_string(),
            }
        );
    }

    const TOML_DATA_KEEP: &str = r#"
        API_KEY = "api key"
        SECRET_KEY = "secret key"
        order_log_path = "data/xyz.txt"
        default_quote_asset="BTC"
        test = true
        confirmation_required = false
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
        assert_eq!(config.keys.api_key, Some("api key".to_string()));
        assert_eq!(config.keys.secret_key, Some("secret key".to_string()));
        assert_eq!(
            config.order_log_path,
            Some(PathBuf::from("data/xyz.txt".to_string()))
        );
        assert_eq!(config.default_quote_asset, "BTC");
        assert_eq!(config.scheme, "http");
        assert_eq!(config.domain, "binance.com");
        assert_eq!(config.test, true);
        assert_eq!(config.verbose, true);
        assert_eq!(config.confirmation_required, false);
        let krs = &config.keep.unwrap();
        assert_eq!(
            krs.get("USD").unwrap(),
            &KeepRec {
                name: "USD".to_string(),
                min: Decimal::MAX,
                quote_asset: "".to_string()
            }
        );
        assert_eq!(
            krs.get("USDT").unwrap(),
            &KeepRec {
                name: "USDT".to_string(),
                min: Decimal::MAX,
                quote_asset: "".to_string()
            }
        );
        assert_eq!(
            krs.get("USDC").unwrap(),
            &KeepRec {
                name: "USDC".to_string(),
                min: Decimal::MAX,
                quote_asset: "".to_string()
            }
        );
        assert_eq!(
            krs.get("BNB").unwrap(),
            &KeepRec {
                name: "BNB".to_string(),
                min: dec!(500),
                quote_asset: "".to_string()
            }
        );

        // ABC says sell everything to BTC
        assert_eq!(
            krs.get("ABC").unwrap(),
            &KeepRec {
                name: "ABC".to_string(),
                min: dec!(0),
                quote_asset: "BTC".to_string()
            }
        );

        // XYZ is odd as nothing will be sold since KeepRec.min default is MAX so quote_asset is ignored
        assert_eq!(
            krs.get("XYZ").unwrap(),
            &KeepRec {
                name: "XYZ".to_string(),
                min: Decimal::MAX,
                quote_asset: "BNB".to_string()
            }
        );
    }
}
