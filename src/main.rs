//use clap::{AppSettings, Clap};
use log::trace;

use chrono::prelude::Utc;

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

mod binance_signature;
#[allow(unused)]
use binance_signature::{binance_signature, query_vec_u8};

fn time_ms_utc_now() -> i64 {
    let now = Utc::now();
    now.timestamp_millis()
}

fn append_signature(query: &mut Vec<u8>, signature: [u8; 32]) {
    let signature_string = hex::encode(&signature);
    trace!("signature_string={}", signature_string);

    let signature_params = vec![("signature", signature_string.as_str())];
    trace!("signature_params={:?}", signature_params);
    query.append(&mut vec![b'&']);
    query.append(&mut query_vec_u8(&signature_params));
}

async fn get_account_info<'e>(
    ctx: &BinanceContext,
) -> Result<AccountInfo, Box<dyn std::error::Error>> {
    trace!("get_account_info: +");

    let sig_key = ctx.opts.secret_key.as_bytes();
    let api_key = ctx.opts.api_key.as_bytes();

    let mut params = vec![];
    let ts_string: String = format!("{}", time_ms_utc_now());
    params.append(&mut vec![("timestamp", ts_string.as_str())]);

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data in qs and query as body
    let signature = binance_signature(&sig_key, &[], &query);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("query_string={}", &query_string);

    let url = ctx.make_url("api", &format!("/api/v3/account?{}", &query_string));
    trace!("get_exchange_info: url={}", url);

    // Build request
    let client = reqwest::Client::builder();
    let req_builder = client
        //.proxy(reqwest::Proxy::https("http://localhost:8080")?)
        .build()?
        .get(url)
        .header("X-MBX-APIKEY", api_key);
    trace!("req_builder={:#?}", req_builder);

    // Send and get response
    let response = req_builder.send().await?;
    trace!("response={:#?}", response);
    let response_status = response.status();
    let response_body = response.text().await?;
    let account_info: AccountInfo = if response_status == 200 {
        let ai: AccountInfo = match serde_json::from_str(&response_body) {
            Ok(info) => info,
            Err(e) => {
                let err = format!(
                    "Error converting body to AccountInfo: e={} body={}",
                    e, response_body
                );
                trace!("get_account_info: err: {}", err);
                return Err(err.into());
            }
        };
        ai
    } else {
        let err = format!("response status={} body={}", response_status, response_body);
        trace!("get_account_info: err: {}", err);
        return Err(err.into());
    };

    trace!("get_account_info: err: -");
    Ok(account_info)
}

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

    if ctx.opts.get_account_info {
        let ai = get_account_info(&ctx).await?;
        println!("ai={:#?}", ai);
    }

    trace!("main: -");
    Ok(())
}
