// Based on https://stackoverflow.com/a/55134333/4812090
use clap::{App, Arg, ArgMatches, SubCommand};
use std::error::Error;

use crate::common::APP_VERSION;

pub fn arg_matches() -> Result<ArgMatches<'static>, Box<dyn Error>> {
    // The config option is the only option that has a default_value,
    // all others get their defaults from the Configuration.
    // Also, these are all "global(true)" for two reasons:
    //  1) This allows them to be used after the subcommand, i.e.:
    //      binance-auto-sell ai --config data/wink-config.toml
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
        .default_value("config.toml")
        .takes_value(true);
    let api_key_arg = Arg::with_name("api-key")
        .global(true)
        .long("api-key")
        .value_name("API-KEY")
        .help("Define the api key")
        .env("BINANCE_US_API_KEY")
        .takes_value(true);
    let secret_key_arg = Arg::with_name("secret-key")
        .global(true)
        .long("secret-key")
        .value_name("SECRET-KEY")
        .help("Define the secret key")
        .env("BINANCE_US_SECRET_KEY")
        .takes_value(true);
    let log_path_arg = Arg::with_name("log-path")
        .global(true)
        .long("log-path")
        .value_name("PATH")
        .help("Define log path")
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

    let matches = App::new("binance-auto-sell")
        .version(APP_VERSION.as_str())
        .about("Binance cli app")
        .arg(config_arg.clone())
        .arg(api_key_arg.clone())
        .arg(secret_key_arg.clone())
        .arg(log_path_arg.clone())
        .arg(default_quote_asset_arg.clone())
        .arg(test_arg.clone())
        .arg(scheme_arg.clone())
        .arg(domain_arg.clone())
        .subcommand(SubCommand::with_name("do-nothing").about("Do nothing used for testing"))
        .subcommand(SubCommand::with_name("ai").about("Display the account info"))
        .subcommand(SubCommand::with_name("ei").about("Display the exchange info"))
        .subcommand(
            SubCommand::with_name("auto-sell")
                .about("Automatically sell assets as defined in the configuration keep section"),
        )
        .subcommand(
            SubCommand::with_name("auto-buy")
                .about("Automatically buy assets as defined in the configuration buy section"),
        )
        .subcommand(
            SubCommand::with_name("sei")
                .about("Display a symbols exchange information")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("sap")
                .about("Display a symbols 5 minute average price")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("skr")
                .about("Display a symbols current kline record")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("skrs")
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
                .about("Display a symbols open orders")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("ao").about("Dispaly all orders"))
        .subcommand(
            SubCommand::with_name("st")
                .about("Display a symbols trades")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("ol").about("Dispaly order log"))
        .subcommand(
            SubCommand::with_name("buy-market-value")
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
            SubCommand::with_name("sell-market-value")
                .about("Sell asset using quote asset value")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("Value")
                        .help("Value of asset to sell in the quote asset")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("sell-market")
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
        .get_matches();

    Ok(matches)
}
