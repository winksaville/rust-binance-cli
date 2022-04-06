use crate::{
    binance_trade::convert,
    common::{create_buf_reader, create_buf_writer, verify_input_files_exist},
    configuration::Configuration,
    date_time_utc::DateTimeUtc,
};

use dec_utils::{dec_to_separated_string, dec_to_usd_string};
use time_ms_conversions::{time_ms_to_utc, utc_now_to_time_ms};

use clap::ArgMatches;
use log::trace;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tokentaxrec::{TokenTaxRec, TokenTaxRecType};

use std::{collections::BTreeMap, fmt::Debug};

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

                if ttr.type_txs == TokenTaxRecType::Income {
                    // Consolidate this entry
                    if cur_ttr.type_txs != TokenTaxRecType::Income {
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
            if (consolidated_quantity != dec!(0)) {
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
    //asset_rec_map: AssetRecMap,
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
            //asset_rec_map: AssetRecMap::new(),
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
                // Can happen but unusual
                //println!("WARNING adding missing other_asset: {}",ttr.get_other_asset());
                AssetRec::new(ttr.get_other_asset())
            });
    }
    if !ttr.fee_currency.is_empty() {
        let _ = arm
            .bt
            .entry(ttr.fee_currency.to_owned())
            .or_insert_with(|| {
                // Can happen but unusual
                //println!("WARNING adding missing fee_currency: {}", ttr.fee_currency);
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
        // I've seen this on two recrods from binance.us, weird so just a warning.
        //assert!(
        //    buy_amount >= dec!(0),
        //    "{}",
        //    format!("Expected buy_amount: {buy_amount} >= dec!(0) at line_number: {line_number}")
        //);
        if buy_amount < dec!(0) {
            println!("WARNING buy_amount: {buy_amount} >= dec!(0) at line_number: {line_number}");
        }
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
        TokenTaxRecType::Unknown => {
            println!(
                "{leading_nl}{} Unknown transaction type: {:?}",
                line_number, ttr
            );
            data.type_txs_unknown_count += 1;
        }
        TokenTaxRecType::Trade => {
            assert!(!ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_trade_count += 1;
        }
        TokenTaxRecType::Deposit => {
            assert!(!ttr.buy_currency.is_empty());
            assert!(ttr.sell_currency.is_empty());
            data.type_txs_deposit_count += 1;
        }
        TokenTaxRecType::Withdrawal => {
            assert!(ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_withdrawal_count += 1;
        }
        TokenTaxRecType::Income => {
            assert!(!ttr.buy_currency.is_empty());
            assert!(ttr.sell_currency.is_empty());
            data.type_txs_income_count += 1;
        }
        TokenTaxRecType::Spend => {
            assert!(ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_spend_count += 1;
        }
        TokenTaxRecType::Lost => {
            assert!(ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_lost_count += 1;
        }
        TokenTaxRecType::Stolen => {
            assert!(ttr.buy_currency.is_empty());
            assert!(!ttr.sell_currency.is_empty());
            data.type_txs_stolen_count += 1;
        }
        TokenTaxRecType::Mining => {
            assert!(!ttr.buy_currency.is_empty());
            assert!(ttr.sell_currency.is_empty());
            data.type_txs_mining_count += 1;
        }
        TokenTaxRecType::Gift => {
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

    let out_tt_file_path = sc_matches.value_of("OUT_FILE");

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
                print!("Processing {line_number} {asset}                        \r");
            }

            process_entry(config, &mut data, &mut asset_rec_map, &ttr, line_number)?;

            data.ttr_vec.push(ttr.clone());
            asset_rec_map.push_rec(ttr);
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
                    dec_to_usd_string(ar.value_usd)
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
            "Total quantity: {}",
            dec_to_separated_string(total_quantity, 8),
        );
        if usd_value_needed {
            println!(" account value: {}", dec_to_usd_string(total_value_usd));
        }
        println!();

        println!(
            "Total txs count: {}",
            dec_to_separated_string(Decimal::from(data.total_count), 0)
        );
        println!(
            "Total asset count: {}",
            dec_to_separated_string(Decimal::from(asset_rec_map.bt.len()), 0)
        );
    }

    Ok(())
}

pub async fn consolidate_token_tax_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("consolidate_token_tax_files:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let leading_nl = if config.progress_info { "\n" } else { "" };

    let mut data = TokenTaxData::new();
    let mut asset_rec_map = AssetRecMap::new();

    let in_tt_file_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();

    let out_tt_file_path = sc_matches.value_of("OUT_FILE");

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

    println!("Read files");
    for f in in_tt_file_paths {
        println!("{leading_nl}file: {f}");
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

            process_entry(config, &mut data, &mut asset_rec_map, &ttr, line_number)?;

            data.ttr_vec.push(ttr.clone());
            asset_rec_map.push_rec(ttr);
        }
    }

    let col_1 = 10;
    let col_2 = 20;
    let col_3 = 15;

    let mut total_pre_len = 0usize;
    let mut total_post_len = 0usize;
    println!("{leading_nl}Consolidate");
    println!(
        "{:<col_1$} {:>col_2$} {:>col_3$}",
        "Asset", "pre count", "post count"
    );

    for (asset, ar) in &mut asset_rec_map.bt {
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

/// uniq_currency_token_tax_files
///
/// The initial goal of this function is to find a subset of transactions
/// such we can test that every asset can converted into a "quote" asset
/// such as USD.
///
/// Another function cound/should probably be written that also includes every
/// type of operation is in the data set is represented in some test data.
///
/// And maybe in an ideal case this code or some other function should create,
/// for each asset, an array of every possible operation and find one transaction
/// with that operation for each asset. It's not expected that every asset
/// would be used in every operation but that we should emit one transaction
/// of each operation that an asset does participate in.
pub async fn uniq_currency_token_tax_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("uniq_currency_token_tax_files:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let leading_nl = if config.progress_info { "\n" } else { "" };

    let mut data = TokenTaxData::new();
    let mut asset_rec_map = AssetRecMap::new();

    let in_tt_file_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();

    let out_tt_file_path = sc_matches.value_of("OUT_FILE");

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

    println!("Read files");
    for f in in_tt_file_paths {
        println!("{leading_nl}file: {f}");
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
                print!("Processing {line_number} {asset}                        \r");
            }

            process_entry(config, &mut data, &mut asset_rec_map, &ttr, line_number)?;

            data.ttr_vec.push(ttr.clone());
            asset_rec_map.push_rec(ttr);
        }
    }

    let mut uc_ttr_vec = Vec::<TokenTaxRec>::new();
    println!(
        "{leading_nl}asset_rec_map.bt.len: {}",
        asset_rec_map.bt.len()
    );
    for ar in &mut asset_rec_map.bt.values() {
        let mut was_pushed = false;
        if let Some(rec) = ar.ttr_vec.get(0) {
            // We have rec, but lets prefer an Income rec as they are simple
            for ttr in ar.ttr_vec.iter() {
                if ttr.type_txs == TokenTaxRecType::Income {
                    uc_ttr_vec.push(ttr.clone());
                    was_pushed = true;
                    break;
                }
            }
            if !was_pushed {
                uc_ttr_vec.push(rec.clone());
            }
        }
    }

    println!("${leading_nl}Sorting");
    uc_ttr_vec.sort();
    println!("Sorting done");

    println!("uc_ttr_vec.len: {}", uc_ttr_vec.len());
    //uc_ttr_vec.iter().for_each(|r| println!("rec: {r}"));

    if let Some(w) = &mut csv_ttr_writer {
        println!("Writing uc_ttr data to {}", out_tt_file_path.unwrap());
        for ttr in &uc_ttr_vec {
            w.serialize(ttr)?;
        }
        w.flush()?;
    }

    println!("Done");
    Ok(())
}

#[cfg(test)]
mod test {

    use rust_decimal_macros::dec;
    use tokentaxrec::{GroupType, TokenTaxRecType};

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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Deposit);
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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Trade);
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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Trade);
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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Income);
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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Withdrawal);
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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Spend);
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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Lost);
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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Stolen);
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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Mining);
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
                            assert_eq!(ttcr.type_txs, TokenTaxRecType::Gift);
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

    #[test]
    fn test_consolidate_income() {
        // Very tricky corner case only one record and with a negative value.
        let csv = r#"
Type,BuyAmount,BuyCurrency,SellAmount,SellCurrency,FeeAmount,FeeCurrency,Exchange,Group,Comment,Date
Income,-6.890204,NANO,,,,,binance.us,,"v4,0,341330,,960697047,Distribution,Others",2022-02-20T07:57:16.000+00:00
"#;

        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);

        let mut ar = AssetRec::new("NANO");
        for result in reader.deserialize() {
            match result {
                Ok(entry) => {
                    ar.ttr_vec.push(entry);
                }
                Err(e) => panic!("Error: {e}")
            }
        }

        let config: Configuration = toml::from_str("").unwrap();
        ar.consolidate_income(&config).unwrap();
        assert_eq!(ar.consolidated_ttr_vec.len(), 1);
        let crec = &ar.consolidated_ttr_vec[0];
        assert_eq!(crec.buy_currency, "NANO");
        assert_eq!(crec.buy_amount, Some(dec!(-6.890204)));
    }
}
