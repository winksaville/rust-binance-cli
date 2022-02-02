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
use std::{collections::BTreeMap, fs::File, io::BufReader, io::BufWriter};

use clap::ArgMatches;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::{
    binance_klines::get_kline_of_primary_asset_for_value_asset,
    common::{dec_to_money_string, dec_to_separated_string, time_ms_to_utc, utc_now_to_time_ms},
    configuration::Configuration,
    de_string_to_utc_time_ms::{de_string_to_utc_time_ms_condaddtzutc, se_time_ms_to_utc_string},
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

#[allow(unused)]
impl AssetRec {
    fn new(asset: &str) -> AssetRec {
        AssetRec {
            asset: asset.to_string(),
            quantity: dec!(0),
            value_usd: dec!(0),
            transaction_count: 0,
        }
    }
}

struct AssetRecMap {
    bt: BTreeMap<String, AssetRec>,
}

impl AssetRecMap {
    fn new() -> AssetRecMap {
        AssetRecMap {
            bt: BTreeMap::<String, AssetRec>::new(),
        }
    }

    fn inc_transaction_count(&mut self, asset: &str) {
        let entry = self.bt.get_mut(asset).unwrap();
        entry.transaction_count += 1;
    }

    fn add_quantity(&mut self, asset: &str, val: Decimal) {
        let entry = self.bt.get_mut(asset).unwrap();
        println!("add_quantity:+ {asset} {val} {}", entry.quantity);
        entry.quantity += val;
        println!("add_quantity:- {asset} {val} {}", entry.quantity);
    }

    #[allow(unused)]
    fn sub_quantity(&mut self, asset: &str, val: Decimal) {
        self.add_quantity(asset, -val)
    }

    #[allow(unused)]
    fn set_value_usd(&mut self, asset: &str, val: Decimal) {
        let entry = self.bt.get_mut(asset).unwrap();
        entry.value_usd = val;
    }
}

#[derive(Debug)]
pub struct ProcessedData {
    //pub asset_rec_map: AssetRecMap,
    pub total_distribution_value_usd: Decimal,
    pub total_count: u64,
    pub distribution_category_count: u64,
    pub distribution_operation_referral_commission_count: u64,
    pub distribution_operation_staking_reward_count: u64,
    pub distribution_operation_others_count: u64,
    pub distribution_operation_unknown_count: u64,
    pub quick_category_count: u64,
    pub quick_buy_operation_buy_count: u64,
    pub quick_buy_base_asset_in_usd_value: Decimal,
    pub quick_sell_operation_sell_count: u64,
    pub quick_sell_base_asset_in_usd_value: Decimal,
    pub quick_operation_unknown_count: u64,
    pub spot_trading_category_count: u64,
    pub spot_trading_operation_unknown_count: u64,
    pub spot_trading_operation_buy_count: u64,
    pub spot_trading_base_asset_buy_in_usd_value: Decimal,
    pub spot_trading_operation_sell_count: u64,
    pub spot_trading_base_asset_sell_in_usd_value: Decimal,
    pub withdrawal_category_count: u64,
    pub withdrawal_operation_crypto_withdrawal_count: u64,
    pub withdrawal_operation_unknown_count: u64,
    pub withdrawal_realized_amount_for_primary_asset_in_usd_value: Decimal,
    pub deposit_category_count: u64,
    pub deposit_operation_crypto_deposit_count: u64,
    pub deposit_operation_unknown_count: u64,
    pub deposit_realized_amount_for_primary_asset_in_usd_value: Decimal,
    pub unprocessed_category_count: u64,
}

impl ProcessedData {
    fn new() -> ProcessedData {
        ProcessedData {
            //asset_rec_map: AssetRecMap::new(),
            total_distribution_value_usd: dec!(0),
            total_count: 0u64,
            distribution_category_count: 0u64,
            distribution_operation_referral_commission_count: 0u64,
            distribution_operation_staking_reward_count: 0u64,
            distribution_operation_others_count: 0u64,
            distribution_operation_unknown_count: 0u64,
            quick_category_count: 0u64,
            quick_buy_operation_buy_count: 0u64,
            quick_buy_base_asset_in_usd_value: dec!(0),
            quick_sell_operation_sell_count: 0u64,
            quick_sell_base_asset_in_usd_value: dec!(0),
            quick_operation_unknown_count: 0u64,
            spot_trading_category_count: 0u64,
            spot_trading_operation_unknown_count: 0u64,
            spot_trading_operation_buy_count: 0u64,
            spot_trading_base_asset_buy_in_usd_value: dec!(0),
            spot_trading_operation_sell_count: 0u64,
            spot_trading_base_asset_sell_in_usd_value: dec!(0),
            withdrawal_category_count: 0u64,
            withdrawal_operation_crypto_withdrawal_count: 0u64,
            withdrawal_operation_unknown_count: 0u64,
            withdrawal_realized_amount_for_primary_asset_in_usd_value: dec!(0),
            deposit_category_count: 0u64,
            deposit_operation_crypto_deposit_count: 0u64,
            deposit_operation_unknown_count: 0u64,
            deposit_realized_amount_for_primary_asset_in_usd_value: dec!(0),
            unprocessed_category_count: 0u64,
        }
    }
}

async fn get_asset_in_usd_value_update_if_none(
    config: &Configuration,
    line_index: usize,
    time: i64,
    asset: &str,
    asset_value: Option<Decimal>,
    usd_value: &mut Option<Decimal>,
    verbose: bool,
) -> Result<Decimal, Box<dyn std::error::Error>> {
    let line_index = line_index + 1usize;
    // Error if there is no asset_value
    let leading_nl = if config.verbose { "\n" } else { "" };
    let asset_value = if let Some(value) = asset_value {
        value
    } else {
        return Err(format!(
            "{leading_nl}No asset_value so unable to convert {asset} at line_index: {line_index} time: {}",
            time_ms_to_utc(time)
        )
        .into());
    };
    let usd = match *usd_value {
        Some(v) => v,
        None => {
            let time_utc = time_ms_to_utc(time);
            let value_assets = ["USD", "USDT", "BUSD"];
            let (sym_name, kr) = match get_kline_of_primary_asset_for_value_asset(
                config,
                time,
                asset,
                &value_assets,
            )
            .await
            {
                Some(r) => r,
                None => {
                    return Err(
                        format!("{leading_nl}Unable to convert {asset} to {value_assets:?} at line_index: {line_index} time: {time_utc}").into()
                    );
                }
            };

            // Calculate the value in usd using the closing price of the kline, other
            // options could be avg of kr open, close, high and low ...
            let value = kr.close * asset_value;

            // Update the passed in value
            *usd_value = Some(value);

            if verbose {
                println!("{leading_nl}Updating {sym_name} value, updated to {value} for line_index: {line_index} time: {time_utc}");
            }

            value
        }
    };

    Ok(usd)
}

async fn update_all_usd_values(
    config: &Configuration,
    dr: &mut DistRec,
    line_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dr.primary_asset.is_empty() {
        let _usd_value = get_asset_in_usd_value_update_if_none(
            config,
            line_index,
            dr.time,
            &dr.primary_asset,
            dr.realized_amount_for_primary_asset,
            &mut dr.realized_amount_for_primary_asset_in_usd_value,
            true,
        )
        .await?;
    }

    if !dr.base_asset.is_empty() {
        let _usd_value = get_asset_in_usd_value_update_if_none(
            config,
            line_index,
            dr.time,
            &dr.base_asset,
            dr.realized_amount_for_base_asset,
            &mut dr.realized_amount_for_base_asset_in_usd_value,
            true,
        )
        .await?;
    }

    if !dr.quote_asset.is_empty() {
        let _usd_value = get_asset_in_usd_value_update_if_none(
            config,
            line_index,
            dr.time,
            &dr.quote_asset,
            dr.realized_amount_for_quote_asset,
            &mut dr.realized_amount_for_quote_asset_in_usd_value,
            true,
        )
        .await?;
    }

    if !dr.fee_asset.is_empty() {
        let _usd_value = get_asset_in_usd_value_update_if_none(
            config,
            line_index,
            dr.time,
            &dr.fee_asset,
            dr.realized_amount_for_fee_asset,
            &mut dr.realized_amount_for_fee_asset_in_usd_value,
            true,
        )
        .await?;
    }

    Ok(())
}

#[allow(unused)]
#[derive(PartialEq)]
enum TradeType {
    Buy,
    Sell,
}

fn trade_asset(tt: TradeType, dr: &DistRec, arm: &mut AssetRecMap) -> Result<(), Box<dyn std::error::Error>> {
    match tt {
        TradeType::Buy => {
            arm.add_quantity(&dr.base_asset, dr.realized_amount_for_base_asset.unwrap());
            arm.sub_quantity(&dr.quote_asset, dr.realized_amount_for_quote_asset.unwrap());
        }
        TradeType::Sell => {
            arm.sub_quantity(&dr.base_asset, dr.realized_amount_for_base_asset.unwrap());
            arm.add_quantity(&dr.quote_asset, dr.realized_amount_for_quote_asset.unwrap());
        }
    }

    arm.sub_quantity(&dr.fee_asset, dr.realized_amount_for_fee_asset.unwrap());

    Ok(())
}

// We assume that update_all_usd_values has been run prior
// to calling process_entry and thus can use unwrap() on
// the Option<Decimal> fields.
fn process_entry(
    config: &Configuration,
    data: &mut ProcessedData,
    arm: &mut AssetRecMap,
    dr: &DistRec,
    line_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    data.total_count += 1;

    // The asset is always either primary_asset or base_asset
    let asset_value: Decimal;
    let asset_value_usd: Decimal;
    let asset = if !dr.primary_asset.is_empty() {
        assert!(dr.base_asset.is_empty());
        asset_value = dr.realized_amount_for_primary_asset.unwrap();
        asset_value_usd = dr.realized_amount_for_primary_asset_in_usd_value.unwrap();
        &dr.primary_asset
    } else {
        asset_value = dr.realized_amount_for_base_asset.unwrap();
        asset_value_usd = dr.realized_amount_for_base_asset_in_usd_value.unwrap();
        &dr.base_asset
    };

    // Get the entry or insert a new one
    {
        let _x = arm
            .bt
            .entry(asset.to_owned())
            .or_insert_with(|| AssetRec::new(asset));
    }

    arm.inc_transaction_count(asset);

    fn dbg_x(
        _x: &str,
        line_index: usize,
        asset: &str,
        asset_value: Decimal,
        asset_value_usd: Decimal,
        category: &str,
        operation: &str,
    ) {
        //if asset == x {
        println!(
            "{line_index} {asset} {asset_value} {} {category} {operation}",
            dec_to_money_string(asset_value_usd)
        );
        //}
    }

    let leading_nl = if config.verbose { "\n" } else { "" };
    match dr.category.as_ref() {
        "Distribution" => {
            // Since invoking `get_asset_in_usd_value_update_if_none` above
            // will return an error, we can safely use unwrap().
            data.distribution_category_count += 1;

            arm.add_quantity(asset, asset_value);
            //entry.value_usd += asset_value_usd;

            dbg_x(
                "BTC",
                line_index,
                asset,
                asset_value,
                asset_value_usd,
                &dr.category,
                &dr.operation,
            );

            data.total_distribution_value_usd += asset_value_usd;
            match dr.operation.as_ref() {
                "Referral Commission" => {
                    data.distribution_operation_referral_commission_count += 1;
                }
                "Staking Rewards" => data.distribution_operation_staking_reward_count += 1,
                "Others" => data.distribution_operation_others_count += 1,
                _ => {
                    data.distribution_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Distribution unknown operation: {}",
                        line_index + 1,
                        dr.primary_asset,
                        dr.operation
                    );
                }
            }
        }
        "Quick Buy" | "Quick Sell" => {
            data.quick_category_count += 1;
            match dr.operation.as_ref() {
                "Buy" | "Sell" => {
                    if dr.operation == "Buy" {
                        trade_asset(TradeType::Buy, &dr, arm)?;

                        // TODO: save as "usd_cost_basis" asset_value_usd;
                        data.quick_buy_operation_buy_count += 1;
                        data.quick_buy_base_asset_in_usd_value += asset_value_usd;
                        dbg_x(
                            "BTC",
                            line_index,
                            asset,
                            asset_value,
                            asset_value_usd,
                            &dr.category,
                            &dr.operation,
                        );
                    } else {
                        trade_asset(TradeType::Sell, &dr, arm)?;

                        // TODO: save as "usd_cost_basis" asset_value_usd;
                        data.quick_sell_operation_sell_count += 1;
                        data.quick_sell_base_asset_in_usd_value += asset_value_usd;
                        dbg_x(
                            "BTC",
                            line_index,
                            asset,
                            -asset_value,
                            asset_value_usd,
                            &dr.category,
                            &dr.operation,
                        );
                    }
                }
                _ => {
                    data.quick_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Quick unknown operation: {}",
                        line_index + 1,
                        dr.base_asset,
                        dr.operation
                    );
                }
            }
        }
        "Spot Trading" => {
            data.spot_trading_category_count += 1;
            match dr.operation.as_ref() {
                "Buy" | "Sell" => {
                    if dr.operation == "Buy" {
                        trade_asset(TradeType::Buy, &dr, arm)?;

                        // TODO: save as "usd_cost_basis" asset_value_usd;
                        data.spot_trading_operation_buy_count += 1;
                        data.spot_trading_base_asset_buy_in_usd_value += asset_value_usd;
                        dbg_x(
                            "BTC",
                            line_index,
                            asset,
                            asset_value,
                            asset_value_usd,
                            &dr.category,
                            &dr.operation,
                        );
                    } else {
                        trade_asset(TradeType::Sell, &dr, arm)?;

                        // TODO: save as "usd_cost_basis" asset_value_usd;
                        data.spot_trading_operation_sell_count += 1;
                        data.spot_trading_base_asset_sell_in_usd_value += asset_value_usd;
                        dbg_x(
                            "BTC",
                            line_index,
                            asset,
                            -asset_value,
                            asset_value_usd,
                            &dr.category,
                            &dr.operation,
                        );
                    }
                }
                _ => {
                    data.spot_trading_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Spot trading unknown operation: {}",
                        line_index + 1,
                        dr.base_asset,
                        dr.operation
                    );
                }
            }
            //println!("{} Spot Trading: {} {entry:?}", line_index + 1, dr.operation);
        }
        "Withdrawal" => {
            data.withdrawal_category_count += 1;
            match dr.operation.as_ref() {
                "Crypto Withdrawal" => {
                    arm.sub_quantity(asset, asset_value);
                    //entry.value_usd -= asset_value_usd;

                    data.withdrawal_operation_crypto_withdrawal_count += 1;
                    data.withdrawal_realized_amount_for_primary_asset_in_usd_value +=
                        asset_value_usd;
                    dbg_x(
                        "BTC",
                        line_index,
                        asset,
                        -asset_value,
                        asset_value_usd,
                        &dr.category,
                        &dr.operation,
                    );
                }
                _ => {
                    data.withdrawal_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Withdrawal unknown operation: {}",
                        line_index + 1,
                        dr.primary_asset,
                        dr.operation
                    );
                }
            }
        }
        "Deposit" => {
            // println!("{} Deposit entry: {entry:?}", line_index + 1);
            data.deposit_category_count += 1;
            match dr.operation.as_ref() {
                "Crypto Deposit" => {
                    arm.add_quantity(asset, asset_value);
                    //entry.value_usd += asset_value_usd;
                    data.deposit_operation_crypto_deposit_count += 1;
                    data.deposit_realized_amount_for_primary_asset_in_usd_value += asset_value_usd;
                    dbg_x(
                        "BTC",
                        line_index,
                        asset,
                        asset_value,
                        asset_value_usd,
                        &dr.category,
                        &dr.operation,
                    );
                }
                "USD Deposit" => {
                    arm.add_quantity(asset, asset_value);
                    //entry.value_usd += asset_value_usd;
                    data.deposit_operation_crypto_deposit_count += 1;
                    data.deposit_realized_amount_for_primary_asset_in_usd_value += asset_value_usd;
                    dbg_x(
                        "BTC",
                        line_index,
                        asset,
                        asset_value,
                        asset_value_usd,
                        &dr.category,
                        &dr.operation,
                    );
                }
                _ => {
                    data.deposit_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Deposit unknown operation: {}",
                        line_index + 1,
                        dr.primary_asset,
                        dr.operation
                    );
                }
            }
        }
        _ => {
            data.unprocessed_category_count += 1;
            println!(
                "{leading_nl}{} Unknown category: {}",
                line_index + 1,
                dr.category
            );
        }
    }

    Ok(())
}

#[derive(PartialEq)]
pub enum ProcessType {
    Update,
    Process,
}

pub async fn process_dist_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
    process_type: ProcessType,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("process_dist_files:+ config: {config:?} sc_matches: {sc_matches:?}");

    let mut data = ProcessedData::new();

    let in_dist_file_path = sc_matches.value_of("IN_FILE").expect("IN_FILE is missing");
    let out_dist_file_path = sc_matches.value_of("OUT_FILE");

    let in_file = if let Ok(in_f) = File::open(in_dist_file_path) {
        in_f
    } else {
        return Err(format!("Unable to open {in_dist_file_path}").into());
    };
    let reader = BufReader::new(in_file);
    let writer = if let Some(out_f_path) = out_dist_file_path {
        let out_file = if let Ok(out_f) = File::create(out_f_path) {
            out_f
        } else {
            return Err(format!("Unable to create {out_f_path}").into());
        };
        Some(BufWriter::new(out_file))
    } else {
        None
    };

    // Create reader and writer
    let mut rdr = csv::Reader::from_reader(reader);

    // Clippy suggested changing this:
    //   let mut wdr = if let Some(wtr) = writer { Some(csv::Writer::from_writer(wtr)) } else { None };
    // To this:
    let mut wdr = writer.map(csv::Writer::from_writer);

    let mut asset_rec_map = AssetRecMap::new();

    for (line_index, result) in rdr.deserialize().enumerate() {
        let mut dr: DistRec = result?;

        if config.verbose {
            let asset = if !dr.primary_asset.is_empty() {
                &dr.primary_asset
            } else {
                &dr.base_asset
            };
            print!(
                "Processing {} {asset}                        \r",
                line_index + 1,
            );
        }

        match process_type {
            ProcessType::Update => update_all_usd_values(config, &mut dr, line_index).await?,
            ProcessType::Process => {
                process_entry(config, &mut data, &mut asset_rec_map, &dr, line_index)?;
            }
        }

        if let Some(w) = &mut wdr {
            w.serialize(&dr)?;
        }
    }

    match process_type {
        ProcessType::Update => println!("\nDone"),
        ProcessType::Process => {
            if config.verbose {
                println!("\n");
            }

            let mut total_value_usd = dec!(0);

            #[allow(clippy::for_kv_map)]
            for (_, ar) in &mut asset_rec_map.bt {
                let mut usd_value: Option<Decimal> = None;
                let usd: Decimal = match get_asset_in_usd_value_update_if_none(
                    config,
                    0,
                    utc_now_to_time_ms(),
                    &ar.asset.clone(),
                    Some(ar.quantity),
                    &mut usd_value,
                    false,
                )
                .await
                {
                    Ok(v) => v,
                    Err(_) => dec!(0),
                };
                ar.value_usd = usd;

                total_value_usd += ar.value_usd;
                println!(
                    "{:10} {:20.6} {:>10} {:>14}",
                    ar.asset,
                    ar.quantity,
                    dec_to_separated_string(Decimal::from(ar.transaction_count), 0),
                    dec_to_money_string(ar.value_usd)
                );
            }

            println!();
            println!(
                "Total account value: {}",
                dec_to_money_string(total_value_usd)
            );
            println!();
            println!(
                "{:>33}: {}",
                "Distribution referral commissions",
                dec_to_separated_string(
                    Decimal::from(data.distribution_operation_referral_commission_count),
                    0
                ),
            );
            println!(
                "{:>33}: {}",
                "Distribution staking rewards",
                dec_to_separated_string(
                    Decimal::from(data.distribution_operation_staking_reward_count),
                    0
                ),
            );
            println!(
                "{:>33}: {}",
                "Distribution others",
                dec_to_separated_string(Decimal::from(data.distribution_operation_others_count), 0),
            );
            //println!(
            //    "{:>33}: {} ",
            //    "Distribution count",
            //    dec_to_separated_string(Decimal::from(data.distribution_category_count), 0)
            //);
            println!(
                "{:>33}: {} ",
                "Total distribution USD",
                dec_to_money_string(data.total_distribution_value_usd)
            );
            println!(
                "{:>33}: {} ",
                "Quick Buy count",
                dec_to_separated_string(Decimal::from(data.quick_buy_operation_buy_count), 0)
            );
            println!(
                "{:>33}: {} ",
                "Quick Buy USD value",
                dec_to_money_string(data.quick_buy_base_asset_in_usd_value),
            );
            println!(
                "{:>33}: {} ",
                "Quick Sell count",
                dec_to_separated_string(Decimal::from(data.quick_sell_operation_sell_count), 0)
            );
            println!(
                "{:>33}: {} ",
                "Quick Sell USD value",
                dec_to_money_string(data.quick_sell_base_asset_in_usd_value),
            );
            //println!(
            //    "{:>33}: {} ",
            //    "Quick count",
            //    dec_to_separated_string(Decimal::from(data.quick_category_count), 0)
            //);
            println!(
                "{:>33}: {} ",
                "Spot Trading buy count",
                dec_to_separated_string(Decimal::from(data.spot_trading_operation_buy_count), 0)
            );
            println!(
                "{:>33}: {} ",
                "Spot Trading buy USD value",
                dec_to_money_string(data.spot_trading_base_asset_buy_in_usd_value),
            );
            println!(
                "{:>33}: {} ",
                "Spot Trading sell count",
                dec_to_separated_string(Decimal::from(data.spot_trading_operation_sell_count), 0)
            );
            println!(
                "{:>33}: {} ",
                "Spot Trading sell USD value",
                dec_to_money_string(data.spot_trading_base_asset_sell_in_usd_value),
            );
            //println!(
            //    "{:>33}: {} ",
            //    "Spot Trading count",
            //    dec_to_separated_string(Decimal::from(data.spot_trading_category_count), 0)
            //);
            println!(
                "{:>33}: {} ",
                "Withdrawal crypto count",
                dec_to_separated_string(
                    Decimal::from(data.withdrawal_operation_crypto_withdrawal_count),
                    0
                )
            );
            println!(
                "{:>33}: {} ",
                "Withdrawal crypto USD value",
                &dec_to_money_string(
                    data.withdrawal_realized_amount_for_primary_asset_in_usd_value
                )
            );
            println!(
                "{:>33}: {} ",
                "Deposit crypto count",
                dec_to_separated_string(
                    Decimal::from(data.deposit_operation_crypto_deposit_count),
                    0
                )
            );
            println!(
                "{:>33}: {} ",
                "Deposit crypto USD value",
                &dec_to_money_string(data.deposit_realized_amount_for_primary_asset_in_usd_value)
            );
            println!(
                "{:>33}: {} ",
                "Total count",
                dec_to_separated_string(Decimal::from(data.total_count), 0)
            );

            // Assertions!
            assert_eq!(std::mem::size_of::<usize>(), std::mem::size_of::<u64>());

            assert_eq!(
                data.distribution_category_count,
                data.distribution_operation_referral_commission_count
                    + data.distribution_operation_staking_reward_count
                    + data.distribution_operation_others_count
                    + data.distribution_operation_unknown_count
            );
            assert_eq!(data.distribution_operation_unknown_count, 0);

            assert_eq!(
                data.quick_category_count,
                data.quick_sell_operation_sell_count
                    + data.quick_buy_operation_buy_count
                    + data.quick_operation_unknown_count
            );
            assert_eq!(data.quick_operation_unknown_count, 0);

            assert_eq!(
                data.spot_trading_category_count,
                data.spot_trading_operation_buy_count
                    + data.spot_trading_operation_sell_count
                    + data.spot_trading_operation_unknown_count
            );
            assert_eq!(data.spot_trading_operation_unknown_count, 0);

            assert_eq!(
                data.withdrawal_category_count,
                data.withdrawal_operation_crypto_withdrawal_count
                    + data.withdrawal_operation_unknown_count
            );
            assert_eq!(data.withdrawal_operation_unknown_count, 0);

            assert_eq!(
                data.deposit_category_count,
                data.deposit_operation_crypto_deposit_count + data.deposit_operation_unknown_count
            );
            assert_eq!(data.deposit_operation_unknown_count, 0);

            assert_eq!(
                data.total_count,
                data.distribution_category_count
                    + data.quick_category_count
                    + data.spot_trading_category_count
                    + data.withdrawal_category_count
                    + data.deposit_category_count
                    + data.unprocessed_category_count
            );
            assert_eq!(data.unprocessed_category_count, 0);
        }
    }

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
