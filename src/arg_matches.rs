// Based on https://stackoverflow.com/a/55134333/4812090
use clap::{App, Arg, ArgMatches, SubCommand};
use std::error::Error;

use crate::common::{APP_NAME, APP_VERSION};

pub fn arg_matches() -> Result<ArgMatches<'static>, Box<dyn Error>> {
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
    let config_arg = Arg::with_name("config")
        .global(true)
        .short("c")
        .long("config")
        .value_name("FILE")
        .help("Sets a custom config file")
        .env("BINANCE_CONFIG")
        .default_value("configs/config.toml")
        .takes_value(true);
    let api_key_arg = Arg::with_name("api-key")
        .global(true)
        .long("api-key")
        .value_name("API-KEY")
        .help("Define the api key")
        .env("BINANCE_API_KEY")
        .takes_value(true);
    let secret_key_arg = Arg::with_name("secret-key")
        .global(true)
        .long("secret-key")
        .value_name("SECRET-KEY")
        .help("Define the secret key")
        .env("BINANCE_SECRET_KEY")
        .takes_value(true);
    let order_log_path_arg = Arg::with_name("order-log-path")
        .global(true)
        .long("order-log-path")
        .value_name("PATH")
        .help("Define order log path")
        .takes_value(true);
    let default_quote_asset_arg = Arg::with_name("default-quote-asset")
        .global(true)
        .long("default-quote-asset")
        .value_name("ASSET")
        .help("The name of the asset that is used to buy or sell another asset")
        .takes_value(true);
    let test_arg = Arg::with_name("test")
        .global(true)
        .short("t")
        .long("test")
        .help("Enable test mode");
    let no_test_arg = Arg::with_name("no-test")
        .global(true)
        .long("no-test")
        .help("Disable test mode");
    let scheme_arg = Arg::with_name("scheme")
        .global(true)
        .long("scheme")
        .value_name("BINANCE_SCHEME")
        .help("Scheme such as https")
        .takes_value(true);
    let domain_arg = Arg::with_name("domain")
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
        .arg(scheme_arg.clone())
        .arg(domain_arg.clone())
        .subcommand(
            SubCommand::with_name("ai")
                .display_order(1)
                .about("Display the account info"),
        )
        .subcommand(
            SubCommand::with_name("auto-buy")
                .display_order(2)
                .about("Automatically buy assets as defined in the configuration buy section"),
        )
        .subcommand(
            SubCommand::with_name("auto-sell")
                .display_order(2)
                .about("Automatically sell assets as defined in the configuration keep section"),
        )
        .subcommand(
            SubCommand::with_name("buy-market-value")
                .display_order(5)
                .about("Buy asset using quote asset value")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("VALUE")
                        .help("Value of asset to buy in the quote asset")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("buy-market")
                .display_order(5)
                .about("Buy a number of assets")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("QUANTITY")
                        .help("Number of assets to buy")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("sell-market")
                .display_order(5)
                .about("Sell a number of assets")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("QUANTITY")
                        .help("Number of assets to sell")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("sell-market-value")
                .display_order(5)
                .about("Sell asset using quote asset value")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("VALUE")
                        .help("Value of asset to sell in the quote asset")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("withdraw")
                .display_order(5)
                .about("Withdraw an asset, either quantity or precent. Examples: `withdraw ETH 1 1543abcd` or `withdraw ETH 1.5% 1543abcd`")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("AMOUNT")
                        .help("The amournt of asset, example `5.2` ETH or `12.3%` of the free ETH owned")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("ADDRESS")
                        .help("The destination address")
                        .required(true)
                        .index(3),
                )
                .arg(
                    Arg::with_name("dest-sec-addr")
                        .global(false)
                        .long("dest-sec-addr")
                        .value_name("DEST_SEC_ADDR")
                        .help("A destination's secondary address")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dest-label")
                        .global(false)
                        .long("dest-label")
                        .value_name("DEST_LABEL")
                        .help("A label identifying a destination address")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("ei")
                .display_order(10)
                .about("Display the exchange info"),
        )
        .subcommand(
            SubCommand::with_name("sei")
                .display_order(10)
                .about("Display a symbols exchange information")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(10),
                ),
        )
        .subcommand(
            SubCommand::with_name("sap")
                .display_order(10)
                .about("Display a symbols 5 minute average price")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(10),
                ),
        )
        .subcommand(
            SubCommand::with_name("skr")
                .display_order(10)
                .about("Display a symbols current kline record")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(10),
                ),
        )
        .subcommand(
            SubCommand::with_name("skrs")
                .display_order(10)
                .about("Display a symbols kline records")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("START-TIME")
                        .short("s")
                        .long("start_time")
                        .value_name("START-TIME")
                        .help("Define the starting time format: YYYY-MM-DDTHR:MIN example: 2021-05-24T16:31")
                        //.default_value("now")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("LIMIT")
                        .short("l")
                        .long("limit")
                        .value_name("LIMIT")
                        .help("Number of kline records to get, value between 1 and 1000")
                        .default_value("1")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("INTERVAL")
                        .short("i")
                        .long("interval")
                        .value_name("INTERVAL")
                        .help("Kline interval, one of: 1m 3m 5m 15m 30m 1h 2h 4h 6h 8h 12h 1d 3d 1w 1M")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("oo")
                .display_order(10)
                .about("Display a symbols open orders")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("dh")
                .display_order(7)
                .about("Display deposit history")
                .arg(
                    Arg::with_name("ASSET")
                        .help("Name of aseet or all assets if absent")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("wh")
                .display_order(7)
                .about("Display withdrawal history")
                .arg(
                    Arg::with_name("ASSET")
                        .help("Name of aseet or all assets if absent")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("fcdh")
                .display_order(7)
                .about("Display fiat currency deposit history")
                .arg(
                    Arg::with_name("FIAT_CURRENCY")
                        .help("Name of fiat currency or USD if absent")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("fcwh")
                .display_order(7)
                .about("Display fiat currency withdraw history")
                .arg(
                    Arg::with_name("FIAT_CURRENCY")
                        .help("Name of fiat currency or USD if absent")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("mt")
                .display_order(8)
                .about("Display my trades for a symbol")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("ao")
                .display_order(9)
                .about("Dispaly all orders"),
        )
        .subcommand(
            SubCommand::with_name("ol")
                .display_order(9)
                .about("Display order log"),
            )
        .subcommand(
            SubCommand::with_name("version")
                .display_order(10)
                .about("Display version"),
        )
        .subcommand(
            SubCommand::with_name("do-nothing")
                .display_order(99)
                .about("Do nothing used for testing"),
        )
        .get_matches();

    Ok(matches)
}
