use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use clap::ArgMatches;
use rust_decimal_macros::dec;
use taxbitrec::{TaxBitRec, TaxBitRecType};
use tokentaxrec::{TokenTaxRec, TokenTaxRecType};

use crate::{
    arg_matches::time_offset_days_to_time_ms_offset,
    common::{create_buf_writer, create_buf_writer_from_path, verify_input_files_exist},
    configuration::Configuration,
};

fn tbr_from_token_tax_rec(ttr: &TokenTaxRec) -> TaxBitRec {
    // TODO: TaxBit "source/destination" fields must be valid
    // TODO: for binance.com and binance.us that is "Binance"
    // TODO: but what about other exchanges, maybe make this
    // TODO: should be a command line option as the user may know?
    let exchange = match ttr.exchange.as_str() {
        "binance.com" => "Binance".to_owned(),
        "binance.us" => "BinanceUS".to_owned(),
        _ => "".to_owned(),
    };

    match ttr.type_txs {
        TokenTaxRecType::Mining | TokenTaxRecType::Income => {
            let mut tbr = TaxBitRec::new();
            tbr.time = ttr.time;
            tbr.type_txs = TaxBitRecType::Income;
            tbr.received_quantity = ttr.buy_amount;
            tbr.received_currency = ttr.buy_currency.clone();
            tbr.receiving_destination = exchange;

            tbr
        }
        TokenTaxRecType::Trade => {
            let mut tbr = TaxBitRec::new();

            if ttr.sell_currency.as_str() == "USD" {
                tbr.type_txs = TaxBitRecType::Buy;
                tbr.sent_quantity = ttr.sell_amount;
                tbr.sent_currency = ttr.sell_currency.clone();
                tbr.received_quantity = ttr.buy_amount;
                tbr.received_currency = ttr.buy_currency.clone();
            } else if ttr.buy_currency.as_str() == "USD" {
                tbr.type_txs = TaxBitRecType::Sale;
                tbr.sent_quantity = ttr.buy_amount;
                tbr.sent_currency = ttr.buy_currency.clone();
                tbr.received_quantity = ttr.sell_amount;
                tbr.received_currency = ttr.sell_currency.clone();
            } else {
                tbr.type_txs = TaxBitRecType::Trade;
                tbr.sent_quantity = ttr.sell_amount;
                tbr.sent_currency = ttr.sell_currency.clone();
                tbr.received_quantity = ttr.buy_amount;
                tbr.received_currency = ttr.buy_currency.clone();
            }
            tbr.time = ttr.time;
            tbr.sending_source = exchange.clone();
            tbr.receiving_destination = exchange;
            tbr.fee_quantity = ttr.fee_amount;
            tbr.fee_currency = ttr.fee_currency.clone();

            // If fee_quantity is zero remove otherwise it is silently rejected by TaxBit
            if let Some(q) = tbr.fee_quantity {
                if q == dec!(0) {
                    tbr.fee_quantity = None;
                    tbr.fee_currency = "".to_owned();
                }
            }

            tbr
        }
        TokenTaxRecType::Deposit => {
            // How to receive cost basis of crypto
            let mut tbr = TaxBitRec::new();
            tbr.time = ttr.time;
            tbr.type_txs = TaxBitRecType::TransferIn;
            tbr.received_quantity = ttr.buy_amount;
            tbr.received_currency = ttr.buy_currency.clone();
            tbr.receiving_destination = exchange;

            tbr
        }
        TokenTaxRecType::Withdrawal => {
            // How to send cost basis of crypto
            let mut tbr = TaxBitRec::new();
            tbr.time = ttr.time;
            tbr.type_txs = TaxBitRecType::TransferOut;
            tbr.sent_quantity = ttr.sell_amount;
            tbr.sent_currency = ttr.sell_currency.clone();
            tbr.sending_source = exchange;

            tbr
        }
        TokenTaxRecType::Spend => {
            let mut tbr = TaxBitRec::new();
            tbr.time = ttr.time;
            tbr.type_txs = TaxBitRecType::Expense;
            tbr.sent_quantity = ttr.sell_amount;
            tbr.sent_currency = ttr.sell_currency.clone();
            tbr.sending_source = exchange;

            tbr
        }
        TokenTaxRecType::Lost => {
            let mut tbr = TaxBitRec::new();
            tbr.time = ttr.time;
            tbr.type_txs = TaxBitRecType::Expense;
            tbr.sent_quantity = ttr.sell_amount;
            tbr.sent_currency = ttr.sell_currency.clone();
            tbr.sending_source = exchange;

            tbr
        }
        TokenTaxRecType::Stolen => {
            let mut tbr = TaxBitRec::new();
            tbr.time = ttr.time;
            tbr.type_txs = TaxBitRecType::Expense;
            tbr.sent_quantity = ttr.sell_amount;
            tbr.sent_currency = ttr.sell_currency.clone();
            tbr.sending_source = exchange;

            tbr
        }
        TokenTaxRecType::Gift => {
            let mut tbr = TaxBitRec::new();
            tbr.time = ttr.time;
            tbr.type_txs = TaxBitRecType::TransferOut;
            tbr.sent_quantity = ttr.sell_amount;
            tbr.sent_currency = ttr.sell_currency.clone();
            tbr.sending_source = exchange;

            tbr
        }
        TokenTaxRecType::Unknown => {
            panic!("Unknown Token Type")
        }
    }
}

fn create_tbr_vec_from_ttr_vec(ttr_vec: &[TokenTaxRec]) -> Vec<TaxBitRec> {
    let mut tbr_vec = Vec::<TaxBitRec>::new();
    for ttr in ttr_vec {
        let tbr = tbr_from_token_tax_rec(ttr);
        tbr_vec.push(tbr);
    }

    tbr_vec
}

// Write TaxBit record from tbr_vec
//
// Returns number of records written
fn write_tbr_vec(
    writer: BufWriter<File>,
    tbr_vec: &[TaxBitRec],
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut cvs_writer = csv::Writer::from_writer(writer);

    let len = tbr_vec.len();

    for tbr in tbr_vec {
        //println!("{tbr}");
        cvs_writer.serialize(tbr)?;
    }
    cvs_writer.flush()?;

    Ok(len)
}

pub async fn tb_file_from_token_tax_file(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("tb_file_from_token_tax_file:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let leading_nl = if config.progress_info { "\n" } else { "" };

    let in_file_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();
    verify_input_files_exist(&in_file_paths)?;

    // Create out_dist_path
    let out_file_path_str = sc_matches
        .value_of("OUT_FILE")
        .unwrap_or_else(|| panic!("out-file option is missing"));
    let out_file_path = Path::new(out_file_path_str);

    let time_ms_offset = time_offset_days_to_time_ms_offset(sc_matches)?;

    let rec_writer = create_buf_writer_from_path(out_file_path)?;

    let mut ttr_vec = Vec::<TokenTaxRec>::new();

    println!("Read files:");
    for (fidx, f) in in_file_paths.into_iter().enumerate() {
        println!("file {fidx}: {f}");
        let in_file = if let Ok(in_f) = File::open(f) {
            in_f
        } else {
            return Err(format!("Unable to open {f}").into());
        };
        let reader = BufReader::new(in_file);

        // record reader
        let mut rec_reader = csv::Reader::from_reader(reader);

        for (rec_idx, result) in rec_reader.deserialize().enumerate() {
            //println!("{rec_index}: {result:?}");
            let line_number = rec_idx + 2;
            let mut ttr: TokenTaxRec = result?;

            if config.progress_info {
                let asset = ttr.get_asset();
                print!("Processing {line_number} {asset}                        \r");
            }

            if let Some(offset) = time_ms_offset {
                ttr.time += offset;
            }

            ttr_vec.push(ttr.clone());
        }
        println!("{leading_nl}");
    }

    println!("Sorting");
    ttr_vec.sort();
    println!("Sorting done");

    let tbr_vec = create_tbr_vec_from_ttr_vec(&ttr_vec);

    println!("Writing TaxBit records to {out_file_path_str}");
    let written = write_tbr_vec(rec_writer, &tbr_vec)?;
    println!("Writing TaxBit records: Done; records written: {written}");

    println!();
    println!("Done");

    Ok(())
}

pub async fn process_tax_bit_files(
    config: &Configuration,
    sc_matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("tb_file_from_token_tax_file:+ config: {config:?}\n\nsc_matches: {sc_matches:?}\n");

    let leading_nl = if config.progress_info { "\n" } else { "" };

    // Create list of input files
    let in_file_paths: Vec<&str> = sc_matches
        .values_of("IN_FILES")
        .expect("files option is missing")
        .collect();
    verify_input_files_exist(&in_file_paths)?;

    // Create csv::Writer if out_file_path exists
    let out_file_path = sc_matches.value_of("OUT_FILE");
    let mut buf_writer = if let Some(fp) = out_file_path {
        let writer = create_buf_writer(fp)?;
        Some(csv::Writer::from_writer(writer))
    } else {
        None
    };

    let mut tbr_vec = Vec::<TaxBitRec>::new();

    println!("Read files:");
    for (fidx, f) in in_file_paths.into_iter().enumerate() {
        println!("file {fidx}: {f}");
        let in_file = if let Ok(in_f) = File::open(f) {
            in_f
        } else {
            return Err(format!("Unable to open {f}").into());
        };
        let reader = BufReader::new(in_file);

        // record reader
        let mut rec_reader = csv::Reader::from_reader(reader);

        for (rec_idx, result) in rec_reader.deserialize().enumerate() {
            //println!("{rec_index}: {result:?}");
            let line_number = rec_idx + 2;
            let ttr: TaxBitRec = result?;

            if config.progress_info {
                let asset = ttr.get_asset();
                print!("Processing {line_number} {asset}                        \r");
            }

            tbr_vec.push(ttr.clone());
        }
        println!("{leading_nl}");
    }

    println!("Sorting");
    tbr_vec.sort();
    println!("Sorting done");

    if let Some(_w) = &mut buf_writer {
        println!(
            "Writing to {}, NOT IMPLEMENTED YET!",
            out_file_path.unwrap()
        );
        //for ttr in &ttr_vec {
        //    w.serialize(ttr)?;
        //}
        //w.flush()?;
        //println!("Writing done");
    } else {
        //println!();
        //println!();
    }
    println!();

    // Print contents for now
    tbr_vec.iter().for_each(|ttr| println!("{ttr}"));

    println!();
    println!("Done");

    Ok(())
}

#[cfg(test)]
mod test {

    use taxbitrec::TaxBitRecType;
    use time_ms_conversions::{dt_str_to_utc_time_ms, TzMassaging};

    use super::*;
    //use rust_decimal::prelude::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_1() {
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
            let ttr: TokenTaxRec = entry.unwrap();
            println!("{idx}: ttr: {ttr:?}");
            let tbr = tbr_from_token_tax_rec(&ttr);
            println!("{idx}: tbr: {tbr:?}");
        }
    }

    #[test]
    fn test_from_token_tax_rec() {
        let ttr_csv = r#"
Type,BuyAmount,BuyCurrency,SellAmount,SellCurrency,FeeAmount,FeeCurrency,Exchange,Group,Comment,Date
Deposit,5125,USD,,,,,binance.us,,"v0,2,1,1,Deposit,USD Deposit",2019-08-01T00:00:00.000+00:00
"#;
        //Income,4.4238,BTT,,,,,binance.com,,"v3,0,2,36757189,Spot,Commission History",2019-05-21T21:39:50.000+00:00
        //Income,0.00014924,BNB,,,,,binance.com,,"v3,1,190834,36757189,Spot,Commission History",2020-07-07T14:34:40.000+00:00
        //Deposit,20.02047168,ETH,,,,,binance.com,,"v3,1,190835,36757189,Spot,Deposit",2020-07-07T14:41:05.000+00:00
        //Trade,2.5087278,USD,0.09847,ETH,0.00105017,BNB,binance.com,,"v3,1,190856,36757189,Spot,Buy",2020-07-07T14:43:42.000+00:00
        //Trade,4.9954623,USD,0.20937,ETH,0.00223338,BNB,binance.com,,"v3,1,190851,36757189,Spot,Buy",2020-07-07T14:43:42.000+00:00
        //Spend,,,2.1,BNB,,,binance.com,,"v3,1,298462,36757189,Spot,Small assets exchange BNB",2020-08-30T16:32:45.000+00:00
        //Spend,,,3.7834,ETH,,,binance.com,,"v3,1,298463,36757189,Spot,Small assets exchange BNB",2020-08-30T16:32:45.000+00:00
        //Withdrawal,,,1.89339667,ETH,,,binance.com,,"v3,1,168433,36757189,Spot,Withdraw",2020-06-15T02:09:23.000+00:00
        //"#;

        let rdr = ttr_csv.as_bytes();
        let mut reader = csv::Reader::from_reader(rdr);
        for (idx, entry) in reader.deserialize().enumerate() {
            let ttr: TokenTaxRec = entry.unwrap();
            println!("{idx}: ttr: {ttr:?}");
            let tbr = tbr_from_token_tax_rec(&ttr);
            println!("{idx}: tbr: {tbr:?}");
            match idx {
                0 => {
                    // Deposit,5125,USD,,,,,binance.us,,,1970-01-01 00:00:00
                    assert_eq!(
                        tbr.time,
                        dt_str_to_utc_time_ms("2019-08-01T00:00:00.000+00:00", TzMassaging::HasTz)
                            .unwrap()
                    );
                    assert_eq!(tbr.type_txs, TaxBitRecType::TransferIn);
                    assert_eq!(tbr.received_quantity, Some(dec!(5125)));
                    assert_eq!(tbr.received_currency, "USD");
                    assert_eq!(tbr.receiving_destination, "BinanceUS");
                    assert_eq!(tbr.sent_quantity, None);
                    assert_eq!(tbr.sent_currency, "");
                    assert_eq!(tbr.sending_source, "");
                    assert_eq!(tbr.fee_quantity, None);
                    assert_eq!(tbr.fee_currency, "");
                }
                //1 => {
                //    // Trade,1,ETH,3123.00,USD,0.00124,BNB,binance.us,,,1970-01-01 00:00:00
                //    assert_eq!(tbr.type_txs, TaxBitRecType::Trade);
                //    assert_eq!(tbr.buy_amount, Some(dec!(1)));
                //    assert_eq!(tbr.buy_currency, "ETH");
                //    assert_eq!(tbr.sell_amount, Some(dec!(3123)));
                //    assert_eq!(tbr.sell_currency, "USD");
                //    assert_eq!(tbr.fee_amount, Some(dec!(0.00124)));
                //    assert_eq!(tbr.fee_currency, "BNB");
                //    assert_eq!(tbr.exchange, "binance.us");
                //    assert_eq!(tbr.group, None);
                //    assert_eq!(tbr.comment, "");
                //    assert_eq!(tbr.time, 0);
                //}
                //2 => {
                //    // Trade,1,ETH,312.00,USD,0.00124,BNB,binance.us,margin,,1970-01-01 00:00:00
                //    assert_eq!(tbr.type_txs, TaxBitRecType::Trade);
                //    assert_eq!(tbr.buy_amount, Some(dec!(1)));
                //    assert_eq!(tbr.buy_currency, "ETH");
                //    assert_eq!(tbr.sell_amount, Some(dec!(312)));
                //    assert_eq!(tbr.sell_currency, "USD");
                //    assert_eq!(tbr.fee_amount, Some(dec!(0.00124)));
                //    assert_eq!(tbr.fee_currency, "BNB");
                //    assert_eq!(tbr.exchange, "binance.us");
                //    assert_eq!(tbr.group, Some(GroupType::Margin));
                //    assert_eq!(tbr.comment, "");
                //    assert_eq!(tbr.time, 0);
                //}
                //3 => {
                //    // Income,0.001,BNB,,,,,binance.us,,\"Referral Commission\",1970-01-01 00:00:00
                //    assert_eq!(tbr.type_txs, TaxBitRecType::Income);
                //    assert_eq!(tbr.buy_amount, Some(dec!(0.001)));
                //    assert_eq!(tbr.buy_currency, "BNB");
                //    assert_eq!(tbr.sell_amount, None);
                //    assert_eq!(tbr.sell_currency, "");
                //    assert_eq!(tbr.fee_amount, None);
                //    assert_eq!(tbr.fee_currency, "");
                //    assert_eq!(tbr.exchange, "binance.us");
                //    assert_eq!(tbr.group, None);
                //    assert_eq!(tbr.comment, "Referral Commission");
                //    assert_eq!(tbr.time, 0);
                //}
                //4 => {
                //    // Withdrawal,,,100,USD,,,some bank,,\"AccountId: 123456\",1970-01-01 00:00:00
                //    assert_eq!(tbr.type_txs, TaxBitRecType::Withdrawal);
                //    assert_eq!(tbr.buy_amount, None);
                //    assert_eq!(tbr.buy_currency, "");
                //    assert_eq!(tbr.sell_amount, Some(dec!(100)));
                //    assert_eq!(tbr.sell_currency, "USD");
                //    assert_eq!(tbr.fee_amount, None);
                //    assert_eq!(tbr.fee_currency, "");
                //    assert_eq!(tbr.exchange, "some bank");
                //    assert_eq!(tbr.group, None);
                //    assert_eq!(tbr.comment, "AccountId: 123456");
                //    assert_eq!(tbr.time, 0);
                //}
                //5 => {
                //    // Spend,,,100,USD,0.01,USD,,,\"Gift for wife\",1970-01-01 00:00:00
                //    assert_eq!(tbr.type_txs, TaxBitRecType::Spend);
                //    assert_eq!(tbr.buy_amount, None);
                //    assert_eq!(tbr.buy_currency, "");
                //    assert_eq!(tbr.sell_amount, Some(dec!(100)));
                //    assert_eq!(tbr.sell_currency, "USD");
                //    assert_eq!(tbr.fee_amount, Some(dec!(0.01)));
                //    assert_eq!(tbr.fee_currency, "USD");
                //    assert_eq!(tbr.exchange, "");
                //    assert_eq!(tbr.group, None);
                //    assert_eq!(tbr.comment, "Gift for wife");
                //    assert_eq!(tbr.time, 0);
                //}
                //6 => {
                //    // Lost,,,1,ETH,,,,,\"Wallet lost\",1970-01-01 00:00:00
                //    assert_eq!(tbr.type_txs, TaxBitRecType::Lost);
                //    assert_eq!(tbr.buy_amount, None);
                //    assert_eq!(tbr.buy_currency, "");
                //    assert_eq!(tbr.sell_amount, Some(dec!(1)));
                //    assert_eq!(tbr.sell_currency, "ETH");
                //    assert_eq!(tbr.fee_amount, None);
                //    assert_eq!(tbr.fee_currency, "");
                //    assert_eq!(tbr.exchange, "");
                //    assert_eq!(tbr.group, None);
                //    assert_eq!(tbr.comment, "Wallet lost");
                //    assert_eq!(tbr.time, 0);
                //}
                //7 => {
                //    // Stolen,,,1,USD,,,,,\"Wallet hacked\",1970-01-01 00:00:00
                //    assert_eq!(tbr.type_txs, TaxBitRecType::Stolen);
                //    assert_eq!(tbr.buy_amount, None);
                //    assert_eq!(tbr.buy_currency, "");
                //    assert_eq!(tbr.sell_amount, Some(dec!(1)));
                //    assert_eq!(tbr.sell_currency, "USD");
                //    assert_eq!(tbr.fee_amount, None);
                //    assert_eq!(tbr.fee_currency, "");
                //    assert_eq!(tbr.exchange, "");
                //    assert_eq!(tbr.group, None);
                //    assert_eq!(tbr.comment, "Wallet hacked");
                //    assert_eq!(tbr.time, 0);
                //}
                //8 => {
                //    // Mining,0.000002,ETH,,,,,binance.us,,\"ETH2 validator reward\",1970-01-01 00:00:00
                //    assert_eq!(tbr.type_txs, TaxBitRecType::Mining);
                //    assert_eq!(tbr.buy_amount, Some(dec!(0.000002)));
                //    assert_eq!(tbr.buy_currency, "ETH");
                //    assert_eq!(tbr.sell_amount, None);
                //    assert_eq!(tbr.sell_currency, "");
                //    assert_eq!(tbr.fee_amount, None);
                //    assert_eq!(tbr.fee_currency, "");
                //    assert_eq!(tbr.exchange, "binance.us");
                //    assert_eq!(tbr.group, None);
                //    assert_eq!(tbr.comment, "ETH2 validator reward");
                //    assert_eq!(tbr.time, 0);
                //}
                //9 => {
                //    // Gift,,,100,USD,,,,,\"Gift to friend\",1970-01-01 00:00:00
                //    assert_eq!(tbr.type_txs, TaxBitRecType::Gift);
                //    assert_eq!(tbr.buy_amount, None);
                //    assert_eq!(tbr.buy_currency, "");
                //    assert_eq!(tbr.sell_amount, Some(dec!(100)));
                //    assert_eq!(tbr.sell_currency, "USD");
                //    assert_eq!(tbr.fee_amount, None);
                //    assert_eq!(tbr.fee_currency, "");
                //    assert_eq!(tbr.exchange, "");
                //    assert_eq!(tbr.group, None);
                //    assert_eq!(tbr.comment, "Gift to friend");
                //    assert_eq!(tbr.time, 0);
                //}
                _ => panic!("Unexpected idx"),
            }
        }
    }

    //    #[test]
    //    fn test_deserialize_token_tax_rec_to_serialized_tax_bit_rec() {
    //        let ttr_csv = r#"Type,BuyAmount,BuyCurrency,SellAmount,SellCurrency,FeeAmount,FeeCurrency,Exchange,Group,Comment,Date
    //Deposit,5125,USD,,,,,binance.us,,"v4,0,2,1,1,Deposit,USD Deposit",2019-08-01T00:00:00.000+00:00
    //Trade,0.00558,BTC,44.959176,USD,0,BTC,binance.us,,"v4,0,3,367670,125143,Spot Trading,Buy",2019-09-28T15:35:02.000+00:00
    //Income,0.0000003,BTC,,,,,binance.us,,"v4,0,4,5442858,17929593,Distribution,Referral Commission",2020-03-02T07:32:05.000+00:00
    //Deposit,45.25785064909286,ETH,,,,,binance.us,,"v4,0,5,17916393,17916393,Deposit,Crypto Deposit",2020-03-23T04:08:20.000+00:00
    //Trade,0.427854,BTC,20.374,ETH,0.16893668,BNB,binance.us,,"v4,0,6,5988456,17916714,Spot Trading,Sell",2020-03-23T04:10:29.000+00:00
    //Trade,0.61,BNB,11.90903,USD,0.0004575,BNB,binance.us,,"v4,0,7,26988333,32890969,Spot Trading,Buy",2020-07-26T15:50:02.000+00:00
    //Withdrawal,,,23.99180186,ETH,0.005,ETH,binance.us,,"v4,0,8,38078398,38078398,Withdrawal,Crypto Withdrawal",2020-08-16T23:54:01.000+00:00
    //Trade,27.4684,USD,0.1,BNB,0.14,USD,binance.us,,"v4,0,9,cf9257c74ea243da9f3e64847ad0233b,171875688,Quick Buy,Buy",2021-03-18T03:49:18.000+00:00
    //Trade,590.5686,USD,0.010946,BTC,2.97,USD,binance.us,,"v4,0,10,87d5c693897c4a0a8a35534782f6c471,179163493,Quick Sell,Sell",2021-03-22T22:33:06.147+00:00
    //"#;
    //        let result_tbr_csv = r#"User_Id,Time,Category,Operation,Order_Id,Transaction_Id,Primary_Asset,Realized_Amount_For_Primary_Asset,Realized_Amount_For_Primary_Asset_In_USD_Value,Base_Asset,Realized_Amount_For_Base_Asset,Realized_Amount_For_Base_Asset_In_USD_Value,Quote_Asset,Realized_Amount_For_Quote_Asset,Realized_Amount_For_Quote_Asset_In_USD_Value,Fee_Asset,Realized_Amount_For_Fee_Asset,Realized_Amount_For_Fee_Asset_In_USD_Value,Payment_Method,Withdrawal_Method,Additional_Note
    //12345678,2019-08-01T00:00:00.000+00:00,Deposit,USD Deposit,1,1,USD,5125,5125,,,,,,,,,,Debit,,
    //12345678,2019-09-28T15:35:02.000+00:00,Spot Trading,Buy,367670,125143,,,,BTC,0.00558,46.012234,USD,44.959176,44.959176,BTC,0,0,Wallet,,
    //12345678,2020-03-02T07:32:05.000+00:00,Distribution,Referral Commission,5442858,17929593,BTC,0.0000003,0.002661,,,,,,,,,,Wallet,,
    //12345678,2020-03-23T04:08:20.000+00:00,Deposit,Crypto Deposit,17916393,17916393,ETH,45.25785064909286,6105.809587,,,,,,,,,,Wallet,,
    //12345678,2020-03-23T04:10:29.000+00:00,Spot Trading,Sell,5988456,17916714,,,,ETH,20.374,2748.689183,BTC,0.427854,2745.245935,BNB,0.16893668,2.047513,Wallet,,
    //12345678,2020-07-26T15:50:02.000+00:00,Spot Trading,Buy,26988333,32890969,,,,BNB,0.61,11.907825,USD,11.90903,11.90903,BNB,0.0004575,0.008931,Wallet,,
    //12345678,2020-08-16T23:54:01.000+00:00,Withdrawal,Crypto Withdrawal,38078398,38078398,ETH,23.99180186,10407.403729,,,,,,,ETH,0.005,2.16895,Wallet,Wallet,
    //12345678,2021-03-18T03:49:18.000+00:00,Quick Buy,Buy,cf9257c74ea243da9f3e64847ad0233b,171875688,,,,USD,27.4684,27.4684,BNB,0.1,26.170481,USD,0.14,0.14,Wallet,,
    //12345678,2021-03-22T22:33:06.147+00:00,Quick Sell,Sell,87d5c693897c4a0a8a35534782f6c471,179163493,,,,BTC,0.010946,596.876028,USD,590.5686,590.5686,USD,2.97,2.97,Wallet,,
    //"#;
    //
    //        let rdr = ttr_csv.as_bytes();
    //        let mut reader = csv::Reader::from_reader(rdr);
    //
    //        let mut wtr = csv::Writer::from_writer(vec![]);
    //        for (idx, entry) in reader.deserialize().enumerate() {
    //            println!("{idx}: entry: {:?}", entry);
    //            let ttr: TokenTaxRec = entry.unwrap();
    //            //dbg!(ttr);
    //
    //            let tbr = TaxBitRec::_from_token_tax_rec(&ttr);
    //            //dbg!(&tbr);
    //
    //            wtr.serialize(&tbr).expect("Error serializing");
    //        }
    //
    //        let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    //        dbg!(&data);
    //
    //        assert_eq!(data, result_tbr_csv);
    //    }
}
