use clap::{AppSettings, Clap};
use log::trace;

#[allow(unused)]
use serde::{Deserialize, Serialize};

mod de_string_or_number;
#[allow(unused)]
use de_string_or_number::{
    de_string_or_number_to_f64, de_string_or_number_to_i64, de_string_or_number_to_u64,
};

mod binance_context;
use binance_context::BinanceContext;

mod exchange_info;
#[allow(unused)]
use exchange_info::ExchangeInfo;

#[derive(Debug, Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Cli {
    #[clap(short, long, env = "SECRET_KEY")]
    secret_key: String,

    #[clap(short, long, env = "API_KEY")]
    api_key: String,

    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

async fn get_exchange_info<'e>(
    ctx: &BinanceContext,
) -> Result<ExchangeInfo<'e>, Box<dyn std::error::Error>> {
    trace!("get_exchange_info: +");

    let url = ctx.make_url("api", "/api/v3/exchangeInfo");
    trace!("get_exchange_info: url={}", url);

    let resp = reqwest::Client::new().get(url).send().await?.text().await?;
    let exchange_info: ExchangeInfo = serde_json::from_str(&resp)?;

    trace!("get_exchange_info: -");
    Ok(exchange_info)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    trace!("+");

    let ctx = BinanceContext::new();

    let args = Cli::parse();

    #[allow(unused)]
    let sec_key: Vec<u8> = args.secret_key.as_bytes().to_vec();
    let api_key: Vec<u8> = args.api_key.as_bytes().to_vec();
    println!(
        "sec_key=secret key is never displayed api_key={}",
        std::str::from_utf8(&api_key).unwrap(),
    );

    let ei = get_exchange_info(&ctx).await?;
    println!("ei.server_time={:#?}", ei.server_time);

    // let eihm = ei.symbols_to_map();
    // println!("eihm.len()={}", eihm.len());
    // if let Some(sym_bnb) = eihm.get("BNBUSD") {
    //     println!("sym_bnb={:#?}", sym_bnb);
    // }

    trace!("-");
    Ok(())
}
