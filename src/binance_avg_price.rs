use log::trace;
use serde::{Deserialize, Serialize};

use crate::binance_context::BinanceContext;
use crate::common::{BinanceError, ResponseErrorRec};
use crate::de_string_or_number::{de_string_or_number_to_f64, de_string_or_number_to_u64};

#[derive(Debug, Deserialize, Serialize)]
pub struct AvgPrice {
    #[serde(deserialize_with = "de_string_or_number_to_u64")]
    pub mins: u64,
    #[serde(deserialize_with = "de_string_or_number_to_f64")]
    pub price: f64,
}

pub async fn get_avg_price<'e>(
    ctx: &BinanceContext,
    symbol: &str,
) -> Result<AvgPrice, Box<dyn std::error::Error>> {
    let url = ctx.make_url("api", &format!("/api/v3/avgPrice?symbol={}", symbol));
    trace!("get_avg_price: url={}", url);

    let response = reqwest::Client::new().get(url.clone()).send().await?;
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
