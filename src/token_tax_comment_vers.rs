//! TokenTax Comment versions
//! This is shared across modules as these must
//! be unique globally

use lazy_static::lazy_static;

lazy_static! {
    pub static ref TT_CMT_VER0_CSV_HEADER: String =
        "version,LineNumber,OrderId,TransactionId,Category,Operation".to_owned();
    pub static ref TT_CMT_VER0: String = "v0".to_owned();
}
