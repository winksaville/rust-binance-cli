// Based on https://stackoverflow.com/a/55134333/4812090
use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use std::error::Error;

pub fn arg_matches() -> Result<ArgMatches<'static>, Box<dyn Error>> {
    let matches = App::new("Exper clap config")
        .version(crate_version!())
        .about("Experiment using a config file")
        .arg(
            // The config option is the only option that has a default_value,
            // all others get their defaults from the Configuration.
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .env("BINANCE_CONFIG")
                .default_value("config.toml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("api-key")
                .short("a")
                .long("api-key")
                .value_name("API-KEY")
                .help("Define the api key")
                .env("BINANCE_US_API_KEY")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("secret-key")
                .short("s")
                .long("secret-key")
                .value_name("SECRET-KEY")
                .help("Define the secret key")
                .env("BINANCE_US_SECRET_KEY")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("log-path")
                .short("l")
                .long("log-path")
                .value_name("PATH")
                .help("Define log path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("default-quote-asset")
                .short("d")
                .long("default-quote-asset")
                .value_name("ASSET")
                .help("The name of the asset that is used to buy or sell another asset")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("test")
                .short("t")
                .long("test")
                .help("Enable test mode"),
        )
        .arg(
            Arg::with_name("scheme")
                .long("scheme")
                .value_name("BINANCE_SCHEME")
                .help("Scheme such as https")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("domain")
                .long("domain")
                .value_name("BINANCE_DOMAIN")
                .help("Domain such as binance.us or binance.com")
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name("ai").about("Display the account info"))
        .subcommand(SubCommand::with_name("ei").about("Display the exchange info"))
        .subcommand(
            SubCommand::with_name("auto-sell")
                .about("Automatically sell assets as defined in the configuration keep section"),
        )
        .subcommand(
            SubCommand::with_name("sei")
                .about("Display a symbols exchange info")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("ap")
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
                    Arg::with_name("start-time")
                        .short("s")
                        .long("start_time")
                        .value_name("START-TIME")
                        .help("Define the starting time format: YYYY-MM-DDTHR:MIN example: 2021-05-24T16:31")
                        .default_value("now")
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
                    Arg::with_name("interval")
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
            SubCommand::with_name("buy-market")
                .about("Buy an asset")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("QUANTITY")
                        .help("Amount of asset to buy")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("sell-market")
                .about("Sell an asset")
                .arg(
                    Arg::with_name("SYMBOL")
                        .help("Name of aseet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("QUANTITY")
                        .help("Amount of asset to sell")
                        .required(true)
                        .index(2),
                ),
        )
        .get_matches();

    Ok(matches)
}
