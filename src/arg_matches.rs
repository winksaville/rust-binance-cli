// Based on https://stackoverflow.com/a/55134333/4812090
use clap::{Arg, ArgMatches, Command};
use std::error::Error;

use crate::common::{APP_NAME, APP_VERSION};

pub fn time_offset_days_to_time_ms_offset(
    sc_matches: &ArgMatches,
) -> Result<Option<i64>, Box<dyn Error>> {
    // Get time-offset which will be added to each transactions time
    let time_ms_offset = if let Some(offset_days) = sc_matches.value_of("TIME_OFFSET_DAYS") {
        let days = offset_days.parse::<i64>();
        //let days = if let Ok(d) = match offset_days.parse::<i64>() {
        let tms = match days {
            Ok(d) => d * 24 * 60 * 60 * 1000,
            Err(e) => return Err(format!("{offset_days} was not a number, {e}").into()),
        };

        Some(tms)
    } else {
        None
    };

    Ok(time_ms_offset)
}

pub fn arg_matches() -> Result<ArgMatches, Box<dyn Error>> {
    // The config option is the only option that has a default_value,
    // all others get their defaults from the Configuration.
    // Also, these are all "global(true)" for two reasons:
    //  1) This allows them to be used after the subcommand, i.e.:
    //      binance-cli ai --config data/wink-config.toml
    //  2) The configuration::update_config() only updates
    //     the configuration struct for "globals". Changing this
    //     mean update_config has to know about each subcommands
    //     options and, at least at the moment, this is more than
    //     good enough.
    let config_arg = Arg::new("config")
        .global(true)
        .short('c')
        .long("config")
        .value_name("FILE")
        .help("Sets a custom config file")
        .env("BINANCE_CONFIG")
        .default_value("configs/config.toml")
        .takes_value(true);
    let api_key_arg = Arg::new("api-key")
        .global(true)
        .long("api-key")
        .value_name("API-KEY")
        .help("Define the api key")
        .env("BINANCE_API_KEY")
        .takes_value(true);
    let secret_key_arg = Arg::new("secret-key")
        .global(true)
        .long("secret-key")
        .value_name("SECRET-KEY")
        .help("Define the secret key")
        .env("BINANCE_SECRET_KEY")
        .takes_value(true);
    let order_log_path_arg = Arg::new("order-log-path")
        .global(true)
        .long("order-log-path")
        .value_name("PATH")
        .help("Define order log path")
        .takes_value(true);
    let default_quote_asset_arg = Arg::new("default-quote-asset")
        .global(true)
        .long("default-quote-asset")
        .value_name("ASSET")
        .help("The name of the asset that is used to buy or sell another asset")
        .takes_value(true);
    let test_arg = Arg::new("test")
        .global(true)
        .short('t')
        .long("test")
        .help("Enable test mode");
    let no_test_arg = Arg::new("no-test")
        .global(true)
        .long("no-test")
        .help("Disable test mode");
    let verbose_arg = Arg::new("verbose")
        .global(true)
        .long("verbose")
        .help("Enable verbose mode");
    let no_verbose_arg = Arg::new("no-verbose")
        .global(true)
        .long("no-verbose")
        .help("Disable verbose mode");
    let progress_info_arg = Arg::new("progress-info")
        .global(true)
        .long("progress-info")
        .help("Enable progress info");
    let no_progress_info_arg = Arg::new("no-progress-info")
        .global(true)
        .long("no-progress-info")
        .help("Disable progress info");
    let throttle_rate_ms_arg = Arg::new("throttle-rate-ms")
        .global(true)
        .long("throttle-rate-ms")
        .help("Throttle some requests, such as converting.")
        .default_value("500")
        .value_name("IN_MILLISECS")
        .takes_value(true);
    let confirmation_required_arg = Arg::new("confirmation-required")
        .global(true)
        .long("confirmation-required")
        .help("Enable comfirmation being required");
    let no_confirmation_required_arg = Arg::new("no-confirmation-required")
        .global(true)
        .long("no-confirmation-required")
        .help("Disable comfirmation being required");
    let scheme_arg = Arg::new("scheme")
        .global(true)
        .long("scheme")
        .value_name("BINANCE_SCHEME")
        .help("Scheme such as https")
        .takes_value(true);
    let domain_arg = Arg::new("domain")
        .global(true)
        .long("domain")
        .value_name("BINANCE_DOMAIN")
        .help("Domain such as binance.us or binance.com")
        .takes_value(true);
    let time_offset_days_arg = Arg::new("TIME_OFFSET_DAYS")
        .global(false)
        .required(false)
        .long("time-offset")
        .help("Add the time-offset parameter to each transaction, time-offset may be positive or negative.")
        .takes_value(true);
    let no_usd_value_needed_arg = Arg::new("no-usd-value-needed")
        .global(false)
        .long("no-usd-value-needed")
        .help("No USD value needed");
    let withdraw_addr_arg = Arg::new("withdraw-addr")
        .global(true)
        .long("withdraw-addr")
        .value_name("ADDR")
        .help("Default destination address for the withdraw command")
        .takes_value(true);

    let matches = Command::new(APP_NAME.as_str())
        .version(APP_VERSION.as_str())
        .about("Binance cli app")
        .arg(config_arg.clone())
        .arg(api_key_arg.clone())
        .arg(secret_key_arg.clone())
        .arg(order_log_path_arg.clone())
        .arg(default_quote_asset_arg.clone())
        .arg(test_arg.clone())
        .arg(no_test_arg.clone())
        .arg(verbose_arg.clone())
        .arg(no_verbose_arg.clone())
        .arg(progress_info_arg.clone())
        .arg(no_progress_info_arg.clone())
        .arg(throttle_rate_ms_arg.clone())
        .arg(confirmation_required_arg.clone())
        .arg(no_confirmation_required_arg.clone())
        .arg(scheme_arg.clone())
        .arg(domain_arg.clone())
        .arg(withdraw_addr_arg.clone())
        .subcommand(
            Command::new("ai")
                .display_order(1)
                .about("Display the account info")
                // Turns out this isn't the time you'd like for information
                // but related to the receive window :()
                //.arg(
                //    Arg::new("TIME")
                //        .help("Time in form of YYYY-MM-DD H:M:S")
                //        .required(false)
                //        .index(1),
                //),
        )
        .subcommand(
            Command::new("auto-buy")
                .display_order(2)
                .about("Automatically buy assets as defined in the configuration buy section"),
        )
        .subcommand(
            Command::new("auto-sell")
                .display_order(2)
                .about("Automatically sell assets as defined in the configuration keep section"),
        )
        .subcommand(
            Command::new("buy-market-value")
                .display_order(5)
                .about("Buy asset using quote asset value")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("VALUE")
                        .help("Value of asset to buy in the quote asset")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("buy-market")
                .display_order(5)
                .about("Buy a number of assets")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("QUANTITY")
                        .help("Number of assets to buy")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("sell-market")
                .display_order(5)
                .about("Sell a number of assets")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("QUANTITY")
                        .help("Number of assets to sell")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("sell-market-value")
                .display_order(5)
                .about("Sell asset using quote asset value")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("VALUE")
                        .help("Value of asset to sell in the quote asset")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("withdraw")
                .display_order(5)
                .about("Withdraw an asset, either quantity, dollars or precent.\nExamples:\n  withdraw ETH '$1000' --withdraw-addr 1543abcd --keep-min \\$200\n  withdraw ETH 100% --withdraw-addr 1543abcd --keep-min '$200'\n  withdraw ETH 100 --withdraw-addr 1543abcd\nNOTE 1: withdraw-addr maybe placed in config.toml as withdraw_addr = \"1543abcd\"\nNOTE 2: Dollar values must be written\n in single quotes '$123' or with a backslash \\$1234")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("AMOUNT")
                        .help("The amournt of asset, examples 5.2 ETH or \\$100 or '$100' or 12.3% of the free ETH owned")
                        .required(true)
                        .index(2),
                )
                .arg(withdraw_addr_arg)
                .arg(
                    Arg::new("keep-min")
                        .global(false)
                        .long("keep-min")
                        .value_name("KEEP_MIN")
                        .help("Minimum amount to keep in USD, percent asset or asset quantity.\nExamples:\n  --keep-min '$200'\n  --keep-min 10%\n  --keep-min 0.1")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("dest-sec-addr")
                        .global(false)
                        .long("dest-sec-addr")
                        .value_name("DEST_SEC_ADDR")
                        .help("A destination's secondary address")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("dest-label")
                        .global(false)
                        .long("dest-label")
                        .value_name("DEST_LABEL")
                        .help("A label identifying a destination address")
                        .takes_value(true),
                ),
        )
        .subcommand(
            Command::new("ei")
                .display_order(10)
                .about("Display the exchange info"),
        )
        .subcommand(
            Command::new("sei")
                .display_order(10)
                .about("Display a symbols exchange information")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("sap")
                .display_order(10)
                .about("Display a symbols 5 minute average price")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("skr")
                .display_order(10)
                .about("Display a symbols current kline record")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("START-TIME-UTC")
                        .help("Start time UTC")
                        .required(false)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("skrs")
                .display_order(10)
                .about("Display a symbols kline records")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("START-TIME")
                        .short('s')
                        .long("start_time")
                        .value_name("START-TIME")
                        .help("Define the starting time format: YYYY-MM-DDTHR:MIN:SEC{TZ} where TZ is offset from UTC and if absent then the users TZ offset is used. Or TZ maybe z or Z for UTC or +/- then 2 hr digits and optionally 2 min digits.\nexamples:\n  2021-05-24T16:31:12       => Local TZ\n  2022-01-17T00:00:00z      => UTC\n  2022-01-17T00:00:00-08    => PST 2 digit hourly\n  2022-01-17T00:09:00+0530  => IST 4 digit with minutes")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("LIMIT")
                        .short('l')
                        .long("limit")
                        .value_name("LIMIT")
                        .help("Number of kline records to get, value between 1 and 1000")
                        .default_value("1")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("INTERVAL")
                        .short('i')
                        .long("interval")
                        .value_name("INTERVAL")
                        .help("Kline interval, one of: 1m 3m 5m 15m 30m 1h 2h 4h 6h 8h 12h 1d 3d 1w 1M")
                        .takes_value(true),
                ),
        )
        .subcommand(
            Command::new("oo")
                .display_order(10)
                .about("Display a symbols open orders")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("dh")
                .display_order(7)
                .about("Display deposit history")
                .arg(
                    Arg::new("ASSET")
                        .help("Name of asset or all assets if absent")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("wh")
                .display_order(7)
                .about("Display withdrawal history")
                .arg(
                    Arg::new("ASSET")
                        .help("Name of asset or all assets if absent")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("fcdh")
                .display_order(7)
                .about("Display fiat currency deposit history")
                .arg(
                    Arg::new("FIAT_CURRENCY")
                        .help("Name of fiat currency or USD if absent")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("fcwh")
                .display_order(7)
                .about("Display fiat currency withdraw history")
                .arg(
                    Arg::new("FIAT_CURRENCY")
                        .help("Name of fiat currency or USD if absent")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("mt")
                .display_order(8)
                .about("Display my trades for a symbol")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Name of asset")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("ao")
                .display_order(9)
                .about("Dispaly all orders"),
        )
        .subcommand(
            Command::new("obid")
                .display_order(9)
                .about("Order by id")
                .arg(
                    Arg::new("SYMBOL")
                        .help("Symbol such as BTC")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("ORDER_ID")
                        .help("Numeric Order id")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new("LIMIT")
                        .help("Numeric limit 1..1000")
                        .required(true)
                        .index(3),
                ),
        )
        .subcommand(
            Command::new("ol")
                .display_order(9)
                .about("Display order log"),
            )
        .subcommand(
            Command::new("pol")
                .display_order(9)
                .about("process order log"),
        )
        .subcommand(
            Command::new("ubudf")
                .display_order(9)
                .about("update binance.us distribution files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(true)
                        .long("out-file")
                        .short('o')
                        .help("The output file")
                        .takes_value(true),
                )
        )
        .subcommand(
            Command::new("pbudf")
                .display_order(9)
                .about("process binance.us distribution files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(false)
                        .long("out-file")
                        .short('o')
                        .help("The optional output file")
                        .takes_value(true),
                )
                .arg(&no_usd_value_needed_arg)
        )
        .subcommand(
            Command::new("cbudf")
                .display_order(9)
                .about("consolidate binance.us distribution files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(true)
                        .long("out-file")
                        .short('o')
                        .help("The output file")
                        .takes_value(true),
                )
                .arg(&time_offset_days_arg)
        )
        .subcommand(
            Command::new("ttffbudf")
                .display_order(9)
                .about("Token Tax file from binance.us distribution files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(true)
                        .long("out-file")
                        .short('o')
                        .help("The output file")
                        .takes_value(true),
                )
                .arg(&time_offset_days_arg)
        )
        .subcommand(
            Command::new("pbcthf")
                .display_order(9)
                .about("process binance.com trade history files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(false)
                        .long("out-file")
                        .short('o')
                        .help("The optional output file")
                        .takes_value(true),
                )
        )
        .subcommand(
            Command::new("cbcthf")
                .display_order(9)
                .about("consolidate binance.com trade history files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(true)
                        .long("out-file")
                        .short('o')
                        .help("The output file")
                        .takes_value(true),
                )
                .arg(&time_offset_days_arg)
        )
        .subcommand(
            Command::new("ttffbcthf")
                .display_order(9)
                .about("Token Tax file from binance.com trade history files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(true)
                        .long("out-file")
                        .short('o')
                        .help("The output file")
                        .takes_value(true),
                )
                .arg(&time_offset_days_arg)
        )
        .subcommand(
            Command::new("pttf")
                .display_order(9)
                .about("process Token Tax files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(false)
                        .long("out-file")
                        .short('o')
                        .help("The optional output file")
                        .takes_value(true),
                )
                .arg(&no_usd_value_needed_arg)
        )
        .subcommand(
            Command::new("cttf")
                .display_order(9)
                .about("consolidate Token Tax files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(false)
                        .long("out-file")
                        .short('o')
                        .help("The optional output file")
                        .takes_value(true),
                )
        )
        .subcommand(
            Command::new("ucttf")
                .display_order(9)
                .about("uniq currency transactions in Token Tax files")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(false)
                        .long("out-file")
                        .short('o')
                        .help("The optional output file")
                        .takes_value(true),
                )
        )
        .subcommand(
            Command::new("ptbf")
                .display_order(9)
                .about("process Tax Bit file")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(false)
                        .long("out-file")
                        .short('o')
                        .help("The output file")
                        .takes_value(true),
                )
        )
        .subcommand(
            Command::new("tbffttf")
                .display_order(9)
                .about("TaxBit file from Token Tax file")
                .arg(
                    Arg::new("IN_FILES")
                        .global(false)
                        .required(true)
                        .long("files")
                        .short('f')
                        .multiple_values(true)
                        .help("List of input files")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUT_FILE")
                        .global(false)
                        .required(true)
                        .long("out-file")
                        .short('o')
                        .help("The output file")
                        .takes_value(true),
                )
                .arg(&time_offset_days_arg)
        )
        .subcommand(
            Command::new("version")
                .display_order(12)
                .about("Display version"),
        )
        .subcommand(
            Command::new("check-params")
                .display_order(99)
                .about("Used for testing")
        )
        .get_matches();

    Ok(matches)
}
