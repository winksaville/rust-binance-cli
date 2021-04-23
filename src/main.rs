//use clap::{AppSettings, Clap};
use log::trace;

#[allow(unused)]
use serde::{Deserialize, Serialize};

mod de_string_or_number;
#[allow(unused)]
use de_string_or_number::{
    de_string_or_number_to_f64, de_string_or_number_to_i64, de_string_or_number_to_u64,
};

mod de_vec_to_hashmap;
#[allow(unused)]
use de_vec_to_hashmap::de_vec_to_hashmap;

mod binance_context;
use binance_context::BinanceContext;

mod exchange_info;
use exchange_info::ExchangeInfo;

mod account_info;
#[allow(unused)]
use account_info::AccountInfo;

async fn get_exchange_info<'e>(
    ctx: &BinanceContext,
) -> Result<ExchangeInfo, Box<dyn std::error::Error>> {
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
    trace!("main: +");

    let ctx = BinanceContext::new();

    // For now we'll always do this so we pass our tests
    #[allow(unused)]
    let sec_key: Vec<u8> = ctx.opts.secret_key.as_bytes().to_vec();
    let api_key: Vec<u8> = ctx.opts.api_key.as_bytes().to_vec();
    if !sec_key.is_empty() || !api_key.is_empty() {
        println!(
            "sec_key=secret key is never displayed api_key={}",
            if api_key.is_empty() {
                "len is 0"
            } else {
                std::str::from_utf8(&api_key).unwrap()
            }
        );
    }

    if std::env::args().len() == 1 {
        let args: Vec<String> = std::env::args().collect();
        let prog_name = std::path::Path::new(&args[0]).file_name();
        let name = match prog_name {
            Some(pn) => match pn.to_str() {
                Some(n) => n,
                None => &args[0],
            },
            None => &args[0],
        };
        println!("Usage: {} -h or --help", name);
        return Ok(());
    }

    if ctx.opts.get_exchange_info || !ctx.opts.symbol.is_empty() {
        let ei = get_exchange_info(&ctx).await?;

        if ctx.opts.get_exchange_info {
            println!("ei={:#?}", ei);
        }

        if !ctx.opts.symbol.is_empty() {
            let sym = ei.get_symbol(&ctx.opts.symbol);
            match sym {
                Some(sym) => println!("{}={:#?}", sym.symbol, sym),
                None => println!("{} not found", ctx.opts.symbol),
            }
        }
    }

    trace!("main: -");

    Ok(())
}
