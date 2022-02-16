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
//!      12345678,2021-12-31 00:07:03.819,Distribution,Referral Commission,88367941,880499527,SUSHI,0.00224,"","","","","","","","","","",Wallet,"",""
//!    So for these I need to "lookup and calcuate" the Realized_Amount_For_Primary_Asset_In_USD_Value.
//!

//!
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::File,
    io::BufReader,
    io::BufWriter,
    path::{Path, PathBuf},
};

use clap::ArgMatches;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::{
    binance_klines::get_kline_of_primary_asset_for_value_asset,
    common::{dec_to_money_string, dec_to_separated_string, time_ms_to_utc, utc_now_to_time_ms},
    configuration::Configuration,
    de_string_to_utc_time_ms::{de_string_to_utc_time_ms_condaddtzutc, se_time_ms_to_utc_string},
    token_tax::{TokenTaxRec, TypeTxs},
    token_tax_comment_vers::TT_CMT_VER0,
};

#[derive(Debug, Default, Deserialize, Serialize, Clone, Ord, Eq, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
struct DistRec {
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

#[allow(unused)]
impl DistRec {
    fn get_asset(&self) -> &str {
        if self.primary_asset.is_empty() {
            assert!(!self.base_asset.is_empty());
            &self.base_asset
        } else {
            &self.primary_asset
        }
    }

    fn get_value(&self) -> Decimal {
        if self.primary_asset.is_empty() {
            self.realized_amount_for_base_asset.expect("WTF")
        } else {
            self.realized_amount_for_primary_asset.expect("WTF")
        }
    }

    fn get_value_usd(&self) -> Decimal {
        if self.primary_asset.is_empty() {
            self.realized_amount_for_base_asset_in_usd_value
                .expect("WTF")
        } else {
            self.realized_amount_for_primary_asset_in_usd_value
                .expect("WTF")
        }
    }
}

#[derive(Debug)]
struct AssetRec {
    asset: String,
    quantity: Decimal,
    value_usd: Decimal,
    transaction_count: u64,
    dist_rec_vec: Vec<DistRec>,
    consolidated_dist_rec_vec: Vec<DistRec>,
}

#[allow(unused)]
impl AssetRec {
    fn new(asset: &str) -> AssetRec {
        AssetRec {
            asset: asset.to_string(),
            quantity: dec!(0),
            value_usd: dec!(0),
            transaction_count: 0,
            dist_rec_vec: Vec::new(),
            consolidated_dist_rec_vec: Vec::new(),
        }
    }

    // This invoking this causes a compile error, maybe make a process macro?
    //    error[E0499]: cannot borrow `*self` as mutable more than once at a time
    //#[allow(unused)]
    //fn consolidate(&mut self, cdr: &mut DistRec, dr: &DistRec) {
    //    //let cdr = self.consolidated_dist_rec_vec.last_mut().expect("WTF");
    //    assert_eq!(cdr.primary_asset, dr.primary_asset);

    //    let a = dr.realized_amount_for_primary_asset.expect("WTF");
    //    let b = cdr.realized_amount_for_primary_asset.expect("WTF");
    //    cdr.realized_amount_for_primary_asset = Some(a + b);

    //    let a = dr
    //        .realized_amount_for_primary_asset_in_usd_value
    //        .expect("WTF");
    //    let b = cdr
    //        .realized_amount_for_primary_asset_in_usd_value
    //        .expect("WTF");
    //    cdr.realized_amount_for_primary_asset_in_usd_value = Some(a + b);
    //}

    // This invoking this causes a compile error, maybe make a process macro?
    #[allow(unused)]
    fn consolidate_x(&self, cdr: &DistRec, dr: &DistRec) -> (Decimal, Decimal) {
        //let cdr = self.consolidated_dist_rec_vec.last_mut().expect("WTF");
        assert_eq!(cdr.primary_asset, dr.primary_asset);

        let a = dr.realized_amount_for_primary_asset.expect("WTF");
        let b = cdr.realized_amount_for_primary_asset.expect("WTF");
        let value = a + b;

        let a = dr
            .realized_amount_for_primary_asset_in_usd_value
            .expect("WTF");
        let b = cdr
            .realized_amount_for_primary_asset_in_usd_value
            .expect("WTF");
        let value_usd = a + b;

        (value, value_usd)
    }

    fn consolidate_distributions(
        &mut self,
        config: &Configuration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //println!("consolidate_distributions:+");

        #[derive(Debug)]
        enum State {
            LookingForDistribution,
            UpdatingDistributionReferral,
            UpdatingDistributionStaking,
            UpdatingDistributionOthers,
        };

        let mut state = State::LookingForDistribution;

        let mut time_of_next_consolidation_window = 0i64;
        const MS_PER_DAY: i64 = (60 * 60 * 24) * 1000;

        for dr in &self.dist_rec_vec {
            let asset = dr.get_asset();
            //println!("{state:?} asset: {asset} category: {}", dr.category);
            match state {
                State::LookingForDistribution => {
                    self.consolidated_dist_rec_vec.push(dr.clone());
                    if dr.category == "Distribution" {
                        match dr.operation.as_str() {
                            "Referral Commission" => state = State::UpdatingDistributionReferral,
                            "Staking Rewards" => state = State::UpdatingDistributionStaking,
                            "Others" => state = State::UpdatingDistributionOthers,
                            _ => panic!("Unknown operation: {}", &dr.operation),
                        }

                        // Time in ms of next consolidation window
                        time_of_next_consolidation_window =
                            ((dr.time + MS_PER_DAY) / MS_PER_DAY) * MS_PER_DAY;
                    } else {
                        //println!(
                        //    "consolidate_distributions: LookingForDistribution found {}",
                        //    dr.category
                        //);
                    }
                }
                State::UpdatingDistributionReferral => {
                    if (dr.category == "Distribution") && (dr.operation == "Referral Commission") {
                        if (dr.time < time_of_next_consolidation_window) {
                            // Same the Current Time Window

                            //let cdr = self.consolidated_dist_rec_vec.last_mut().expect("WTF");
                            //self.consolidate(cdr, dr); // error[E0499]: cannot borrow `*self` as mutable more than once at a time
                            let cdr = self.consolidated_dist_rec_vec.last().expect("WTF");
                            let (value, value_usd) = self.consolidate_x(cdr, dr);

                            let cdr = self.consolidated_dist_rec_vec.last_mut().expect("WTF");
                            cdr.realized_amount_for_primary_asset = Some(value);
                            cdr.realized_amount_for_primary_asset_in_usd_value = Some(value_usd);
                            cdr.time = dr.time; // Last entry will be used as the time for the consolidated record
                            cdr.order_id = dr.order_id.clone();
                            cdr.transaction_id = dr.transaction_id;
                        } else {
                            // Add this record as we're Starting a new time window
                            self.consolidated_dist_rec_vec.push(dr.clone());

                            // Calculate New Time Window
                            time_of_next_consolidation_window =
                                ((dr.time + MS_PER_DAY) / MS_PER_DAY) * MS_PER_DAY;
                        }
                    } else {
                        //println!(
                        //    "consolidate_distributions {asset}: Not Distribution Referral Comission, back to looking"
                        //);
                        self.consolidated_dist_rec_vec.push(dr.clone());
                        state = State::LookingForDistribution;
                    }
                }
                State::UpdatingDistributionStaking => {
                    if (dr.category == "Distribution") && (dr.operation == "Staking Rewards") {
                        if (dr.time < time_of_next_consolidation_window) {
                            // Same the Current Time Window

                            //let cdr = self.consolidated_dist_rec_vec.last_mut().expect("WTF");
                            //self.consolidate(cdr, dr); // error[E0499]: cannot borrow `*self` as mutable more than once at a time
                            let cdr = self.consolidated_dist_rec_vec.last().expect("WTF");
                            let (value, value_usd) = self.consolidate_x(cdr, dr);

                            let cdr = self.consolidated_dist_rec_vec.last_mut().expect("WTF");
                            cdr.realized_amount_for_primary_asset = Some(value);
                            cdr.realized_amount_for_primary_asset_in_usd_value = Some(value_usd);
                            cdr.time = dr.time; // Last entry will be used as the time for the consolidated record
                            cdr.order_id = dr.order_id.clone();
                            cdr.transaction_id = dr.transaction_id;
                        } else {
                            // Add this record as we're Starting a new time window
                            self.consolidated_dist_rec_vec.push(dr.clone());

                            // Calculate New Time Window
                            time_of_next_consolidation_window =
                                ((dr.time + MS_PER_DAY) / MS_PER_DAY) * MS_PER_DAY;
                        }
                    } else {
                        //println!(
                        //    "consolidate_distributions {asset}: Not Distribution Staking Rewards, back to looking"
                        //);
                        self.consolidated_dist_rec_vec.push(dr.clone());
                        state = State::LookingForDistribution;
                    }
                }
                State::UpdatingDistributionOthers => {
                    if (dr.category == "Distribution") && (dr.operation == "Others") {
                        if (dr.time < time_of_next_consolidation_window) {
                            // Same the Current Time Window

                            //let cdr = self.consolidated_dist_rec_vec.last_mut().expect("WTF");
                            //self.consolidate(cdr, dr); // error[E0499]: cannot borrow `*self` as mutable more than once at a time
                            let cdr = self.consolidated_dist_rec_vec.last().expect("WTF");
                            let (value, value_usd) = self.consolidate_x(cdr, dr);

                            let cdr = self.consolidated_dist_rec_vec.last_mut().expect("WTF");
                            cdr.realized_amount_for_primary_asset = Some(value);
                            cdr.realized_amount_for_primary_asset_in_usd_value = Some(value_usd);
                            cdr.time = dr.time; // Last entry will be used as the time for the consolidated record
                            cdr.order_id = dr.order_id.clone();
                            cdr.transaction_id = dr.transaction_id;
                        } else {
                            // Add this record as we're Starting a new time window
                            self.consolidated_dist_rec_vec.push(dr.clone());

                            // Calculate New Time Window
                            time_of_next_consolidation_window =
                                ((dr.time + MS_PER_DAY) / MS_PER_DAY) * MS_PER_DAY;
                        }
                    } else {
                        //println!(
                        //    "consolidate_distributions {asset}: Not Distribution Others, back to looking"
                        //);
                        self.consolidated_dist_rec_vec.push(dr.clone());
                        state = State::LookingForDistribution;
                    }
                }
            }
        }

        //println!("consolidate_distributions:-");
        Ok(())
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

    fn add_dr(&mut self, dr: DistRec, line_number: usize) {
        // The asset is always either primary_asset or base_asset
        let asset = if !dr.primary_asset.is_empty() {
            assert!(dr.base_asset.is_empty());
            &dr.primary_asset
        } else if !dr.base_asset.is_empty() {
            &dr.base_asset
        } else {
            panic!("No primary_asset or base_asset at line {line_number}");
        };

        let entry = self
            .bt
            .entry(asset.to_owned())
            .or_insert_with(|| AssetRec::new(asset));
        entry.dist_rec_vec.push(dr);
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
struct ProcessedData {
    dist_rec_vec: Vec<DistRec>,
    consolidated_dist_rec_vec: Vec<DistRec>,
    asset_rec_map: AssetRecMap,
    others_rec_map: AssetRecMap,
    total_count: u64,
    distribution_operation_referral_commission_value_usd: Decimal,
    distribution_operation_staking_rewards_value_usd: Decimal,
    distribution_operation_others_value_usd: Decimal,
    distribution_category_count: u64,
    distribution_operation_referral_commission_count: u64,
    distribution_operation_staking_reward_count: u64,
    distribution_operation_others_count: u64,
    distribution_operation_unknown_count: u64,
    quick_category_count: u64,
    quick_buy_operation_buy_count: u64,
    quick_buy_base_asset_in_usd_value: Decimal,
    quick_buy_operation_buy_fee_in_usd_value: Decimal,
    quick_sell_operation_sell_count: u64,
    quick_sell_base_asset_in_usd_value: Decimal,
    quick_sell_operation_sell_fee_in_usd_value: Decimal,
    quick_operation_unknown_count: u64,
    spot_trading_category_count: u64,
    spot_trading_operation_unknown_count: u64,
    spot_trading_operation_buy_count: u64,
    spot_trading_operation_buy_base_asset_in_usd_value: Decimal,
    spot_trading_operation_buy_fee_in_usd_value: Decimal,
    spot_trading_operation_sell_count: u64,
    spot_trading_operation_sell_base_asset_in_usd_value: Decimal,
    spot_trading_operation_sell_fee_in_usd_value: Decimal,
    withdrawal_category_count: u64,
    withdrawal_operation_crypto_withdrawal_count: u64,
    withdrawal_operation_crypto_withdrawal_usd_value: Decimal,
    withdrawal_operation_crypto_withdrawal_fee_count: u64,
    withdrawal_operation_crypto_withdrawal_fee_in_usd_value: Decimal,
    withdrawal_operation_unknown_count: u64,
    deposit_category_count: u64,
    deposit_operation_crypto_deposit_count: u64,
    deposit_operation_crypto_deposit_usd_value: Decimal,
    deposit_operation_crypto_deposit_fee_count: u64,
    deposit_operation_usd_deposit_count: u64,
    deposit_operation_usd_deposit_usd_value: Decimal,
    deposit_operaiton_usd_deposit_fee_count: u64,
    deposit_operation_usd_deposit_fee_usd_value: Decimal,
    deposit_operation_unknown_count: u64,
    unprocessed_category_count: u64,
}

impl ProcessedData {
    fn new() -> ProcessedData {
        ProcessedData {
            dist_rec_vec: Vec::new(),
            consolidated_dist_rec_vec: Vec::new(),
            asset_rec_map: AssetRecMap::new(),
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

impl TokenTaxRec {
    fn format_tt_cmt_ver0(line_number: usize, dr: &DistRec) -> String {
        let ver = TT_CMT_VER0.as_str();
        format!(
            "{ver},{line_number},{},{},{},{}",
            dr.order_id, dr.transaction_id, dr.category, dr.operation
        )
    }

    fn from_dist_rec(line_number: usize, dr: &DistRec) -> TokenTaxRec {
        let mut ttr = TokenTaxRec {
            type_txs: TypeTxs::Trade,
            buy_amount: None,
            buy_currency: "".to_owned(),
            sell_amount: None,
            sell_currency: "".to_owned(),
            fee_amount: None,
            fee_currency: "".to_owned(),
            exchange: "binance.us".to_owned(),
            group: None,
            comment: TokenTaxRec::format_tt_cmt_ver0(line_number, dr),
            time: dr.time,
        };
        //dbg!(&dr.operation);
        //dbg!(&ttr);

        match dr.category.as_ref() {
            "Distribution" => {
                ttr.type_txs = TypeTxs::Income;
                ttr.buy_amount = dr.realized_amount_for_primary_asset;
                ttr.buy_currency = dr.primary_asset.clone();
                ttr.fee_amount = dr.realized_amount_for_fee_asset;
                ttr.fee_currency = dr.fee_asset.clone();

                ttr
            }
            "Quick Buy" | "Quick Sell" | "Spot Trading" => {
                ttr.type_txs = TypeTxs::Trade;
                ttr.fee_amount = dr.realized_amount_for_fee_asset;
                ttr.fee_currency = dr.fee_asset.clone();
                match dr.operation.as_ref() {
                    "Buy" => {
                        ttr.buy_amount = dr.realized_amount_for_base_asset;
                        ttr.buy_currency = dr.base_asset.clone();
                        ttr.sell_amount = dr.realized_amount_for_quote_asset;
                        ttr.sell_currency = dr.quote_asset.clone();

                        ttr
                    }
                    "Sell" => {
                        ttr.buy_amount = dr.realized_amount_for_quote_asset;
                        ttr.buy_currency = dr.quote_asset.clone();
                        ttr.sell_amount = dr.realized_amount_for_base_asset;
                        ttr.sell_currency = dr.base_asset.clone();

                        ttr
                    }
                    _ => {
                        panic!(
                            "{} {} {} unknown operation: {}",
                            line_number, dr.base_asset, dr.category, dr.operation
                        );
                    }
                }
            }
            "Withdrawal" => {
                ttr.type_txs = TypeTxs::Withdrawal;
                ttr.sell_amount = dr.realized_amount_for_primary_asset;
                ttr.sell_currency = dr.primary_asset.clone();
                ttr.fee_amount = dr.realized_amount_for_fee_asset;
                ttr.fee_currency = dr.fee_asset.clone();

                ttr
            }
            "Deposit" => {
                ttr.type_txs = TypeTxs::Deposit;
                ttr.buy_amount = dr.realized_amount_for_primary_asset;
                ttr.buy_currency = dr.primary_asset.clone();
                ttr.fee_amount = dr.realized_amount_for_fee_asset;
                ttr.fee_currency = dr.fee_asset.clone();

                ttr
            }
            _ => {
                panic!("{} Unknown category: {}", line_number, dr.category);
            }
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
    let asset = dr.get_asset();
    let asset_value = dr.get_value();
    let asset_value_usd = dr.get_value_usd();

    // Add missing AssetRecMap entries that might be needed
    // Adding them here means less surprises later and we can
    // use "unwarp()".
    let _ = arm.bt.entry(asset.to_owned()).or_insert_with(|| {
        // This happens the first time an asset is seen and is not unusual
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
        if let Some(r) = sc_matches.value_of("OUT_FILE") {
            Some(r)
        } else {
            return Err("Expected --out-file parameter".into());
        }
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
                let asset = dr.get_asset();
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

fn verify_input_files_exist(in_file_paths: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    for f in &*in_file_paths {
        if !Path::new(*f).exists() {
            return Err(format!("{} does not exist", *f).into());
        }
    }

    Ok(())
}

fn create_buf_writer(out_file_path: &str) -> Result<BufWriter<File>, Box<dyn std::error::Error>> {
    let out_file = File::create(out_file_path)?;
    Ok(BufWriter::new(out_file))
}

#[allow(unused)]
fn write_dist_rec_vec(
    writer: BufWriter<File>,
    dist_rec_vec: &[DistRec],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a data record writer
    let mut dist_rec_writer = csv::Writer::from_writer(writer);

    // Output the data
    println!("Output dist recs: len={}", dist_rec_vec.len());
    for dr in dist_rec_vec {
        dist_rec_writer.serialize(dr)?;
    }
    println!("Output dist recs: Done");

    Ok(())
}

#[allow(unused)]
fn write_dist_rec_vec_as_token_tax(
    writer: BufWriter<File>,
    dist_rec_vec: &[DistRec],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a token tax writer
    let mut token_tax_writer = csv::Writer::from_writer(writer);

    // Output the data
    println!("Output token tax recs: len={}", dist_rec_vec.len());
    for (idx, dr) in dist_rec_vec.iter().enumerate() {
        let line_number = idx + 2;
        let dr: &DistRec = dr;
        let ttr: TokenTaxRec = TokenTaxRec::from_dist_rec(line_number, dr);
        token_tax_writer.serialize(ttr)?;
    }
    println!("Output token tax recs: Done");

    Ok(())
}

// Write the dist_rec's for an asset, used for debugging
#[allow(unused)]
fn write_dist_rec_vec_for_asset(
    data: &ProcessedData,
    asset: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ar = if let Some(v) = data.asset_rec_map.bt.get(asset) {
        v
    } else {
        panic!("No USD asset record");
    };
    let usd_wtr = create_buf_writer(format!("{asset}_dr.csv").as_str())?;
    write_dist_rec_vec(usd_wtr, &ar.dist_rec_vec)?;

    Ok(())
}

pub async fn consolidate_dist_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("consoldiate_dist_files:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let mut data = ProcessedData::new();

    let in_dist_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();
    verify_input_files_exist(&in_dist_paths)?;

    // Create out_dist_path
    let out_dist_path = sc_matches
        .value_of("OUT_FILE")
        .unwrap_or_else(|| panic!("out-file option is missing"));
    let out_dist_path = Path::new(out_dist_path);

    // Determine parent path, file_stem and extension so we can construct out_token_tax_path
    let out_parent_path = if let Some(pp) = out_dist_path.parent() {
        pp
    } else {
        Path::new(".")
    };

    let out_path_file_stem = if let Some(stem) = out_dist_path.file_stem() {
        stem
    } else {
        return Err(format!("There was no file in: '{out_dist_path:?}").into());
    };

    let out_path_extension = if let Some(csv_extension) = out_dist_path.extension() {
        let csv_extension = csv_extension.to_string_lossy().to_string();
        if csv_extension != "csv" {
            return Err(
                format!("Expecting file extension to be 'csv' found '{csv_extension}").into(),
            );
        }

        csv_extension
    } else {
        "csv".to_string()
    };

    // Construct the out_token_tax_path with adding "tt" before extension
    let out_token_tax_path = PathBuf::from(out_parent_path);
    let mut filename = out_path_file_stem.to_os_string();
    let ttx = OsString::from_str(".tt.").unwrap();
    let extx = OsString::from_str(out_path_extension.as_str()).unwrap();
    filename.push(ttx);
    filename.push(extx);

    let out_token_tax_path = out_token_tax_path.join(filename);

    let f = File::create(out_dist_path)?;
    let dist_rec_writer = BufWriter::new(f);

    let f = File::create(out_token_tax_path)?;
    let token_tax_rec_writer = BufWriter::new(f);

    println!("Read files");
    for f in &in_dist_paths {
        let in_file = if let Ok(in_f) = File::open(*f) {
            in_f
        } else {
            return Err(format!("Unable to open {f}").into());
        };
        let reader = BufReader::new(in_file);

        // DataRec reader
        let mut data_rec_reader = csv::Reader::from_reader(reader);

        for (rec_index, result) in data_rec_reader.deserialize().enumerate() {
            //println!("{rec_index}: {result:?}");
            let line_number = rec_index + 2;
            let dr: DistRec = result?;

            if config.verbose {
                let asset = dr.get_asset();
                print!("Processing {line_number} {asset}                        \r",);
            }

            data.dist_rec_vec.push(dr.clone());
            data.asset_rec_map.add_dr(dr, line_number);
        }
    }

    println!();
    println!();
    let col_1 = 7;
    let col_2 = 15;
    let col_3 = 15;

    let mut total_pre_len = 0usize;
    let mut total_post_len = 0usize;
    println!("Consolidate");
    println!(
        "{:<col_1$} {:>col_2$} {:>col_3$}",
        "Asset", "pre count", "post count"
    );

    //let mut state = ConsolidateState { prev_dr: Default::default() };
    for (asset, ar) in &mut data.asset_rec_map.bt {
        let pre_len = ar.dist_rec_vec.len();
        total_pre_len += pre_len;

        ar.consolidate_distributions(config)?;

        let post_len = ar.consolidated_dist_rec_vec.len();
        total_post_len += post_len;

        // Append the ar.consolidated_dis_rec_vec to end of data.consolidated_dist_rec_vec
        for x in &ar.consolidated_dist_rec_vec {
            data.consolidated_dist_rec_vec.push(x.clone());
        }

        println!(
            "{:<col_1$} {:>col_2$} {:>col_3$}",
            asset,
            dec_to_separated_string(Decimal::from_f64(pre_len as f64).unwrap(), 0),
            dec_to_separated_string(Decimal::from_f64(post_len as f64).unwrap(), 0),
        );
    }
    println!("Consolidated from {} to {}", total_pre_len, total_post_len);

    data.consolidated_dist_rec_vec.sort();

    // Output consolidated data as dist records and token_tax records
    println!("Writing disttribution records");
    write_dist_rec_vec(dist_rec_writer, &data.consolidated_dist_rec_vec)?;
    println!("Writing token tax records");
    write_dist_rec_vec_as_token_tax(token_tax_rec_writer, &data.consolidated_dist_rec_vec)?;

    // For debug
    //write_dist_rec_vec_for_asset(&data, "USD")?;

    println!();
    println!("Done");

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

    #[test]
    fn test_deserialize_dist_rec_from_csv() {
        let csv = "User_Id,Time,Category,Operation,Order_Id,Transaction_Id,Primary_Asset,Realized_Amount_For_Primary_Asset,Realized_Amount_For_Primary_Asset_In_USD_Value,Base_Asset,Realized_Amount_For_Base_Asset,Realized_Amount_For_Base_Asset_In_USD_Value,Quote_Asset,Realized_Amount_For_Quote_Asset,Realized_Amount_For_Quote_Asset_In_USD_Value,Fee_Asset,Realized_Amount_For_Fee_Asset,Realized_Amount_For_Fee_Asset_In_USD_Value,Payment_Method,Withdrawal_Method,Additional_Note
12345678,2019-08-01T00:00:00.000+00:00,Deposit,USD Deposit,1,1,USD,5125,5125,,,,,,,,,,Debit,,
12345678,2019-09-28T15:35:02.000+00:00,Spot Trading,Buy,367670,125143,,,,BTC,0.00558,46.012234,USD,44.959176,44.959176,BTC,0,0,Wallet,,
12345678,2020-03-02T07:32:05.000+00:00,Distribution,Referral Commission,5442858,17929593,BTC,0.0000003,0.002661,,,,,,,,,,Wallet,,
12345678,2020-03-23T04:08:20.000+00:00,Deposit,Crypto Deposit,17916393,17916393,ETH,45.25785064909286,6105.809587,,,,,,,,,,Wallet,,
12345678,2020-03-23T04:10:29.000+00:00,Spot Trading,Sell,5988456,17916714,,,,ETH,20.374,2748.689183,BTC,0.427854,2745.245935,BNB,0.16893668,2.047513,Wallet,,
12345678,2020-07-26T15:50:02.000+00:00,Spot Trading,Buy,26988333,32890969,,,,BNB,0.61,11.907825,USD,11.90903,11.90903,BNB,0.0004575,0.008931,Wallet,,
12345678,2020-08-16T23:54:01.000+00:00,Withdrawal,Crypto Withdrawal,38078398,38078398,ETH,23.99180186,10407.403729,,,,,,,ETH,0.005,2.16895,Wallet,Wallet,
12345678,2021-03-18T03:49:18.000+00:00,Quick Buy,Buy,cf9257c74ea243da9f3e64847ad0233b,171875688,,,,USD,27.4684,27.4684,BNB,0.1,26.170481,USD,0.14,0.14,Wallet,,
12345678,2021-03-22T22:33:06.147+00:00,Quick Sell,Sell,87d5c693897c4a0a8a35534782f6c471,179163493,,,,BTC,0.010946,596.876028,USD,590.5686,590.5686,USD,2.97,2.97,Wallet,,
";

        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);
        //let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for (idx, entry) in reader.deserialize().enumerate() {
            println!("{idx}: entry: {:?}", entry);
            match entry {
                Ok(dr) => {
                    let dr: DistRec = dr;
                    println!("tr: {:?}", dr);
                    match idx {
                        0 => {
                            assert_eq!(dr.category, "Deposit");
                            assert_eq!(dr.operation, "USD Deposit");
                        }
                        1 => {
                            assert_eq!(dr.category, "Spot Trading");
                            assert_eq!(dr.operation, "Buy");
                        }
                        2 => {
                            assert_eq!(dr.category, "Distribution");
                            assert_eq!(dr.operation, "Referral Commission");
                        }
                        3 => {
                            assert_eq!(dr.category, "Deposit");
                            assert_eq!(dr.operation, "Crypto Deposit");
                        }
                        4 => {
                            assert_eq!(dr.category, "Spot Trading");
                            assert_eq!(dr.operation, "Sell");
                        }
                        5 => {
                            assert_eq!(dr.category, "Spot Trading");
                            assert_eq!(dr.operation, "Buy");
                        }
                        6 => {
                            assert_eq!(dr.category, "Withdrawal");
                            assert_eq!(dr.operation, "Crypto Withdrawal");
                        }
                        7 => {
                            assert_eq!(dr.category, "Quick Buy");
                            assert_eq!(dr.operation, "Buy");
                        }
                        8 => {
                            assert_eq!(dr.category, "Quick Sell");
                            assert_eq!(dr.operation, "Sell");
                        }
                        _ => panic!("Unexpected idx"),
                    }
                }
                Err(e) => panic!("Error: {e}"),
            }
        }
    }

    #[test]
    fn test_serialize_binance_us_dist_rec_to_csv() {
        let csv = "User_Id,Time,Category,Operation,Order_Id,Transaction_Id,Primary_Asset,Realized_Amount_For_Primary_Asset,Realized_Amount_For_Primary_Asset_In_USD_Value,Base_Asset,Realized_Amount_For_Base_Asset,Realized_Amount_For_Base_Asset_In_USD_Value,Quote_Asset,Realized_Amount_For_Quote_Asset,Realized_Amount_For_Quote_Asset_In_USD_Value,Fee_Asset,Realized_Amount_For_Fee_Asset,Realized_Amount_For_Fee_Asset_In_USD_Value,Payment_Method,Withdrawal_Method,Additional_Note
12345678,2019-08-01T00:00:00.000+00:00,Deposit,USD Deposit,1,1,USD,5125,5125,,,,,,,,,,Debit,,
12345678,2019-09-28T15:35:02.000+00:00,Spot Trading,Buy,367670,125143,,,,BTC,0.00558,46.012234,USD,44.959176,44.959176,BTC,0,0,Wallet,,
12345678,2020-03-02T07:32:05.000+00:00,Distribution,Referral Commission,5442858,17929593,BTC,0.0000003,0.002661,,,,,,,,,,Wallet,,
12345678,2020-03-23T04:08:20.000+00:00,Deposit,Crypto Deposit,17916393,17916393,ETH,45.25785064909286,6105.809587,,,,,,,,,,Wallet,,
12345678,2020-03-23T04:10:29.000+00:00,Spot Trading,Sell,5988456,17916714,,,,ETH,20.374,2748.689183,BTC,0.427854,2745.245935,BNB,0.16893668,2.047513,Wallet,,
12345678,2020-07-26T15:50:02.000+00:00,Spot Trading,Buy,26988333,32890969,,,,BNB,0.61,11.907825,USD,11.90903,11.90903,BNB,0.0004575,0.008931,Wallet,,
12345678,2020-08-16T23:54:01.000+00:00,Withdrawal,Crypto Withdrawal,38078398,38078398,ETH,23.99180186,10407.403729,,,,,,,ETH,0.005,2.16895,Wallet,Wallet,
12345678,2021-03-18T03:49:18.000+00:00,Quick Buy,Buy,cf9257c74ea243da9f3e64847ad0233b,171875688,,,,USD,27.4684,27.4684,BNB,0.1,26.170481,USD,0.14,0.14,Wallet,,
12345678,2021-03-22T22:33:06.147+00:00,Quick Sell,Sell,87d5c693897c4a0a8a35534782f6c471,179163493,,,,BTC,0.010946,596.876028,USD,590.5686,590.5686,USD,2.97,2.97,Wallet,,
";

        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);

        let mut wtr = csv::Writer::from_writer(vec![]);
        for (_idx, entry) in reader.deserialize().enumerate() {
            //println!("{_idx}: entry: {:?}", entry);
            let record: DistRec = entry.unwrap();
            wtr.serialize(record).expect("Error serializing");
        }

        let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        //dbg!(&data);

        assert_eq!(data, csv);
    }

    #[test]
    fn test_deserialize_binance_us_dist_rec_to_serialized_token_tax_rec() {
        let csv = "User_Id,Time,Category,Operation,Order_Id,Transaction_Id,Primary_Asset,Realized_Amount_For_Primary_Asset,Realized_Amount_For_Primary_Asset_In_USD_Value,Base_Asset,Realized_Amount_For_Base_Asset,Realized_Amount_For_Base_Asset_In_USD_Value,Quote_Asset,Realized_Amount_For_Quote_Asset,Realized_Amount_For_Quote_Asset_In_USD_Value,Fee_Asset,Realized_Amount_For_Fee_Asset,Realized_Amount_For_Fee_Asset_In_USD_Value,Payment_Method,Withdrawal_Method,Additional_Note
12345678,2019-08-01T00:00:00.000+00:00,Deposit,USD Deposit,1,1,USD,5125,5125,,,,,,,,,,Debit,,
12345678,2019-09-28T15:35:02.000+00:00,Spot Trading,Buy,367670,125143,,,,BTC,0.00558,46.012234,USD,44.959176,44.959176,BTC,0,0,Wallet,,
12345678,2020-03-02T07:32:05.000+00:00,Distribution,Referral Commission,5442858,17929593,BTC,0.0000003,0.002661,,,,,,,,,,Wallet,,
12345678,2020-03-23T04:08:20.000+00:00,Deposit,Crypto Deposit,17916393,17916393,ETH,45.25785064909286,6105.809587,,,,,,,,,,Wallet,,
12345678,2020-03-23T04:10:29.000+00:00,Spot Trading,Sell,5988456,17916714,,,,ETH,20.374,2748.689183,BTC,0.427854,2745.245935,BNB,0.16893668,2.047513,Wallet,,
12345678,2020-07-26T15:50:02.000+00:00,Spot Trading,Buy,26988333,32890969,,,,BNB,0.61,11.907825,USD,11.90903,11.90903,BNB,0.0004575,0.008931,Wallet,,
12345678,2020-08-16T23:54:01.000+00:00,Withdrawal,Crypto Withdrawal,38078398,38078398,ETH,23.99180186,10407.403729,,,,,,,ETH,0.005,2.16895,Wallet,Wallet,
12345678,2021-03-18T03:49:18.000+00:00,Quick Buy,Buy,cf9257c74ea243da9f3e64847ad0233b,171875688,,,,USD,27.4684,27.4684,BNB,0.1,26.170481,USD,0.14,0.14,Wallet,,
12345678,2021-03-22T22:33:06.147+00:00,Quick Sell,Sell,87d5c693897c4a0a8a35534782f6c471,179163493,,,,BTC,0.010946,596.876028,USD,590.5686,590.5686,USD,2.97,2.97,Wallet,,
";
        let result_ttr_csv = "Type,BuyAmount,BuyCurrency,SellAmount,SellCurrency,FeeAmount,FeeCurrency,Exchange,Group,Comment,Date
Deposit,5125,USD,,,,,binance.us,,\"v0,2,1,1,Deposit,USD Deposit\",2019-08-01T00:00:00.000+00:00
Trade,0.00558,BTC,44.959176,USD,0,BTC,binance.us,,\"v0,3,367670,125143,Spot Trading,Buy\",2019-09-28T15:35:02.000+00:00
Income,0.0000003,BTC,,,,,binance.us,,\"v0,4,5442858,17929593,Distribution,Referral Commission\",2020-03-02T07:32:05.000+00:00
Deposit,45.25785064909286,ETH,,,,,binance.us,,\"v0,5,17916393,17916393,Deposit,Crypto Deposit\",2020-03-23T04:08:20.000+00:00
Trade,0.427854,BTC,20.374,ETH,0.16893668,BNB,binance.us,,\"v0,6,5988456,17916714,Spot Trading,Sell\",2020-03-23T04:10:29.000+00:00
Trade,0.61,BNB,11.90903,USD,0.0004575,BNB,binance.us,,\"v0,7,26988333,32890969,Spot Trading,Buy\",2020-07-26T15:50:02.000+00:00
Withdrawal,,,23.99180186,ETH,0.005,ETH,binance.us,,\"v0,8,38078398,38078398,Withdrawal,Crypto Withdrawal\",2020-08-16T23:54:01.000+00:00
Trade,27.4684,USD,0.1,BNB,0.14,USD,binance.us,,\"v0,9,cf9257c74ea243da9f3e64847ad0233b,171875688,Quick Buy,Buy\",2021-03-18T03:49:18.000+00:00
Trade,590.5686,USD,0.010946,BTC,2.97,USD,binance.us,,\"v0,10,87d5c693897c4a0a8a35534782f6c471,179163493,Quick Sell,Sell\",2021-03-22T22:33:06.147+00:00
";

        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);

        let mut wtr = csv::Writer::from_writer(vec![]);
        for (idx, entry) in reader.deserialize().enumerate() {
            let line_number = idx + 2;
            println!("{idx}: entry: {:?}", entry);
            let dr: DistRec = entry.unwrap();
            //dbg!(dr);

            let ttr = TokenTaxRec::from_dist_rec(line_number, &dr);
            //dbg!(&ttr);
            wtr.serialize(&ttr).expect("Error serializing");
        }

        let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        dbg!(&data);

        assert_eq!(data, result_ttr_csv);
    }
}
