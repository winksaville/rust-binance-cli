# Binance auto sell
[![codecov](https://codecov.io/gh/winksaville/rust-binance-auto-sell/branch/main/graph/badge.svg?token=5l3L7yVGTj)](https://codecov.io/gh/winksaville/rust-binance-auto-sell)

A command line interface to some of the binance.us REST API's.
Including some higher level capabilities such as automatically
selling some assets. Eventually other higher level capabilities
maybe provoded.

## Prerequisites

Along with the normal rust tools installed via `rustup` and `cargo install`.

> Note: rustup must be 1.24.1 as I'm using `rust-toolchain.toml` for rustup
> configuration.
(Add more specific docs here)

You must install a version of Tarpaulin >= `0.18.0-alpha1` as I'm using the
--follow-exec option, currently I'm installing with:
```
$ cargo install --git https://github.com/xd009642/tarpaulin.git --branch develop cargo-tarpaulin
```

You should verify `follow-exec` is in the help:
```
$ cargo tarpaulin --help | grep 'follow-exe'
        --follow-exec            Follow executed processes capturing coverage information if they're part of your
```

Copy `cargo-precommit` to ~/.cargo/bin/
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cp ./cargo-precommit ~/.cargo/bin/
```

## Before committing

Run `cargo pre-commit` it runs cargo check, fmt, test, tarpaulin and clippy

```
$ cargo pre-commit
check
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
fmt
test
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished test [unoptimized + debuginfo] target(s) in 2.95s
     Running target/debug/deps/binance_auto_sell-10c26e09c2475d07

running 24 tests
test binance_account_info::test::test_account_info ... ok
test binance_signature::test::test_binance_example ... ok

...

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

tarpaulin
May 07 09:02:43.508  INFO cargo_tarpaulin: Running Tarpaulin
May 07 09:02:43.508  INFO cargo_tarpaulin: Building project
May 07 09:02:43.750  INFO cargo_tarpaulin::cargo: Cleaning project
   Compiling autocfg v1.0.1

...

test binance_verify_order::test::test_adj_quantity_verify_market_lot_size ... ok
test binance_exchange_info::test::test_exchange_info ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

May 07 09:03:13.315  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src/binance_account_info.rs: 27/100
|| src/binance_auto_sell.rs: 46/70
|| src/binance_avg_price.rs: 0/16
|| src/binance_context.rs: 25/26
|| src/binance_exchange_info.rs: 202/275
|| src/binance_open_orders.rs: 0/49
|| src/binance_order_response.rs: 24/38
|| src/binance_sell_market.rs: 0/25
|| src/binance_signature.rs: 114/114
|| src/binance_trade.rs: 0/79
|| src/binance_verify_order.rs: 91/145
|| src/common.rs: 60/70
|| src/de_string_or_number.rs: 52/55
|| src/main.rs: 0/46
|| 
57.85% coverage, 641/1108 lines covered
clean
clippy
   Compiling proc-macro2 v1.0.26

...

    Checking reqwest v0.11.3
   Compiling rust_decimal_macros v1.12.4
    Checking binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished dev [unoptimized + debuginfo] target(s) in 14.62s
```

## Building and run

Building
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo clean ; cargo build
   Compiling proc-macro2 v1.0.26
   Compiling unicode-xid v0.2.2
...
   Compiling reqwest v0.11.3
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished dev [unoptimized + debuginfo] target(s) in 23.47s
```

Run with no parameters
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/binance-auto-sell`
Usage: binance-auto-sell help, --help or -h
```

And here we build and then run showing the help message:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (wip)
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s

wink@3900x:~/prgs/rust/projects/binance-auto-sell (wip)
$ ./target/debug/binance-auto-sell help
binance-auto-sell 0.1.0-e865e61
Binance cli app

USAGE:
    binance-auto-sell [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -t, --test       Enable test mode
    -V, --version    Prints version information

OPTIONS:
        --api-key <API-KEY>              Define the api key [env: BINANCE_API_KEY=]
    -c, --config <FILE>                  Sets a custom config file [env: BINANCE_CONFIG=]  [default: config.toml]
        --default-quote-asset <ASSET>    The name of the asset that is used to buy or sell another asset
        --domain <BINANCE_DOMAIN>        Domain such as binance.us or binance.com
        --order-log-path <PATH>          Define order log path
        --scheme <BINANCE_SCHEME>        Scheme such as https
        --secret-key <SECRET-KEY>        Define the secret key [env: BINANCE_SECRET_KEY=]

SUBCOMMANDS:
    ai                   Display the account info
    ao                   Dispaly all orders
    auto-buy             Automatically buy assets as defined in the configuration buy section
    auto-sell            Automatically sell assets as defined in the configuration keep section
    buy-market           Buy a number of assets
    buy-market-value     Buy asset using quote asset value
    do-nothing           Do nothing used for testing
    ei                   Display the exchange info
    help                 Prints this message or the help of the given subcommand(s)
    ol                   Dispaly order log
    oo                   Display a symbols open orders
    sap                  Display a symbols 5 minute average price
    sei                  Display a symbols exchange information
    sell-market          Sell a number of assets
    sell-market-value    Sell asset using quote asset value
    skr                  Display a symbols current kline record
    skrs                 Display a symbols kline records
    st                   Display a symbols trades```
```

> Note: you'll need
> [binance API_KEY and SECRET_KEY](https://www.binance.com/en/support/faq/360002502072)
> to use most commands. I suggest `mkdir data ; cp config.toml data/config.toml`
> adding a real real keys to `data/config.toml` and then
> `export BINANCE_CONFIG=data/config.toml`. As you can see in the `OPTIONS`
> section if you prefer you can `export BINANCE_SECRET_KEY=xxx` and
> `export BINANCE_API_KEY=yyy` or pass them on the command line with
> `--secret-key xxx` and `--api-key=yyy`.

## Debug

There are trace!() statements they can be seen by using `RUST_LOG=trace cargo run`.
Too see only "binance-auto-sell" traces use `RUST_LOG=binance_auto_sell=trace cargo run`.
And as a small real example, to see only traces from the configuration module use.
Here we see that there is an error because the default config.toml doesn't
have real keys, API_KEY and SECRET_KEY:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (wip)
$ RUST_LOG=binance_auto_sell::configuration=trace cargo run ai
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/binance-auto-sell ai`
[2021-05-29T01:38:13Z TRACE binance_auto_sell::configuration] Configuration::new: opt_config=Some(
        "config.toml",
    )
[2021-05-29T01:38:13Z TRACE binance_auto_sell::configuration] config from file:
    Configuration {
        secret_key: "a secret key",
        api_key: "an api key",
        order_log_path: None,
        default_quote_asset: "USD",
        test: false,
        keep: {},
        buy: {},
        scheme: "https",
        domain: "binance.us",
    }
[2021-05-29T01:38:13Z TRACE binance_auto_sell::configuration] config after update_config:
    Configuration {
        secret_key: "a secret key",
        api_key: "an api key",
        order_log_path: None,
        default_quote_asset: "USD",
        test: false,
        keep: {},
        buy: {},
        scheme: "https",
        domain: "binance.us",
    }
Error: "response status=401 Unauthorized body={\"code\":-2014,\"msg\":\"API-key format invalid.\"}"
```

I've also dabbled using vscode debugger and there is a
`.vscode/launch.json` file, but YMMV.


## Test

Test using `cargo test`:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo test
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished test [unoptimized + debuginfo] target(s) in 4.22s
     Running target/debug/deps/binance_auto_sell-e94ca3f569a2653a

running 34 tests
test binance_order_response::test::test_order_response_header_rec_app_rec_versions ... ok
test binance_account_info::test::test_account_info ... ok
test binance_klines::test::test_kline_rec ... ok
test binance_order_response::test::test_order_response_header_rec_min ... ok
test binance_order_response::test::test_order_response_semver ... ok
test binance_order_response::test::test_order_response_header_rec_max ... ok
test binance_order_response::test::test_order_response_success_unknown ... ok
test binance_order_response::test::test_order_response_success_ack ... ok
test binance_order_response::test::test_order_response_success_result ... ok
test binance_auto_sell::test::test_config_auto_sell_all ... ok
test binance_signature::test::test_append_signature ... ok
test binance_signature::test::test_binance_example ... ok
test binance_order_response::test::test_order_response_success_full ... ok
test binance_signature::test::test_binance_signature_body_only ... ok
test binance_signature::test::test_binance_signature_no_query_string_no_body ... ok
test binance_signature::test::test_binance_signature_query_string_and_body ... ok
test binance_signature::test::test_query_vec_u8 ... ok
test binance_signature::test::test_binance_signature_query_string_only ... ok
test binance_signature::test::test_query_vec_u8_no_data ... ok
test binance_exchange_info::test::test_exchange_info ... ok
test binance_trade::test::test_log_order_response ... ok
test common::test::test_binance_response_error_rec ... ok
test common::test::test_internal_error ... ok
test common::test::test_binance_response_error_rec_bad_body ... ok
test common::test::test_timestamp_ms_to_secs_nsecs ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_f64_errors ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_i64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_numbers ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_u64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_strings ... ok
test binance_verify_order::test::test_adj_quantity_verify_lot_size ... ok
test common::test::test_utc_now_to_time_ms ... ok
test binance_trade::test::test_convertcommission ... ok
test binance_trade::test::test_convert ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

     Running target/debug/deps/cli-165364053a395b57

running 4 tests
test test_no_params ... ok
test test_req_params ... ok
test test_req_params_as_env_vars ... ok
test test_help ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Code coverage

> Note: tarpaulin is being used for code coverage, but you must use 0.18.0+.
> Because of [issue #1 in this repo](https://github.com/winksaville/rust-binance-auto-sell/issues/1)
> when you want to run `cargo build|run|test` after `cargo tarpaulin` you
> must do a `cargo clean` first.

So the first time run `cargo tarpaulin`:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo tarpaulin
May 26 08:31:33.926  INFO cargo_tarpaulin: Running Tarpaulin
May 26 08:31:33.926  INFO cargo_tarpaulin: Building project
May 26 08:31:34.102  INFO cargo_tarpaulin::cargo: Cleaning project
   Compiling proc-macro2 v1.0.26
   Compiling autocfg v1.0.1
   Compiling unicode-xid v0.2.2
...
   Compiling function_name v0.2.0
   Compiling hyper-tls v0.5.0
   Compiling reqwest v0.11.3
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished test [unoptimized + debuginfo] target(s) in 26.27s
May 26 08:32:00.474  INFO cargo_tarpaulin::process_handling::linux: Launching test
May 26 08:32:00.474  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/cli-165364053a395b57

running 4 tests
test test_req_params_as_env_vars ... ok
test test_req_params ... ok
test test_no_params ... ok
test test_help ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

May 26 08:32:01.667  INFO cargo_tarpaulin::process_handling::linux: Launching test
May 26 08:32:01.667  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/binance_auto_sell-e94ca3f569a2653a

running 34 tests
test binance_signature::test::test_query_vec_u8_no_data ... ok
test binance_order_response::test::test_order_response_semver ... ok
test binance_signature::test::test_binance_example ... ok
test binance_signature::test::test_binance_signature_no_query_string_no_body ... ok
test binance_order_response::test::test_order_response_header_rec_min ... ok
test binance_signature::test::test_query_vec_u8 ... ok
test binance_order_response::test::test_order_response_header_rec_app_rec_versions ... ok
test binance_trade::test::test_log_order_response ... ok
test binance_signature::test::test_binance_signature_query_string_only ... ok
test binance_signature::test::test_binance_signature_query_string_and_body ... ok
test binance_signature::test::test_binance_signature_body_only ... ok
test binance_order_response::test::test_order_response_header_rec_max ... ok
test binance_signature::test::test_append_signature ... ok
test binance_order_response::test::test_order_response_success_unknown ... ok
test common::test::test_internal_error ... ok
test binance_order_response::test::test_order_response_success_ack ... ok
test common::test::test_binance_response_error_rec_bad_body ... ok
test common::test::test_binance_response_error_rec ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_u64_errors ... ok
test binance_klines::test::test_kline_rec ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_i64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_strings ... ok
test de_string_or_number::tests::test_de_string_or_number_from_numbers ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_f64_errors ... ok
test common::test::test_utc_now_to_time_ms ... ok
test binance_order_response::test::test_order_response_success_result ... ok
test binance_order_response::test::test_order_response_success_full ... ok
test binance_account_info::test::test_account_info ... ok
test common::test::test_timestamp_ms_to_secs_nsecs ... ok
test binance_auto_sell::test::test_config_auto_sell_all ... ok
test binance_verify_order::test::test_adj_quantity_verify_lot_size ... ok
test binance_exchange_info::test::test_exchange_info ... ok
test binance_trade::test::test_convert ... ok
test binance_trade::test::test_convertcommission ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

May 26 08:32:06.567  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src/arg_matches.rs: 0/45
|| src/binance_account_info.rs: 27/106
|| src/binance_auto_sell.rs: 37/136
|| src/binance_avg_price.rs: 0/16
|| src/binance_exchange_info.rs: 202/271
|| src/binance_get_klines_cmd.rs: 0/22
|| src/binance_klines.rs: 59/129
|| src/binance_market_order_cmd.rs: 0/67
|| src/binance_my_trades.rs: 0/50
|| src/binance_order_response.rs: 134/225
|| src/binance_orders.rs: 0/64
|| src/binance_signature.rs: 114/114
|| src/binance_trade.rs: 55/165
|| src/binance_verify_order.rs: 59/116
|| src/common.rs: 85/145
|| src/configuration.rs: 22/57
|| src/de_string_or_number.rs: 52/55
|| src/main.rs: 0/112
|| 
44.64% coverage, 846/1895 lines covered
```

For subsequent runs you can use `--skip-clean` to save time:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo tarpaulin --skip-clean
May 26 08:33:36.424  INFO cargo_tarpaulin: Running Tarpaulin
May 26 08:33:36.424  INFO cargo_tarpaulin: Building project
    Finished test [unoptimized + debuginfo] target(s) in 0.04s
May 26 08:33:36.631  INFO cargo_tarpaulin::process_handling::linux: Launching test
May 26 08:33:36.631  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/binance_auto_sell-e94ca3f569a2653a

running 34 tests
test binance_signature::test::test_query_vec_u8_no_data ... ok
test binance_order_response::test::test_order_response_semver ... ok
test binance_signature::test::test_binance_example ... ok
test binance_signature::test::test_binance_signature_no_query_string_no_body ... ok
test binance_order_response::test::test_order_response_header_rec_min ... ok
test binance_signature::test::test_query_vec_u8 ... ok
test binance_order_response::test::test_order_response_header_rec_app_rec_versions ... ok
test binance_signature::test::test_binance_signature_query_string_only ... ok
test binance_signature::test::test_binance_signature_query_string_and_body ... ok
test binance_signature::test::test_binance_signature_body_only ... ok
test binance_order_response::test::test_order_response_header_rec_max ... ok
test binance_trade::test::test_log_order_response ... ok
test binance_signature::test::test_append_signature ... ok
test binance_order_response::test::test_order_response_success_unknown ... ok
test common::test::test_internal_error ... ok
test common::test::test_binance_response_error_rec_bad_body ... ok
test common::test::test_binance_response_error_rec ... ok
test binance_order_response::test::test_order_response_success_ack ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_u64_errors ... ok
test binance_klines::test::test_kline_rec ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_i64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_strings ... ok
test de_string_or_number::tests::test_de_string_or_number_from_numbers ... ok
test common::test::test_utc_now_to_time_ms ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_f64_errors ... ok
test binance_order_response::test::test_order_response_success_result ... ok
test binance_account_info::test::test_account_info ... ok
test binance_order_response::test::test_order_response_success_full ... ok
test common::test::test_timestamp_ms_to_secs_nsecs ... ok
test binance_auto_sell::test::test_config_auto_sell_all ... ok
test binance_verify_order::test::test_adj_quantity_verify_lot_size ... ok
test binance_exchange_info::test::test_exchange_info ... ok
test binance_trade::test::test_convert ... ok
test binance_trade::test::test_convertcommission ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s

May 26 08:33:41.647  INFO cargo_tarpaulin::process_handling::linux: Launching test
May 26 08:33:41.647  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/cli-165364053a395b57

running 4 tests
test test_req_params_as_env_vars ... ok
test test_req_params ... ok
test test_no_params ... ok
test test_help ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

May 26 08:33:42.795  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src/arg_matches.rs: 0/45 +0%
|| src/binance_account_info.rs: 27/106 +0%
|| src/binance_auto_sell.rs: 37/136 +0%
|| src/binance_avg_price.rs: 0/16 +0%
|| src/binance_exchange_info.rs: 202/271 +0%
|| src/binance_get_klines_cmd.rs: 0/22 +0%
|| src/binance_klines.rs: 59/129 +0%
|| src/binance_market_order_cmd.rs: 0/67 +0%
|| src/binance_my_trades.rs: 0/50 +0%
|| src/binance_order_response.rs: 134/225 +0%
|| src/binance_orders.rs: 0/64 +0%
|| src/binance_signature.rs: 114/114 +0%
|| src/binance_trade.rs: 55/165 +0%
|| src/binance_verify_order.rs: 59/116 +0%
|| src/common.rs: 85/145 +0%
|| src/configuration.rs: 22/57 +0%
|| src/de_string_or_number.rs: 52/55 +0%
|| src/main.rs: 0/112 +0%
|| 
44.64% coverage, 846/1895 lines covered, +0% change in coverage
```

**But**, as mentioned above, run `cargo build|run|test`
commands the first one will need to be preceeded by a `cargo clean`
otherwise you'll get an error from the linker:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo run help
   Compiling proc-macro2 v1.0.26
   Compiling unicode-xid v0.2.2
...
          /home/wink/prgs/rust/projects/binance-auto-sell/src/common.rs:20: undefined reference to `core::ptr::drop_in_place<alloc::string::String>'
          /usr/bin/ld: /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/binance_auto_sell-f54f02dcf7d07658.y4lov3w4ip2lx2c.rcgu.o:/home/wink/prgs/rust/projects/binance-auto-sell/src/common.rs:20: more undefined references to `core::ptr::drop_in_place<alloc::string::String>' follow
          collect2: error: ld returned 1 exit status
          

error: aborting due to previous error

error: could not compile `binance-auto-sell`

To learn more, run the command again with --verbose.
```

Here is what you need to do:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo clean ; cargo run help
   Compiling proc-macro2 v1.0.26
   Compiling unicode-xid v0.2.2
...
   Compiling reqwest v0.11.3
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished dev [unoptimized + debuginfo] target(s) in 23.45s
     Running `target/debug/binance-auto-sell help`
Exper clap config 0.1.0
Experiment using a config file

USAGE:
    binance-auto-sell [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -t, --test       Enable test mode
    -V, --version    Prints version information

OPTIONS:
        --api-key <API-KEY>              Define the api key [env: BINANCE_US_API_KEY=]
    -c, --config <FILE>                  Sets a custom config file [env: BINANCE_CONFIG=]  [default: config.toml]
        --default-quote-asset <ASSET>    The name of the asset that is used to buy or sell another asset
        --domain <BINANCE_DOMAIN>        Domain such as binance.us or binance.com
        --log-path <PATH>                Define log path
        --scheme <BINANCE_SCHEME>        Scheme such as https
        --secret-key <SECRET-KEY>        Define the secret key [env: BINANCE_US_SECRET_KEY=]

SUBCOMMANDS:
    ai             Display the account info
    ao             Dispaly all orders
    auto-sell      Automatically sell assets as defined in the configuration keep section
    buy-market     Buy an asset
    do-nothing     Do nothing used for testing
    ei             Display the exchange info
    help           Prints this message or the help of the given subcommand(s)
    ol             Dispaly order log
    oo             Display a symbols open orders
    sap            Display a symbols 5 minute average price
    sei            Display a symbols exchange information
    sell-market    Sell an asset
    skr            Display a symbols current kline record
    skrs           Display a symbols kline records
    st             Display a symbols trades
For subseqent `build|run|test`'s it is not necessary:
```

And, of course, subquent runs don't need the `cargo clean`:
``
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
