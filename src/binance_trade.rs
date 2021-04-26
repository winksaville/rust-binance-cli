use strum_macros::IntoStaticStr;

use crate::order_response::OrderResponse;

use crate::binance_signature::{append_signature, binance_signature, query_vec_u8};

use crate::binance_context::BinanceContext;

use crate::common::utc_now_to_time_ms;

pub enum MarketQuantityType {
    Quantity(f64),
    //QuoteOrderQty(f64),
}

#[derive(IntoStaticStr)]
pub enum OrderType {
    Market(MarketQuantityType),
    // Limit,
    // StopLoss,
    // StopLossLimit,
    // TakeProfit,
    // TakeProfitLimit,
    // LimitMaker,
}

#[derive(IntoStaticStr)]
#[allow(clippy::upper_case_acronyms)]
pub enum Side {
    BUY,
    SELL,
}

pub async fn binance_new_order_or_test(
    ctx: BinanceContext,
    symbol: &str,
    side: Side,
    order_type: OrderType,
    test: bool,
) -> Result<OrderResponse, Box<dyn std::error::Error>> {
    const DEBUG: bool = true;

    let sig_key = ctx.opts.secret_key.as_bytes();
    let api_key = ctx.opts.api_key.as_bytes();

    let side_str: &str = side.into();
    let mut params = vec![
        ("recvWindow", "5000"),
        ("symbol", symbol),
        ("side", side_str),
    ];

    let astring: String;
    match order_type {
        OrderType::Market(MarketQuantityType::Quantity(qty)) => {
            params.push(("type", "MARKET"));
            astring = format!("{}", qty);
            params.push(("quantity", &astring));
        } //_ => return Err("Unknown order_type")?,
    };

    let ts_string: String = format!("{}", utc_now_to_time_ms());
    params.push(("timestamp", ts_string.as_str()));

    if DEBUG {
        println!("binanace_new_order_or_test: params={:#?}", params);
    }

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data is qs and query as body
    let signature = binance_signature(&sig_key, &[], &query);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query).unwrap();
    if DEBUG {
        println!("query_string={}", &query_string);
    }

    let path = if test {
        "/api/v3/order/test"
    } else {
        "/api/v3/order"
    };
    let url = "https://api.binance.us".to_string() + path;

    // Build request
    let client = reqwest::Client::builder();
    let req_builder = client
        //.proxy(reqwest::Proxy::https("http://localhost:8080")?)
        .build()?
        .post(url)
        .header("X-MBX-APIKEY", api_key)
        .body(query_string);
    if DEBUG {
        println!("req_builder={:#?}", req_builder);
    }

    // Send and get response
    let response = req_builder.send().await?;
    if DEBUG {
        println!("response={:#?}", &response);
    }
    let response_status = response.status();
    let response_body = response.text().await?;
    if response_status == 200 {
        if DEBUG {
            println!("response_body={}", response_body);
        }
        let mut order_resp = OrderResponse::default();
        if !test {
            order_resp = serde_json::from_str(&&response_body)?;
        } else {
            order_resp.test = true;
        }
        if DEBUG {
            println!(
                "binance_market_order_or_test: symbol={} side={} test={} order_response={:#?}",
                symbol, side_str, test, order_resp
            );
        }
        Ok(order_resp)
    } else {
        return Err(format!(
            "Error response status={} symbol={} side={} body={}",
            response_status, symbol, side_str, response_body
        )
        .into());
    }
}
