use log::trace;
use std::path::PathBuf;

// Using structopt and clap v2
use structopt::{clap::AppSettings, StructOpt};

use rust_decimal::prelude::*;

// When I tried clap version 3.0.0-beta.2
// "optional" string parameters such as:
//    #[clap(short, long, required = false, env = "SECRET_KEY", default_value)]
//    pub secret_key: String,
//
// Caused an error:
//   $ cargo run
//       Finished dev [unoptimized + debuginfo] target(s) in 0.03s
//        Running `target/debug/binance-auto-sell`
//   thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', src/binance_context.rs:9:21
//   note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//use clap::{AppSettings, Clap};

#[derive(Debug, Clone, Default, StructOpt)]
#[structopt(
    global_settings = &[ AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp ],
    version = env!("CARGO_PKG_VERSION"),
)]
pub struct Opts {
    /// SECRET key
    #[structopt(long, required = false, env = "SECRET_KEY", default_value)]
    pub secret_key: String,

    /// API Key
    #[structopt(long, required = false, env = "API_KEY", default_value)]
    pub api_key: String,

    /// Order log full path
    #[structopt(
        short = "L",
        long,
        required = false,
        env = "ORDER_LOG_PATH",
        default_value = "data/order_log.txt"
    )]
    pub order_log_path: PathBuf,

    /// Symbol name such as; BNBUSD
    #[structopt(short = "S", long, required = false, default_value)]
    pub symbol: String,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Get exchange info and display parts of it
    #[structopt(short = "E", long)]
    pub get_exchange_info: bool,

    /// Get account info and display it
    #[structopt(short = "A", long)]
    pub get_account_info: bool,

    /// Get average price and display it, -P=BTCUSD
    #[structopt(short = "P", long, required = false, default_value)]
    pub get_avg_price: String,

    /// Get open orders, -O <Optional SYMBOL>. If no SYMBOL
    /// return all open orders (Expensive Weight: 40).
    #[structopt(short = "O", long)]
    pub get_open_orders: Option<Option<String>>,

    /// Get all orders, --all-orders <Optional SYMBOL>. If no SYMBOL
    /// return all orders (Expensive Weight: 5 * number of exchange symbols).
    #[structopt(long)]
    pub get_all_orders: Option<Option<String>>,

    /// Sell Symbol, --sell=BNDUSD
    #[structopt(long, required = false, default_value)]
    pub sell: String,

    /// Quantity to buy or sell
    #[structopt(long, required = false, default_value)]
    pub quantity: Decimal,

    /// Auto Sell using config file, --auto-sell=data/wink-config-auto-sell.toml
    #[structopt(long, required = false, default_value)]
    pub auto_sell: String,
}

#[derive(Debug, Clone)]
pub struct Keys {
    pub secret_key: String,
    pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct BinanceContext {
    pub opts: Opts,
    pub keys: Keys,
    pub scheme: String,
    pub domain: String,
}

impl BinanceContext {
    pub fn new() -> Self {
        let opts = if cfg!(test) {
            trace!("cfg: test");
            Opts::default()
        } else {
            trace!("cfg: not testing");
            Opts::from_args()
        };

        let order_log_path = opts.order_log_path.clone();
        if let Some(prefix) = order_log_path.parent() {
            if let Err(e) = std::fs::create_dir_all(prefix) {
                panic!("Error creating {:?} e={}", order_log_path, e);
            }
        }

        let sk = opts.secret_key.clone();
        let ak = opts.api_key.clone();
        Self {
            opts,
            keys: Keys {
                api_key: ak,
                secret_key: sk,
            },
            scheme: "https".to_string(),
            domain: "binance.us".to_string(),
        }
    }

    pub fn make_url(&self, subdomain: &str, full_path: &str) -> String {
        let sd = if !subdomain.is_empty() {
            format!("{}.", subdomain)
        } else {
            "".to_string()
        };

        format!("{}://{}{}{}", self.scheme, sd, self.domain, full_path)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_env_log::test;

    #[test]
    fn test_new() {
        let ctx = BinanceContext::new();
        assert_eq!(ctx.scheme, "https");
        assert_eq!(ctx.domain, "binance.us");
    }

    #[test]
    fn test_make_url() {
        let ctx = BinanceContext::new();
        let url = ctx.make_url("api", "/api/v3/exchangeInfo");
        assert_eq!(url, "https://api.binance.us/api/v3/exchangeInfo");

        let url = ctx.make_url("", "/api/v3/exchangeInfo");
        assert_eq!(url, "https://binance.us/api/v3/exchangeInfo");
    }
}
