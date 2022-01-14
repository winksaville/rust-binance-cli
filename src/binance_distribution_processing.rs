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

    //let mut rdr = csv::Reader::from_reader(line.as_bytes());
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
    _config: &Configuration,
    _subcmd: &SubCommand<'static>,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("process_dist_files:+\n config={:?} \nsubcmd={:?}", config, _subcmd);

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

    let mut last_line_index = 0usize;

    let init_hm_entry =
        |dr: &mut DistRec, line_index: usize| -> Result<(), Box<dyn std::error::Error>> {
            let rpa = match dr.realized_amount_for_primary_asset {
                Some(v) => v,
                None => dec!(0),
            };
            let rpa_usd = match dr.realized_amount_for_primary_asset_in_usd_value {
                Some(v) => v,
                None => dec!(0),
            };

            let entry = hm.entry(dr.primary_asset.clone()).or_insert(AssetRec::new(
                &dr.primary_asset,
                rpa,
                rpa_usd,
                0,
            ));
            entry.transaction_count += 1;
            if entry.transaction_count > 1 {
                // Sum realized amounts
                entry.quantity += rpa;
                entry.value_usd += rpa_usd;
            }
            total += rpa_usd;
            //println!("{}: {:?}", line_index, entry);
            print!(
                "{} {} {} {} {}                                               \r",
                line_index, entry.asset, rpa_usd, entry.value_usd, total
            );

            last_line_index = line_index;
            Ok(())
        };
    //iterate_dist_file("./test_data/dist_file_several_recs.csv", init_hm_entry).await?;
    iterate_dist_file("./data/binance.us-distribution-2021.csv", init_hm_entry).await?;
    //println!("\nhm: {:#?}", hm);

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
    assert!(std::mem::size_of::<usize>() >= std::mem::size_of::<u64>());
    assert_eq!(last_line_index as u64 + 1, total_hm_transaction_count);
    assert_eq!(total, total_hm_value_usd);
    println!(
        "Transactions: {}",
        dec_to_separated_string(Decimal::from(total_hm_transaction_count), 0)
    );
    println!("   Total USD: {} ", dec_to_money_string(total_hm_value_usd));

    //println!("process_dist_files:-");
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_1() {
        let csv = "year,make,model,description
        1948,Porsche,356,Luxury sports car
        1967,Ford,Mustang fastback 1967,American car";

        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for record in reader.records() {
            let record = record.expect("error converting record");
            println!(
                "In {}, {} built the {} model. It is a {}.",
                &record[0], &record[1], &record[2], &record[3]
            );
        }
    }

    #[test]
    fn test_2() {
        let csv = "
year,make,model,description
1948,Porsche,356,Luxury sport cars
1967,Ford,Mustang fastback 1967,American car";

        #[derive(Debug, Deserialize, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CarInfo {
            year: u64,
            make: String,
            model: String,
            description: String,
        }

        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for result in reader.deserialize() {
            println!("result: {:?}", result);
            let dr: CarInfo = result.expect("eror converting to DistRec");
            println!("dr: {:?}", dr);
        }
    }
}
