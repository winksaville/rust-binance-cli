use log::trace;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::{
    fmt::{self, Display},
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::{
    binance_exchange_info::ExchangeInfo,
    binance_klines::{get_kline, get_kline_of_primary_asset_for_value_asset},
    binance_order_response::{
        AckTradeResponseRec, FullTradeResponseRec, ResultTradeResponseRec, TestTradeResponseRec,
        TradeResponse, UnknownTradeResponseRec,
    },
    binance_signature::{append_signature, binance_signature, query_vec_u8},
    common::{post_req_get_response, utc_now_to_time_ms, ResponseErrorRec, Side, VALUE_ASSETS},
    configuration::Configuration,
};

#[derive(Debug, Clone)]
pub enum MarketQuantityType {
    Quantity(Decimal),
    QuoteOrderQty(Decimal),
}

impl Display for MarketQuantityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let qty_str = match self {
            MarketQuantityType::Quantity(qty) => {
                format!("Quantity:{}", qty)
            }
            MarketQuantityType::QuoteOrderQty(qty) => {
                format!("QuoteOrderQty:{}", qty)
            }
        };

        write!(f, "{}", qty_str)
    }
}

#[derive(Debug, Clone)]
pub enum TradeOrderType {
    Market(MarketQuantityType),
    // Limit,
    // StopLoss,
    // StopLossLimit,
    // TakeProfit,
    // TakeProfitLimit,
    // LimitMaker,
}

impl Display for TradeOrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeOrderType::Market(mot) => {
                write!(f, "Market::{}", mot)
            }
        }
    }
}

pub fn order_log_file(order_log_path: &Path) -> Result<File, Box<dyn std::error::Error>> {
    if let Some(parent_dirs) = order_log_path.parent() {
        // Be sure the parent directories exist
        std::fs::create_dir_all(parent_dirs)?;
    }

    Ok(OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(order_log_path)?)
}

pub fn log_order_response(
    mut writer: &mut dyn Write,
    order_response: &TradeResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer(&mut writer, order_response)?;
    writer.write_all(b"\n")?;
    Ok(())
}

// Convert quantity of asset to the quantity in other_asset at UTC time_ms
pub async fn convert(
    config: &Configuration,
    time_ms: i64,
    asset: &str,
    quantity: Decimal,
    other_asset: &str,
) -> Result<Decimal, Box<dyn std::error::Error>> {
    trace!(
        "convert:+ asset: {} quantity: {} other_asset: {}",
        asset,
        quantity,
        other_asset
    );
    let other_quantity: Decimal = if asset == other_asset {
        let new_quantity = quantity;
        trace!(
            "convert:- asset: {} quantity: {} to {}: {}",
            asset,
            quantity,
            other_asset,
            new_quantity
        );
        new_quantity
    } else {
        // If the asset is one of the VALUE_ASSETS then invert the conversion
        let (lhs, rhs, invert_result) = if VALUE_ASSETS.contains(&asset.to_owned()) {
            (other_asset, asset, true)
        } else {
            (asset, other_asset, false)
        };

        // Try to directly convert it
        let cvrt_asset_name = lhs.to_string() + rhs;
        let result = match get_kline(config, &cvrt_asset_name, time_ms).await {
            Ok(kr) => {
                let direct_result = if invert_result {
                    let tmp = quantity / kr.close;
                    trace!(
                        "{asset}{other_asset}: INVERTED direct_result: {} = quantity: {} / kr.close: {}",
                        tmp, quantity, kr.close
                    );

                    tmp
                } else {
                    let tmp = quantity * kr.close;
                    trace!(
                        "{asset}{other_asset}: direct_result {} = quantity: {} * kr.close: {}",
                        tmp,
                        quantity,
                        kr.close
                    );

                    tmp
                };

                direct_result
            }
            Err(_) => {
                trace!("{cvrt_asset_name} failed: try getting kline of {lhs} in {VALUE_ASSETS:?}");
                let indirect_result = if let Some((sym_name, kr)) =
                    get_kline_of_primary_asset_for_value_asset(config, time_ms, lhs, &VALUE_ASSETS)
                        .await
                {
                    let one_lhs_value_in_usd = kr.close;
                    trace!("{sym_name}: one_lhs_value_in_usd: {one_lhs_value_in_usd} try getting kline of {rhs} in {VALUE_ASSETS:?}");
                    let indirect_result = if let Some((sym_name, kr)) =
                        get_kline_of_primary_asset_for_value_asset(
                            config,
                            time_ms,
                            rhs,
                            &VALUE_ASSETS,
                        )
                        .await
                    {
                        let one_rhs_value_in_usd = kr.close;
                        trace!("{sym_name}: one_rhs_value_in_usd: {one_rhs_value_in_usd}");
                        let ir = if invert_result {
                            let tmp = quantity * (one_rhs_value_in_usd / one_lhs_value_in_usd);
                            trace!("{asset}{other_asset}: INVERTED indirect_result: {} = quantity: {} * (one_rhs_value_in_usd: {} / one_lhs_value_in_usd: {}))",
                                tmp, quantity, one_rhs_value_in_usd, one_lhs_value_in_usd);
                            tmp
                        } else {
                            let tmp = quantity * (one_lhs_value_in_usd / one_rhs_value_in_usd);
                            trace!("{asset}{other_asset}: indirect_result: {} = quantity: {} * (one_lhs_value_in_usd: {} / one_rhs_value_in_usd: {})",
                                tmp, quantity, one_lhs_value_in_usd, one_rhs_value_in_usd);

                            tmp
                        };

                        ir
                    } else {
                        return Err(format!(
                            "convert error, asset: {} not convertable to {}",
                            asset, other_asset
                        )
                        .into());
                    };

                    indirect_result
                } else {
                    return Err(format!(
                        "convert error, asset: {} not convertable to {}",
                        asset, other_asset
                    )
                    .into());
                };

                indirect_result
            }
        };
        trace!(
            "convert:- asset: {} value: {} to {}: {}",
            asset,
            quantity,
            other_asset,
            result
        );

        result
    };

    Ok(other_quantity)
}

async fn convert_commission(
    config: &Configuration,
    order_response: &FullTradeResponseRec,
    fee_asset: &str,
) -> Result<Decimal, Box<dyn std::error::Error>> {
    let mut commission_value = dec!(0);
    for f in &order_response.fills {
        commission_value += convert(
            config,
            order_response.transact_time,
            &f.commission_asset,
            f.commission,
            fee_asset,
        )
        .await?;
    }
    Ok(commission_value)
}

pub async fn binance_new_order_or_test(
    config: &Configuration,
    mut log_writer: &mut dyn Write,
    ei: &ExchangeInfo,
    symbol: &str,
    side: Side,
    order_type: TradeOrderType,
    test: bool,
) -> Result<TradeResponse, Box<dyn std::error::Error>> {
    let ei_symbol = match ei.get_symbol(symbol) {
        Some(s) => s,
        None => {
            return Err(format!("{} was not found in exchange_info", symbol).into());
        }
    };

    let api_key = config.keys.get_ak_or_err()?;
    let secret_key = &config.keys.get_sk_vec_u8_or_err()?;

    let side_str: &str = side.into();
    let mut params = vec![
        ("recvWindow", "5000"),
        ("symbol", symbol),
        ("side", side_str),
        ("newOrderRespType", "FULL"), // Manually tested, "FULL", "RESULT", "ACK" and "XYZ".
                                      // making ADAUSD buys. "XYZ" generated an error which
                                      // was handled properly.
    ];

    let qty_string: String;
    match order_type {
        TradeOrderType::Market(MarketQuantityType::Quantity(qty)) => {
            params.push(("type", "MARKET"));
            qty_string = format!("{}", qty);
            params.push(("quantity", &qty_string));
        }
        TradeOrderType::Market(MarketQuantityType::QuoteOrderQty(qty)) => {
            params.push(("type", "MARKET"));
            qty_string = format!("{}", qty);
            params.push(("quoteOrderQty", &qty_string));
        }
    };

    let ts_string: String = format!("{}", utc_now_to_time_ms());
    params.push(("timestamp", ts_string.as_str()));

    trace!("binanace_new_order_or_test: params={:#?}", params);

    let mut query = query_vec_u8(&params);

    // Calculate the signature using sig_key and the data is qs and query as body
    let signature = binance_signature(secret_key, &[], &query);

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
    let response_headers = response.headers().clone();
    trace!("response_headers={:#?}", response_headers);
    let response_status = response.status();
    trace!("response_status={:#?}", response_status);
    let response_body = response.text().await?;
    trace!("response_body={:#?}", response_body);

    // Log the response
    let result = if response_status == 200 {
        let order_resp = match serde_json::from_str::<FullTradeResponseRec>(&response_body) {
            Ok(mut full) => {
                full.test = test;
                full.query = query_string.clone();
                full.value_usd = if full.cummulative_quote_qty > dec!(0) {
                    // TODO: Erroring is wrong, maybe dec!(0) plus an error alert sent to the programmer!
                    convert(
                        config,
                        full.transact_time,
                        &ei_symbol.quote_asset,
                        full.cummulative_quote_qty,
                        "USD",
                    )
                    .await?
                } else {
                    dec!(0)
                };
                full.commission_usd = if !full.fills.is_empty() {
                    // TODO: Erroring is wrong, maybe dec!(0) and an error alert sent to the programmer!
                    convert_commission(config, &full, "USD").await?
                } else {
                    dec!(0)
                };

                TradeResponse::SuccessFull(full)
            }
            Err(_) => match serde_json::from_str::<ResultTradeResponseRec>(&response_body) {
                Ok(mut result) => {
                    result.test = test;
                    result.query = query_string.clone();
                    result.value_usd = if result.status.eq("FILLED") {
                        // TODO: Erroring is wrong, maybe dec!(0) plus an error alert sent to the programmer!
                        convert(
                            config,
                            result.transact_time,
                            &ei_symbol.quote_asset,
                            result.cummulative_quote_qty,
                            "USD",
                        )
                        .await?
                    } else {
                        dec!(0)
                    };
                    result.commission_usd = dec!(0);

                    TradeResponse::SuccessResult(result)
                }
                Err(_) => match serde_json::from_str::<AckTradeResponseRec>(&response_body) {
                    Ok(mut ack) => {
                        ack.test = test;
                        ack.query = query_string.clone();
                        TradeResponse::SuccessAck(ack)
                    }
                    Err(_) => {
                        if test {
                            TradeResponse::SuccessTest(TestTradeResponseRec {
                                test,
                                query: query_string,
                                response_body,
                            })
                        } else {
                            TradeResponse::SuccessUnknown(UnknownTradeResponseRec {
                                test,
                                query: query_string,
                                response_body,
                                error_internal: "Unexpected trade response body".to_string(),
                            })
                        }
                    }
                },
            },
        };

        trace!(
            "binance_market_order_or_test: symbol={} side={} test={} order_response={:#?}",
            symbol,
            side_str,
            test,
            order_resp
        );
        // TODO: Erroring is wrong, maybe dec!(0) plus an error alert sent to the programmer!
        log_order_response(&mut log_writer, &order_resp)?;

        Ok(order_resp)
    } else {
        let rer = ResponseErrorRec::new(
            test,
            response_status.as_u16(),
            &query_string,
            response_headers,
            &response_body,
        );
        let order_resp = TradeResponse::FailureResponse(rer);

        // TODO: Erroring is wrong, maybe dec!(0) plus an error alert sent to the programmer!
        log_order_response(&mut log_writer, &order_resp)?;

        trace!(
            "{}",
            format!(
                "binance_market_order_or_test: symbol={} side={} test={} order_resp={:#?}",
                symbol, side_str, test, order_resp
            )
        );

        Err(order_resp.into())
    };

    result
}

#[cfg(test)]
mod test {
    use std::io::{Read, Seek, SeekFrom};

    use super::*;

    const SUCCESS_FULL: &str = r#"{
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

    async fn convert_test(asset: &str, quantity: Decimal, other_asset: &str) {
        // TODO: we should use a "mock" so the results are predictable and a
        // network connection is not needed

        let mut config = Configuration::default();
        config.keys.api_key = Some("a_key".to_string());
        config.keys.secret_key = Some("a_secret_key".to_string());

        // Use a time 10 minutes earlier than "now" otherwise the kline
        // changes as it's being actively being updated by binance.
        let time_ms = utc_now_to_time_ms() - 10 * 60 * 1000;

        let asset_to_other_value = convert(&config, time_ms, asset, quantity, other_asset)
            .await
            .unwrap();
        println!(
            "converted {} {asset} -> {} {other_asset}",
            quantity, asset_to_other_value
        );

        let other_to_asset_value = convert(&config, time_ms, other_asset, quantity, asset)
            .await
            .unwrap();

        println!(
            "converted {} {other_asset} -> {} {asset}",
            quantity, other_to_asset_value
        );

        // Validate the convsersion by checking
        //    ((convert(asset, quantity, other_asset) * convert(other_asset, quantity, asset)) / quantity * quantity)
        // is approximately equal to 1
        //
        // Proof:
        //  fn convert(asset: &str, quantity: Decimal , other_asset: &str) -> Decimal {
        //      let va = value_of_an_asset_per_x(asset);
        //      let vo = value_of_an_asset_per_x(other_asset);
        //
        //      quantity * (va / vo)
        //  }
        //
        //  So:
        //      let quantity_a = convert(asset, quantity, other_asset)
        //      let quantity_a = quantity * va/vo
        //      let quantity_b = convert(other_asset, quantity, asset)
        //      let quantity_b = quantity * vo/va
        //
        //      assert_approx_eq!((quantity_a * quantity_b) / (quantity * quantity), 1)
        //      assert_approx_eq!(((quantity * va / vo) * (quantity * (vo / va))) / quantity^2, 1)
        //      assert_approx_eq!((quantity * quantity * (va / vo) * (vo / va)) / quantity^2, 1)
        //      assert_approx_eq!((quantity^2 * (va * vo) / (vo * va)) / quantity^2, 1)
        //      assert_approx_eq!((quantity^2 * (va * vo) / (va * vo)) / quantity^2, 1)
        //      assert_approx_eq!((quantity^2 * 1) / quantity^2, 1)
        //      assert_approx_eq!(quantity^2 / quantity^2, 1)
        //      assert_approx_eq!(1, 1)
        let approx_one = (asset_to_other_value * other_to_asset_value) / (quantity * quantity);
        println!(
            "approx_one: {} = {asset}{other_asset}: {} * {other_asset}{asset}: {} / (quantity^2: {} )",
            approx_one, asset_to_other_value, other_to_asset_value, quantity * quantity
        );

        // Take the difference between 1 and approx_one and assert it's small
        let diff = dec!(1) - approx_one;
        println!("diff: {} = {} - approx_one: {}", diff, dec!(1), approx_one);

        //assert!(diff.abs() < dec!(0.0000000000001));
        assert!(diff.abs() < dec!(0.01));
    }

    #[tokio::test]
    async fn test_convert_value_assets_to_value_assets() {
        for asset in VALUE_ASSETS.iter() {
            for other_asset in VALUE_ASSETS.iter() {
                convert_test(asset, dec!(10), other_asset).await;
                convert_test(other_asset, dec!(10), asset).await;
            }
        }
    }

    #[tokio::test]
    async fn test_convert_shib_to_value_assets() {
        for asset in VALUE_ASSETS.iter() {
            convert_test("SHIB", dec!(10), asset).await;
            convert_test(asset, dec!(10), "SHIB").await;
        }
    }

    #[tokio::test]
    async fn test_convert_direct() {
        convert_test("BAT", dec!(1), "USD").await;
        convert_test("BAT", dec!(10), "USD").await;

        convert_test("USD", dec!(1), "BAT").await;
        convert_test("USD", dec!(10), "BAT").await;
    }

    #[tokio::test]
    async fn test_convert_indirect() {
        convert_test("BAT", dec!(1), "BNB").await;
        convert_test("BNB", dec!(1), "BAT").await;

        convert_test("BAT", dec!(123.4), "BNB").await;
        convert_test("BNB", dec!(123.4), "BAT").await;
    }

    #[tokio::test]
    async fn test_convertcommission() {
        let mut config = Configuration::default();
        config.keys.api_key = Some("a_key".to_string());
        config.keys.secret_key = Some("a_secret_key".to_string());

        let order_response: FullTradeResponseRec = serde_json::from_str(SUCCESS_FULL).unwrap();

        // TODO: Need to "mock" get_kline so order_response.fills[0].commission_asset ("BNB") always returns a specific value.
        let commission_usd = convert_commission(&config, &order_response, "USD")
            .await
            .unwrap();
        // assert_eq!(commission_usd, dec!(xxx))
        println!(
            "convert {} BNBUSB: {}",
            order_response.fills[0].commission, commission_usd
        );

        assert!(commission_usd > dec!(0));
    }

    #[tokio::test]
    async fn test_log_order_response() {
        let order_response: FullTradeResponseRec = serde_json::from_str(SUCCESS_FULL).unwrap();
        let order_resp = TradeResponse::SuccessFull(order_response);

        // Create a cursor buffer and log to it
        let mut buff = std::io::Cursor::new(vec![0; 100]);
        log_order_response(&mut buff, &order_resp).unwrap();
        let buff_len = buff.stream_position().unwrap();

        // Convert to a string so we can inspect it easily, but we must seek to 0 first
        let mut buff_string = String::with_capacity(100);
        buff.seek(SeekFrom::Start(0)).unwrap();
        let buff_string_len = buff
            .read_to_string(&mut buff_string)
            .unwrap()
            .to_u64()
            .unwrap();
        //println!("buff: len: {} string: {}", buff_string_len, buff_string);

        // The length of the string and buffer should be the same
        assert_eq!(buff_len, buff_string_len);

        // Check that it contains 1.6365.  This will assert if the rust_decimal
        // feature, "serde-float", is enabled in Cargo.toml:
        //   rust_decimal = { version = "1.12.4", features = ["serde-arbitrary-precision", "serde-float"] }
        // As we see the following in buff_string:
        //   "price":1.6364999999999998
        //
        // If "serde-float" is NOT enabled:
        //   rust_decimal = { version = "1.12.4", features = ["serde-arbitrary-precision"] }
        // then we see value "correct" price:
        //   "price":"1.6365"
        assert!(buff_string.contains("1.6365"));
    }
}
