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
fn de_vec_keep_rec_to_hashmap<'de, D>(
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct KeepRec {
    pub name: String,

    #[serde(default = "default_min")]
    pub min: Decimal,

    #[serde(default)]
    pub sell_to_asset: String,
}

fn default_min() -> Decimal {
    Decimal::MAX
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigurationX {
    #[serde(rename = "SECRET_KEY")]
    //#[serde(default)]
    pub secret_key: String,

    #[serde(rename = "API_KEY")]
    //#[serde(default)]
    pub api_key: String,

    pub log_path: Option<PathBuf>,

    #[serde(default = "default_sell_to_asset")]
    pub default_quote_asset: String,

    pub test: bool,

    #[serde(deserialize_with = "de_vec_keep_rec_to_hashmap")]
    //#[serde(default)]
    pub keep: HashMap<String, KeepRec>,

    pub scheme: String,

    pub domain: String,
}

fn default_sell_to_asset() -> String {
    "USD".to_string()
}

impl Default for ConfigurationX {
    fn default() -> Self {
        ConfigurationX {
            api_key: "".to_string(),
            secret_key: "".to_string(),
            log_path: None,
            default_quote_asset: default_sell_to_asset(),
            test: false,
            scheme: "".to_string(),
            domain: "".to_string(),
            keep: HashMap::<String, KeepRec>::new(),
        }
    }
}

impl ConfigurationX {
    pub fn new(matches: &ArgMatches) -> Self {
        let mut config = if let Some(path_str) = matches.value_of("config") {
            let config_file_path = PathBuf::from(path_str.to_string());
            let config: ConfigurationX = match read_to_string(config_file_path) {
                Ok(str) => match toml::from_str(&str) {
                    Ok(cfg) => {
                        trace!("config from file:\n{:#?}", cfg);
                        cfg
                    }
                    Err(_) => ConfigurationX::default(),
                },
                Err(_) => ConfigurationX::default(),
            };
            config
        } else {
            ConfigurationX::default()
        };

        config.update_config(&matches);
        trace!("config after update_config:\n{:#?}", config);

        config
    }

    fn update_config(&mut self, matches: &ArgMatches) {
        if let Some(value) = matches.value_of("api-key") {
            self.api_key = value.to_string();
        }

        if let Some(value) = matches.value_of("secret-key") {
            self.secret_key = value.to_string();
        }

        if let Some(value) = matches.value_of("log-path") {
            let path_buf = PathBuf::from(value.to_string());
            self.log_path = Some(path_buf);
        }

        if let Some(value) = matches.value_of("default-quote-asset") {
            self.default_quote_asset = value.to_string();
        }

        if matches.is_present("test") {
            self.test = true;
        }

        if let Some(value) = matches.value_of("scheme") {
            self.scheme = value.to_string();
        }

        if let Some(value) = matches.value_of("domain") {
            self.domain = value.to_string();
        }
    }
}
