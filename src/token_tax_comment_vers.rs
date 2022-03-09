//! TokenTax Comment versions
//! This is shared across modules as these must
//! be unique globally

use lazy_static::lazy_static;

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
