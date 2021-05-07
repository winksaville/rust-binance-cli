use log::trace;

use rust_decimal::prelude::*;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::common::{post_req_get_response, BinanceError, ResponseErrorRec};

use crate::binance_order_response::{FullTradeResponseRec, TradeResponse};

use crate::binance_signature::{append_signature, binance_signature, query_vec_u8};

use crate::binance_context::BinanceContext;

use crate::common::{utc_now_to_time_ms, Side};
pub enum MarketQuantityType {
    Quantity(Decimal),
    //QuoteOrderQty(Decimal),
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

struct OrderLogger {
    file: File,
}

impl OrderLogger {
    fn new(order_log_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(prefix) = order_log_path.parent() {
            if let Err(e) = std::fs::create_dir_all(prefix) {
                panic!("Error creating {:?} e={}", order_log_path, e);
            }
        }

        let order_log_file: File = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(order_log_path)
        {
            Ok(file) => file,
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(OrderLogger {
            file: order_log_file,
        })
    }

    fn log_order_response(
        &mut self,
        order_response: &TradeResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        serde_json::to_writer(&self.file, order_response)?;
        self.file.write_all(b"\n")?;

        Ok(())
    }
}

// THIS DOES NOT Compile, but the above works fine, WHY?
//     fn log_order_response(
//         file: &mut File,
//         order_resp: &TradeResponse,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         serde_json::to_writer(file, order_resp)?;
//         file.write_all(b"\n")?;
//         Ok(())
//     }
//
// Here is the error:
//
//    wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
//    $ cargo check
//        Checking binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
//    error[E0382]: borrow of moved value: `file`
//      --> src/binance_trade.rs:73:5
//       |
//    69 |     file: &mut File,
//       |     ---- move occurs because `file` has type `&mut std::fs::File`, which does not implement the `Copy` trait
//    ...
//    72 |     serde_json::to_writer(file, order_resp)?;
//       |                           ---- value moved here
//    73 |     file.write_all(b"\n")?;
//       |     ^^^^ value borrowed here after move
//
//    error: aborting due to previous error
//
//    For more information about this error, try `rustc --explain E0382`.
//    error: could not compile `binance-auto-sell`
//
//    To learn more, run the command again with --verbose.

pub async fn binance_new_order_or_test(
    ctx: &mut BinanceContext,
    symbol: &str,
    side: Side,
    order_type: TradeOrderType,
    test: bool,
) -> Result<TradeResponse, Box<dyn std::error::Error>> {
    let mut ol = OrderLogger::new(&ctx.opts.order_log_path)?;

    let secret_key = ctx.keys.secret_key.as_bytes();
    let api_key = &ctx.keys.api_key;

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
    let signature = binance_signature(&secret_key, &[], &query);

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

    let response = post_req_get_response(api_key, &url, &query_string).await?;
    trace!("response={:#?}", response);
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

        ol.log_order_response(&order_resp)?;

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

        ol.log_order_response(&order_resp)?;

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
