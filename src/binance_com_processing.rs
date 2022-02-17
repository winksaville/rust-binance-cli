//! This file processes binance.com commission files.
//!
use crate::{
    common::{create_buf_reader, dec_to_separated_string, verify_input_files_exist},
    configuration::Configuration,
    de_string_to_utc_time_ms::{de_string_to_utc_time_ms_condaddtzutc, se_time_ms_to_utc_string},
    token_tax::{TokenTaxRec, TypeTxs},
    token_tax_comment_vers::{TT_CMT_VER1, TT_CMT_VER2},
};
use clap::ArgMatches;
use rust_decimal::Decimal;
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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
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

        match bctr.account.as_str() {
            "Coin-Futures" => match bctr.operation.as_str() {
                "Referrer rebates" => {
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                    //   123456789,2021-01-01 02:33:56,Coin-Futures,Referrer rebates,BNB,0.00066774,""
                }
                "transfer_in" | "transfer_out" => {
                    // Ignore
                    return Ok(None);
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
                }
                "transfer_in" | "transer_out" => {
                    // Ignore
                    return Ok(None);
                }
                _ => {
                    return Err(format!(
                        "Unknown bctr acccount: {} operation: {}",
                        bctr.account, bctr.operation
                    )
                    .into())
                }
            },
            "Pool" => match bctr.operation.as_str() {
                "Pool Distribution" => {
                    ttr.type_txs = TypeTxs::Income;
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
                }
                "Commission History" => {
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                    //   123456789,2021-01-01 00:00:31,Spot,Commission History,DOT,0.00505120,""
                }
                "Commission Rebate" => {
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                    //   123456789,2021-06-23 03:54:55,Spot,Commission Rebate,BTC,2.2E-7,""
                }
                "Deposit" => {
                    //??
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                }
                "Distribution" => {
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                    //  123456789,2020-01-03 05:58:34,Spot,Distribution,ALGO,0.08716713,""
                }
                "ETH 2.0 Staking Rewards" => {
                    //??
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                }
                "Savings Interest" => {
                    //??
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                }
                "Savings Principal redemption" => {
                    //??
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                }
                "Savings purchase" => {
                    //??
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                }
                "Small assets exchange BNB" => {
                    //??
                    // Income nothing more to do:
                    //   User_ID,UTC_Time,Account,Operation,Coin,Change,Remark
                }
                "transfer_in" | "transfer_out" => return Ok(None),
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
                }
                _ => {
                    return Err(format!(
                        "Unknown bctr acccount: {} operation: {}",
                        bctr.account, bctr.operation
                    )
                    .into());
                }
            },
            _ => panic!("Unknown bctr.account: {}", bctr.account),
            _ => return Err(format!("Unknown bctr acccount: {}", bctr.account).into()),
        };

        Ok(Some(ttr))
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
    total_count: u64,
}

impl BcData {
    fn new() -> BcData {
        BcData {
            tr_vec: Vec::new(),
            total_count: 0u64,
        }
    }
}

pub async fn process_binance_com_trade_history_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("process_trade_history_files:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let in_dist_file_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();

    // Verify all input files exist
    verify_input_files_exist(&in_dist_file_paths)?;

    let mut data = BcData::new();

    for f in in_dist_file_paths {
        let reader = create_buf_reader(f)?;

        // Create csv reader
        let mut rdr = csv::Reader::from_reader(reader);

        for (rec_index, result) in rdr.deserialize().enumerate() {
            let line_number = rec_index + 2;
            let tr: TradeRec = result?;

            data.tr_vec.push(tr.clone());

            if config.verbose {
                print!(
                    "Processing {line_number} {} {}                     \r",
                    tr.operation, tr.coin
                );
            }

            data.total_count += 1;
        }
    }

    println!(
        "Total count: {}",
        dec_to_separated_string(Decimal::from(data.total_count), 0)
    );
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::token_tax::{TokenTaxRec, TypeTxs};
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
        println!("csv: {csv:?}");
        let mut bctr_a = Vec::<TradeRec>::new();

        let rdr = csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);
        //let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for (idx, entry) in reader.deserialize().enumerate() {
            println!("{idx}: entry: {:?}", entry);
            match entry {
                Ok(bctr) => {
                    bctr_a.push(bctr);
                }
                Err(e) => panic!("Error: {e}"),
            }
        }
        println!("bctr_a: {bctr_a:?}");

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
            println!("{idx}: entry: {:?}", entry);
            match entry {
                Ok(bccr) => {
                    let bccr: CommissionRec = bccr;
                    println!("bcr: {:?}", bccr);
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
        println!("bcr: {bccr:?}");

        let ttr = TokenTaxRec::from_commission_rec(1, &bccr);
        println!("ttr: {ttr:?}");
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
            println!("{idx}: entry: {:?}", entry);
            match entry {
                Ok(bctr) => {
                    let bctr: TradeRec = bctr;
                    println!("bcr: {:?}", bctr);
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
    fn test_trade_to_tt() {
        let bctr = TradeRec {
            user_id: "123456789".to_string(),
            time: 1609459231000,
            account: "Spot".to_string(),
            operation: "Commission History".to_string(),
            coin: "DOT".to_string(),
            change: dec!(0.00505120),
            remark: "".to_string(),
            //usd_value: dec!(0),
        };
        println!("bctr: {bctr:?}");

        let ttr = TokenTaxRec::from_trade_rec(1, &bctr).unwrap().unwrap();
        println!("ttr: {ttr:?}");
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
}
