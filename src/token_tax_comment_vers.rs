//! TokenTax Comment versions
//! This is shared across modules as these must
//! be unique globally

use lazy_static::lazy_static;

use crate::{
    process_binance_com::{CommissionRec, TradeRec},
    process_binance_us::DistRec,
};

lazy_static! {
    //#[deprecated(since="0.3.16", note = "use version 4")]
    //pub static ref TT_CMT_VER0_CSV_HEADER: String = "version,LineNumber,OrderId,TransactionId,Category,Operation".to_owned();
    //#[deprecated(since="0.3.16", note = "use version 4")]
    //pub static ref TT_CMT_VER0: String = "v0".to_owned();

    pub static ref TT_CMT_VER1_CSV_HEADER: String = "version,FileIdx,LineNumber,OrderType,FriendsIdSpot,CommissionEarnedUsdt,RegistrationTime,ReferralId".to_owned();
    pub static ref TT_CMT_VER1: String = "v1".to_owned();

    //#[deprecated(since="0.3.16", note = "use version 3")]
    //pub static ref TT_CMT_VER2_CSV_HEADER: String = "version,LineNumber,UserId,Account,Operation".to_owned();
    //#[deprecated(since="0.3.16", note = "use version 3")]
    //pub static ref TT_CMT_VER2: String = "v2".to_owned();

    pub static ref TT_CMT_VER3_CSV_HEADER: String = "version,FileIdx,LineNumber,UserId,Account,Operation".to_owned();
    pub static ref TT_CMT_VER3: String = "v3".to_owned();

    pub static ref TT_CMT_VER4_CSV_HEADER: String = "version,FileIdx,LineNumber,OrderId,TransactionId,Category,Operation".to_owned();
    pub static ref TT_CMT_VER4: String = "v4".to_owned();
}

pub fn create_tt_cmt_ver1_string(bccr: &CommissionRec) -> String {
    let ver = TT_CMT_VER1.as_str();
    format!(
        "{ver},{},{},{},{},{},{},{},{}",
        bccr.file_idx,
        bccr.line_number,
        bccr.order_type,
        bccr.friends_id_spot,
        bccr.friends_sub_id_spot,
        bccr.commission_earned_usdt,
        bccr.registration_time,
        bccr.referral_id
    )
}

pub fn create_tt_cmt_ver3_string(bctr: &TradeRec) -> String {
    let ver = TT_CMT_VER3.as_str();
    format!(
        "{ver},{},{},{},{},{}",
        bctr.file_idx, bctr.line_number, bctr.user_id, bctr.account, bctr.operation,
    )
}

pub fn create_tt_cmt_ver4_string(dr: &DistRec) -> String {
    let ver = TT_CMT_VER4.as_str();
    format!(
        "{},{},{},{},{},{},{}",
        ver, dr.file_idx, dr.line_number, dr.order_id, dr.transaction_id, dr.category, dr.operation
    )
}
