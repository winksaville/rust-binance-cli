use log::trace;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

use crate::common::{BinanceError, ResponseErrorRec};
use crate::{binance_context::BinanceContext, common::get_req_get_response};

#[derive(Debug, Deserialize, Serialize)]
pub struct AvgPrice {
    pub mins: Decimal,
    pub price: Decimal,
}

pub async fn get_avg_price<'e>(
    ctx: &BinanceContext,
    symbol: &str,
) -> Result<AvgPrice, Box<dyn std::error::Error>> {
    let url = ctx.make_url("api", &format!("/api/v3/avgPrice?symbol={}", symbol));
    trace!("get_avg_price: url={}", url);

    let response = get_req_get_response(&ctx.opts.api_key, &url).await?;
    let response_status = response.status();
    let response_body = response.text().await?;

    // Log the response
    let result = if response_status == 200 {
        let avg_price: AvgPrice = serde_json::from_str(&response_body)?;
        trace!("get_avg_price: avg_price={:?}", avg_price);
        Ok(avg_price)
    } else {
        let response_error_rec =
            ResponseErrorRec::new(false, response_status.as_u16(), &url, &response_body);
        let binance_error_response = BinanceError::Response(response_error_rec);

        trace!(
            "get_avg_price: error symbol={} resp_failure={:?}",
            symbol,
            binance_error_response,
        );

        Err(binance_error_response.into())
    };

    result
}
