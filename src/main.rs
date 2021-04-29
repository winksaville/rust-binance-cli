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
use account_info::AccountInfo;

mod binance_signature;
use binance_signature::{append_signature, binance_signature, query_vec_u8};

mod order_response;
#[allow(unused)]
use order_response::OrderResponse;

mod binance_trade;
use binance_trade::{binance_new_order_or_test, MarketQuantityType, OrderType, Side};

mod binance_avg_price;
use binance_avg_price::{get_avg_price, AvgPrice};

mod common;
use common::{time_ms_to_utc, utc_now_to_time_ms};

async fn get_account_info<'e>(
    ctx: &BinanceContext,
) -> Result<AccountInfo, Box<dyn std::error::Error>> {
    trace!("get_account_info: +");

    let sig_key = ctx.opts.secret_key.as_bytes();
    let api_key = ctx.opts.api_key.as_bytes();

    let mut params = vec![];
    let ts_string: String = format!("{}", utc_now_to_time_ms());
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
        //println!("ai={:#?}", ai);
        println!("     account_type: {}", ai.account_type);
        println!("      can_deposit: {}", ai.can_deposit);
        println!("        can_trade: {}", ai.can_trade);
        println!("     can_withdraw: {}", ai.can_withdraw);
        println!(" buyer_commission: {}", ai.buyer_commission);
        println!(" maker_commission: {}", ai.maker_commission);
        println!("seller_commission: {}", ai.seller_commission);
        println!(" taker_commission: {}", ai.taker_commission);
        println!("      update_time: {}", time_ms_to_utc(ai.update_time));
        println!("      permissions: {:?}", ai.permissions);
        for balance in ai.balances {
            if balance.free > 0.0 || balance.locked > 0.0 {
                println!(
                    "{}: free: {} locked: {}",
                    balance.asset, balance.free, balance.locked
                );
            }
        }
    }

    if !ctx.opts.get_avg_price.is_empty() {
        let ap: AvgPrice = get_avg_price(&ctx, &ctx.opts.get_avg_price).await?;
        println!("ap: mins={} price={}", ap.mins, ap.price);
    }

    if !ctx.opts.sell.is_empty() {
        let symbol = ctx.opts.sell.clone();
        let quantity = ctx.opts.quantity;
        if quantity <= 0.0 {
            return Err(format!("Can't sell {} quantity", quantity).into());
        }
        let response = binance_new_order_or_test(
            ctx,
            &symbol,
            Side::SELL,
            OrderType::Market(MarketQuantityType::Quantity(quantity)),
            true,
        )
        .await?;
        println!("Sell reponse: {:#?}", response);
    }

    trace!("main: -");
    Ok(())
}
