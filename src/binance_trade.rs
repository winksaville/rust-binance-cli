use log::trace;

use crate::common::{BinanceError, ResponseErrorRec};

use crate::binance_order_response::{FullTradeResponseRec, TradeResponse};

use crate::binance_signature::{append_signature, binance_signature, query_vec_u8};

use crate::binance_context::BinanceContext;

use crate::common::{utc_now_to_time_ms, Side};
pub enum MarketQuantityType {
    Quantity(f64),
    //QuoteOrderQty(f64),
}

pub enum TradeOrderType {
    Market(MarketQuantityType),
    // Limit,
    // StopLoss,
    // StopLossLimit,
    // TakeProfit,
    // TakeProfitLimit,
    // LimitMaker,
}

pub async fn binance_new_order_or_test(
    mut ctx: BinanceContext,
    symbol: &str,
    side: Side,
    order_type: TradeOrderType,
    test: bool,
) -> Result<TradeResponse, Box<dyn std::error::Error>> {
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
        TradeOrderType::Market(MarketQuantityType::Quantity(qty)) => {
            params.push(("type", "MARKET"));
            astring = format!("{}", qty);
            params.push(("quantity", &astring));
        } //_ => return Err("Unknown order_type")?,
    };

    let ts_string: String = format!("{}", utc_now_to_time_ms());
    params.push(("timestamp", ts_string.as_str()));

    trace!("binanace_new_order_or_test: params={:#?}", params);

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data is qs and query as body
    let signature = binance_signature(&sig_key, &[], &query);

    // Append the signature to query
    append_signature(&mut query, signature);

    // Convert to a string
    let query_string = String::from_utf8(query)?;
    trace!("query_string={}", &query_string);

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
        .body(query_string.clone());
    trace!("req_builder={:#?}", req_builder);

    // Send and get response
    let response = req_builder.send().await?;
    trace!("response={:#?}", &response);

    let response_status = response.status();
    let response_body = response.text().await?;

    // Log the response
    let result = if response_status == 200 {
        trace!("response_body={}", response_body);
        let mut order_resp_success = FullTradeResponseRec::default();
        if !test {
            order_resp_success = serde_json::from_str(&&response_body)?;
        } else {
            order_resp_success.test = true;
        }
        order_resp_success.query = query_string;
        let order_resp = TradeResponse::SuccessFull(order_resp_success);
        trace!(
            "binance_market_order_or_test: symbol={} side={} test={} order_response={:#?}",
            symbol,
            side_str,
            test,
            order_resp
        );
        //Ok(ctx.log_order_response(&order_resp)?)
        ctx.log_order_response(&order_resp)?;

        Ok(order_resp)
    } else {
        let response_error_rec = ResponseErrorRec::new(
            test,
            response_status.as_u16(),
            &query_string,
            &response_body,
        );
        let binance_error_response = BinanceError::Response(response_error_rec);
        let order_resp = TradeResponse::Failure(binance_error_response.clone());
        ctx.log_order_response(&order_resp)?;

        trace!(
            "{}",
            format!(
                "binance_market_order_or_test: symbol={} side={} test={} order_resp={:#?}",
                symbol, side_str, test, order_resp
            )
        );

        Err(binance_error_response.into())
    };

    result
}
