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
use std::{collections::BTreeMap, fs::File, io::BufReader, io::BufWriter, path::Path};

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

#[derive(Debug)]
pub struct AssetRecMap {
    bt: BTreeMap<String, AssetRec>,
}

impl AssetRecMap {
    fn new() -> AssetRecMap {
        AssetRecMap {
            bt: BTreeMap::<String, AssetRec>::new(),
        }
    }

    fn add_or_update(&mut self, asset: &str, quantity: Decimal, value_usd: Decimal) {
        let entry = self
            .bt
            .entry(asset.to_owned())
            .or_insert_with(|| AssetRec::new(asset));
        entry.quantity += quantity;
        entry.value_usd += value_usd;
        entry.transaction_count += 1;
    }

    fn inc_transaction_count(&mut self, asset: &str) {
        let entry = self.bt.get_mut(asset).unwrap();
        entry.transaction_count += 1;
    }

    fn add_quantity(&mut self, asset: &str, val: Decimal) {
        let entry = self.bt.get_mut(asset).unwrap();
        entry.quantity += val;
    }

    fn sub_quantity(&mut self, asset: &str, val: Decimal) {
        self.add_quantity(asset, -val)
    }

    #[allow(unused)]
    fn add_value_usd(&mut self, asset: &str, val: Decimal) {
        let entry = self.bt.get_mut(asset).unwrap();
        entry.value_usd += val;
    }
}

#[derive(Debug)]
pub struct ProcessedData {
    pub others_rec_map: AssetRecMap,
    pub total_count: u64,
    pub distribution_operation_referral_commission_value_usd: Decimal,
    pub distribution_operation_staking_rewards_value_usd: Decimal,
    pub distribution_operation_others_value_usd: Decimal,
    pub distribution_category_count: u64,
    pub distribution_operation_referral_commission_count: u64,
    pub distribution_operation_staking_reward_count: u64,
    pub distribution_operation_others_count: u64,
    pub distribution_operation_unknown_count: u64,
    pub quick_category_count: u64,
    pub quick_buy_operation_buy_count: u64,
    pub quick_buy_base_asset_in_usd_value: Decimal,
    pub quick_buy_operation_buy_fee_in_usd_value: Decimal,
    pub quick_sell_operation_sell_count: u64,
    pub quick_sell_base_asset_in_usd_value: Decimal,
    pub quick_sell_operation_sell_fee_in_usd_value: Decimal,
    pub quick_operation_unknown_count: u64,
    pub spot_trading_category_count: u64,
    pub spot_trading_operation_unknown_count: u64,
    pub spot_trading_operation_buy_count: u64,
    pub spot_trading_operation_buy_base_asset_in_usd_value: Decimal,
    pub spot_trading_operation_buy_fee_in_usd_value: Decimal,
    pub spot_trading_operation_sell_count: u64,
    pub spot_trading_operation_sell_base_asset_in_usd_value: Decimal,
    pub spot_trading_operation_sell_fee_in_usd_value: Decimal,
    pub withdrawal_category_count: u64,
    pub withdrawal_operation_crypto_withdrawal_count: u64,
    pub withdrawal_operation_crypto_withdrawal_usd_value: Decimal,
    pub withdrawal_operation_crypto_withdrawal_fee_count: u64,
    pub withdrawal_operation_crypto_withdrawal_fee_in_usd_value: Decimal,
    pub withdrawal_operation_unknown_count: u64,
    pub deposit_category_count: u64,
    pub deposit_operation_crypto_deposit_count: u64,
    pub deposit_operation_crypto_deposit_usd_value: Decimal,
    pub deposit_operation_crypto_deposit_fee_count: u64,
    pub deposit_operation_usd_deposit_count: u64,
    pub deposit_operation_usd_deposit_usd_value: Decimal,
    pub deposit_operaiton_usd_deposit_fee_count: u64,
    pub deposit_operation_usd_deposit_fee_usd_value: Decimal,
    pub deposit_operation_unknown_count: u64,
    pub unprocessed_category_count: u64,
}

impl ProcessedData {
    fn new() -> ProcessedData {
        ProcessedData {
            others_rec_map: AssetRecMap::new(),
            total_count: 0u64,
            distribution_operation_referral_commission_value_usd: dec!(0),
            distribution_operation_staking_rewards_value_usd: dec!(0),
            distribution_operation_others_value_usd: dec!(0),
            distribution_category_count: 0u64,
            distribution_operation_referral_commission_count: 0u64,
            distribution_operation_staking_reward_count: 0u64,
            distribution_operation_others_count: 0u64,
            distribution_operation_unknown_count: 0u64,
            quick_category_count: 0u64,
            quick_buy_operation_buy_count: 0u64,
            quick_buy_base_asset_in_usd_value: dec!(0),
            quick_buy_operation_buy_fee_in_usd_value: dec!(0),
            quick_sell_operation_sell_count: 0u64,
            quick_sell_base_asset_in_usd_value: dec!(0),
            quick_sell_operation_sell_fee_in_usd_value: dec!(0),
            quick_operation_unknown_count: 0u64,
            spot_trading_category_count: 0u64,
            spot_trading_operation_unknown_count: 0u64,
            spot_trading_operation_buy_count: 0u64,
            spot_trading_operation_buy_base_asset_in_usd_value: dec!(0),
            spot_trading_operation_buy_fee_in_usd_value: dec!(0),
            spot_trading_operation_sell_count: 0u64,
            spot_trading_operation_sell_base_asset_in_usd_value: dec!(0),
            spot_trading_operation_sell_fee_in_usd_value: dec!(0),
            withdrawal_category_count: 0u64,
            withdrawal_operation_crypto_withdrawal_count: 0u64,
            withdrawal_operation_crypto_withdrawal_usd_value: dec!(0),
            withdrawal_operation_crypto_withdrawal_fee_count: 0u64,
            withdrawal_operation_crypto_withdrawal_fee_in_usd_value: dec!(0),
            withdrawal_operation_unknown_count: 0u64,
            deposit_category_count: 0u64,
            deposit_operation_crypto_deposit_count: 0u64,
            deposit_operation_crypto_deposit_usd_value: dec!(0),
            deposit_operation_crypto_deposit_fee_count: 0u64,
            deposit_operation_usd_deposit_count: 0u64,
            deposit_operation_usd_deposit_usd_value: dec!(0),
            deposit_operaiton_usd_deposit_fee_count: 0u64,
            deposit_operation_usd_deposit_fee_usd_value: dec!(0),
            deposit_operation_unknown_count: 0u64,
            unprocessed_category_count: 0u64,
        }
    }
}

async fn get_asset_in_usd_value_update_if_none(
    config: &Configuration,
    line_number: usize,
    time: i64,
    asset: &str,
    asset_value: Option<Decimal>,
    usd_value: &mut Option<Decimal>,
    verbose: bool,
) -> Result<Decimal, Box<dyn std::error::Error>> {
    if asset == "USD" {
        *usd_value = asset_value;
        let v = asset_value.unwrap();
        return Ok(v);
    }

    // Error if there is no asset_value
    let leading_nl = if config.verbose { "\n" } else { "" };
    let asset_value = if let Some(value) = asset_value {
        value
    } else {
        return Err(format!(
            "{leading_nl}No asset_value so unable to convert {asset} at line_number: {line_number} time: {}",
            time_ms_to_utc(time)
        )
        .into());
    };
    let time_utc = time_ms_to_utc(time);
    let usd = match *usd_value {
        Some(v) => {
            //if verbose {
            //    println!("{leading_nl}USD value for {asset} is {v} for line_number: {line_number} time: {time_utc}");
            //}

            v
        }
        None => {
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
                        format!("{leading_nl}Unable to convert {asset} to {value_assets:?} at line_number: {line_number} time: {time_utc}").into()
                    );
                }
            };

            // Calculate the value in usd using the closing price of the kline, other
            // options could be avg of kr open, close, high and low ...
            let value = kr.close * asset_value;

            // Update the passed in value
            *usd_value = Some(value);

            if verbose {
                println!("{leading_nl}Updating {sym_name} value to {value} for line_number: {line_number} time: {time_utc}");
            }

            value
        }
    };

    Ok(usd)
}

async fn update_all_usd_values(
    config: &Configuration,
    dr: &mut DistRec,
    line_number: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    //let line_number = rec_index + 2;
    if !dr.primary_asset.is_empty() {
        let _usd_value = get_asset_in_usd_value_update_if_none(
            config,
            line_number,
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
            line_number,
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
            line_number,
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
            line_number,
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
fn dbg_x(
    x: &str,
    line_number: usize,
    asset: &str,
    asset_value: Decimal,
    asset_value_usd: Decimal,
    category: &str,
    operation: &str,
) {
    if x.is_empty() || asset == x {
        println!(
            "{line_number} {asset} {asset_value} {} {category} {operation}",
            dec_to_money_string(asset_value_usd)
        );
    }
}

#[derive(PartialEq, Debug)]
enum TradeType {
    Buy,
    Sell,
}

fn trade_asset(
    tt: TradeType,
    dr: &DistRec,
    arm: &mut AssetRecMap,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("trade_asset:+\ntt: {tt:?}\nbase_asset ar: {:?}\nquote_asset ar: {:?}\nfee_asset ar: {:?}",
    //     arm.bt.get(&dr.base_asset), arm.bt.get(&dr.quote_asset), arm.bt.get(&dr.fee_asset));
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
    line_number: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    data.total_count += 1;

    // The asset is always either primary_asset or base_asset
    let asset_value: Decimal;
    let asset_value_usd: Decimal;
    let asset = if !dr.primary_asset.is_empty() {
        assert!(dr.base_asset.is_empty());
        asset_value = dr.realized_amount_for_primary_asset.unwrap_or_else(|| {
            panic!(
                "{} has no realized_amount_for_primary_asset at line {line_number}",
                dr.primary_asset
            )
        });
        asset_value_usd = dr.realized_amount_for_primary_asset_in_usd_value
                        .unwrap_or_else(|| {
                            panic!("{} has no realized_amount_for_primary_asset_in_usd_value at line {line_number}", dr.primary_asset)
                        });
        &dr.primary_asset
    } else if !dr.base_asset.is_empty() {
        asset_value = dr.realized_amount_for_base_asset.unwrap_or_else(|| {
            panic!(
                "{} has no realized_amount_for_base_asset_in_usd_value at line {line_number}",
                dr.primary_asset
            )
        });
        asset_value_usd = dr
            .realized_amount_for_base_asset_in_usd_value
            .unwrap_or_else(|| {
                panic!(
                    "{} has no realized_amount_for_base_asset_in_usd_value at line {line_number}",
                    dr.primary_asset
                )
            });
        &dr.base_asset
    } else {
        panic!("No primary_asset or base_asset at line {line_number}");
    };

    // Add possibly missing AssetRecMap entries
    {
        let _ = arm.bt.entry(asset.to_owned()).or_insert_with(|| {
            //println!("Adding missing asset: {}", asset);
            AssetRec::new(asset)
        });
        if !dr.quote_asset.is_empty() {
            let _ = arm.bt.entry(dr.quote_asset.to_owned()).or_insert_with(|| {
                println!("WARNING adding missing quote_asset: {}", dr.quote_asset);
                AssetRec::new(&dr.quote_asset)
            });
        }
        if !dr.fee_asset.is_empty() {
            let _ = arm.bt.entry(dr.fee_asset.to_owned()).or_insert_with(|| {
                println!("WARNING adding missing fee_asset: {}", dr.fee_asset);
                AssetRec::new(&dr.fee_asset)
            });
        }
    }

    arm.inc_transaction_count(asset);

    let leading_nl = if config.verbose { "\n" } else { "" };

    // TODO: For all the category and operations we need to save asset_value_usd as "usd_cost_basis"
    match dr.category.as_ref() {
        "Distribution" => {
            // Since invoking `get_asset_in_usd_value_update_if_none` above
            // will return an error, we can safely use unwrap().
            data.distribution_category_count += 1;

            arm.add_quantity(asset, asset_value);
            if !dr.fee_asset.is_empty() {
                println!(
                    "Distribution fee: {} {:?}",
                    dr.fee_asset, dr.realized_amount_for_fee_asset
                );
                arm.sub_quantity(&dr.fee_asset, dr.realized_amount_for_fee_asset.unwrap());
            }

            match dr.operation.as_ref() {
                "Referral Commission" => {
                    data.distribution_operation_referral_commission_count += 1;
                    data.distribution_operation_referral_commission_value_usd += asset_value_usd;
                }
                "Staking Rewards" => {
                    data.distribution_operation_staking_reward_count += 1;
                    data.distribution_operation_staking_rewards_value_usd += asset_value_usd;
                }
                "Others" => {
                    data.distribution_operation_others_count += 1;
                    data.distribution_operation_others_value_usd += asset_value_usd;
                    data.others_rec_map
                        .add_or_update(asset, asset_value, asset_value_usd);
                }
                _ => {
                    data.distribution_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Distribution unknown operation: {}",
                        line_number, dr.primary_asset, dr.operation
                    );
                }
            }
        }
        "Quick Buy" | "Quick Sell" => {
            data.quick_category_count += 1;
            match dr.operation.as_ref() {
                "Buy" => {
                    trade_asset(TradeType::Buy, dr, arm)?;

                    data.quick_buy_operation_buy_count += 1;
                    data.quick_buy_base_asset_in_usd_value += asset_value_usd;
                    data.quick_buy_operation_buy_fee_in_usd_value += dr
                        .realized_amount_for_fee_asset_in_usd_value
                        .unwrap_or_else(|| {
                            panic!("Quick Buy of {asset} has no fee at line {line_number}")
                        });
                }
                "Sell" => {
                    trade_asset(TradeType::Sell, dr, arm)?;

                    data.quick_sell_operation_sell_count += 1;
                    data.quick_sell_base_asset_in_usd_value += asset_value_usd;
                    data.quick_sell_operation_sell_fee_in_usd_value += dr
                        .realized_amount_for_fee_asset_in_usd_value
                        .unwrap_or_else(|| {
                            panic!("Quick Sell of {asset} has no fee at line {line_number}")
                        });
                }
                _ => {
                    data.quick_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Quick unknown operation: {}",
                        line_number, dr.base_asset, dr.operation
                    );
                }
            }
        }
        "Spot Trading" => {
            data.spot_trading_category_count += 1;
            match dr.operation.as_ref() {
                "Buy" => {
                    trade_asset(TradeType::Buy, dr, arm)?;

                    data.spot_trading_operation_buy_count += 1;
                    data.spot_trading_operation_buy_base_asset_in_usd_value += asset_value_usd;
                    data.spot_trading_operation_buy_fee_in_usd_value += dr
                        .realized_amount_for_fee_asset_in_usd_value
                        .unwrap_or_else(|| {
                            panic!("Spot Trading Buy of {asset} has no fee at line {line_number}")
                        });
                }
                "Sell" => {
                    trade_asset(TradeType::Sell, dr, arm)?;

                    data.spot_trading_operation_sell_count += 1;
                    data.spot_trading_operation_sell_base_asset_in_usd_value += asset_value_usd;
                    data.spot_trading_operation_sell_fee_in_usd_value += dr
                        .realized_amount_for_fee_asset_in_usd_value
                        .unwrap_or_else(|| {
                            panic!("Spot Trading Sell of {asset} has no fee at line {line_number}")
                        });
                }
                _ => {
                    data.spot_trading_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Spot trading unknown operation: {}",
                        line_number, dr.base_asset, dr.operation
                    );
                }
            }
            //println!("{} Spot Trading: {} {entry:?}", line_number, dr.operation);
        }
        "Withdrawal" => {
            data.withdrawal_category_count += 1;
            match dr.operation.as_ref() {
                "Crypto Withdrawal" => {
                    arm.sub_quantity(asset, asset_value);
                    if !dr.fee_asset.is_empty() {
                        //println!("Crypto Withdrawal fee: {} {} {:?}", dr.fee_asset, dec_to_money_string(dr.realized_amount_for_fee_asset_in_usd_value.unwrap()), dr.realized_amount_for_fee_asset);
                        arm.sub_quantity(&dr.fee_asset, dr.realized_amount_for_fee_asset.unwrap());
                        data.withdrawal_operation_crypto_withdrawal_fee_count += 1;
                        data.withdrawal_operation_crypto_withdrawal_fee_in_usd_value +=
                            dr.realized_amount_for_fee_asset_in_usd_value.unwrap();
                    }

                    data.withdrawal_operation_crypto_withdrawal_count += 1;
                    data.withdrawal_operation_crypto_withdrawal_usd_value += asset_value_usd;
                }
                _ => {
                    data.withdrawal_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Withdrawal unknown operation: {}",
                        line_number, dr.primary_asset, dr.operation
                    );
                }
            }
        }
        "Deposit" => {
            // println!("{} Deposit entry: {entry:?}", line_number);
            data.deposit_category_count += 1;
            match dr.operation.as_ref() {
                "Crypto Deposit" => {
                    arm.add_quantity(asset, asset_value);
                    if !dr.fee_asset.is_empty() {
                        println!(
                            "Crypto Deposit fee: {} {:?}",
                            dr.fee_asset, dr.realized_amount_for_fee_asset
                        );
                        data.deposit_operation_crypto_deposit_fee_count += 1;
                        // TODO: CryptoDepositFee:
                        //   If this occurs it needs to be per asset, and
                        //    we'll need to add a new field to the AssetRec or
                        //    keep a separate BTreeMap<AssetRec> with fees!
                        //data.total_crypto_deposit_fee += dr.realized_amount_for_fee_asset_in_usd_value;
                    }

                    //entry.value_usd += asset_value_usd;
                    data.deposit_operation_crypto_deposit_count += 1;
                    data.deposit_operation_crypto_deposit_usd_value += asset_value_usd;
                }
                "USD Deposit" => {
                    arm.add_quantity(asset, asset_value);
                    if !dr.fee_asset.is_empty() {
                        // This is subtracted on the way in so this needs to be tracked in a separate
                        // "external_fees: BTreeMap<AssetRec>" collection. Especially if total_crypto_deposit_fee_count != 0.
                        println!(
                            "USD Deposit fee: {} {:?}",
                            dr.fee_asset, dr.realized_amount_for_fee_asset
                        );
                        data.deposit_operaiton_usd_deposit_fee_count += 1;
                        data.deposit_operation_usd_deposit_fee_usd_value +=
                            dr.realized_amount_for_fee_asset_in_usd_value.unwrap();
                    }

                    data.deposit_operation_usd_deposit_count += 1;
                    data.deposit_operation_usd_deposit_usd_value += asset_value_usd;
                }
                _ => {
                    data.deposit_operation_unknown_count += 1;
                    println!(
                        "{leading_nl}{} {} Deposit unknown operation: {}",
                        line_number, dr.primary_asset, dr.operation
                    );
                }
            }
        }
        _ => {
            data.unprocessed_category_count += 1;
            println!(
                "{leading_nl}{} Unknown category: {}",
                line_number, dr.category
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

#[derive(PartialEq)]
pub enum ProcessDistSubCommand {
    Udf,
    Pdf,
}

pub async fn process_dist_files(
    config: &Configuration,
    subcmd: ProcessDistSubCommand,
    sc_matches: &ArgMatches,
    process_type: ProcessType,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("process_dist_files:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let mut data = ProcessedData::new();
    let mut asset_rec_map = AssetRecMap::new();

    let in_dist_file_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();
    let out_dist_file_path = if subcmd == ProcessDistSubCommand::Udf {
        sc_matches.value_of("OUT_FILE")
    } else {
        None
    };

    //println!("in_dist_file_path: {in_dist_file_paths:?}");
    //println!("out_dist_file_path: {out_dist_file_path:?}");

    // Verify all input files exist
    for f in &in_dist_file_paths {
        if !Path::new(f).exists() {
            return Err(format!("{} does not exist", *f).into());
        };
    }

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

    // Clippy suggested changing this:
    //   let mut wdr = if let Some(wtr) = writer { Some(csv::Writer::from_writer(wtr)) } else { None };
    // To this:
    let mut wdr = writer.map(csv::Writer::from_writer);

    for f in &in_dist_file_paths {
        let in_file = if let Ok(in_f) = File::open(*f) {
            in_f
        } else {
            return Err(format!("Unable to open {f}").into());
        };
        let reader = BufReader::new(in_file);

        // Create reader
        let mut rdr = csv::Reader::from_reader(reader);

        for (rec_index, result) in rdr.deserialize().enumerate() {
            let line_number = rec_index + 2;
            let mut dr: DistRec = result?;

            if config.verbose {
                let asset = if !dr.primary_asset.is_empty() {
                    &dr.primary_asset
                } else {
                    &dr.base_asset
                };
                print!("Processing {line_number} {asset}                        \r",);
            }

            match process_type {
                ProcessType::Update => update_all_usd_values(config, &mut dr, line_number).await?,
                ProcessType::Process => {
                    process_entry(config, &mut data, &mut asset_rec_map, &dr, line_number)?;
                }
            }

            if let Some(w) = &mut wdr {
                w.serialize(&dr)?;
            }
        }
    }

    match process_type {
        ProcessType::Update => println!("\nDone"),
        ProcessType::Process => {
            if config.verbose {
                println!("\n");
            }

            if config.verbose {
                let mut total_value_usd = dec!(0);

                let col_1_width = 10;
                let col_2_width = 20;
                let col_3_width = 10;
                let col_4_width = 14;
                println!(
                    "{:col_1_width$} {:>col_2_width$} {:>col_3_width$} {:>col_4_width$}",
                    "Asset", "Quantity", "Txs count", "USD value today",
                );

                #[allow(clippy::for_kv_map)]
                for (_, ar) in &mut asset_rec_map.bt {
                    let mut _usd_value: Option<Decimal> = None;
                    ar.value_usd = match get_asset_in_usd_value_update_if_none(
                        config,
                        0,
                        utc_now_to_time_ms(),
                        &ar.asset.clone(),
                        Some(ar.quantity),
                        &mut _usd_value,
                        false,
                    )
                    .await
                    {
                        Ok(v) => v,
                        Err(_) => dec!(0),
                    };

                    total_value_usd += ar.value_usd;
                    println!(
                        "{:col_1_width$} {:>col_2_width$} {:>col_3_width$} {:>col_4_width$}",
                        ar.asset,
                        dec_to_separated_string(ar.quantity, 8),
                        dec_to_separated_string(Decimal::from(ar.transaction_count), 0),
                        dec_to_money_string(ar.value_usd)
                    );
                }

                println!();
                println!(
                    "Total account value: {}",
                    dec_to_money_string(total_value_usd)
                );
            }

            let lbl_width = 45;
            let cnt_width = 10;
            let val_width = 14;
            let fee_width = 14;
            println!(
                "{:>lbl_width$}  {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Operation", "Count", "USD Value", "Fee USD Value",
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Distribution Referral Commission USD value",
                dec_to_separated_string(
                    Decimal::from(data.distribution_operation_referral_commission_count),
                    0
                ),
                dec_to_money_string(data.distribution_operation_referral_commission_value_usd),
                "",
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Distribution Staking Reward USD value",
                dec_to_separated_string(
                    Decimal::from(data.distribution_operation_staking_reward_count),
                    0
                ),
                dec_to_money_string(data.distribution_operation_staking_rewards_value_usd),
                "",
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "* Distribution Others USD value",
                dec_to_separated_string(Decimal::from(data.distribution_operation_others_count), 0),
                dec_to_money_string(data.distribution_operation_others_value_usd),
                "",
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Quick Buy",
                dec_to_separated_string(Decimal::from(data.quick_buy_operation_buy_count), 0),
                dec_to_money_string(data.quick_buy_base_asset_in_usd_value),
                dec_to_money_string(data.quick_buy_operation_buy_fee_in_usd_value)
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Quick Sell",
                dec_to_separated_string(Decimal::from(data.quick_sell_operation_sell_count), 0),
                dec_to_money_string(data.quick_sell_base_asset_in_usd_value),
                dec_to_money_string(data.quick_sell_operation_sell_fee_in_usd_value)
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Spot Trading Buy",
                dec_to_separated_string(Decimal::from(data.spot_trading_operation_buy_count), 0),
                dec_to_money_string(data.spot_trading_operation_buy_base_asset_in_usd_value),
                dec_to_money_string(data.spot_trading_operation_buy_fee_in_usd_value)
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Spot Trading Sell",
                dec_to_separated_string(Decimal::from(data.spot_trading_operation_sell_count), 0),
                dec_to_money_string(data.spot_trading_operation_sell_base_asset_in_usd_value),
                dec_to_money_string(data.spot_trading_operation_sell_fee_in_usd_value)
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Withdrawal crypto",
                dec_to_separated_string(
                    Decimal::from(data.withdrawal_operation_crypto_withdrawal_count),
                    0
                ),
                dec_to_money_string(data.withdrawal_operation_crypto_withdrawal_usd_value),
                dec_to_money_string(data.withdrawal_operation_crypto_withdrawal_fee_in_usd_value)
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Deposit crypto",
                dec_to_separated_string(
                    Decimal::from(data.deposit_operation_crypto_deposit_count),
                    0
                ),
                dec_to_money_string(data.deposit_operation_crypto_deposit_usd_value),
                "",
            );
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Deposit USD",
                dec_to_separated_string(
                    Decimal::from(data.deposit_operation_crypto_deposit_count),
                    0
                ),
                dec_to_money_string(data.deposit_operation_usd_deposit_usd_value),
                dec_to_money_string(data.deposit_operation_usd_deposit_fee_usd_value)
            );
            let fees_usd_value = data.quick_buy_operation_buy_fee_in_usd_value
                + data.quick_sell_operation_sell_fee_in_usd_value
                + data.spot_trading_operation_buy_fee_in_usd_value
                + data.spot_trading_operation_sell_fee_in_usd_value
                + data.withdrawal_operation_crypto_withdrawal_fee_in_usd_value
                + data.deposit_operation_usd_deposit_fee_usd_value;
            println!(
                "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
                "Totals",
                dec_to_separated_string(Decimal::from(data.total_count), 0),
                "",
                dec_to_money_string(fees_usd_value),
            );

            println!();
            println!("* Distribution Others:");
            // Output others
            let col_1_width = 10;
            let col_2_width = 20;
            let col_3_width = 10;
            let col_4_width = 14;
            println!(
                "{:col_1_width$} {:>col_2_width$} {:>col_3_width$} {:>col_4_width$}",
                "Asset", "Quantity", "Txs count", "USD value",
            );

            let mut others_value = dec!(0);

            #[allow(clippy::for_kv_map)]
            for (_, entry) in &data.others_rec_map.bt {
                others_value += entry.value_usd;
                println!(
                    "{:col_1_width$} {:>col_2_width$} {:>col_3_width$} {:>col_4_width$}",
                    entry.asset,
                    entry.quantity,
                    entry.transaction_count,
                    dec_to_money_string(entry.value_usd),
                );
            }
            assert_eq!(others_value, data.distribution_operation_others_value_usd);

            // Assertions!
            assert_eq!(std::mem::size_of::<usize>(), std::mem::size_of::<u64>());

            assert_eq!(
                data.deposit_operation_crypto_deposit_fee_count, 0,
                "See TODO: CryptoDepositFee"
            );

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
                data.deposit_operation_crypto_deposit_count
                    + data.deposit_operation_usd_deposit_count
                    + data.deposit_operation_unknown_count
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
