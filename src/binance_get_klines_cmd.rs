use log::trace;

use crate::{
    binance_klines::{get_klines, KlineInterval, KlineRec},
    configuration::Configuration,
};

use time_ms_conversions::{
    dt_str_to_utc_time_ms, time_ms_to_utc,
    TzMassaging::{HasTz, LocalTz},
};

#[derive(Debug, Clone, Default)]
pub struct GetKlinesCmdRec {
    // Symbol name
    pub sym_name: String,

    // Start date and time
    pub start_date_time: Option<String>,

    // Limit 1 to 1000
    pub limit: Option<u16>,

    // Kline interval
    pub interval: Option<String>,
}

pub async fn get_klines_cmd(
    config: &Configuration,
    rec: &GetKlinesCmdRec,
) -> Result<(), Box<dyn std::error::Error>> {
    // Seconds and minutes in milli-seconds
    const SEC: i64 = 1000;
    const MIN: i64 = 60 * SEC;

    trace!("get_klines_cmd: rec: {:#?}", rec);

    let start_time_ms = if let Some(dt_str) = &rec.start_date_time {
        match dt_str_to_utc_time_ms(dt_str, LocalTz) {
            Ok(ndt) => {
                println!("get_klines_cmd: ndt={ndt}");
                Some(ndt)
            }
            Err(_) => match dt_str_to_utc_time_ms(dt_str, HasTz) {
                Ok(dt) => {
                    println!("get_klines_cmd: dt={dt}");
                    Some(dt)
                }
                Err(_) => None,
            },
        }
    } else {
        None
    };

    let limit: u16 = if let Some(v) = rec.limit { v } else { 1 };

    let interval = if let Some(s) = &rec.interval {
        KlineInterval::from_string(s)?
    } else {
        KlineInterval::Mins1
    };

    let krs: Vec<KlineRec> = get_klines(
        config,
        &rec.sym_name,
        interval,
        start_time_ms,
        None,
        Some(limit),
    )
    .await?;

    for kr in &krs {
        println!(
            "Open time: {} Close time: {} diff minutes: {}",
            time_ms_to_utc(kr.open_time),
            time_ms_to_utc(kr.close_time),
            (kr.close_time - kr.open_time) as f64 / MIN as f64
        );
        println!("{kr:#?}");
    }

    Ok(())
}
