use crate::{
    binance_trade::convert,
    common::{
        create_buf_reader, create_buf_writer, dec_to_money_string, dec_to_separated_string,
        time_ms_to_utc, time_ms_to_utc_string, utc_now_to_time_ms, verify_input_files_exist,
    },
    configuration::Configuration,
    date_time_utc::DateTimeUtc,
    de_string_to_utc_time_ms::{de_string_to_utc_time_ms_condaddtzutc, se_time_ms_to_utc_string},
};

use clap::ArgMatches;
use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum TypeTxs {
    Income, // Income must sort as "smallest"
    Trade,
    Deposit,
    Withdrawal,
    Spend,
    Lost,
    Stolen,
    Mining,
    Gift,
    Unknown,
}

impl Display for TypeTxs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
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
}

impl Display for TokenTaxRec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "time: {} type_txs: {} buy_amount: {:?} buy_currency {} sell_amount: {:?} sell_currency: {} fee_amount: {:?} fee_currency: {} exchange: {} group: {:?} comment: {}",
            time_ms_to_utc_string(self.time),
            self.type_txs,
            self.buy_amount,
            self.buy_currency,
            self.sell_amount,
            self.sell_currency,
            self.fee_amount,
            self.fee_currency,
            self.exchange,
            self.group,
            self.comment,
        )
    }
}

// TODO: Add tests for Eq Ord PartialEq PartialOrd
impl Eq for TokenTaxRec {}

// Manually imiplement PartialEq so time is sorted first
impl PartialEq for TokenTaxRec {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
            && self.type_txs == other.type_txs
            && self.buy_currency == other.buy_currency
            && self.sell_currency == other.sell_currency
            && self.fee_currency == other.fee_currency
            && self.buy_amount == other.buy_amount
            && self.sell_amount == other.sell_amount
            && self.fee_amount == other.fee_amount
            && self.exchange == other.exchange
            && self.group == other.group
            && self.comment == other.comment
    }
}

impl PartialOrd for TokenTaxRec {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.time.partial_cmp(&other.time) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.type_txs.partial_cmp(&other.type_txs) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.buy_currency.partial_cmp(&other.buy_currency) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.sell_currency.partial_cmp(&other.sell_currency) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.fee_currency.partial_cmp(&other.fee_currency) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.buy_amount.partial_cmp(&other.buy_amount) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.sell_amount.partial_cmp(&other.sell_amount) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.fee_amount.partial_cmp(&other.fee_amount) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.exchange.partial_cmp(&other.exchange) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.group.partial_cmp(&other.group) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.comment.partial_cmp(&other.comment)
    }
}

impl Ord for TokenTaxRec {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(ord) => ord,
            None => panic!("WTF"),
        }
    }
}

#[allow(unused)]
fn ttr_cmp_income_time(lhs: &TokenTaxRec, rhs: &TokenTaxRec) -> std::cmp::Ordering {
    #[inline(always)]
    fn done(ord: Option<std::cmp::Ordering>) -> std::cmp::Ordering {
        //println!("PartialOrd::partial_cmp:- {ord:?}");
        match ord {
            Some(o) => o,
            None => panic!("WTF"),
        }
    }

    //println!("PartialOrd::partial_cmp:+\n lhs: {lhs:?}\n rhs: {rhs:?}");

    // We assume Income sorts as "smallest"
    match lhs.type_txs.partial_cmp(&rhs.type_txs) {
        Some(core::cmp::Ordering::Equal) => {}
        ord => return done(ord),
    }
    done(lhs.time.partial_cmp(&rhs.time))
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AssetRec {
    asset: String,
    quantity: Decimal,
    value_usd: Decimal,
    transaction_count: u64,
    ttr_vec: Vec<TokenTaxRec>,
    consolidated_ttr_vec: Vec<TokenTaxRec>,
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
            consolidated_ttr_vec: Vec::new(),
        }
    }

    fn consolidate(
        &self,
        ttr: &TokenTaxRec,
        period_time: &DateTimeUtc,
        quantity: Decimal,
    ) -> TokenTaxRec {
        let mut ttr = ttr.clone();
        ttr.time = period_time.time_ms();
        ttr.buy_amount = Some(quantity);

        ttr
    }

    fn consolidate_income(
        &mut self,
        config: &Configuration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        trace!("consolidate_income:+");

        self.ttr_vec.sort();

        if let Some(first_ttr) = self.ttr_vec.get(0) {
            let mut first_dt = DateTimeUtc::from_utc_time_ms(first_ttr.time);
            let mut start_period = first_dt.beginning_of_this_month();
            let mut end_period = start_period.beginning_of_next_month();
            let mut end_period_ms = end_period.time_ms();
            let mut consolidated_quantity = dec!(0);
            let mut cur_ttr = first_ttr.clone();
            trace!("first_dt: {first_dt} start_period: {start_period} end_period: {end_period} cur_ttr: {cur_ttr}");

            for (idx, ttr) in self.ttr_vec.iter().enumerate() {
                trace!("TOL           ttr: {ttr}  *** end_period: {end_period}");
                if ttr.time >= end_period_ms {
                    // This can never happen the first time through the loop
                    // because the first_dt was used to initialize start_period and end_period
                    assert!(idx != 0);

                    if consolidated_quantity > dec!(0) {
                        // Push consolidated record
                        let cttr = self.consolidate(&cur_ttr, &first_dt, consolidated_quantity);
                        trace!("End Period       Q {consolidated_quantity} push {cttr}");
                        self.consolidated_ttr_vec.push(cttr);
                    }

                    cur_ttr = ttr.clone();
                    first_dt = DateTimeUtc::from_utc_time_ms(cur_ttr.time);
                    start_period =
                        DateTimeUtc::from_utc_time_ms(ttr.time).beginning_of_this_month();
                    end_period = start_period.beginning_of_next_month();
                    end_period_ms = end_period.time_ms();
                    consolidated_quantity = dec!(0);
                    trace!(
                        "New  period      Q {consolidated_quantity} start_period:  {start_period} end_period: {end_period}"
                    );
                }

                if ttr.type_txs == TypeTxs::Income {
                    // Consolidate this entry
                    if cur_ttr.type_txs != TypeTxs::Income {
                        // And this is the first one
                        cur_ttr = ttr.clone();
                        first_dt = DateTimeUtc::from_utc_time_ms(cur_ttr.time);
                        consolidated_quantity = dec!(0);
                        trace!("First Income     Q {consolidated_quantity}");
                    }
                    consolidated_quantity += ttr.buy_amount.unwrap();
                    trace!("Income           Q {consolidated_quantity}");
                } else {
                    trace!("Not Income");
                    self.consolidated_ttr_vec.push(ttr.clone());
                }
            }
            // Do the last entry
            if (consolidated_quantity > dec!(0)) {
                // Push consolidated record
                let cttr = self.consolidate(&cur_ttr, &first_dt, consolidated_quantity);
                trace!("End Period       Q {consolidated_quantity} push {cttr}");
                self.consolidated_ttr_vec.push(cttr);
            }
        }

        trace!("consolidate_income:-");
        Ok(())
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
        // The asset is always either buy_currency or sell_currency
        let asset = ttr.get_asset();

        let entry = self
            .bt
            .entry(asset.to_owned())
            .or_insert_with(|| AssetRec::new(asset));

        entry.ttr_vec.push(ttr.clone());
        //println!("AssetRecMap.push_rec: asset: {asset} ttr_vec.len: {} ttr: {ttr:?}", entry.ttr_vec.len());
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

    fn add_quantity(&mut self, asset: &str, val: Decimal) {
        let entry = self.bt.get_mut(asset).unwrap();
        entry.quantity += val;
    }

    fn sub_quantity(&mut self, asset: &str, val: Decimal) {
        self.add_quantity(asset, -val)
    }
}

#[derive(Debug)]
struct TokenTaxData {
    ttr_vec: Vec<TokenTaxRec>,
    consolidated_ttr_vec: Vec<TokenTaxRec>,
    asset_rec_map: AssetRecMap,
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
}

impl TokenTaxData {
    fn new() -> TokenTaxData {
        TokenTaxData {
            ttr_vec: Vec::new(),
            consolidated_ttr_vec: Vec::new(),
            asset_rec_map: AssetRecMap::new(),
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
        }
    }
}

// We assume that update_all_usd_values has been run prior
// to calling process_entry and thus can use unwrap() on
// the Option<Decimal> fields.
fn process_entry(
    config: &Configuration,
    data: &mut TokenTaxData,
    arm: &mut AssetRecMap,
    ttr: &TokenTaxRec,
    line_number: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let leading_nl = if config.progress_info { "\n" } else { "" };

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

    if let Some(buy_amount) = ttr.buy_amount {
        assert!(
            !ttr.buy_currency.is_empty(),
            "{}",
            format!("line_number {line_number}: type_txs: {}, buy_currency: is_empty(), buy_amount: {buy_amount}, comment: {}", ttr.type_txs, ttr.comment)
        );
        assert!(
            buy_amount >= dec!(0),
            "{}",
            format!("Expected buy_amount: {buy_amount} >= dec!(0) at line_number: {line_number}")
        );
        arm.add_quantity(&ttr.buy_currency, buy_amount);
    }

    if let Some(sell_amount) = ttr.sell_amount {
        assert!(sell_amount >= dec!(0));
        assert!(!ttr.sell_currency.is_empty());
        arm.sub_quantity(&ttr.sell_currency, sell_amount);
    }

    if let Some(fee_amount) = ttr.fee_amount {
        assert!(fee_amount >= dec!(0));
        assert!(!ttr.fee_currency.is_empty());
        arm.sub_quantity(&ttr.fee_currency, fee_amount);
    }

    // TODO: For all the category and operations we need to save asset_value_usd as "usd_cost_basis"
    match ttr.type_txs {
        TypeTxs::Unknown => {
            println!(
                "{leading_nl}{} Unknown transaction type: {:?}",
                line_number, ttr
            );
            data.type_txs_unknown_count += 1;
        }
        TypeTxs::Trade => {
            assert!(!ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_trade_count += 1;
        }
        TypeTxs::Deposit => {
            assert!(!ttr.buy_currency.is_empty());
            assert!(ttr.sell_currency.is_empty());
            data.type_txs_deposit_count += 1;
        }
        TypeTxs::Withdrawal => {
            assert!(ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_withdrawal_count += 1;
        }
        TypeTxs::Income => {
            assert!(!ttr.buy_currency.is_empty());
            assert!(ttr.sell_currency.is_empty());
            data.type_txs_income_count += 1;
        }
        TypeTxs::Spend => {
            assert!(ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_spend_count += 1;
        }
        TypeTxs::Lost => {
            assert!(ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_lost_count += 1;
        }
        TypeTxs::Stolen => {
            assert!(ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_stolen_count += 1;
        }
        TypeTxs::Mining => {
            assert!(!ttr.buy_currency.is_empty());
            assert!(ttr.sell_currency.is_empty());
            data.type_txs_mining_count += 1;
        }
        TypeTxs::Gift => {
            assert!(ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_gift_count += 1;
        }
    }

    Ok(())
}

pub async fn process_token_tax_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let leading_nl = if config.progress_info { "\n" } else { "" };
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

    // usd_value_need is true unless --no-usd-value-need is present
    let usd_value_needed = !sc_matches.is_present("no-usd-value-needed");

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

    println!("Read files");
    for f in in_tt_file_paths {
        println!("{leading_nl}file: {f}");
        let reader = create_buf_reader(f)?;

        // Create reader
        let mut rdr = csv::Reader::from_reader(reader);

        for (rec_index, result) in rdr.deserialize().enumerate() {
            let line_number = rec_index + 2;
            let ttr: TokenTaxRec = result?;

            if config.progress_info {
                let asset = ttr.get_asset();
                print!("Processing {line_number} {asset}                        \r",);
            }

            process_entry(config, &mut data, &mut asset_rec_map, &ttr, line_number)?;

            data.ttr_vec.push(ttr.clone());
            data.asset_rec_map.push_rec(ttr.clone());
        }
    }
    println!();

    println!("Sorting");
    data.ttr_vec.sort();
    println!("Sorting done");

    println!("ttr_vec: len: {}", data.ttr_vec.len());

    if let Some(w) = &mut csv_ttr_writer {
        println!("Writing to {}", out_tt_file_path.unwrap());
        for dr in &data.ttr_vec {
            w.serialize(dr)?;
        }
        w.flush()?;
        println!("Writing done");
    }
    println!();

    if config.verbose {
        let mut total_value_usd = dec!(0);
        let mut total_quantity = dec!(0);

        let ten_minutes_ago = utc_now_to_time_ms() - (10 * 60 * 1000);
        let convert_time = if let Some(last_rec) = data.ttr_vec.last() {
            // The beginning of the next_day
            let last_time = DateTimeUtc::from_utc_time_ms(last_rec.time);
            let next_day = last_time.beginning_of_next_day();
            let time_ms = next_day.time_ms();

            time_ms.min(ten_minutes_ago)
        } else {
            ten_minutes_ago
        };

        let time_utc_string = if usd_value_needed {
            let dt_utc = time_ms_to_utc(convert_time);
            dt_utc.to_string()
        } else {
            "".to_owned()
        };

        let col_1_width = 10;
        let col_2_width = 20;
        let col_3_width = 10;
        let col_4_width = if usd_value_needed { 14 } else { 0 };
        println!(
            "{:col_1_width$} {:>col_2_width$} {:>col_3_width$}{}{:>col_4_width$}{}{}",
            "Asset",
            "Quantity",
            "Txs count",
            if usd_value_needed { " " } else { "" },
            if usd_value_needed { " USD value" } else { "" },
            if usd_value_needed { " " } else { "" },
            time_utc_string,
        );

        #[allow(clippy::for_kv_map)]
        for (_, ar) in &mut asset_rec_map.bt {
            //println!("TOL {ar:?}");
            let value_usd_string = if usd_value_needed {
                if let Ok(value_usd) =
                    convert(config, convert_time, &ar.asset, ar.quantity, "USD").await
                {
                    ar.value_usd = value_usd;
                    dec_to_money_string(ar.value_usd)
                } else {
                    ar.value_usd = dec!(0);
                    "?".to_owned()
                }
            } else {
                ar.value_usd = dec!(0);
                "".to_owned()
            };
            total_value_usd += ar.value_usd;
            total_quantity += ar.quantity;

            println!(
                "{:col_1_width$} {:>col_2_width$} {:>col_3_width$}{}{:>col_4_width$}",
                ar.asset,
                dec_to_separated_string(ar.quantity, 8),
                dec_to_separated_string(Decimal::from(ar.transaction_count), 0),
                if usd_value_needed { " " } else { "" },
                value_usd_string,
            );
        }

        println!();
        println!(
            "Total quantity: {} account value: {}",
            dec_to_separated_string(total_quantity, 8),
            dec_to_money_string(total_value_usd)
        );

        println!(
            "total txs count: {}",
            dec_to_separated_string(Decimal::from(data.total_count), 0)
        );
    }

    Ok(())
}

pub async fn consolidate_token_tax_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("consolidate_token_tax_files:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let mut data = TokenTaxData::new();
    //let mut asset_rec_map = AssetRecMap::new();

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

    trace!("in_tt_file_path: {in_tt_file_paths:?}");
    trace!("out_tt_file_path: {out_tt_file_path:?}");

    // Verify all input files exist
    verify_input_files_exist(&in_tt_file_paths)?;

    // Create csv::Writer if out_file_path exists
    let mut csv_ttr_writer = if let Some(out_file_path) = out_tt_file_path {
        let writer = create_buf_writer(out_file_path)?;
        Some(csv::Writer::from_writer(writer))
    } else {
        None
    };

    print!("Read files");
    for f in in_tt_file_paths {
        println!("\nfile: {f}");
        trace!("top loop: f: {f}");
        let reader = create_buf_reader(f)?;

        // Create reader
        let mut rdr = csv::Reader::from_reader(reader);

        for (rec_index, result) in rdr.deserialize().enumerate() {
            let line_number = rec_index + 2;
            let ttr: TokenTaxRec = result?;

            //println!("{line_number}: {ttr}");

            if config.progress_info {
                let asset = ttr.get_asset();
                print!("Processing {line_number} {asset}                        \r",);
            }

            data.ttr_vec.push(ttr.clone());
            data.asset_rec_map.push_rec(ttr.clone());
        }
    }

    let col_1 = 10;
    let col_2 = 20;
    let col_3 = 15;

    let mut total_pre_len = 0usize;
    let mut total_post_len = 0usize;
    println!("Consolidate");
    println!(
        "{:<col_1$} {:>col_2$} {:>col_3$}",
        "Asset", "pre count", "post count"
    );

    for (asset, ar) in &mut data.asset_rec_map.bt {
        //trace!("consolidating: {asset}");
        let pre_len = ar.ttr_vec.len();
        total_pre_len += pre_len;

        ar.consolidate_income(config)?;

        let post_len = ar.consolidated_ttr_vec.len();
        total_post_len += post_len;

        // Append the ar.consolidated_ttr_vec to end of data.consolidated_ttr_vec
        for x in &ar.consolidated_ttr_vec {
            data.consolidated_ttr_vec.push(x.clone());
        }

        println!(
            "{:<col_1$} {:>col_2$} {:>col_3$}",
            asset,
            dec_to_separated_string(Decimal::from_f64(pre_len as f64).unwrap(), 0),
            dec_to_separated_string(Decimal::from_f64(post_len as f64).unwrap(), 0),
        );
    }
    println!("Consolidated from {} to {}", total_pre_len, total_post_len);

    data.consolidated_ttr_vec.sort();
    //println!("consolidate_tt_files:");
    //for ttr in &data.consolidated_ttr_vec {
    //    println!("{ttr}");
    //}

    if let Some(w) = &mut csv_ttr_writer {
        println!("Writing consolidated data to {}", out_tt_file_path.unwrap());
        for ttr in &data.consolidated_ttr_vec {
            w.serialize(&ttr)?;
        }
        w.flush()?;
    }

    println!("Done");
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
