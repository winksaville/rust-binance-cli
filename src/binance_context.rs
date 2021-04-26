// Using structopt and clap v2
use std::fs::{File, OpenOptions};
use structopt::{clap::AppSettings, StructOpt};
//use std::io::prelude::*;

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

//#[derive(Debug, Clap)]
//#[clap(version = env!("CARGO_PKG_VERSION"), setting = AppSettings::ColoredHelp)]
#[derive(Debug, StructOpt)]
//#[structopt(version = env!("CARGO_PKG_VERSION"), setting = AppSettings::ColoredHelp, AppSettings::ArgRequiredElseHelp)]
#[structopt(
    global_settings = &[ AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp ],
    version = env!("CARGO_PKG_VERSION"),
)]
pub struct Opts {
    /// SECRET key
    //#[clap(short, long, required = false, env = "SECRET_KEY", default_value)]
    #[structopt(short, long, required = false, env = "SECRET_KEY", default_value)]
    pub secret_key: String,

    /// API key
    //#[clap(short, long, required = false, env = "SECRET_KEY", default_value)]
    #[structopt(short, long, required = false, env = "API_KEY", default_value)]
    pub api_key: String,

    /// Symbol name such as; BNBUSD
    #[structopt(short = "S", long, required = false, default_value)]
    pub symbol: String,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    //#[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Get exchange info and display parts of it
    #[structopt(short = "E", long)]
    //#[clap(short, long)]
    pub get_exchange_info: bool,

    /// Get account info and display it
    #[structopt(short = "A", long)]
    //#[clap(short, long)]
    pub get_account_info: bool,

    /// Sell Symbol, --sell=BNDUSD
    #[structopt(long, required = false, default_value)]
    //#[clap(long)]
    pub sell: String,

    /// Quantity to buy or sell
    #[structopt(long, required = false, default_value)]
    //#[clap(long)]
    pub quantity: f64,
}

pub struct BinanceContext {
    pub opts: Opts,
    pub order_log_file: File,
    pub scheme: String,
    pub domain: String,
}

impl BinanceContext {
    pub fn new() -> Self {
        Self {
            opts: Opts::from_args(),
            order_log_file: match OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open("data/order_logger.txt")
            {
                Ok(file) => file,
                Err(e) => panic!("Could not create order_logger.txt e: {}", e),
            },
            //opts: Opts::parse(),
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
