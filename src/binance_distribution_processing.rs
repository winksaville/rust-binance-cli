//use std::{error::Error, fmt, fs::File, io::BufRead, io::BufReader, path::PathBuf};

use clap::SubCommand;
//use log::trace;
//use serde::{Deserialize, Serialize};

//use rust_decimal::prelude::*;
//use rust_decimal_macros::dec;
//use semver::Version;

use crate::{
    //common::{dec_to_money_string, time_ms_to_utc, InternalErrorRec, ResponseErrorRec, Side},
    configuration::Configuration,
    //de_string_or_number::{de_string_or_number_to_i64, de_string_or_number_to_u64},
};

pub async fn process_dist_files(
    config: &Configuration,
    _subcmd: &SubCommand<'static>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("process_dist_files:\n config={:?} \nsubcmd={:?}", config, _subcmd);

    //iterate_dist_file(config.order_log_path.clone(), process_order_log_line).await?;

    Ok(())
}

#[cfg(test)]
mod test {

    //use super::*;

    #[test]
    fn test_1() {
    }
}
