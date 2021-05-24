use log::trace;
use structopt::StructOpt;

use function_name::named;

use crate::{
    binance_klines::{get_klines, KlineInterval, KlineRec},
    common::{dt_str_to_utc_time_ms, time_ms_to_utc, utc_now_to_time_ms},
    configuration::ConfigurationX,
};

// Seconds and minutes in milli-seconds
const SEC: i64 = 1000;
const MIN: i64 = 60 * SEC;

#[derive(Debug, Clone, Default, StructOpt)]
pub struct GetKlinesCmdRec {
    /// Symbol name
    pub sym_name: String,

    /// Start date and time, Optional second positional argument
    #[structopt(short = "s", long)]
    pub start_date_time: Option<String>,

    /// Limit 1 to 1000
    #[structopt(short = "l", long)]
    pub limit: Option<u16>,

    /// Kline interval
    #[structopt(short = "i", long)]
    pub interval: Option<String>,
}

#[named]
pub async fn get_klines_cmd(
    config: &ConfigurationX,
    rec: &GetKlinesCmdRec,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("get_klines_cmd: rec: {:#?}", rec);

    let start_time_ms = if let Some(naive_dt_str) = &rec.start_date_time {
        Some(dt_str_to_utc_time_ms(naive_dt_str)?)
    } else {
        None
    };

    let limit: u16 = if let Some(v) = rec.limit { v } else { 1 };

    let interval = if let Some(s) = &rec.interval {
        KlineInterval::from_string(s)?
    } else {
        KlineInterval::Mins1
    };

    let krs: Vec<KlineRec> =
        //get_klines(ctx, &rec.sym_name, interval, Some(start_time_ms), None, Some(limit)).await?;
        get_klines(config, &rec.sym_name, interval, start_time_ms, None, Some(limit)).await?;

    for kr in &krs {
        println!(
            "Now UTC: {} Open time: {} Close time: {} diff: {}",
            time_ms_to_utc(utc_now_to_time_ms()),
            time_ms_to_utc(kr.open_time),
            time_ms_to_utc(kr.close_time),
            (kr.close_time - kr.open_time) as f64 / MIN as f64
        );
        println!("{:#?}", kr);
    }

    Ok(())
}
