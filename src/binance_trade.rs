//use chrono::Utc;
use log::trace;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::{
    binance_avg_price::get_avg_price,
    binance_context::BinanceContext,
    binance_exchange_info::ExchangeInfo,
    binance_order_response::{FullTradeResponseRec, TradeResponse},
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    common::{post_req_get_response, utc_now_to_time_ms, BinanceError, ResponseErrorRec, Side},
};

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

#[allow(unused)]
async fn convert(
    ctx: &BinanceContext,
    asset: &str,
    value: Decimal,
    other_asset: &str,
) -> Result<Decimal, Box<dyn std::error::Error>> {
    let other_value: Decimal = if asset == other_asset {
        let new_value = value;
        println!(
            "convert: asset: {} value: {} to {}: {}",
            asset, value, other_asset, new_value
        );
        new_value
    } else {
        // Try to directly convert it
        let cvrt_asset_name = asset.to_string() + other_asset;
        match get_avg_price(ctx, &cvrt_asset_name).await {
            Ok(ap) => {
                let new_value = ap.price * value;
                println!(
                    "convert: asset: {} value: {} to {}: {}",
                    asset, value, other_asset, new_value
                );
                new_value
            }
            Err(_) => {
                return Err(format!(
                    "convert error, asset: {} not convertalbe to {}",
                    asset, other_asset
                )
                .into());
            }
        }
    };

    Ok(other_value)
}

async fn convert_commission(
    ctx: &BinanceContext,
    order_response: &FullTradeResponseRec,
    fee_asset: &str,
) -> Result<Decimal, Box<dyn std::error::Error>> {
    let mut commission_value = dec!(0);
    for f in &order_response.fills {
        commission_value += convert(&ctx, &f.commission_asset, f.commission, fee_asset).await?;
    }
    Ok(commission_value)
}

#[cfg(test)]
mod test {
    use super::*;

    const FULL_TRADE_RESPONSE_REC_SUCCESS_FULL: &str = r#"{
        "symbol":"ADAUSD",
        "clientOrderId":"2K956RjiRG7mJfk06skarQ",
        "orderId":108342146,
        "orderListId":-1,
        "transactTime":1620435240708,
        "price":"0.0000",
        "origQty":"6.20000000",
        "executedQty":"6.20000000",
        "cummulativeQuoteQty":"10.1463",
        "status":"FILLED",
        "timeInForce":"GTC",
        "type":"MARKET",
        "side":"SELL",
        "fills":[
            {
                "commissionAsset":"BNB",
                "commission":"0.00001209",
                "price":"1.6365",
                "qty":"6.20000000",
                "tradeId":5579228
            }
        ]
    }"#;

    #[tokio::test]
    async fn test_convert() {
        let ctx = BinanceContext::new();
        let order_response: FullTradeResponseRec =
            serde_json::from_str(FULL_TRADE_RESPONSE_REC_SUCCESS_FULL).unwrap();
        let mut commission_usd = dec!(0);
        for f in order_response.fills {
            commission_usd += convert(&ctx, &f.commission_asset, f.commission, "USD")
                .await
                .unwrap();
        }

        // TODO: Need to "mock" get_avg_price.
        assert!(commission_usd > dec!(0));
    }

    #[tokio::test]
    async fn test_convert_commission() {
        let ctx = BinanceContext::new();
        let order_response: FullTradeResponseRec =
            serde_json::from_str(FULL_TRADE_RESPONSE_REC_SUCCESS_FULL).unwrap();
        let commission_usd = convert_commission(&ctx, &order_response, "USD")
            .await
            .unwrap();

        // TODO: Need to "mock" get_avg_price.
        assert!(commission_usd > dec!(0));
    }
}

pub async fn binance_new_order_or_test(
    ctx: &mut BinanceContext,
    ei: &ExchangeInfo,
    symbol: &str,
    side: Side,
    order_type: TradeOrderType,
    test: bool,
) -> Result<TradeResponse, Box<dyn std::error::Error>> {
    let mut ol = OrderLogger::new(&ctx.opts.order_log_path)?;

    let ei_symbol = match ei.get_symbol(symbol) {
        Some(s) => s,
        None => {
            return Err(format!("{} was not found in exchange_info", symbol).into());
        }
    };

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
    trace!("response_status={:#?}", response_status);
    let response_body = response.text().await?;
    trace!("response_body={:#?}", response_body);

    // Log the response
    let result = if response_status == 200 {
        let mut order_resp_success = FullTradeResponseRec::default();
        if !test {
            order_resp_success = serde_json::from_str(&&response_body)?;
        } else {
            order_resp_success.test = true;
        }

        order_resp_success.query = query_string;
        order_resp_success.cost_basis_usd = convert(
            ctx,
            &ei_symbol.quote_asset,
            order_resp_success.cummulative_quote_qty,
            "USD",
        )
        .await?;
        order_resp_success.commission_usd =
            convert_commission(&ctx, &order_resp_success, "USD").await?;

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
