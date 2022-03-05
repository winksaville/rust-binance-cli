//! This file processes binance.com commission files.
//!
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fmt::Display,
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::{
    arg_matches::time_offset_days_to_time_ms_offset,
    common::{
        create_buf_reader, create_buf_writer, create_buf_writer_from_path, dec_to_separated_string,
        time_ms_to_utc_string, verify_input_files_exist,
    },
    configuration::Configuration,
    de_string_to_utc_time_ms::{de_string_to_utc_time_ms_condaddtzutc, se_time_ms_to_utc_string},
    process_token_tax::{TokenTaxRec, TypeTxs},
    token_tax_comment_vers::{TT_CMT_VER1, TT_CMT_VER2},
};
use clap::ArgMatches;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone, Ord, Eq, PartialEq, PartialOrd)]
// Order Type,Friend's ID(Spot),Friend's sub ID (Spot),Commission Asset,Commission Earned,Commission Earned (USDT),Commission Time,Registration Time,Referral ID
//USDT-futures,42254326,"",USDT,0.00608292,0.00608300,2022-01-01 07:49:33,2021-03-31 21:58:24,bpcode
struct CommissionRec {
    #[serde(rename = "Order Type")]
    order_type: String,

    #[serde(rename = "Friend's ID(Spot)")]
    friends_id_spot: u64,

    #[serde(rename = "Friend's sub ID (Spot)")]
    friends_sub_id_spot: String,

    #[serde(rename = "Commission Asset")]
    commission_asset: String,

    #[serde(rename = "Commission Earned")]
    commission_earned: Decimal,

    #[serde(rename = "Commission Earned (USDT)")]
    commission_earned_usdt: Decimal,

    #[serde(rename = "Commission Time")]
    #[serde(deserialize_with = "de_string_to_utc_time_ms_condaddtzutc")]
    #[serde(serialize_with = "se_time_ms_to_utc_string")]
    commission_time: i64,

    #[serde(rename = "Registration Time")]
    #[serde(deserialize_with = "de_string_to_utc_time_ms_condaddtzutc")]
    #[serde(serialize_with = "se_time_ms_to_utc_string")]
    registration_time: i64,

    #[serde(rename = "Referral ID")]
    referral_id: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Ord, Eq, PartialEq, PartialOrd)]
// User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
// 123456789,2021-01-01 00:00:31,Spot,Commission History,DOT,0.00505120,""
struct TradeRec {
    #[serde(rename = "User_ID")]
    user_id: String,

    #[serde(rename = "UTC_Time")]
    #[serde(deserialize_with = "de_string_to_utc_time_ms_condaddtzutc")]
    #[serde(serialize_with = "se_time_ms_to_utc_string")]
    time: i64,

    #[serde(rename = "Account")]
    account: String,

    #[serde(rename = "Operation")]
    operation: String,

    #[serde(rename = "Coin")]
    coin: String,

    #[serde(rename = "Change")]
    change: Decimal,

    #[serde(rename = "Remark")]
    remark: String,
}

impl TradeRec {
    fn new() -> TradeRec {
        Default::default()
    }
}

impl Display for TradeRec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.user_id,
            time_ms_to_utc_string(self.time),
            self.account,
            self.operation,
            self.coin,
            self.change,
            self.remark
        )
    }
}

fn ttr_cmp_no_change_no_remark(lhs: &TradeRec, rhs: &TradeRec) -> std::cmp::Ordering {
    #[inline(always)]
    fn done(ord: Option<std::cmp::Ordering>) -> std::cmp::Ordering {
        //println!("PartialOrd::partial_cmp:- {ord:?}");
        match ord {
            Some(o) => o,
            None => panic!("WTF"),
        }
    }
    //println!("PartialOrd::partial_cmp:+\n lhs: {lhs:?}\n rhs: {rhs:?}");
    match lhs.user_id.partial_cmp(&rhs.user_id) {
        Some(core::cmp::Ordering::Equal) => {}
        ord => return done(ord),
    }
    match lhs.time.partial_cmp(&rhs.time) {
        Some(core::cmp::Ordering::Equal) => {}
        ord => return done(ord),
    }
    match lhs.account.partial_cmp(&rhs.account) {
        Some(core::cmp::Ordering::Equal) => {}
        ord => return done(ord),
    }
    match lhs.operation.partial_cmp(&rhs.operation) {
        Some(core::cmp::Ordering::Equal) => {}
        ord => return done(ord),
    }
    done(lhs.coin.partial_cmp(&rhs.coin))
}

#[derive(Debug)]
struct BcAssetRec {
    asset: String,
    quantity: Decimal,
    transaction_count: usize,
    tr_vec: Vec<TradeRec>,
    consolidated_tr_vec: Vec<TradeRec>,
}

impl BcAssetRec {
    fn new(asset: &str) -> BcAssetRec {
        BcAssetRec {
            asset: asset.to_owned(),
            quantity: dec!(0),
            transaction_count: 0,
            tr_vec: Vec::new(),
            consolidated_tr_vec: Vec::new(),
        }
    }
}

impl Display for BcAssetRec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<16} quantity: {:>20}   transaction_count: {:>10}",
            self.asset,
            dec_to_separated_string(self.quantity, 4),
            dec_to_separated_string(Decimal::from(self.transaction_count), 0)
        )
    }
}

impl BcAssetRec {
    // Consolidate records in BcAssetRec.tr_vec to BcAssetRec::consolidated_tr_vec
    // if the account and operation match
    fn consolidate_account_operation(&mut self, account: &str, operation: &str) {
        //println!("Consolidating {}", self.asset);

        const MS_PER_DAY: i64 = (60 * 60 * 24) * 1000;
        struct State {
            tr: TradeRec,
            time_of_next_consolidation_window: i64,
        }
        let mut state = State {
            tr: TradeRec::new(),
            time_of_next_consolidation_window: 0,
        };

        for tr in &mut self.tr_vec {
            if tr.account == account && tr.operation == operation {
                if state.time_of_next_consolidation_window == 0 {
                    state.time_of_next_consolidation_window =
                        ((tr.time + MS_PER_DAY) / MS_PER_DAY) * MS_PER_DAY;
                    state.tr = tr.clone();
                    *tr = TradeRec::new();
                    //println!("Consolidating {} window: {} tr.time: {} change: {} tr: {tr}", self.asset, time_ms_to_utc_string(state.time_of_next_consolidation_window), time_ms_to_utc_string(tr.time), state.tr.change);
                } else if tr.time < state.time_of_next_consolidation_window {
                    state.tr.change += tr.change;
                    *tr = TradeRec::new();
                    //println!("Consolidating {} window: {} tr.time: {} change: {} tr: {tr}", self.asset, time_ms_to_utc_string(state.time_of_next_consolidation_window), time_ms_to_utc_string(tr.time), state.tr.change);
                } else {
                    // Calculate New Time Window
                    state.time_of_next_consolidation_window =
                        ((tr.time + MS_PER_DAY) / MS_PER_DAY) * MS_PER_DAY;

                    // Add this record as we're Starting a new time window
                    let consolidated_tr = state.tr.clone();
                    state.tr = tr.clone();
                    *tr = TradeRec::new();

                    //println!("CONSOLIDATED  {} window: {} tr.time: {} change: {} tr: {tr} consolidated: {consolidated_tr}", self.asset, time_ms_to_utc_string(state.time_of_next_consolidation_window), time_ms_to_utc_string(tr.time), state.tr.change);
                    self.consolidated_tr_vec.push(consolidated_tr);
                }
            }
        }
        if !state.tr.user_id.is_empty() {
            //let tr = state.tr.clone();
            //println!("LAST one      {} window: {} tr.time: {} change: {} tr: {tr} consolidated: {tr}", self.asset, time_ms_to_utc_string(state.time_of_next_consolidation_window), time_ms_to_utc_string(tr.time), state.tr.change);
            self.consolidated_tr_vec.push(state.tr.clone());
        }
        //println!("consolidate_account_operation:- {account} {operation}");
        //for tr in &self.consolidated_tr_vec {
        //    println!("{}", tr);
        //}
    }

    #[allow(clippy::needless_return)]
    fn consolidate_trade_recs(
        &mut self,
        _config: &Configuration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //println!("consolidate_trade_recs:+");

        let mut sorted_by_account_operation = self.tr_vec.clone();
        sorted_by_account_operation.sort_by(|lhs, rhs| {
            assert!(self.asset == lhs.coin);
            assert!(self.asset == rhs.coin);
            assert!(lhs.user_id == rhs.user_id);
            match lhs.account.partial_cmp(&rhs.account) {
                Some(core::cmp::Ordering::Equal) => {}
                Some(ord) => return ord,
                None => panic!("WFT"),
            }

            match lhs.operation.partial_cmp(&rhs.operation) {
                Some(ord) => return ord,
                None => panic!("WFT"),
            }
        });

        self.consolidate_account_operation("Coin-Futures", "Referrer rebates");
        self.consolidate_account_operation("Pool", "Pool Distribution");
        self.consolidate_account_operation("Spot", "Commission History");
        self.consolidate_account_operation("Spot", "Commission Rebate");
        self.consolidate_account_operation("Spot", "ETH 2.0 Staking Rewards");
        self.consolidate_account_operation("USDT-Futures", "Referrer rebates");

        //println!("Consolidatation done          len: {}", self.consolidated_tr_vec.len());

        // Move the non-consolidated entries to consolidated_tr_vec
        for tr in &self.tr_vec {
            if !tr.coin.is_empty() {
                self.consolidated_tr_vec.push(tr.clone());
            }
        }

        // tr_vec is no longer has valid data, clear it
        self.tr_vec.clear();

        self.consolidated_tr_vec
            .sort_by(ttr_cmp_no_change_no_remark);

        // Update the quantity and transaction_count
        self.quantity = dec!(0);
        for tr in &self.consolidated_tr_vec {
            self.quantity += tr.change;
            self.transaction_count = self.consolidated_tr_vec.len();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct BcAssetRecMap {
    bt: BTreeMap<String, BcAssetRec>,
}

impl BcAssetRecMap {
    fn new() -> BcAssetRecMap {
        BcAssetRecMap {
            bt: BTreeMap::<String, BcAssetRec>::new(),
        }
    }

    fn add_tr(&mut self, tr: TradeRec) {
        // The asset is always either primary_asset or base_asset
        let asset = tr.coin.as_str();

        let entry = self
            .bt
            .entry(asset.to_owned())
            .or_insert_with(|| BcAssetRec::new(asset));
        entry.quantity += tr.change;
        entry.transaction_count += 1;
        entry.tr_vec.push(tr);
        assert_eq!(entry.transaction_count, entry.tr_vec.len());
    }

    //fn add_or_update(&mut self, asset: &str, quantity: Decimal) {
    //    let entry = self
    //        .bt
    //        .entry(asset.to_owned())
    //        .or_insert_with(|| BcAssetRec::new(asset));
    //    entry.quantity += quantity;
    //    entry.transaction_count += 1;
    //}

    //fn inc_transaction_count(&mut self, asset: &str) {
    //    let entry = self.bt.get_mut(asset).unwrap();
    //    entry.transaction_count += 1;
    //}

    //fn add_quantity(&mut self, asset: &str, val: Decimal) {
    //    let entry = self.bt.get_mut(asset).unwrap();
    //    entry.quantity += val;
    //}

    //fn sub_quantity(&mut self, asset: &str, val: Decimal) {
    //    self.add_quantity(asset, -val)
    //}
}

impl TokenTaxRec {
    fn format_tt_cmt_ver1(line_number: usize, bccr: &CommissionRec) -> String {
        let ver = TT_CMT_VER1.as_str();
        format!(
            "{ver},{line_number},{},{},{},{},{},{}",
            bccr.order_type,
            bccr.friends_id_spot,
            bccr.friends_sub_id_spot,
            bccr.commission_earned_usdt,
            bccr.registration_time,
            bccr.referral_id
        )
    }

    fn format_tt_cmt_ver2(line_number: usize, bctr: &TradeRec) -> String {
        let ver = TT_CMT_VER2.as_str();
        format!(
            "{ver},{line_number},{},{},{}",
            bctr.user_id, bctr.account, bctr.operation,
        )
    }

    #[allow(unused)]
    fn from_commission_rec(line_number: usize, bccr: &CommissionRec) -> TokenTaxRec {
        let mut ttr = TokenTaxRec::new();

        ttr.type_txs = TypeTxs::Income;
        ttr.buy_amount = Some(bccr.commission_earned);
        ttr.buy_currency = bccr.commission_asset.clone();
        ttr.exchange = "binance.com".to_owned();
        ttr.comment = TokenTaxRec::format_tt_cmt_ver1(line_number, bccr);
        ttr.time = bccr.commission_time;

        ttr
    }

    // Create a TokenTaxRec from a TradeRec.
    //
    // Returns: Ok(Some(TokenTaxRec)) if conversion was successful
    //          Ok(None) if the TradeRec::account,operation should be ignored
    //          Err if an error typically the account,operation pair were Unknown
    #[allow(unused)]
    fn from_trade_rec(
        line_number: usize,
        bctr: &TradeRec,
    ) -> Result<Option<TokenTaxRec>, Box<dyn std::error::Error>> {
        // User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
        // 123456789,2021-01-01 00:00:31,Spot,Commission History,DOT,0.00505120,""
        //
        // 2021:
        // wink@3900x:~/Documents/crypto/binance.com-trade-history/2021
        // $ cat part-0000* | csvcut -K 2,3 | sort | uniq -c
        //       3 Account,Operation
        //  188590 Coin-Futures,Referrer rebates
        //      99 Coin-Futures,transfer_out
        //     742 Pool,Pool Distribution
        //      18 Spot,Buy
        //  551603 Spot,Commission History
        //  335953 Spot,Commission Rebate
        //      91 Spot,Distribution
        //     322 Spot,ETH 2.0 Staking Rewards
        //      18 Spot,Fee
        //      18 Spot,Transaction Related
        //     168 Spot,transfer_in
        //       1 Spot,transfer_out
        //      47 Spot,Withdraw
        // 1080073 USDT-Futures,Referrer rebates
        //       1 USDT-Futures,transfer_in
        //      69 USDT-Futures,transfer_out
        //
        // 2020:
        // wink@3900x:~/prgs/rust/myrepos/binance-cli (Add-binance-com-processing)
        // $ cat data/b.com-trade-history/2020/part-0000* | csvcut -K 2,3 | sort | uniq -c
        //       1 Account,Operation
        //    3376 Coin-Futures,Referrer rebates
        //     323 Spot,Buy
        //  338433 Spot,Commission History
        //       1 Spot,Deposit
        //     193 Spot,Distribution
        //     322 Spot,Fee
        //      28 Spot,Savings Interest
        //       2 Spot,Savings Principal redemption
        //      32 Spot,Savings purchase
        //    1150 Spot,Small assets exchange BNB
        //     323 Spot,Transaction Related
        //       3 Spot,Withdraw
        //  133604 USDT-Futures,Referrer rebates

        // Combined 2020-2021
        // wink@3900x:~/prgs/rust/myrepos/binance-cli (Add-binance-com-processing)
        // $ cat data/b.com-trade-history/2020/part-0000* data/b.com-trade-history/2021/part-0000* | csvcut -K 2,3 | sort | uniq -c
        //       4 Account,Operation
        //  191966 Coin-Futures,Referrer rebates
        //      99 Coin-Futures,transfer_out
        //     742 Pool,Pool Distribution
        //     341 Spot,Buy
        //  890036 Spot,Commission History
        //  335953 Spot,Commission Rebate
        //       1 Spot,Deposit
        //     284 Spot,Distribution
        //     322 Spot,ETH 2.0 Staking Rewards
        //     340 Spot,Fee
        //      28 Spot,Savings Interest
        //       2 Spot,Savings Principal redemption
        //      32 Spot,Savings purchase
        //    1150 Spot,Small assets exchange BNB
        //     341 Spot,Transaction Related
        //     168 Spot,transfer_in
        //       1 Spot,transfer_out
        //      50 Spot,Withdraw
        // 1213677 USDT-Futures,Referrer rebates
        //       1 USDT-Futures,transfer_in
        //      69 USDT-Futures,transfer_out

        // TODO: Handle the all of the above "Account,Operations".
        let mut ttr = TokenTaxRec::new();

        // For all acount/operations these are the same
        ttr.exchange = "binance.com".to_owned();
        ttr.comment = TokenTaxRec::format_tt_cmt_ver2(line_number, bctr);
        ttr.time = bctr.time;

        // Most other account/operations are Income so
        // we'll assume them too.
        ttr.type_txs = TypeTxs::Income;
        ttr.buy_amount = Some(bctr.change);
        ttr.buy_currency = bctr.coin.clone();

        // Income should have no seller or fee information
        assert_eq!(ttr.sell_amount, None);
        assert_eq!(ttr.sell_currency, "");
        assert_eq!(ttr.fee_amount, None);
        assert_eq!(ttr.fee_currency, "");

        let result = match bctr.account.as_str() {
            "Coin-Futures" => match bctr.operation.as_str() {
                "Referrer rebates" => {
                    // Income: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=id.2kugk142pi0
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                    //   123456789,2021-01-01 02:33:56,Coin-Futures,Referrer rebates,BNB,0.00066774,""
                    Some(ttr)
                }
                "transfer_out" => {
                    // Non-taxable event: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=id.9y3dhg3cp1y8
                    None
                }
                "transfer_in" => {
                    // Non-taxable event: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=id.6ucacaaia5sl
                    None
                }
                _ => {
                    return Err(format!(
                        "Unknown bctr acccount: {} operation: {}",
                        bctr.account, bctr.operation
                    )
                    .into())
                }
            },
            "USDT-Futures" => match bctr.operation.as_str() {
                "Referrer rebates" => {
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                    //   123456789,2021-01-01 00:00:38,USDT-Futures,Referrer rebates,BNB,0.00237605,""
                    Some(ttr)
                }
                "transfer_out" => {
                    // Non-taxable event: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=id.9y3dhg3cp1y8
                    None
                }
                "transfer_in" => {
                    // Non-taxable event: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=id.6ucacaaia5sl
                    None
                }
                _ => {
                    return Err(format!(
                        "Unknown bctr acccount: {} operation: {}",
                        bctr.account, bctr.operation
                    )
                    .into());
                }
            },
            "Pool" => match bctr.operation.as_str() {
                "Pool Distribution" => {
                    ttr.type_txs = TypeTxs::Income;

                    Some(ttr)
                }
                _ => {
                    return Err(format!(
                        "Unknown bctr acccount: {} operation: {}",
                        bctr.account, bctr.operation
                    )
                    .into())
                }
            },
            "Spot" => match bctr.operation.as_str() {
                "Buy" => {
                    // Buy, Transaction Related and Fee transactions are part of a "Trade".
                    //
                    // Observations:
                    //  1) All have the same timestamp
                    //  2) Buy and Transaction Related are alway present, Fee was missing once
                    //  3) They are not in in any particular order
                    //  4) There maybe other transactions with the same timestamp
                    //  5) The Fee asset is always BNB
                    //  6) For all Trades with the same timestamp there is a unique Buy asset
                    //     and Transaction Related asset pair thus they maybe consolidated
                    //     into one Buy, Transaction Related and Fee transaction i.e. "Trade"
                    // Prove 6)
                    //    $ cat 2020/part-0000* 2021/part-0000* | csvgrep -p Operation/Buy | csvcut -K 1,2,3,4 | uniq -c > Buy.csv
                    //    $ head -2 Buy.csv
                    //          1 UTC_Time,Account,Operation,Coin
                    //          1 2020-05-08 01:31:06,Spot,Buy,BTC

                    //    $ cat 2020/part-0000* 2021/part-0000* | csvgrep -p "Operation/Transaction Related" | csvcut -K 1,2,3,4 | uniq -c > TransactionRelated.csv
                    //    $ head -2 TransactionRelated.csv
                    //          1 UTC_Time,Account,Operation,Coin
                    //          1 2020-05-08 01:31:06,Spot,Transaction Related,USDT

                    //    $ cat Buy.csv | csvcut -K 0,1 > Buy-count-time.csv
                    //    $ head -2 Buy-count-time.csv
                    //          1 UTC_Time,Account
                    //          1 2020-05-08 01:31:06,Spot

                    //    $ cat TransactionRelated.csv | csvcut -K 0,1 > TransactionRelated-count-time.csv
                    //    $ head -2 TransactionRelated-count-time.csv
                    //          1 UTC_Time,Account
                    //          1 2020-05-08 01:31:06,Spot

                    //    $ diff -s Buy-count-time.csv TransactionRelated-count-time.csv
                    //    Files Buy-count-time.csv and TransactionRelated-count-time.csv are identical

                    // This sequence shows the "unorderness"
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00014357,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00267651,""
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00195765,""
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00014356,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00255590,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-0.11000000,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00008040,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-1.50000000,""
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00465885,""
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00445039,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00112574,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-3.41000000,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-0.11000000,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-3.57000000,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00008040,""
                    //
                    // How to match Buy, Transaction Related and Fee:
                    //  if there re 3 less transactions {
                    //    use them as the transaction
                    //  } else {
                    //     Convert all transactions to a common asset, USD or USDT.
                    //     Sort each the Operations based on value of the common asset.
                    //     Merge sort them into trades (Note: Fee could be absent)
                    //  }
                    // Merge sort the resulting Operations into a Trade.
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00014357,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00267651,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-0.11000000,""
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00195765,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00255590,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-1.50000000,""
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00014356,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00008040,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-3.41000000,""
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00465885,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00112574,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-0.11000000,""
                    //  123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00445039,""
                    //  123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00008040,""
                    //  123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-3.57000000,""
                    //
                    //
                    // Here is an example of non-Trade transaction being interspersed in a Trade:
                    //  123456789,2020-05-09 20:42:56,Spot,Fee,BNB,-0.00123363,""
                    //  123456789,2020-05-09 20:42:56,Spot,Transaction Related,USDT,-27.99945000,""
                    //  123456789,2020-05-09 20:42:56,Spot,Commission History,BNB,0.00021562,""
                    //  123456789,2020-05-09 20:42:56,Spot,Buy,BTC,0.00289400,""
                    //
                    // Here is the transaction without a fee at 2020-05-09 05:12:24:
                    //  123456789,2020-05-09 05:02:32,Spot,Commission History,BNB,0.00028619,""
                    //  123456789,2020-05-09 05:08:29,Spot,Fee,BNB,-0.00109485,""
                    //  123456789,2020-05-09 05:08:29,Spot,Transaction Related,ETC,-3.52000000,""
                    //  123456789,2020-05-09 05:08:29,Spot,Buy,BTC,0.00256960,""
                    //  123456789,2020-05-09 05:09:31,Spot,Commission History,VET,1.71544000,""
                    //  123456789,2020-05-09 05:11:01,Spot,Buy,BTC,0.00418506,""
                    //  123456789,2020-05-09 05:11:01,Spot,Transaction Related,XRP,-187.00000000,""
                    //  123456789,2020-05-09 05:11:01,Spot,Fee,BNB,-0.00178794,""
                    //  123456789,2020-05-09 05:12:23,Spot,Fee,BNB,-0.00155840,""
                    //  123456789,2020-05-09 05:12:23,Spot,Transaction Related,USDT,-35.79505902,""
                    //  123456789,2020-05-09 05:12:23,Spot,Buy,BTC,0.00365400,""
                    //  123456789,2020-05-09 05:12:24,Spot,Transaction Related,BUSD,-35.05460000,""
                    //  123456789,2020-05-09 05:12:24,Spot,Buy,BTC,0.00357700,""
                    //  123456789,2020-05-09 05:12:29,Spot,Commission History,BNB,0.00023283,""
                    //

                    // TODO:
                    Some(ttr)
                }
                "Fee" | "Transaction Related" => {
                    // 123456789,2021-01-24 21:41:09,Spot,Fee,BNB,-0.00409488,""
                    // 123456789,2021-01-24 21:41:09,Spot,Transaction Related,ADA,-648.00000000,""
                    ttr.type_txs = TypeTxs::Spend;
                    if bctr.change < dec!(0) {
                        ttr.sell_amount = Some(-bctr.change);
                        ttr.sell_currency = bctr.coin.clone();
                        ttr.buy_amount = None;
                        ttr.buy_currency = "".to_owned();
                    } else {
                        return Err(format!(
                            "{line_number}: account: {} operation: {} the change: {} was expected to be negative",
                            bctr.account, bctr.operation, bctr.change,
                        )
                        .into());
                    }

                    Some(ttr)
                }
                "Commission History" => {
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                    //   123456789,2021-01-01 00:00:31,Spot,Commission History,DOT,0.00505120,""

                    Some(ttr)
                }
                "Commission Rebate" => {
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                    //   123456789,2021-06-23 03:54:55,Spot,Commission Rebate,BTC,2.2E-7,""
                    Some(ttr)
                }
                "Deposit" => {
                    // Deposit: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=id.9q4kesdhtivv
                    ttr.type_txs = TypeTxs::Deposit;
                    assert_eq!(ttr.buy_amount, Some(bctr.change));
                    assert_eq!(ttr.buy_currency, bctr.coin);
                    assert_eq!(ttr.sell_amount, Some(dec!(0)));
                    assert_eq!(ttr.sell_currency, "");
                    assert_eq!(ttr.fee_amount, Some(dec!(0)));
                    assert_eq!(ttr.fee_currency, "");

                    Some(ttr)
                }
                "Distribution" => {
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                    //  123456789,2020-01-03 05:58:34,Spot,Distribution,ALGO,0.08716713,""

                    Some(ttr)
                }
                "ETH 2.0 Staking Rewards" => {
                    //??
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark

                    Some(ttr)
                }
                "Savings Interest" => {
                    // Income: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=kix.b5b1syp9wm44
                    Some(ttr)
                }
                "Savings Principal redemption" => {
                    // Non-taxable event: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=kix.qcstrov9fmvj
                    None
                }
                "Savings purchase" => {
                    // Non-taxable event: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=kix.joyiqsumphny
                    None
                }
                "Small assets exchange BNB" => {
                    // This implements Option 2 of https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=id.ui3g3olz647l
                    if bctr.coin != "BNB" {
                        // Assert the change is negative
                        assert!(bctr.change < dec!(0));
                        ttr.type_txs = TypeTxs::Spend;
                        ttr.buy_amount = Some(dec!(0));
                        ttr.buy_currency = "".to_owned();
                        ttr.sell_amount = Some(-bctr.change);
                        ttr.sell_currency = bctr.coin.clone();
                    } else {
                        // Assert the change is positive and coin is BNB
                        assert!(bctr.coin == "BNB"); // This is redundant but just incase!!
                        assert!(bctr.change > dec!(0));
                        ttr.type_txs = TypeTxs::Income;
                        ttr.buy_amount = Some(bctr.change);
                        ttr.buy_currency = bctr.coin.clone();
                        ttr.sell_amount = Some(dec!(0));
                        ttr.sell_currency = "".to_owned();
                    }

                    Some(ttr)
                }
                "transfer_out" => {
                    // Non-taxable event: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=id.9y3dhg3cp1y8
                    None
                }
                "transfer_in" => {
                    // Non-taxable event: https://docs.google.com/document/d/1O1kSLV81cHmFDZVC12OhwRGj8z9tm83LHpcPrETSSYs/edit#bookmark=id.6ucacaaia5sl
                    None
                }
                "Withdraw" => {
                    // 123456789,2021-01-24 22:24:15,Spot,Withdraw,USDT,-2179.39975800,Withdraw fee is included
                    ttr.type_txs = TypeTxs::Withdrawal;
                    if bctr.change < dec!(0) {
                        ttr.sell_amount = Some(-bctr.change);
                        ttr.sell_currency = bctr.coin.clone();
                        ttr.buy_amount = None;
                        ttr.buy_currency = "".to_owned();
                    } else {
                        return Err(format!(
                            "{line_number}: account: {} operation: {} the change: {} was expected to be negative",
                            bctr.account, bctr.operation, bctr.change,
                        )
                        .into());
                    }

                    Some(ttr)
                }
                _ => {
                    return Err(format!(
                        "Unknown bctr acccount: {} operation: {}",
                        bctr.account, bctr.operation
                    )
                    .into());
                }
            },
            _ => return Err(format!("Unknown bctr acccount: {}", bctr.account).into()),
        };

        Ok(result)
    }
}

// From bctr_buy, _sell and _fee records create a TokenTax trade record
#[allow(unused)]
async fn to_tt_trade_rec(
    _config: &Configuration,
    bctr_buy: &TradeRec,
    bctr_sell: &TradeRec,
    bctr_fee: Option<&TradeRec>,
) -> Result<TokenTaxRec, Box<dyn std::error::Error>> {
    let mut ttr = TokenTaxRec::new();

    assert_eq!(bctr_buy.time, bctr_sell.time);
    let fee_time = if let Some(fee) = bctr_fee {
        fee.time
    } else {
        bctr_buy.time
    };
    assert_eq!(bctr_buy.time, fee_time);

    ttr.type_txs = TypeTxs::Trade;
    ttr.buy_amount = Some(bctr_buy.change);
    ttr.buy_currency = bctr_buy.coin.to_owned();
    ttr.sell_amount = Some(bctr_sell.change);
    ttr.sell_currency = bctr_sell.coin.to_owned();

    if let Some(fee) = bctr_fee {
        ttr.fee_amount = Some(fee.change);
        ttr.fee_currency = fee.coin.to_owned();
    }

    Ok(ttr)
}

#[derive(Debug)]
struct BcData {
    tr_vec: Vec<TradeRec>,
    bc_asset_rec_map: BcAssetRecMap,
    bc_consolidated_tr_vec: Vec<TradeRec>,
    transfer_in_count: u64,
    transfer_out_count: u64,
    savings_principal: Decimal,
    savings_principal_redemption_of_ldbtc: u64,
    total_count: u64,
}

impl BcData {
    fn new() -> BcData {
        BcData {
            tr_vec: Vec::new(),
            bc_asset_rec_map: BcAssetRecMap::new(),
            bc_consolidated_tr_vec: Vec::new(),
            transfer_in_count: 0u64,
            transfer_out_count: 0u64,
            savings_principal: dec!(0),
            savings_principal_redemption_of_ldbtc: 0u64,
            total_count: 0u64,
        }
    }
}

fn process_entry(bc_data: &mut BcData, bctr: &TradeRec) -> Result<(), Box<dyn std::error::Error>> {
    let ar = bc_data
        .bc_asset_rec_map
        .bt
        .entry(bctr.coin.clone())
        .or_insert_with(|| {
            // This happens the first time an asset is seen and is not unusual
            //println!("Adding missing asset: {}", asset);
            BcAssetRec::new(&bctr.coin)
        });

    bc_data.tr_vec.push(bctr.clone());

    ar.transaction_count += 1;
    ar.quantity += bctr.change;

    match bctr.account.as_str() {
        "Coin-Futures" => match bctr.operation.as_str() {
            "Referrer rebates" => {
                // ?
            }
            "transfer_out" => {
                assert!(bctr.change <= dec!(0));
                bc_data.transfer_out_count += 1;
            }
            "transfer_in" => {
                assert!(bctr.change >= dec!(0));
                bc_data.transfer_in_count += 1;
            }
            _ => {
                return Err(format!(
                    "Unknown bctr acccount: {} operation: {}",
                    bctr.account, bctr.operation
                )
                .into())
            }
        },
        "USDT-Futures" => match bctr.operation.as_str() {
            "Referrer rebates" => {
                // ?
            }
            "transfer_out" => {
                assert!(bctr.change <= dec!(0));
                bc_data.transfer_out_count += 1;
            }
            "transfer_in" => {
                assert!(bctr.change >= dec!(0));
                bc_data.transfer_in_count += 1;
            }
            _ => {
                return Err(format!(
                    "Unknown bctr acccount: {} operation: {}",
                    bctr.account, bctr.operation
                )
                .into());
            }
        },
        "Pool" => match bctr.operation.as_str() {
            "Pool Distribution" => {
                // ?
            }
            _ => {
                return Err(format!(
                    "Unknown bctr acccount: {} operation: {}",
                    bctr.account, bctr.operation
                )
                .into())
            }
        },
        "Spot" => match bctr.operation.as_str() {
            "Buy" => {
                // ?
            }
            "Fee" | "Transaction Related" => {
                // ?
            }
            "Commission History" => {
                // ?
            }
            "Commission Rebate" => {
                // ?
            }
            "Deposit" => {
                // ?
            }
            "Distribution" => {
                // ?
            }
            "ETH 2.0 Staking Rewards" => {
                // ?
            }
            "Savings Interest" => {
                assert!(bctr.coin == "BTC");
                assert!(bctr.change >= dec!(0));
            }
            "Savings Principal redemption" => {
                // maybe either positive or negative
                match bctr.coin.as_str() {
                    "BTC" => {
                        bc_data.savings_principal += bctr.change;
                    }
                    "LDBTC" => {
                        bc_data.savings_principal_redemption_of_ldbtc += 1;
                    }
                    _ => {
                        return Err(format!(
                            "Unexpected coin {}, in {}",
                            bctr.account, bctr.operation
                        )
                        .into());
                    }
                }
            }
            "Savings purchase" => {
                assert!(bctr.change <= dec!(0));
                bc_data.savings_principal += bctr.change;
            }
            "Small assets exchange BNB" => {
                // ?
            }
            "transfer_out" => {
                assert!(bctr.change <= dec!(0));
                bc_data.transfer_out_count += 1;
            }
            "transfer_in" => {
                assert!(bctr.change >= dec!(0));
                bc_data.transfer_in_count += 1;
            }
            "Withdraw" => {
                assert!(bctr.change < dec!(0));
            }
            _ => {
                return Err(format!(
                    "Unknown bctr acccount: {} operation: {}",
                    bctr.account, bctr.operation
                )
                .into());
            }
        },
        _ => return Err(format!("Unknown bctr acccount: {}", bctr.account).into()),
    };

    Ok(())
}

// Process binance.com trade history files.
//
// TODO: process_binance_com_trade_history_files started as a copy
// of process_binance_us_dist_files and they should share code!!!
pub async fn process_binance_com_trade_history_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let leading_nl = if config.progress_info { "\n" } else { "" };

    //println!("process_trade_history_files:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let in_th_file_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();

    // Verify all input files exist
    verify_input_files_exist(&in_th_file_paths)?;

    // Create csv::Writer if out_file_path exists
    let out_file_path = sc_matches.value_of("OUT_FILE");
    let mut wdr = if let Some(fp) = out_file_path {
        let writer = create_buf_writer(fp)?;
        Some(csv::Writer::from_writer(writer))
    } else {
        None
    };

    let mut bc_data = BcData::new();

    print!("Read files");
    for f in in_th_file_paths {
        println!("{leading_nl}file: {f}");
        let reader = create_buf_reader(f)?;

        // Create csv reader
        let mut rdr = csv::Reader::from_reader(reader);

        let mut first_tr = TradeRec::new();
        for (rec_index, result) in rdr.deserialize().enumerate() {
            let line_number = rec_index + 2;
            let tr: TradeRec = result?;

            if config.progress_info {
                print!(
                    "Processing {line_number} {} {}                     \r",
                    tr.operation, tr.coin
                );
            }

            // Guarantee the user_id is always the same
            if first_tr.user_id.is_empty() {
                first_tr = tr.clone();
            }
            assert_eq!(first_tr.user_id, tr.user_id);

            process_entry(&mut bc_data, &tr)?;

            bc_data.total_count += 1;
        }
    }
    println!();

    println!("Sorting");
    bc_data.tr_vec.sort();
    println!("Sorting done");

    println!("tr_vec: len: {}", bc_data.tr_vec.len());

    if let Some(w) = &mut wdr {
        println!("Writing to {}", out_file_path.unwrap());
        for dr in &bc_data.tr_vec {
            w.serialize(dr)?;
        }
        w.flush()?;
        println!("Writing done");
    }
    println!();

    if config.verbose {
        let col_1_width = 10;
        let col_2_width = 20;
        let col_3_width = 10;
        println!(
            "{:<col_1_width$} {:>col_2_width$} {:>col_3_width$}",
            "Asset", "Quantity", "Txs count",
        );

        let mut total_quantity = dec!(0);
        let mut total_transaction_count = 0usize;

        for ar in bc_data.bc_asset_rec_map.bt.values_mut() {
            total_quantity += ar.quantity;
            total_transaction_count += ar.transaction_count;

            println!(
                "{:<col_1_width$} {:>col_2_width$} {:>col_3_width$}",
                ar.asset,
                dec_to_separated_string(ar.quantity, 4),
                dec_to_separated_string(Decimal::from(ar.transaction_count), 0)
            );
        }

        assert_eq!(bc_data.total_count as usize, total_transaction_count);
        println!();
        println!(
            "Total quantity: {} ",
            dec_to_separated_string(total_quantity, 8)
        );
        println!(
            "Total txs count: {}",
            dec_to_separated_string(Decimal::from(total_transaction_count), 0)
        );
        println!(
            "Total asset count: {}",
            dec_to_separated_string(Decimal::from(bc_data.bc_asset_rec_map.bt.len()), 0)
        );
        println!(
            "Total savings principal: {}",
            dec_to_separated_string(bc_data.savings_principal, 0)
        );
    }

    // Asserts
    assert_eq!(bc_data.transfer_in_count, bc_data.transfer_out_count);

    // This may need to be approximately == 0
    assert_eq!(bc_data.savings_principal, dec!(0));

    // At most one odd redemption ?
    assert!(bc_data.savings_principal_redemption_of_ldbtc <= 1);

    Ok(())
}

// Consolidate binance.com trade history files.
//
// TODO: consolidate_binance_com_trade_history_files started as a copy
// of consolidate_binance_us_dist_files and they should share code!!!
pub async fn consolidate_binance_com_trade_history_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("consoldiate_binance_com_trade_history:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let mut bc_data = BcData::new();

    let in_th_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();
    verify_input_files_exist(&in_th_paths)?;

    // Create out_tr_path
    let out_tr_path_str = sc_matches
        .value_of("OUT_FILE")
        .unwrap_or_else(|| panic!("out-file option is missing"));
    let out_tr_path = Path::new(out_tr_path_str);

    let time_ms_offset = time_offset_days_to_time_ms_offset(sc_matches)?;

    // Determine parent path, file_stem and extension so we can construct out_token_tax_path
    let out_parent_path = if let Some(pp) = out_tr_path.parent() {
        pp
    } else {
        Path::new(".")
    };

    let out_path_file_stem = if let Some(stem) = out_tr_path.file_stem() {
        stem
    } else {
        return Err(format!("There was no file in: '{out_tr_path:?}").into());
    };

    let out_path_extension = if let Some(csv_extension) = out_tr_path.extension() {
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
    let ttx: OsString = OsString::from_str(".tt.").unwrap();
    let extx: OsString = OsString::from_str(out_path_extension.as_str()).unwrap();
    filename.push(ttx);
    filename.push(extx);
    let _out_token_tax_path = &(*out_token_tax_path.join(filename));

    let tr_writer = create_buf_writer_from_path(out_tr_path)?;

    print!("Read files");
    for f in in_th_paths {
        println!("\nfile: {f}");
        let reader = create_buf_reader(f)?;

        // DataRec reader
        let mut data_rec_reader = csv::Reader::from_reader(reader);

        // Read the TradeRecs and append them to data.tr_vec and at the
        // same add them them in to per asset map.
        let mut first_tr = TradeRec::new();
        for (rec_index, result) in data_rec_reader.deserialize().enumerate() {
            //println!("{rec_index}: {result:?}");
            let line_number = rec_index + 2;
            let mut tr: TradeRec = result?;

            if config.progress_info {
                let asset = &tr.coin;
                print!("Processing {line_number} {asset}                        \r",);
            }

            // Guarantee the user_id is always the same
            if first_tr.user_id.is_empty() {
                first_tr = tr.clone();
            }
            assert_eq!(first_tr.user_id, tr.user_id);

            // Update the time by the offset if present
            if let Some(offset) = time_ms_offset {
                tr.time += offset;
            }

            bc_data.tr_vec.push(tr.clone());
            bc_data.bc_asset_rec_map.add_tr(tr);
        }
    }
    println!();
    println!("Consolidate");

    // Loop through the asset records and consolidating each
    // and then append them to consolidated_tr_vec.
    for ar in bc_data.bc_asset_rec_map.bt.values_mut() {
        ar.consolidate_trade_recs(config)?;

        // Append the ar.consolidated_tr_vec to end of bc_data.consolidated_tr_vec
        for tr in &ar.consolidated_tr_vec {
            bc_data.bc_consolidated_tr_vec.push(tr.clone());
        }
    }

    println!("Sorting");
    bc_data
        .bc_consolidated_tr_vec
        .sort_by(ttr_cmp_no_change_no_remark);
    println!("Sorting done");

    println!(
        "consolidated_tr_vec: len: {}",
        bc_data.bc_consolidated_tr_vec.len()
    );

    // Output consolidated data as tr records and token_tax records
    println!("Writing to {out_tr_path_str}");
    write_tr_vec(tr_writer, &bc_data.bc_consolidated_tr_vec)?;
    println!("Writing done");

    if config.verbose {
        let mut total_quantity = dec!(0);
        let col_1_width = 10;
        let col_2_width = 20;
        let col_3_width = 10;

        println!(
            "{:<col_1_width$} {:>col_2_width$} {:>col_3_width$}",
            "Asset", "Quantity", "Tx count"
        );

        // Loop through the asset records printing them
        for (asset, ar) in &bc_data.bc_asset_rec_map.bt {
            assert_eq!(ar.transaction_count, ar.consolidated_tr_vec.len());
            total_quantity += ar.quantity;

            println!(
                "{:<col_1_width$} {:>col_2_width$} {:>col_3_width$}",
                asset,
                dec_to_separated_string(ar.quantity, 4),
                dec_to_separated_string(Decimal::from(ar.transaction_count), 0),
            );
        }

        println!(
            "Total quantity: {}",
            dec_to_separated_string(total_quantity, 8),
        );
        println!(
            "Consolidated from {} to {}",
            dec_to_separated_string(Decimal::from(bc_data.tr_vec.len()), 0),
            bc_data.bc_consolidated_tr_vec.len()
        );
        println!(
            "Total asset count: {}",
            dec_to_separated_string(Decimal::from(bc_data.bc_asset_rec_map.bt.len()), 0)
        );
    }

    Ok(())
}

fn write_tr_vec(
    writer: BufWriter<File>,
    tr_vec: &[TradeRec],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a data record writer
    let mut tr_writer = csv::Writer::from_writer(writer);

    // Output the data
    for dr in tr_vec {
        tr_writer.serialize(dr)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::process_token_tax::{TokenTaxRec, TypeTxs};
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_tt_trade_rec() {
        let csv = r#"User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00014357,""
123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00267651,""
123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-0.11000000,""
123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00195765,""
123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00255590,""
123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-1.50000000,""
123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00014356,""
123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00008040,""
123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-3.41000000,""
123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00465885,""
123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00112574,""
123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-0.11000000,""
123456789,2020-12-26 18:36:01,Spot,Buy,BTC,0.00445039,""
123456789,2020-12-26 18:36:01,Spot,Fee,BNB,-0.00008040,""
123456789,2020-12-26 18:36:01,Spot,Transaction Related,BNB,-3.57000000,""
"#;
        //println!("csv: {csv:?}");
        let mut bctr_a = Vec::<TradeRec>::new();

        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);
        //let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for (_idx, entry) in reader.deserialize().enumerate() {
            //println!("{_idx}: entry: {:?}", entry);
            match entry {
                Ok(bctr) => {
                    bctr_a.push(bctr);
                }
                Err(e) => panic!("Error: {e}"),
            }
        }
        //println!("bctr_a: {bctr_a:?}");

        //TODO: implement create_tt_trade_rec
        //tt_trade_rec = create_tt_trade_rec(&bctr_a)?;
    }

    #[test]
    fn test_deserialize_commission_from_csv() {
        let csv = r#"Order Type,Friend's ID(Spot),Friend's sub ID (Spot),Commission Asset,Commission Earned,Commission Earned (USDT),Commission Time,Registration Time,Referral ID
USDT-futures,42254326,"",USDT,0.00608292,0.00608300,2022-01-01 07:49:33,2021-03-31 21:58:24,bpcode
"#;
        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);
        //let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for (idx, entry) in reader.deserialize().enumerate() {
            //println!("{idx}: entry: {:?}", entry);
            match entry {
                Ok(bccr) => {
                    let bccr: CommissionRec = bccr;
                    //println!("bcr: {:?}", bccr);
                    match idx {
                        0 => {
                            assert_eq!(bccr.order_type, "USDT-futures");
                            assert_eq!(bccr.friends_id_spot, 42254326);
                            assert!(bccr.friends_sub_id_spot.is_empty());
                        }
                        _ => panic!("Unexpected idx"),
                    }
                }
                Err(e) => panic!("Error: {e}"),
            }
        }
    }

    #[test]
    fn test_commission_to_tt() {
        let bccr = CommissionRec {
            order_type: "USDT_futures".to_string(),
            friends_id_spot: 42254326,
            friends_sub_id_spot: "".to_string(),
            commission_asset: "USDT".to_string(),
            commission_earned: dec!(123),
            commission_earned_usdt: dec!(123),
            commission_time: 321,
            registration_time: 213,
            referral_id: "bpcode".to_string(),
        };
        //println!("bcr: {bccr:?}");

        let ttr = TokenTaxRec::from_commission_rec(1, &bccr);
        //println!("ttr: {ttr:?}");
        assert_eq!(ttr.type_txs, TypeTxs::Income);
        assert_eq!(ttr.buy_amount, Some(dec!(123)));
        assert_eq!(ttr.buy_currency, "USDT");
        assert_eq!(ttr.sell_amount, None);
        assert!(ttr.sell_currency.is_empty());
        assert_eq!(ttr.fee_amount, None);
        assert!(ttr.fee_currency.is_empty());
        assert_eq!(ttr.exchange, "binance.com");
        assert_eq!(ttr.group, None);
        assert_eq!(ttr.comment, "v1,1,USDT_futures,42254326,,123,213,bpcode");
        assert_eq!(ttr.time, 321);
    }

    #[test]
    fn test_deserialize_trade_from_csv() {
        let csv = r#"User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
123456789,2021-01-01 00:00:31,Spot,Commission History,DOT,0.00505120,""
"#;
        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);
        //let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for (idx, entry) in reader.deserialize().enumerate() {
            //println!("{idx}: entry: {:?}", entry);
            match entry {
                Ok(bctr) => {
                    let bctr: TradeRec = bctr;
                    //println!("bcr: {:?}", bctr);
                    match idx {
                        0 => {
                            assert_eq!(bctr.user_id, "123456789");
                            assert_eq!(bctr.time, 1609459231000);
                            assert_eq!(bctr.account, "Spot");
                            assert_eq!(bctr.operation, "Commission History");
                            assert_eq!(bctr.coin, "DOT");
                            assert_eq!(bctr.change, dec!(0.00505120));
                            assert!(bctr.remark.is_empty());
                        }
                        _ => panic!("Unexpected idx"),
                    }
                }
                Err(e) => panic!("Error: {e}"),
            }
        }
    }

    #[test]
    fn test_tr_commission_history_to_tt() {
        let bctr = TradeRec {
            user_id: "123456789".to_string(),
            time: 1609459231000,
            account: "Spot".to_string(),
            operation: "Commission History".to_string(),
            coin: "DOT".to_string(),
            change: dec!(0.00505120),
            remark: "".to_string(),
        };
        //println!("bctr: {bctr:?}");

        let ttr = TokenTaxRec::from_trade_rec(1, &bctr).unwrap().unwrap();
        //println!("ttr: {ttr:?}");
        assert!(bctr.remark.is_empty());
        assert_eq!(ttr.type_txs, TypeTxs::Income);
        assert_eq!(ttr.buy_amount, Some(dec!(0.0050512)));
        assert_eq!(ttr.buy_currency, "DOT");
        assert_eq!(ttr.sell_amount, None);
        assert!(ttr.sell_currency.is_empty());
        assert_eq!(ttr.fee_amount, None);
        assert!(ttr.fee_currency.is_empty());
        assert_eq!(ttr.exchange, "binance.com");
        assert_eq!(ttr.group, None);
        assert_eq!(ttr.comment, "v2,1,123456789,Spot,Commission History");
        assert_eq!(ttr.time, 1609459231000);
    }

    #[test]
    fn test_tr_small_assets_exchange_bnb_to_tt() {
        let csv = r#"User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
12345678,2020-05-11T21:41:11.000+00:00,Spot,Small assets exchange BNB,ANKR,-0.6765,
12345678,2020-05-11T21:41:11.000+00:00,Spot,Small assets exchange BNB,DOGE,-1.39892,
12345678,2020-05-11T21:41:11.000+00:00,Spot,Small assets exchange BNB,ENJ,-0.34686,
12345678,2020-05-11T21:41:11.000+00:00,Spot,Small assets exchange BNB,GRS,-0.14514,
12345678,2020-05-11T21:41:11.000+00:00,Spot,Small assets exchange BNB,GXS,-0.04182,
12345678,2020-05-11T21:41:11.000+00:00,Spot,Small assets exchange BNB,HBAR,-0.041,
12345678,2020-05-11T21:41:11.000+00:00,Spot,Small assets exchange BNB,LEND,-4.09221,
12345678,2020-05-11T21:41:11.000+00:00,Spot,Small assets exchange BNB,RVN,-9.219219,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,ALGO,-1.3555789,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,BAND,-0.30012,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,BAT,-0.10004,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,BCH,-0.00247263,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,BTG,-0.0008651,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,CND,-3.56167,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,COCOS,-406.392,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,COS,-8.783143,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,EDO,-0.11316,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,ETC,-0.0553295,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,FUN,-8.14137,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,HOT,-22.796,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,MBL,-36.2276,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,MCO,-0.03184306,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,NANO,-0.0031816,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,QTUM,-0.02939659,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,REN,-0.27142,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,SNT,-7.25085,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,TNT,-7.57188,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,TOMO,-0.09553,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,TRX,-0.62405378,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,VIA,-6.39108,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,XRP,-0.575435,
12345678,2020-05-11T21:41:12.000+00:00,Spot,Small assets exchange BNB,ZRX,-2.4887,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,APPC,-0.58179,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,ARPA,-1.59408,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,BNB,0.37247015,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,BUSD,-0.03237096,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,ELF,-0.02911,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,RCN,-0.23042,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,REP,-0.00166009,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,USDC,-0.23266911,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,XMR,-0.00108827,
12345678,2020-05-11T21:41:13.000+00:00,Spot,Small assets exchange BNB,ZIL,-2.71379,
"#;

        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);
        for (idx, entry) in reader.deserialize().enumerate() {
            let line_number = idx + 2;
            //println!("{idx}: entry: {:?}", entry);
            match entry {
                Ok(bctr) => {
                    let bctr: TradeRec = bctr;
                    //println!("bcr: {:?}", bctr);
                    let ttr = TokenTaxRec::from_trade_rec(line_number, &bctr)
                        .unwrap()
                        .unwrap();
                    if bctr.coin != "BNB" {
                        assert_ne!(line_number, 36);
                        assert_eq!(ttr.type_txs, TypeTxs::Spend);
                        assert_eq!("", ttr.buy_currency);
                        assert_eq!(Some(dec!(0)), ttr.buy_amount);
                        assert_eq!(bctr.coin, ttr.sell_currency);
                        assert_eq!(Some(-bctr.change), ttr.sell_amount);
                    } else {
                        assert_eq!(line_number, 36);
                        assert_eq!(ttr.type_txs, TypeTxs::Income);
                        assert_eq!(bctr.coin, ttr.buy_currency);
                        assert_eq!(Some(bctr.change), ttr.buy_amount);
                        assert_eq!("", ttr.sell_currency);
                        assert_eq!(Some(dec!(0)), ttr.sell_amount);
                    }
                }
                Err(e) => panic!("Error: {e}"),
            }
        }
    }

    #[test]
    fn test_tr_lt() {
        let mut ttr1 = TradeRec::new();
        ttr1.user_id = "1".to_owned();
        let mut ttr2 = TradeRec::new();
        ttr2.user_id = "2".to_owned();

        assert!(ttr1 < ttr2);
    }

    #[test]
    fn test_tr_le() {
        let mut ttr1 = TradeRec::new();
        ttr1.user_id = "1".to_owned();
        ttr1.change = dec!(0);
        let mut ttr2 = TradeRec::new();
        ttr2.user_id = "1".to_owned();
        ttr2.change = dec!(1);

        assert!(ttr1 <= ttr2);
        //println!("{:?}", ttr_cmp_no_change_no_remark(&ttr1, &ttr2));
        assert_eq!(
            ttr_cmp_no_change_no_remark(&ttr1, &ttr2),
            core::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_tr_eq() {
        let mut ttr1 = TradeRec::new();
        ttr1.user_id = "1".to_owned();
        ttr1.remark = "2".to_owned();
        let mut ttr2 = TradeRec::new();
        ttr2.user_id = "1".to_owned();
        ttr1.change = dec!(1);
        ttr1.remark = "3".to_owned();

        assert!(ttr1 != ttr2);
        assert_eq!(
            ttr_cmp_no_change_no_remark(&ttr1, &ttr2),
            core::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_tr_ge() {
        let mut ttr1 = TradeRec::new();
        ttr1.user_id = "2".to_owned();
        let mut ttr2 = TradeRec::new();
        ttr2.user_id = "1".to_owned();

        assert!(ttr1 >= ttr2);
    }

    #[test]
    fn test_tr_gt() {
        let mut ttr1 = TradeRec::new();
        ttr1.user_id = "2".to_owned();
        let mut ttr2 = TradeRec::new();
        ttr2.user_id = "1".to_owned();

        assert!(ttr1 > ttr2);
    }

    #[test]
    fn test_tr_ne() {
        let mut ttr1 = TradeRec::new();
        ttr1.user_id = "2".to_owned();
        let mut ttr2 = TradeRec::new();
        ttr2.user_id = "1".to_owned();

        assert!(ttr1 != ttr2);
    }
}