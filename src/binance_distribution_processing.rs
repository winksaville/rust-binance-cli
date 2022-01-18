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
use std::{collections::HashMap, fs::File, io::BufReader};

use clap::SubCommand;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::{
    common::{dec_to_money_string, dec_to_separated_string},
    configuration::Configuration,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistRec {
    #[serde(rename = "User_Id")]
    pub user_id: String,
    #[serde(rename = "Time")]
    pub time: String, // TODO: Convert to an i64 assume UTC
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

/// Iterate over a reader which returns lines from the distribution file.
///
/// TODO: How to allow the reader to be a file or a buffer. Specifically
/// I'd like to provide data in a buffer for testing and not have to use
/// ./test_data/ like I am now!
pub async fn iterate_dist_reader(
    reader: BufReader<File>,
    mut process_line: impl FnMut(&mut DistRec, usize) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("iterate_dist_reader:+");

    let mut rdr = csv::Reader::from_reader(reader);
    for (line_index, result) in rdr.deserialize().enumerate() {
        let mut dr = result?;
        process_line(&mut dr, line_index)?;
    }

    //println!("iterate_dist_reader:-");
    Ok(())
}

pub async fn iterate_dist_file(
    dist_file_str: &str,
    process_line: impl FnMut(&mut DistRec, usize) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("iterate_distribution_file:+ dist_file_path={}", dist_file_str);

    let file = File::open(dist_file_str)?;
    let reader = BufReader::new(file);
    iterate_dist_reader(reader, process_line).await?;

    //println!("iterate_distribution_file:- dist_file_path={}", dist_file_str);
    Ok(())
}

#[allow(unused)]
pub fn display_rec(dr: &mut DistRec, line_index: usize) -> Result<(), Box<dyn std::error::Error>> {
    let rpa = match dr.realized_amount_for_primary_asset {
        Some(v) => v,
        None => dec!(0),
    };
    let rpa_usd = match dr.realized_amount_for_primary_asset_in_usd_value {
        Some(v) => v,
        None => dec!(0),
    };

    println!(
        "{:9}: {:>10}     {:25} {:5} {:16.8}  {:>12}  {:>12}",
        line_index + 1,
        dr.user_id,
        dr.time,
        dr.primary_asset,
        rpa,
        dec_to_money_string(rpa_usd),
        dec_to_money_string(rpa_usd / rpa),
    );
    Ok(())
}

#[allow(unused)]
pub fn display_full_rec(
    dr: &mut DistRec,
    line_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}: {:?}", line_index + 1, dr);
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

    //iterate_dist_file("./test_data/dist_file_one_rec.csv", display_rec).await?;
    //iterate_dist_file("./test_data/dist_file_several_recs.csv", display_rec).await?;

    #[derive(Debug)]
    #[allow(unused)]
    struct AssetRec {
        pub asset: String,
        pub quantity: Decimal,
        pub value_usd: Decimal,
        pub transaction_count: u64,
    }

    impl AssetRec {
        fn new(
            asset: &str,
            quantity: Decimal,
            value_usd: Decimal,
            transaction_count: u64,
        ) -> AssetRec {
            AssetRec {
                asset: asset.to_string(),
                quantity,
                value_usd,
                transaction_count,
            }
        }
    }

    type AssetRecHashMap = HashMap<String, AssetRec>;
    let mut hm = AssetRecHashMap::new();

    let mut total = dec!(0);
    let mut empty_rpa = 0u64;
    let mut empty_rpa_usd = 0u64;
    let mut total_count = 0u64;
    let mut distribution_category_count = 0u64;
    let mut quick_buy_category_count = 0u64;
    let mut quick_sell_category_count = 0u64;
    let mut spot_trading_category_count = 0u64;
    let mut withdrawal_category_count = 0u64;
    let mut unprocessed_category_count = 0u64;
    let process_hm_entry =
        |dr: &mut DistRec, line_index: usize| -> Result<(), Box<dyn std::error::Error>> {
            total_count += 1;
            match dr.category.as_ref() {
                "Distribution" => {
                    distribution_category_count += 1;
                    let rpa = match dr.realized_amount_for_primary_asset {
                        Some(v) => v,
                        None => {
                            empty_rpa += 1;
                            dec!(0)
                        }
                    };
                    let rpa_usd = match dr.realized_amount_for_primary_asset_in_usd_value {
                        Some(v) => v,
                        None => {
                            empty_rpa_usd += 1;
                            dec!(0)
                        }
                    };

                    let entry = hm
                        .entry(dr.primary_asset.clone())
                        .or_insert_with(|| AssetRec::new(&dr.primary_asset, rpa, rpa_usd, 0));
                    entry.transaction_count += 1;
                    if entry.transaction_count > 1 {
                        // Sum realized amounts
                        entry.quantity += rpa;
                        entry.value_usd += rpa_usd;
                    }
                    total += rpa_usd;
                    if config.verbose {
                        print!(
                            "{} {} {} {} {}                                               \r",
                            line_index + 1,
                            entry.asset,
                            rpa_usd,
                            entry.value_usd,
                            total
                        );
                    }
                }
                "Quick Buy" => {
                    quick_buy_category_count += 1;
                }
                "Quick Sell" => {
                    quick_sell_category_count += 1;
                }
                "Spot Trading" => {
                    spot_trading_category_count += 1;
                }
                "Withdrawal" => {
                    withdrawal_category_count += 1;
                }
                _ => {
                    unprocessed_category_count += 1;
                }
            }

            Ok(())
        };

    let file_path = subcmd.matches.value_of("FILE").expect("FILE is missing");
    iterate_dist_file(file_path, process_hm_entry).await?;
    //dbg!("\nhm: {:#?}", hm);

    if config.verbose {
        println!("\n");
    }

    let mut total_hm_value_usd = dec!(0);
    let mut total_hm_transaction_count = 0u64;
    for (_, ar) in hm {
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
        dec_to_separated_string(Decimal::from(empty_rpa_usd), 0),
    );
    println!(
        "{:>27}: {} ",
        "Total USD",
        dec_to_money_string(total_hm_value_usd)
    );
    println!(
        "{:>27}: {} ",
        "Distribution count",
        dec_to_separated_string(Decimal::from(distribution_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Quick Buy count",
        dec_to_separated_string(Decimal::from(quick_buy_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Quick Sell count",
        dec_to_separated_string(Decimal::from(quick_sell_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Spot Trading count",
        dec_to_separated_string(Decimal::from(spot_trading_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Withdrawal count",
        dec_to_separated_string(Decimal::from(withdrawal_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Unprocessed count",
        dec_to_separated_string(Decimal::from(unprocessed_category_count), 0)
    );
    println!(
        "{:>27}: {} ",
        "Total count",
        dec_to_separated_string(Decimal::from(total_count), 0)
    );

    // Assertions!
    assert_eq!(std::mem::size_of::<usize>(), std::mem::size_of::<u64>());
    assert_eq!(total, total_hm_value_usd);
    assert_eq!(empty_rpa, 0);
    assert_eq!(total_hm_transaction_count, distribution_category_count);
    assert_eq!(
        total_count,
        distribution_category_count
            + quick_buy_category_count
            + quick_sell_category_count
            + spot_trading_category_count
            + withdrawal_category_count
            + unprocessed_category_count
    );

    //println!("process_dist_files:-");
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::de_string_to_utc_time_ms::de_string_to_utc_time_ms;

    #[derive(Debug, Serialize, Deserialize)]
    struct TimeRec {
        #[serde(rename = "Time")]
        #[serde(deserialize_with = "de_string_to_utc_time_ms")]
        time: i64,
    }

    #[test]
    fn test_serde_from_csv() {
        let csv = "
Time
1970-01-01 00:00:00
1970-01-01 00:00:00.123";

        let mut reader = csv::Reader::from_reader(csv.as_bytes());
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
}
