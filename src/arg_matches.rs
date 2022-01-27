// Based on https://stackoverflow.com/a/55134333/4812090
use clap::{App, Arg, ArgMatches};
use std::error::Error;

use crate::common::{APP_NAME, APP_VERSION};

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

    let matches = App::new(APP_NAME.as_str())
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
        .arg(confirmation_required_arg.clone())
        .arg(no_confirmation_required_arg.clone())
        .arg(scheme_arg.clone())
        .arg(domain_arg.clone())
        .subcommand(
            App::new("ai")
                .display_order(1)
                .about("Display the account info"),
        )
        .subcommand(
            App::new("auto-buy")
                .display_order(2)
                .about("Automatically buy assets as defined in the configuration buy section"),
        )
        .subcommand(
            App::new("auto-sell")
                .display_order(2)
                .about("Automatically sell assets as defined in the configuration keep section"),
        )
        .subcommand(
            App::new("buy-market-value")
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
            App::new("buy-market")
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
            App::new("sell-market")
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
            App::new("sell-market-value")
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
            App::new("withdraw")
                .display_order(5)
                .about("Withdraw an asset, either quantity, dollars or precent.\nExamples:\n  withdraw ETH '$1000' 1543abcd --keep-min \\$200\n  withdraw ETH 100% 1543abcd --keep-min '$200'\n  withdraw ETH 100 1543abcd\n NOTE: Dollar values must be written\n in single quotes '$123' or with a backslash \\$1234")
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
                .arg(
                    Arg::new("ADDRESS")
                        .help("The destination address")
                        .required(true)
                        .index(3),
                )
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
            App::new("ei")
                .display_order(10)
                .about("Display the exchange info"),
        )
        .subcommand(
            App::new("sei")
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
            App::new("sap")
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
            App::new("skr")
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
            App::new("skrs")
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
            App::new("oo")
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
            App::new("dh")
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
            App::new("wh")
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
            App::new("fcdh")
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
            App::new("fcwh")
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
            App::new("mt")
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
            App::new("ao")
                .display_order(9)
                .about("Dispaly all orders"),
        )
        .subcommand(
            App::new("ol")
                .display_order(9)
                .about("Display order log"),
            )
        .subcommand(
            App::new("pol")
                .display_order(9)
                .about("process order log"),
        )
        .subcommand(
            App::new("pdf")
                .display_order(9)
                .about("process distribution files")
                .arg(
                    Arg::new("IN_FILE")
                    .help("File to process")
                    .required(true)
                    .index(1),
                )
                .arg(
                    Arg::new("OUT_FILE")
                    .help("Output File, optional")
                    .required(false)
                    .index(2),
                ),
        )
        .subcommand(
            App::new("version")
                .display_order(12)
                .about("Display version"),
        )
        .subcommand(
            App::new("check-params")
                .display_order(99)
                .about("Used for testing")
        )
        .get_matches();

    Ok(matches)
}
