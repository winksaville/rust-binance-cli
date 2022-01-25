//! This file processes binance.us distributation files.
//!
//! Information I've learned
//!  * Contents adhere to the CSV format
//!  * First line is contains headers
//!  * Subsequent lines contine the comma seperated fields
//!  * Empty line contain an empty string; "" other wise no quotes are used.
//!    This means empty numeric fields must be defined as using Option<T>.
//!    If they had be "blank" i.e. just adjacent commas, serde would have defaulted to 0, I believe.
//!  * Using the following `awk` and `sort` yields there are 4 catagories:
//!    Distributation, Quick Buy, Quick Sell, Spot Trading and Withdrawal
//!    ```
//!    wink@3900x:~/prgs/rust/myrepos/binance-cli/data
//!    $ awk -F, '{ print $3 }' binance.us-distribution-2021.csv | sort -u
//!    Category
//!    Distribution
//!    Quick Buy
//!    Quick Sell
//!    Spot Trading
//!    Withdrawal
//!    ```
//!  * I think I need to process only records with Category == Distribution.
//!  * Some Category == Distribution records have an empty
//!    Realized_Amount_For_Primary_Asset_In_USD_Value field that is empty.
//!    Such as:
//!      35002704,2021-12-31 00:07:03.819,Distribution,Referral Commission,88367941,880499527,SUSHI,0.00224,"","","","","","","","","","",Wallet,"",""
//!    So for these I need to "lookup and calcuate" the Realized_Amount_For_Primary_Asset_In_USD_Value.
//!

//!
use std::{collections::HashMap, fs::File, io::BufReader, io::BufWriter};

use clap::SubCommand;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::{
    common::{dec_to_money_string, dec_to_separated_string},
    configuration::Configuration,
    de_string_to_utc_time_ms::{de_string_to_utc_time_ms_condaddtzutc, se_time_ms_to_utc_string}, //binance_klines::get_kline,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistRec {
    #[serde(rename = "User_Id")]
    pub user_id: String,
    #[serde(rename = "Time")]
    #[serde(deserialize_with = "de_string_to_utc_time_ms_condaddtzutc")]
    #[serde(serialize_with = "se_time_ms_to_utc_string")]
    pub time: i64,
    #[serde(rename = "Category")]
    pub category: String,
    #[serde(rename = "Operation")]
    pub operation: String,
    #[serde(rename = "Order_Id")]
    pub order_id: String,
    #[serde(rename = "Transaction_Id")]
    pub transaction_id: u64,
    #[serde(rename = "Primary_Asset")]
    pub primary_asset: String,
    #[serde(rename = "Realized_Amount_For_Primary_Asset")]
    pub realized_amount_for_primary_asset: Option<Decimal>,
    #[serde(rename = "Realized_Amount_For_Primary_Asset_In_USD_Value")]
    pub realized_amount_for_primary_asset_in_usd_value: Option<Decimal>,
    #[serde(rename = "Base_Asset")]
    pub base_asset: String,
    #[serde(rename = "Realized_Amount_For_Base_Asset")]
    pub realized_amount_for_base_asset: Option<Decimal>,
    #[serde(rename = "Realized_Amount_For_Base_Asset_In_USD_Value")]
    pub realized_amount_for_base_asset_in_usd_value: Option<Decimal>,
    #[serde(rename = "Quote_Asset")]
    pub quote_asset: String,
    #[serde(rename = "Realized_Amount_For_Quote_Asset")]
    pub realized_amount_for_quote_asset: Option<Decimal>,
    #[serde(rename = "Realized_Amount_For_Quote_Asset_In_USD_Value")]
    pub realized_amount_for_quote_asset_in_usd_value: Option<Decimal>,
    #[serde(rename = "Fee_Asset")]
    pub fee_asset: String,
    #[serde(rename = "Realized_Amount_For_Fee_Asset")]
    pub realized_amount_for_fee_asset: Option<Decimal>,
    #[serde(rename = "Realized_Amount_For_Fee_Asset_In_USD_Value")]
    pub realized_amount_for_fee_asset_in_usd_value: Option<Decimal>,
    #[serde(rename = "Payment_Method")]
    pub payment_method: String,
    #[serde(rename = "Withdrawal_Method")]
    pub withdrawal_method: String,
    #[serde(rename = "Additional_Note")]
    pub additional_note: String,
}

#[derive(Debug)]
pub struct AssetRec {
    pub asset: String,
    pub quantity: Decimal,
    pub value_usd: Decimal,
    pub transaction_count: u64,
}

impl AssetRec {
    fn new(asset: &str, quantity: Decimal, value_usd: Decimal, transaction_count: u64) -> AssetRec {
        AssetRec {
            asset: asset.to_string(),
            quantity,
            value_usd,
            transaction_count,
        }
    }
}

pub type AssetRecHashMap = HashMap<String, AssetRec>;

#[derive(Debug)]
pub struct ProcessedData {
    pub hm: AssetRecHashMap,
    pub total_rpa_usd: Decimal,
    pub empty_rpa: u64,
    pub empty_rpa_usd: u64,
    pub total_count: u64,
    pub distribution_category_count: u64,
    pub quick_buy_category_count: u64,
    pub quick_sell_category_count: u64,
    pub spot_trading_category_count: u64,
    pub withdrawal_category_count: u64,
    pub unprocessed_category_count: u64,
}

impl ProcessedData {
    fn new() -> ProcessedData {
        ProcessedData {
            hm: AssetRecHashMap::new(),
            total_rpa_usd: dec!(0),
            empty_rpa: 0u64,
            empty_rpa_usd: 0u64,
            total_count: 0u64,
            distribution_category_count: 0u64,
            quick_buy_category_count: 0u64,
            quick_sell_category_count: 0u64,
            spot_trading_category_count: 0u64,
            withdrawal_category_count: 0u64,
            unprocessed_category_count: 0u64,
        }
    }
}

fn process_hm_entry(
    config: &Configuration,
    data: &mut ProcessedData,
    dr: &mut DistRec,
    line_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    data.total_count += 1;
    match dr.category.as_ref() {
        "Distribution" => {
            data.distribution_category_count += 1;
            let rpa = match dr.realized_amount_for_primary_asset {
                Some(v) => v,
                None => {
                    data.empty_rpa += 1;
                    dec!(0)
                }
            };
            let rpa_usd = match dr.realized_amount_for_primary_asset_in_usd_value {
                Some(v) => v,
                None => {
                    //let kr = get_kline(config, &(dr.primary_asset.to_ownede() + "USD"), dr.time).await?;
                    data.empty_rpa_usd += 1;
                    dec!(0)
                }
            };

            let entry = data
                .hm
                .entry(dr.primary_asset.clone())
                .or_insert_with(|| AssetRec::new(&dr.primary_asset, rpa, rpa_usd, 0));
            entry.transaction_count += 1;
            if entry.transaction_count > 1 {
                // Sum realized amounts
                entry.quantity += rpa;
                entry.value_usd += rpa_usd;
            }
            data.total_rpa_usd += rpa_usd;
            if config.verbose {
                print!(
                    "{} {} {} {} {}                                               \r",
                    line_index + 1,
                    entry.asset,
                    rpa_usd,
                    entry.value_usd,
                    data.total_rpa_usd
                );
            }
        }
        "Quick Buy" => {
            data.quick_buy_category_count += 1;
        }
        "Quick Sell" => {
            data.quick_sell_category_count += 1;
        }
        "Spot Trading" => {
            data.spot_trading_category_count += 1;
        }
        "Withdrawal" => {
            data.withdrawal_category_count += 1;
        }
        _ => {
            data.unprocessed_category_count += 1;
        }
    }

    Ok(())
}

pub async fn process_dist_files(
    config: &Configuration,
    subcmd: &SubCommand<'static>,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.verbose {
        println!(
            "process_dist_files:+\n config={:?} \nsubcmd={:?}",
            config, subcmd
        );
    }
    let mut data = ProcessedData::new();

    let in_dist_file_path = subcmd.matches.value_of("IN_FILE").expect("FILE is missing");
    let out_dist_file_path = subcmd.matches.value_of("OUT_FILE");

    //iterate_dist_file( config, &mut data, in_dist_file_path, out_dist_file_path, process_hm_entry).await?;
    let in_file = File::open(in_dist_file_path)?;
    let reader = BufReader::new(in_file);
    let writer = if let Some(of) = out_dist_file_path {
        let out_file = File::create(of)?;
        Some(BufWriter::new(out_file))
    } else {
        None
    };

    //iterate_dist_processor(config, data, reader, writer, process_line).await?;
    let mut rdr = csv::Reader::from_reader(reader);

    // Clippy suggested changing this:
    //   let mut wdr = if let Some(wtr) = writer { Some(csv::Writer::from_writer(wtr)) } else { None };
    // To this:
    let mut wdr = writer.map(csv::Writer::from_writer);

    for (line_index, result) in rdr.deserialize().enumerate() {
        let mut dr = result?;
        process_hm_entry(config, &mut data, &mut dr, line_index)?;
        if let Some(w) = &mut wdr {
            w.serialize(&dr)?;
        }
    }

    if config.verbose {
        println!("\n");
    }

    let mut total_hm_value_usd = dec!(0);
    let mut total_hm_transaction_count = 0u64;
    for (_, ar) in data.hm {
        total_hm_value_usd += ar.value_usd;
        total_hm_transaction_count += ar.transaction_count; // as usize;
        println!(
            "{:10} {:20.6} {:>10} {:>14}",
            ar.asset,
            ar.quantity,
            dec_to_separated_string(Decimal::from(ar.transaction_count), 0),
            dec_to_money_string(ar.value_usd)
        );
    }
    println!("\n");
    println!(
        "{:>27}: {} no USD amount: {}",
        "Distribution Transactions",
        dec_to_separated_string(Decimal::from(total_hm_transaction_count), 0),
        dec_to_separated_string(Decimal::from(data.empty_rpa_usd), 0),
    );
    println!(
        "{:>27}: {} ",
        "Total USD",
        dec_to_money_string(total_hm_value_usd)
    );
    println!(
        "{:>27}: {} ",
        "Distribution count",
        dec_to_separated_string(Decimal::from(data.distribution_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Quick Buy count",
        dec_to_separated_string(Decimal::from(data.quick_buy_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Quick Sell count",
        dec_to_separated_string(Decimal::from(data.quick_sell_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Spot Trading count",
        dec_to_separated_string(Decimal::from(data.spot_trading_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Withdrawal count",
        dec_to_separated_string(Decimal::from(data.withdrawal_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Unprocessed count",
        dec_to_separated_string(Decimal::from(data.unprocessed_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Total count",
        dec_to_separated_string(Decimal::from(data.total_count), 0)
    );

    // Assertions!
    assert_eq!(std::mem::size_of::<usize>(), std::mem::size_of::<u64>());
    assert_eq!(data.total_rpa_usd, total_hm_value_usd);
    assert_eq!(data.empty_rpa, 0);
    assert_eq!(total_hm_transaction_count, data.distribution_category_count);
    assert_eq!(
        data.total_count,
        data.distribution_category_count
            + data.quick_buy_category_count
            + data.quick_sell_category_count
            + data.spot_trading_category_count
            + data.withdrawal_category_count
            + data.unprocessed_category_count
    );

    //println!("process_dist_files:-");
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::de_string_to_utc_time_ms::{
        de_string_to_utc_time_ms_condaddtzutc, se_time_ms_to_utc_string,
    };

    #[derive(Debug, Serialize, Deserialize)]
    struct TimeRec {
        #[serde(rename = "Time")]
        #[serde(deserialize_with = "de_string_to_utc_time_ms_condaddtzutc")]
        #[serde(serialize_with = "se_time_ms_to_utc_string")]
        time: i64,
    }

    #[test]
    fn test_deserialize_from_csv() {
        let csv = "
Time
1970-01-01 00:00:00
1970-01-01 00:00:00.123";

        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);
        //let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for (idx, entry) in reader.deserialize().enumerate() {
            println!("{idx}: entry: {:?}", entry);
            match entry {
                Ok(tr) => {
                    let tr: TimeRec = tr;
                    println!("tr: {:?}", tr);
                    match idx {
                        0 => assert_eq!(tr.time, 0),
                        1 => assert_eq!(tr.time, 123),
                        _ => panic!("Unexpected idx"),
                    }
                }
                Err(e) => panic!("Error: {e}"),
            }
        }
    }

    #[test]
    fn test_serialize_to_csv() {
        let trs = vec![TimeRec { time: 0 }, TimeRec { time: 123 }];

        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.serialize(trs.get(0)).expect("Error serializing");
        wtr.serialize(trs.get(1)).expect("Error serializing");

        let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        dbg!(&data);

        assert_eq!(
            data,
            "1970-01-01T00:00:00.000+00:00\n1970-01-01T00:00:00.123+00:00\n"
        );
    }
}
