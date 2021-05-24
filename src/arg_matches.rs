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
                .help("Enable test mode")
        )
        .subcommand(
            SubCommand::with_name("auto-sell")
                .about("Automatically sell assets as defined in the configuration keep section"),
        )
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
        .get_matches();

    Ok(matches)
}
