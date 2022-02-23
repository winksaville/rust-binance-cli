use crate::{
    binance_trade::convert,
    common::{
        create_buf_reader, create_buf_writer, dec_to_money_string, dec_to_separated_string,
        utc_now_to_time_ms, verify_input_files_exist,
    },
    configuration::Configuration,
    de_string_to_utc_time_ms::{de_string_to_utc_time_ms_condaddtzutc, se_time_ms_to_utc_string},
};

use clap::ArgMatches;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum TypeTxs {
    Unknown,
    Trade,
    Deposit,
    Withdrawal,
    Income,
    Spend,
    Lost,
    Stolen,
    Mining,
    Gift,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum GroupType {
    #[serde(rename = "margin")]
    Margin,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TokenTaxRec {
    #[serde(rename = "Type")]
    pub type_txs: TypeTxs,

    pub buy_amount: Option<Decimal>,
    pub buy_currency: String,
    pub sell_amount: Option<Decimal>,
    pub sell_currency: String,
    pub fee_amount: Option<Decimal>,
    pub fee_currency: String,
    pub exchange: String,
    pub group: Option<GroupType>,
    pub comment: String,

    #[serde(rename = "Date")]
    #[serde(deserialize_with = "de_string_to_utc_time_ms_condaddtzutc")]
    #[serde(serialize_with = "se_time_ms_to_utc_string")]
    pub time: i64,
}

impl TokenTaxRec {
    pub fn new() -> TokenTaxRec {
        TokenTaxRec {
            type_txs: TypeTxs::Unknown,
            buy_amount: None,
            buy_currency: "".to_string(),
            sell_amount: None,
            sell_currency: "".to_string(),
            fee_amount: None,
            fee_currency: "".to_string(),
            exchange: "".to_string(),
            group: None,
            comment: "".to_string(),
            time: 0,
        }
    }

    fn get_asset(&self) -> &str {
        match self.type_txs {
            TypeTxs::Unknown => "",
            TypeTxs::Trade => &self.buy_currency,
            TypeTxs::Deposit => &self.buy_currency,
            TypeTxs::Withdrawal => &self.sell_currency,
            TypeTxs::Income => &self.buy_currency,
            TypeTxs::Spend => &self.sell_currency,
            TypeTxs::Lost => &self.sell_currency,
            TypeTxs::Stolen => &self.sell_currency,
            TypeTxs::Mining => &self.buy_currency,
            TypeTxs::Gift => &self.sell_currency,
        }
    }

    fn get_quantity(&self) -> Decimal {
        match self.type_txs {
            TypeTxs::Unknown => panic!("WTF"),
            TypeTxs::Trade => self.buy_amount.expect("WTF"),
            TypeTxs::Deposit => self.buy_amount.expect("WTF"),
            TypeTxs::Withdrawal => self.sell_amount.expect("WTF"),
            TypeTxs::Income => self.buy_amount.expect("WTF"),
            TypeTxs::Spend => self.sell_amount.expect("WTF"),
            TypeTxs::Lost => self.sell_amount.expect("WTF"),
            TypeTxs::Stolen => self.sell_amount.expect("WTF"),
            TypeTxs::Mining => self.buy_amount.expect("WTF"),
            TypeTxs::Gift => self.sell_amount.expect("WTF"),
        }
    }

    fn get_other_asset(&self) -> &str {
        match self.type_txs {
            TypeTxs::Unknown => "",
            TypeTxs::Trade => &self.sell_currency,
            TypeTxs::Deposit => &self.sell_currency,
            TypeTxs::Withdrawal => &self.buy_currency,
            TypeTxs::Income => &self.sell_currency,
            TypeTxs::Spend => &self.buy_currency,
            TypeTxs::Lost => &self.buy_currency,
            TypeTxs::Stolen => &self.buy_currency,
            TypeTxs::Mining => &self.sell_currency,
            TypeTxs::Gift => &self.buy_currency,
        }
    }

    //fn get_other_value(&self) -> Decimal {
    //    match self.type_txs {
    //        TypeTxs::Unknown => panic!("WTF"),
    //        TypeTxs::Trade => self.sell_amount.expect("WTF"),
    //        TypeTxs::Deposit => self.sell_amount.expect("WTF"),
    //        TypeTxs::Withdrawal => self.buy_amount.expect("WTF"),
    //        TypeTxs::Income => self.sell_amount.expect("WTF"),
    //        TypeTxs::Spend => self.buy_amount.expect("WTF"),
    //        TypeTxs::Lost => self.buy_amount.expect("WTF"),
    //        TypeTxs::Stolen => self.buy_amount.expect("WTF"),
    //        TypeTxs::Mining => self.sell_amount.expect("WTF"),
    //        TypeTxs::Gift => self.buy_amount.expect("WTF"),
    //    }
    //}

    //fn sum_amount(&self, ttr: &TokenTaxRec) -> Decimal {
    //    self.get_value() + ttr.get_value()
    //}

    //fn consolidate(&mut self, dr: &DistRec) {
    //    //let cdr = self.consolidated_dist_rec_vec.last().expect("WTF");
    //    let (quantity, value_usd) = self.sum_quantity_and_value_usd(dr);

    //    //let cdr = self.consolidated_dist_rec_vec.last_mut().expect("WTF");
    //    self.realized_amount_for_primary_asset = Some(quantity);
    //    self.realized_amount_for_primary_asset_in_usd_value = Some(value_usd);
    //    //cdr.time = dr.time; // Last entry will be used as the time for the consolidated record, otherwise first entry is used
    //    self.order_id = dr.order_id.clone();
    //    self.transaction_id = dr.transaction_id;
    //}
}

#[derive(Debug)]
struct AssetRec {
    asset: String,
    quantity: Decimal,
    value_usd: Decimal,
    transaction_count: u64,
    ttr_vec: Vec<TokenTaxRec>,
    //consolidated_dist_rec_vec: Vec<DistRec>,
}

#[allow(unused)]
impl AssetRec {
    fn new(asset: &str) -> AssetRec {
        AssetRec {
            asset: asset.to_string(),
            quantity: dec!(0),
            value_usd: dec!(0),
            transaction_count: 0,
            ttr_vec: Vec::new(),
            //consolidated_ttr_vec: Vec::new(),
        }
    }
}

// Consider making a trait as this is being used in binance_us and binance_com too?
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

    fn push_rec(&mut self, ttr: TokenTaxRec) {
        // The asset is always either primary_asset or base_asset
        let asset = ttr.get_asset();

        let entry = self
            .bt
            .entry(asset.to_owned())
            .or_insert_with(|| AssetRec::new(asset));
        entry.ttr_vec.push(ttr);
    }

    fn _add_or_update(&mut self, asset: &str, quantity: Decimal, value_usd: Decimal) {
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

    fn _add_quantity(&mut self, asset: &str, val: Decimal) {
        let entry = self.bt.get_mut(asset).unwrap();
        entry.quantity += val;
    }

    fn _sub_quantity(&mut self, asset: &str, val: Decimal) {
        self._add_quantity(asset, -val)
    }
}

#[derive(Debug)]
struct TokenTaxData {
    ttr_vec: Vec<TokenTaxRec>,
    //consolidated_ttr_vec: Vec<TokenTaxRec>,
    asset_rec_map: AssetRecMap,
    //others_rec_map: AssetRecMap,
    total_count: u64,
    type_txs_unknown_count: u64,
    type_txs_trade_count: u64,
    type_txs_deposit_count: u64,
    type_txs_withdrawal_count: u64,
    type_txs_income_count: u64,
    type_txs_spend_count: u64,
    type_txs_lost_count: u64,
    type_txs_stolen_count: u64,
    type_txs_mining_count: u64,
    type_txs_gift_count: u64,
    //distribution_operation_referral_commission_value_usd: Decimal,
    //distribution_operation_staking_rewards_value_usd: Decimal,
    //distribution_operation_others_value_usd: Decimal,
    //distribution_category_count: u64,
    //distribution_operation_referral_commission_count: u64,
    //distribution_operation_staking_reward_count: u64,
    //distribution_operation_others_count: u64,
    //distribution_operation_unknown_count: u64,
    //quick_category_count: u64,
    //quick_buy_operation_buy_count: u64,
    //quick_buy_base_asset_in_usd_value: Decimal,
    //quick_buy_operation_buy_fee_in_usd_value: Decimal,
    //quick_sell_operation_sell_count: u64,
    //quick_sell_base_asset_in_usd_value: Decimal,
    //quick_sell_operation_sell_fee_in_usd_value: Decimal,
    //quick_operation_unknown_count: u64,
    //spot_trading_category_count: u64,
    //spot_trading_operation_unknown_count: u64,
    //spot_trading_operation_buy_count: u64,
    //spot_trading_operation_buy_base_asset_in_usd_value: Decimal,
    //spot_trading_operation_buy_fee_in_usd_value: Decimal,
    //spot_trading_operation_sell_count: u64,
    //spot_trading_operation_sell_base_asset_in_usd_value: Decimal,
    //spot_trading_operation_sell_fee_in_usd_value: Decimal,
    //withdrawal_category_count: u64,
    //withdrawal_operation_crypto_withdrawal_count: u64,
    //withdrawal_operation_crypto_withdrawal_usd_value: Decimal,
    //withdrawal_operation_crypto_withdrawal_fee_count: u64,
    //withdrawal_operation_crypto_withdrawal_fee_in_usd_value: Decimal,
    //withdrawal_operation_unknown_count: u64,
    //deposit_category_count: u64,
    //deposit_operation_crypto_deposit_count: u64,
    //deposit_operation_crypto_deposit_usd_value: Decimal,
    //deposit_operation_crypto_deposit_fee_count: u64,
    //deposit_operation_usd_deposit_count: u64,
    //deposit_operation_usd_deposit_usd_value: Decimal,
    //deposit_operaiton_usd_deposit_fee_count: u64,
    //deposit_operation_usd_deposit_fee_usd_value: Decimal,
    //deposit_operation_unknown_count: u64,
    //unprocessed_category_count: u64,
}

impl TokenTaxData {
    fn new() -> TokenTaxData {
        TokenTaxData {
            ttr_vec: Vec::new(),
            //consolidated_dist_rec_vec: Vec::new(),
            asset_rec_map: AssetRecMap::new(),
            //others_rec_map: AssetRecMap::new(),
            total_count: 0u64,
            type_txs_unknown_count: 0u64,
            type_txs_trade_count: 0u64,
            type_txs_deposit_count: 0u64,
            type_txs_withdrawal_count: 0u64,
            type_txs_income_count: 0u64,
            type_txs_spend_count: 0u64,
            type_txs_lost_count: 0u64,
            type_txs_stolen_count: 0u64,
            type_txs_mining_count: 0u64,
            type_txs_gift_count: 0u64,
            //distribution_operation_referral_commission_value_usd: dec!(0),
            //distribution_operation_staking_rewards_value_usd: dec!(0),
            //distribution_operation_others_value_usd: dec!(0),
            //distribution_category_count: 0u64,
            //distribution_operation_referral_commission_count: 0u64,
            //distribution_operation_staking_reward_count: 0u64,
            //distribution_operation_others_count: 0u64,
            //distribution_operation_unknown_count: 0u64,
            //quick_category_count: 0u64,
            //quick_buy_operation_buy_count: 0u64,
            //quick_buy_base_asset_in_usd_value: dec!(0),
            //quick_buy_operation_buy_fee_in_usd_value: dec!(0),
            //quick_sell_operation_sell_count: 0u64,
            //quick_sell_base_asset_in_usd_value: dec!(0),
            //quick_sell_operation_sell_fee_in_usd_value: dec!(0),
            //quick_operation_unknown_count: 0u64,
            //spot_trading_category_count: 0u64,
            //spot_trading_operation_unknown_count: 0u64,
            //spot_trading_operation_buy_count: 0u64,
            //spot_trading_operation_buy_base_asset_in_usd_value: dec!(0),
            //spot_trading_operation_buy_fee_in_usd_value: dec!(0),
            //spot_trading_operation_sell_count: 0u64,
            //spot_trading_operation_sell_base_asset_in_usd_value: dec!(0),
            //spot_trading_operation_sell_fee_in_usd_value: dec!(0),
            //withdrawal_category_count: 0u64,
            //withdrawal_operation_crypto_withdrawal_count: 0u64,
            //withdrawal_operation_crypto_withdrawal_usd_value: dec!(0),
            //withdrawal_operation_crypto_withdrawal_fee_count: 0u64,
            //withdrawal_operation_crypto_withdrawal_fee_in_usd_value: dec!(0),
            //withdrawal_operation_unknown_count: 0u64,
            //deposit_category_count: 0u64,
            //deposit_operation_crypto_deposit_count: 0u64,
            //deposit_operation_crypto_deposit_usd_value: dec!(0),
            //deposit_operation_crypto_deposit_fee_count: 0u64,
            //deposit_operation_usd_deposit_count: 0u64,
            //deposit_operation_usd_deposit_usd_value: dec!(0),
            //deposit_operaiton_usd_deposit_fee_count: 0u64,
            //deposit_operation_usd_deposit_fee_usd_value: dec!(0),
            //deposit_operation_unknown_count: 0u64,
            //unprocessed_category_count: 0u64,
        }
    }
}

// We assume that update_all_usd_values has been run prior
// to calling process_entry and thus can use unwrap() on
// the Option<Decimal> fields.
fn process_entry(
    _config: &Configuration,
    data: &mut TokenTaxData,
    arm: &mut AssetRecMap,
    ttr: &TokenTaxRec,
    _line_number: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    data.total_count += 1;

    let asset = ttr.get_asset();
    let _quantity = ttr.get_quantity();

    // Add missing AssetRecMap entries that might be needed
    // Adding them here means less surprises later and we can
    // use "unwarp()".
    let _ = arm.bt.entry(asset.to_owned()).or_insert_with(|| {
        // This happens the first time an asset is seen and is not unusual
        //println!("Adding missing asset: {}", asset);
        AssetRec::new(asset)
    });
    if !ttr.get_other_asset().is_empty() {
        let _ = arm
            .bt
            .entry(ttr.get_other_asset().to_owned())
            .or_insert_with(|| {
                println!(
                    "WARNING adding missing other_asset: {}",
                    ttr.get_other_asset()
                );
                AssetRec::new(ttr.get_other_asset())
            });
    }
    if !ttr.fee_currency.is_empty() {
        let _ = arm
            .bt
            .entry(ttr.fee_currency.to_owned())
            .or_insert_with(|| {
                println!("WARNING adding missing fee_currency: {}", ttr.fee_currency);
                AssetRec::new(&ttr.fee_currency)
            });
    }

    arm.inc_transaction_count(asset);

    //let leading_nl = if config.verbose { "\n" } else { "" };

    // TODO: For all the category and operations we need to save asset_value_usd as "usd_cost_basis"
    match ttr.type_txs {
        TypeTxs::Unknown => {
            data.type_txs_unknown_count += 1;
        }
        TypeTxs::Trade => {
            data.type_txs_trade_count += 1;
        }
        TypeTxs::Deposit => {
            data.type_txs_deposit_count += 1;
        }
        TypeTxs::Withdrawal => {
            data.type_txs_withdrawal_count += 1;
        }
        TypeTxs::Income => {
            data.type_txs_income_count += 1;
        }
        TypeTxs::Spend => {
            data.type_txs_spend_count += 1;
        }
        TypeTxs::Lost => {
            data.type_txs_lost_count += 1;
        }
        TypeTxs::Stolen => {
            data.type_txs_stolen_count += 1;
        }
        TypeTxs::Mining => {
            data.type_txs_mining_count += 1;
        }
        TypeTxs::Gift => {
            data.type_txs_gift_count += 1;
        }
    }

    Ok(())
}
pub async fn process_token_tax_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("process_token_tax_files:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let mut data = TokenTaxData::new();
    let mut asset_rec_map = AssetRecMap::new();

    let in_tt_file_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();

    // Clippy suggests:
    //    let out_tt_file_path = sc_matches.value_of("OUT_FILE").map(|r| r);
    // I feel that is "obtuse", so for me I'm using this more obvious style
    #[allow(clippy::manual_map)]
    let out_tt_file_path = if let Some(r) = sc_matches.value_of("OUT_FILE") {
        Some(r)
    } else {
        None
    };

    //println!("in_dist_file_path: {in_dist_file_paths:?}");
    //println!("out_dist_file_path: {out_dist_file_path:?}");

    // Verify all input files exist
    verify_input_files_exist(&in_tt_file_paths)?;

    // Create csv::Writer if out_file_path exists
    let mut csv_ttr_writer = if let Some(out_file_path) = out_tt_file_path {
        let writer = create_buf_writer(out_file_path)?;
        Some(csv::Writer::from_writer(writer))
    } else {
        None
    };

    for f in in_tt_file_paths {
        let reader = create_buf_reader(f)?;

        // Create reader
        let mut rdr = csv::Reader::from_reader(reader);

        for (rec_index, result) in rdr.deserialize().enumerate() {
            let line_number = rec_index + 2;
            let ttr: TokenTaxRec = result?;

            if config.verbose {
                let asset = ttr.get_asset();
                print!("Processing {line_number} {asset}                        \r",);
            }

            process_entry(config, &mut data, &mut asset_rec_map, &ttr, line_number)?;

            data.ttr_vec.push(ttr.clone());
            data.asset_rec_map.push_rec(ttr.clone());
            data.total_count += 1;

            if let Some(w) = &mut csv_ttr_writer {
                w.serialize(&ttr)?;
            }
        }
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
            let value_usd_string = if let Ok(value_usd) =
                convert(config, utc_now_to_time_ms(), &ar.asset, ar.quantity, "USD").await
            {
                ar.value_usd = value_usd;
                total_value_usd += ar.value_usd;

                dec_to_money_string(ar.value_usd)
            } else {
                // If an error display a ?
                "?".to_owned()
            };
            println!(
                "{:col_1_width$} {:>col_2_width$} {:>col_3_width$} {:>col_4_width$}",
                ar.asset,
                dec_to_separated_string(ar.quantity, 8),
                dec_to_separated_string(Decimal::from(ar.transaction_count), 0),
                value_usd_string,
            );
        }

        println!();
        println!(
            "Total account value: {}",
            dec_to_money_string(total_value_usd)
        );
    }

    //let lbl_width = 45;
    //let cnt_width = 10;
    //let val_width = 14;
    //let fee_width = 14;
    //println!(
    //    "{:>lbl_width$}  {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Operation", "Count", "USD Value", "Fee USD Value",
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Distribution Referral Commission USD value",
    //    dec_to_separated_string(
    //        Decimal::from(data.distribution_operation_referral_commission_count),
    //        0
    //    ),
    //    dec_to_money_string(data.distribution_operation_referral_commission_value_usd),
    //    "",
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Distribution Staking Reward USD value",
    //    dec_to_separated_string(
    //        Decimal::from(data.distribution_operation_staking_reward_count),
    //        0
    //    ),
    //    dec_to_money_string(data.distribution_operation_staking_rewards_value_usd),
    //    "",
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "* Distribution Others USD value",
    //    dec_to_separated_string(Decimal::from(data.distribution_operation_others_count), 0),
    //    dec_to_money_string(data.distribution_operation_others_value_usd),
    //    "",
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Quick Buy",
    //    dec_to_separated_string(Decimal::from(data.quick_buy_operation_buy_count), 0),
    //    dec_to_money_string(data.quick_buy_base_asset_in_usd_value),
    //    dec_to_money_string(data.quick_buy_operation_buy_fee_in_usd_value)
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Quick Sell",
    //    dec_to_separated_string(Decimal::from(data.quick_sell_operation_sell_count), 0),
    //    dec_to_money_string(data.quick_sell_base_asset_in_usd_value),
    //    dec_to_money_string(data.quick_sell_operation_sell_fee_in_usd_value)
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Spot Trading Buy",
    //    dec_to_separated_string(Decimal::from(data.spot_trading_operation_buy_count), 0),
    //    dec_to_money_string(data.spot_trading_operation_buy_base_asset_in_usd_value),
    //    dec_to_money_string(data.spot_trading_operation_buy_fee_in_usd_value)
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Spot Trading Sell",
    //    dec_to_separated_string(Decimal::from(data.spot_trading_operation_sell_count), 0),
    //    dec_to_money_string(data.spot_trading_operation_sell_base_asset_in_usd_value),
    //    dec_to_money_string(data.spot_trading_operation_sell_fee_in_usd_value)
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Withdrawal crypto",
    //    dec_to_separated_string(
    //        Decimal::from(data.withdrawal_operation_crypto_withdrawal_count),
    //        0
    //    ),
    //    dec_to_money_string(data.withdrawal_operation_crypto_withdrawal_usd_value),
    //    dec_to_money_string(data.withdrawal_operation_crypto_withdrawal_fee_in_usd_value)
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Deposit crypto",
    //    dec_to_separated_string(
    //        Decimal::from(data.deposit_operation_crypto_deposit_count),
    //        0
    //    ),
    //    dec_to_money_string(data.deposit_operation_crypto_deposit_usd_value),
    //    "",
    //);
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Deposit USD",
    //    dec_to_separated_string(
    //        Decimal::from(data.deposit_operation_crypto_deposit_count),
    //        0
    //    ),
    //    dec_to_money_string(data.deposit_operation_usd_deposit_usd_value),
    //    dec_to_money_string(data.deposit_operation_usd_deposit_fee_usd_value)
    //);
    //let fees_usd_value = data.quick_buy_operation_buy_fee_in_usd_value
    //    + data.quick_sell_operation_sell_fee_in_usd_value
    //    + data.spot_trading_operation_buy_fee_in_usd_value
    //    + data.spot_trading_operation_sell_fee_in_usd_value
    //    + data.withdrawal_operation_crypto_withdrawal_fee_in_usd_value
    //    + data.deposit_operation_usd_deposit_fee_usd_value;
    //println!(
    //    "{:>lbl_width$}: {:>cnt_width$} {:>val_width$} {:>fee_width$}",
    //    "Totals",
    //    dec_to_separated_string(Decimal::from(data.total_count), 0),
    //    "",
    //    dec_to_money_string(fees_usd_value),
    //);

    //println!();
    //println!("* Distribution Others:");
    //// Output others
    //let col_1_width = 10;
    //let col_2_width = 20;
    //let col_3_width = 10;
    //let col_4_width = 14;
    //println!(
    //    "{:col_1_width$} {:>col_2_width$} {:>col_3_width$} {:>col_4_width$}",
    //    "Asset", "Quantity", "Txs count", "USD value",
    //);

    //let mut others_value = dec!(0);

    //#[allow(clippy::for_kv_map)]
    //for (_, entry) in &data.others_rec_map.bt {
    //    others_value += entry.value_usd;
    //    println!(
    //        "{:col_1_width$} {:>col_2_width$} {:>col_3_width$} {:>col_4_width$}",
    //        entry.asset,
    //        entry.quantity,
    //        entry.transaction_count,
    //        dec_to_money_string(entry.value_usd),
    //    );
    //}
    //assert_eq!(others_value, data.distribution_operation_others_value_usd);

    //// Assertions!
    //assert_eq!(std::mem::size_of::<usize>(), std::mem::size_of::<u64>());

    //assert_eq!(
    //    data.deposit_operation_crypto_deposit_fee_count, 0,
    //    "See TODO: CryptoDepositFee"
    //);

    //assert_eq!(
    //    data.distribution_category_count,
    //    data.distribution_operation_referral_commission_count
    //        + data.distribution_operation_staking_reward_count
    //        + data.distribution_operation_others_count
    //        + data.distribution_operation_unknown_count
    //);
    //assert_eq!(data.distribution_operation_unknown_count, 0);

    //assert_eq!(
    //    data.quick_category_count,
    //    data.quick_sell_operation_sell_count
    //        + data.quick_buy_operation_buy_count
    //        + data.quick_operation_unknown_count
    //);
    //assert_eq!(data.quick_operation_unknown_count, 0);

    //assert_eq!(
    //    data.spot_trading_category_count,
    //    data.spot_trading_operation_buy_count
    //        + data.spot_trading_operation_sell_count
    //        + data.spot_trading_operation_unknown_count
    //);
    //assert_eq!(data.spot_trading_operation_unknown_count, 0);

    //assert_eq!(
    //    data.withdrawal_category_count,
    //    data.withdrawal_operation_crypto_withdrawal_count
    //        + data.withdrawal_operation_unknown_count
    //);
    //assert_eq!(data.withdrawal_operation_unknown_count, 0);

    //assert_eq!(
    //    data.deposit_category_count,
    //    data.deposit_operation_crypto_deposit_count
    //        + data.deposit_operation_usd_deposit_count
    //        + data.deposit_operation_unknown_count
    //);
    //assert_eq!(data.deposit_operation_unknown_count, 0);

    //assert_eq!(
    //    data.total_count,
    //    data.distribution_category_count
    //        + data.quick_category_count
    //        + data.spot_trading_category_count
    //        + data.withdrawal_category_count
    //        + data.deposit_category_count
    //        + data.unprocessed_category_count
    //);
    //assert_eq!(data.unprocessed_category_count, 0);

    //println!("process_token_tax_files:-");

    Ok(())
}

#[cfg(test)]
mod test {

    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_deserialize_from_csv() {
        let csv = "

Type,BuyAmount,BuyCurrency,SellAmount,SellCurrency,FeeAmount,FeeCurrency,Exchange,Group,Comment,Date
Deposit,5125,USD,,,,,binance.us,,,1970-01-01 00:00:00 
Trade,1,ETH,3123.00,USD,0.00124,BNB,binance.us,,,1970-01-01 00:00:00 
Trade,1,ETH,312.00,USD,0.00124,BNB,binance.us,margin,,1970-01-01 00:00:00 
Income,0.001,BNB,,,,,binance.us,,\"Referral Commission\",1970-01-01 00:00:00 
Withdrawal,,,100,USD,,,some bank,,\"AccountId: 123456\",1970-01-01 00:00:00 
Spend,,,100,USD,0.01,USD,,,\"Gift for wife\",1970-01-01 00:00:00 
Lost,,,1,ETH,,,,,\"Wallet lost\",1970-01-01 00:00:00 
Stolen,,,1,USD,,,,,\"Wallet hacked\",1970-01-01 00:00:00 
Mining,0.000002,ETH,,,,,binance.us,,\"ETH2 validator reward\",1970-01-01 00:00:00 
Gift,,,100,USD,,,,,\"Gift to friend\",1970-01-01 00:00:00 
";

        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);
        for (idx, entry) in reader.deserialize().enumerate() {
            println!("{idx}: entry: {:?}", entry);
            match entry {
                Ok(rec) => {
                    let ttcr: TokenTaxRec = rec;
                    println!("tr: {:?}", ttcr);
                    match idx {
                        0 => {
                            // Deposit,5125,USD,,,,,binance.us,,,1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Deposit);
                            assert_eq!(ttcr.buy_amount, Some(dec!(5125)));
                            assert_eq!(ttcr.buy_currency, "USD");
                            assert_eq!(ttcr.sell_amount, None);
                            assert_eq!(ttcr.sell_currency, "");
                            assert_eq!(ttcr.fee_amount, None);
                            assert_eq!(ttcr.fee_currency, "");
                            assert_eq!(ttcr.exchange, "binance.us");
                            assert_eq!(ttcr.group, None);
                            assert_eq!(ttcr.comment, "");
                            assert_eq!(ttcr.time, 0);
                        }
                        1 => {
                            // Trade,1,ETH,3123.00,USD,0.00124,BNB,binance.us,,,1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Trade);
                            assert_eq!(ttcr.buy_amount, Some(dec!(1)));
                            assert_eq!(ttcr.buy_currency, "ETH");
                            assert_eq!(ttcr.sell_amount, Some(dec!(3123)));
                            assert_eq!(ttcr.sell_currency, "USD");
                            assert_eq!(ttcr.fee_amount, Some(dec!(0.00124)));
                            assert_eq!(ttcr.fee_currency, "BNB");
                            assert_eq!(ttcr.exchange, "binance.us");
                            assert_eq!(ttcr.group, None);
                            assert_eq!(ttcr.comment, "");
                            assert_eq!(ttcr.time, 0);
                        }
                        2 => {
                            // Trade,1,ETH,312.00,USD,0.00124,BNB,binance.us,margin,,1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Trade);
                            assert_eq!(ttcr.buy_amount, Some(dec!(1)));
                            assert_eq!(ttcr.buy_currency, "ETH");
                            assert_eq!(ttcr.sell_amount, Some(dec!(312)));
                            assert_eq!(ttcr.sell_currency, "USD");
                            assert_eq!(ttcr.fee_amount, Some(dec!(0.00124)));
                            assert_eq!(ttcr.fee_currency, "BNB");
                            assert_eq!(ttcr.exchange, "binance.us");
                            assert_eq!(ttcr.group, Some(GroupType::Margin));
                            assert_eq!(ttcr.comment, "");
                            assert_eq!(ttcr.time, 0);
                        }
                        3 => {
                            // Income,0.001,BNB,,,,,binance.us,,\"Referral Commission\",1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Income);
                            assert_eq!(ttcr.buy_amount, Some(dec!(0.001)));
                            assert_eq!(ttcr.buy_currency, "BNB");
                            assert_eq!(ttcr.sell_amount, None);
                            assert_eq!(ttcr.sell_currency, "");
                            assert_eq!(ttcr.fee_amount, None);
                            assert_eq!(ttcr.fee_currency, "");
                            assert_eq!(ttcr.exchange, "binance.us");
                            assert_eq!(ttcr.group, None);
                            assert_eq!(ttcr.comment, "Referral Commission");
                            assert_eq!(ttcr.time, 0);
                        }
                        4 => {
                            // Withdrawal,,,100,USD,,,some bank,,\"AccountId: 123456\",1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Withdrawal);
                            assert_eq!(ttcr.buy_amount, None);
                            assert_eq!(ttcr.buy_currency, "");
                            assert_eq!(ttcr.sell_amount, Some(dec!(100)));
                            assert_eq!(ttcr.sell_currency, "USD");
                            assert_eq!(ttcr.fee_amount, None);
                            assert_eq!(ttcr.fee_currency, "");
                            assert_eq!(ttcr.exchange, "some bank");
                            assert_eq!(ttcr.group, None);
                            assert_eq!(ttcr.comment, "AccountId: 123456");
                            assert_eq!(ttcr.time, 0);
                        }
                        5 => {
                            // Spend,,,100,USD,0.01,USD,,,\"Gift for wife\",1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Spend);
                            assert_eq!(ttcr.buy_amount, None);
                            assert_eq!(ttcr.buy_currency, "");
                            assert_eq!(ttcr.sell_amount, Some(dec!(100)));
                            assert_eq!(ttcr.sell_currency, "USD");
                            assert_eq!(ttcr.fee_amount, Some(dec!(0.01)));
                            assert_eq!(ttcr.fee_currency, "USD");
                            assert_eq!(ttcr.exchange, "");
                            assert_eq!(ttcr.group, None);
                            assert_eq!(ttcr.comment, "Gift for wife");
                            assert_eq!(ttcr.time, 0);
                        }
                        6 => {
                            // Lost,,,1,ETH,,,,,\"Wallet lost\",1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Lost);
                            assert_eq!(ttcr.buy_amount, None);
                            assert_eq!(ttcr.buy_currency, "");
                            assert_eq!(ttcr.sell_amount, Some(dec!(1)));
                            assert_eq!(ttcr.sell_currency, "ETH");
                            assert_eq!(ttcr.fee_amount, None);
                            assert_eq!(ttcr.fee_currency, "");
                            assert_eq!(ttcr.exchange, "");
                            assert_eq!(ttcr.group, None);
                            assert_eq!(ttcr.comment, "Wallet lost");
                            assert_eq!(ttcr.time, 0);
                        }
                        7 => {
                            // Stolen,,,1,USD,,,,,\"Wallet hacked\",1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Stolen);
                            assert_eq!(ttcr.buy_amount, None);
                            assert_eq!(ttcr.buy_currency, "");
                            assert_eq!(ttcr.sell_amount, Some(dec!(1)));
                            assert_eq!(ttcr.sell_currency, "USD");
                            assert_eq!(ttcr.fee_amount, None);
                            assert_eq!(ttcr.fee_currency, "");
                            assert_eq!(ttcr.exchange, "");
                            assert_eq!(ttcr.group, None);
                            assert_eq!(ttcr.comment, "Wallet hacked");
                            assert_eq!(ttcr.time, 0);
                        }
                        8 => {
                            // Mining,0.000002,ETH,,,,,binance.us,,\"ETH2 validator reward\",1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Mining);
                            assert_eq!(ttcr.buy_amount, Some(dec!(0.000002)));
                            assert_eq!(ttcr.buy_currency, "ETH");
                            assert_eq!(ttcr.sell_amount, None);
                            assert_eq!(ttcr.sell_currency, "");
                            assert_eq!(ttcr.fee_amount, None);
                            assert_eq!(ttcr.fee_currency, "");
                            assert_eq!(ttcr.exchange, "binance.us");
                            assert_eq!(ttcr.group, None);
                            assert_eq!(ttcr.comment, "ETH2 validator reward");
                            assert_eq!(ttcr.time, 0);
                        }
                        9 => {
                            // Gift,,,100,USD,,,,,\"Gift to friend\",1970-01-01 00:00:00
                            assert_eq!(ttcr.type_txs, TypeTxs::Gift);
                            assert_eq!(ttcr.buy_amount, None);
                            assert_eq!(ttcr.buy_currency, "");
                            assert_eq!(ttcr.sell_amount, Some(dec!(100)));
                            assert_eq!(ttcr.sell_currency, "USD");
                            assert_eq!(ttcr.fee_amount, None);
                            assert_eq!(ttcr.fee_currency, "");
                            assert_eq!(ttcr.exchange, "");
                            assert_eq!(ttcr.group, None);
                            assert_eq!(ttcr.comment, "Gift to friend");
                            assert_eq!(ttcr.time, 0);
                        }
                        _ => panic!("Unexpected idx"),
                    }
                }
                Err(e) => panic!("Error: {e}"),
            }
        }
    }
}
