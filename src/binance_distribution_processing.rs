//use std::{error::Error, fmt, fs::File, io::BufRead, io::BufReader, path::PathBuf};
use std::{fs::File, io::BufReader}; //, Read}, path::PathBuf};

use clap::SubCommand;
//use log::trace;
//use serde::{Deserialize, Serialize};

use rust_decimal::prelude::*;
//use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
//use semver::Version;

use crate::{common::dec_to_money_string, configuration::Configuration};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistRec {
    #[serde(rename = "User_Id")]
    pub user_id: u64,
    #[serde(rename = "Time")]
    pub time: String, // TODO: Convert to an i64 assume UTC
    #[serde(rename = "Category")]
    pub category: String,
    #[serde(rename = "Operation")]
    pub operation: String,
    #[serde(rename = "Order_Id")]
    pub order_id: u64,
    #[serde(rename = "Transaction_Id")]
    pub transaction_id: u64,
    #[serde(rename = "Primary_Asset")]
    pub primary_asset: String,
    #[serde(rename = "Realized_Amount_For_Primary_Asset")]
    pub realized_amount_for_primary_asset: Decimal,
    #[serde(rename = "Realized_Amount_For_Primary_Asset_In_USD_Value")]
    pub realized_amount_for_primary_asset_in_usd_value: Decimal,
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
    process_line: impl Fn(&DistRec, usize) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("iterate_dist_reader:+");

    //let mut rdr = csv::Reader::from_reader(line.as_bytes());
    let mut rdr = csv::Reader::from_reader(reader);
    for (line_index, result) in rdr.deserialize().enumerate() {
        let dr = result?;
        process_line(&dr, line_index)?;
    }

    //println!("iterate_dist_reader:-");
    Ok(())
}

pub async fn iterate_dist_file(
    dist_file_str: &str,
    process_line: impl Fn(&DistRec, usize) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("iterate_distribution_file:+ dist_file_path={}", dist_file_str);

    let file = File::open(dist_file_str)?;
    let reader = BufReader::new(file);
    iterate_dist_reader(reader, process_line).await?;

    //println!("iterate_distribution_file:- dist_file_path={}", dist_file_str);
    Ok(())
}

pub fn display_rec(dr: &DistRec, line_index: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{:9}: {:>10}     {:25} {:5} {:16.8}  {:>12}  {:>12}",
        line_index + 1,
        dr.user_id,
        dr.time,
        dr.primary_asset,
        dr.realized_amount_for_primary_asset,
        dec_to_money_string(dr.realized_amount_for_primary_asset_in_usd_value),
        dec_to_money_string(
            dr.realized_amount_for_primary_asset_in_usd_value
                / dr.realized_amount_for_primary_asset
        )
    );
    Ok(())
}

#[allow(unused)]
pub fn display_full_rec(dr: &DistRec, line_index: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}: {:?}", line_index + 1, dr);
    Ok(())
}

pub async fn process_dist_files(
    _config: &Configuration,
    _subcmd: &SubCommand<'static>,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("process_dist_files:+\n config={:?} \nsubcmd={:?}", config, _subcmd);

    //iterate_dist_file("./test_data/dist_file_one_rec.csv", display_rec).await?;
    iterate_dist_file("./test_data/dist_file_several_recs.csv", display_rec).await?;

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
