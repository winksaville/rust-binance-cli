use crate::{
    binance_us_processing::DistRec,
    de_string_to_utc_time_ms::{de_string_to_utc_time_ms_condaddtzutc, se_time_ms_to_utc_string},
};
use lazy_static::lazy_static;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum TypeTxs {
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum GroupType {
    #[serde(rename = "margin")]
    Margin,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TokenTaxRec {
    #[serde(rename = "Type")]
    type_txs: TypeTxs,

    buy_amount: Option<Decimal>,
    buy_currency: String,
    sell_amount: Option<Decimal>,
    sell_currency: String,
    fee_amount: Option<Decimal>,
    fee_currency: String,
    exchange: String,
    group: Option<GroupType>,
    comment: String,

    #[serde(rename = "Date")]
    #[serde(deserialize_with = "de_string_to_utc_time_ms_condaddtzutc")]
    #[serde(serialize_with = "se_time_ms_to_utc_string")]
    time: i64,
}

lazy_static! {
    pub static ref VER: String = "v0".to_string();
}

impl TokenTaxRec {
    pub fn from_dist_rec(line_number: usize, dr: &DistRec) -> TokenTaxRec {
        let ver = VER.as_str();
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
            comment: format!(
                "{ver},{line_number},{},{},{},{}",
                dr.order_id, dr.transaction_id, dr.category, dr.operation
            ),
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

    #[test]
    fn test_dist_rec_to_serialized_token_tax_rec() {
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
