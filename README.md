# Binance-cli

![ci-stable](https://github.com/winksaville/rust-binance-cli/actions/workflows/ci-stable.yml/badge.svg)
![ci-nightly](https://github.com/winksaville/rust-binance-cli/actions/workflows/ci-nightly.yml/badge.svg)
[![codecov](https://codecov.io/gh/winksaville/rust-binance-cli/branch/main/graph/badge.svg?token=cowZtK1KK1)](https://codecov.io/gh/winksaville/rust-binance-cli)

> **Note: In no case can the authors of this program be held responsible
> for any damanges or monetary losses.**

## Table of Contents

- [Introduction](#introduction)
- [Prerequistites](#prerequistites)
- [Build and run](#build-and-run)
- [Debug](#debug)
- [Test](#test)
- [Introduction](#introduction)
- [Code coverage](#code-coverage)
- [Before committing](#before-committing)
- [Other things](#other-things)
- [License](#license)

## Introduction

This program provides a command line interface to some of the
binance.us API's. It also Includes some higher level capabilities
such as automatically buying and selling of assets. Eventually other
higher level capabilities maybe provided.

Also, there are some **really dangerous** subcommands, such as
`auto-sell` and `auto-buy`. The `auto-sell` is particularly **DANGEROUS**
because, by default, it will **SELL** all of your assets.

There is an attempt to mitigate this behavior. The default configuration
has test mode enabled and it should not do any trades. Also, when not in
test mode the program will prompt you to be sure you want to proceed.
You must type, Yes, with a capital "Y" to proceed. A convenient parameter
is `--no-test`, this allows you to leave the default configuration with
`test = true` and then when you're ready to do trades, automatic or
manual, pass the `--no-test` flag.

As mentioned above the program uses a configuration file. There are
comments, lines with "#" in them, that provide additional information
about each field. Refer to that file for up-to-date information.

A suggestion is to copy `config.toml` to a `configs` diretory and then
have your keys in the config file. See `config.toml` for more options.

Finally, use the `help` subcommand or `--help` or `-h` flags as a
source of information while using the program.

```
wink@3900x:~/prgs/rust/myrepos/binance-cli (main)
$ cargo run help
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/binance-cli help`
binance-cli 0.6.2-2df56c3
Binance cli app

USAGE:
    binance-cli [OPTIONS] [SUBCOMMAND]

OPTIONS:
        --api-key <API-KEY>
            Define the api key [env: BINANCE_API_KEY=]

    -c, --config <FILE>
            Sets a custom config file [env: BINANCE_CONFIG=] [default: configs/config.toml]

        --confirmation-required
            Enable comfirmation being required

        --default-quote-asset <ASSET>
            The name of the asset that is used to buy or sell another asset

        --domain <BINANCE_DOMAIN>
            Domain such as binance.us or binance.com

    -h, --help
            Print help information

        --no-confirmation-required
            Disable comfirmation being required

        --no-progress-info
            Disable progress info

        --no-test
            Disable test mode

        --no-verbose
            Disable verbose mode

        --order-log-path <PATH>
            Define order log path

        --progress-info
            Enable progress info

        --scheme <BINANCE_SCHEME>
            Scheme such as https

        --secret-key <SECRET-KEY>
            Define the secret key [env: BINANCE_SECRET_KEY=]

    -t, --test
            Enable test mode

        --throttle-rate-ms <IN_MILLISECS>
            Throttle some requests, such as converting. [default: 500]

    -V, --version
            Print version information

        --verbose
            Enable verbose mode

SUBCOMMANDS:
    ai                   Display the account info
    auto-buy             Automatically buy assets as defined in the configuration buy section
    auto-sell            Automatically sell assets as defined in the configuration keep section
    buy-market           Buy a number of assets
    buy-market-value     Buy asset using quote asset value
    sell-market          Sell a number of assets
    sell-market-value    Sell asset using quote asset value
    withdraw             Withdraw an asset, either quantity, dollars or precent.
                             Examples:
                               withdraw ETH '$1000' 1543abcd --keep-min \$200
                               withdraw ETH 100% 1543abcd --keep-min '$200'
                               withdraw ETH 100 1543abcd
                              NOTE: Dollar values must be written
                              in single quotes '$123' or with a backslash \$1234
    dh                   Display deposit history
    fcdh                 Display fiat currency deposit history
    fcwh                 Display fiat currency withdraw history
    wh                   Display withdrawal history
    mt                   Display my trades for a symbol
    ao                   Dispaly all orders
    cbcthf               consolidate binance.com trade history files
    cbudf                consolidate binance.us distribution files
    cttf                 consolidate Token Tax files
    obid                 Order by id
    ol                   Display order log
    pbcthf               process binance.com trade history files
    pbudf                process binance.us distribution files
    pol                  process order log
    ptbf                 process Tax Bit file
    pttf                 process Token Tax files
    tbffttf              TaxBit file from Token Tax file
    ttffbcthf            Token Tax file from binance.com trade history files
    ttffbudf             Token Tax file from binance.us distribution files
    ubudf                update binance.us distribution files
    ucttf                uniq currency transactions in Token Tax files
    ei                   Display the exchange info
    oo                   Display a symbols open orders
    sap                  Display a symbols 5 minute average price
    sei                  Display a symbols exchange information
    skr                  Display a symbols current kline record
    skrs                 Display a symbols kline records
    version              Display version
    check-params         Used for testing
    help                 Print this message or the help of the given subcommand(s)
```

## Prerequisites

1.  Install git, this is platform specific [here](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
    is a place to start.

1.  Clone this repo, for instance:
    ```
    git clone https://github.com/winksaville/rust-binance-cli
    ```

1.  [Install rust](https://www.rust-lang.org/tools/install) on your computer,
    this needs to include the `rust compiler` and `cargo`. Typically this
    should be done using `rustup`.

    > Note: rustup must be 1.24.1 as I'm using `rust-toolchain.toml` for rustup
    > configuration.
    (Add more specific docs here)

1.  Install a version of Tarpaulin >= `0.20.0` as I'm using the --follow-exec option,
    currently I'm installing it with with:
    ```
    cargo install cargo-tarpaulin
    ```

    You should verify `follow-exec` is in the help:
    ```
    cargo tarpaulin --help | grep 'follow-exe'
            --follow-exec            Follow executed processes capturing coverage information if they're part of your
    ```

1.  Copy `cargo-precommit` to `~/.cargo/bin/`
    ```
    cp ./cargo-precommit ~/.cargo/bin/
    ```

1.  Copy `config.toml` to `configs/`
    ```
    wink@3900x:~/prgs/rust/projects/binance-cli (main)
    mkdir configs
    cp config.toml configs/
    ```

1.  Edit configs/config.toml:
    ```
    <editor> configs/config.toml
    ```

## Build and run

Build
```
wink@3900x:~/prgs/rust/projects/binance-cli (main)
$ cargo clean ; cargo build
   Compiling libc v0.2.94
   Compiling proc-macro2 v1.0.26
   Compiling unicode-xid v0.2.2
   Compiling autocfg v1.0.1
   Compiling syn v1.0.71
...
   Compiling hyper-tls v0.5.0
   Compiling reqwest v0.11.3
    Finished dev [unoptimized + debuginfo] target(s) in 25.10s
```

Run with no parameters
```
wink@3900x:~/prgs/rust/myrepos/binance-cli (main)
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/binance-cli`
Usage:   binance-cli help, --help or -h
app-ver: 0.3.0-ca767f1
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
To see only "binance-cli" traces use `RUST_LOG=binance_cli=trace cargo run`.
As a small real example, to see only traces from the configuration module add
`RUST_LOG=binance_cli::configuration=trace`:
```
wink@3900x:~/prgs/rust/projects/binance-cli (main)
$ RUST_LOG=binance_cli::configuration=trace cargo run ai -c config.toml
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/binance-cli ai -c config.toml`
[2021-06-02T20:14:04Z TRACE binance_cli::configuration] Configuration::new: opt_config=Some(
        "config.toml",
    )
Error: "InternalErrorRec: app-ver: 0.2.0-a6cd35a file: src/configuration.rs line:234 code: 9 msg: Error processing config.toml: invalid TOML value, did you mean to use a quoted string? at line 68 column 5"
```
The above error is expected, to limit the possibility that someone
could accidentally use the example `config.toml` file.

I've also dabbled using vscode debugger and there is a
`.vscode/launch.json` file, but YMMV.


## Test

Test using `cargo test`:
```
wink@3900x:~/prgs/rust/projects/binance-cli (main)
$ cargo test
    Finished test [unoptimized + debuginfo] target(s) in 0.05s
     Running target/debug/deps/binance_cli-27b6fa3c5914ceca

running 41 tests
test binance_history::test::test_history_rec ... ok
test binance_my_trades::test::test_trade_rec ... ok
...
test binance_klines::test::test_kline_rec ... ok
test binance_trade::test::test_convertcommission ... ok
test binance_trade::test::test_convert ... ok

test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s

     Running target/debug/deps/cli-1bc2ccfeadf46ce1

running 4 tests
test test_no_params ... ok
test test_help ... ok
test test_req_params ... ok
test test_req_params_as_env_vars ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Code coverage

> Note: tarpaulin is being used for code coverage, but you must use 0.18.0+.
> Because of [issue #1 in this repo](https://github.com/winksaville/rust-binance-cli/issues/1)
> when you want to run `cargo build|run|test` after `cargo tarpaulin` you
> must do a `cargo clean` first.

So the first time run `cargo tarpaulin`:
```
wink@3900x:~/prgs/rust/projects/binance-cli (main)
$ cargo tarpaulin
Jun 02 13:31:02.703  INFO cargo_tarpaulin: Running Tarpaulin
Jun 02 13:31:02.703  INFO cargo_tarpaulin: Building project
Jun 02 13:31:02.981  INFO cargo_tarpaulin::cargo: Cleaning project
   Compiling libc v0.2.94
   Compiling autocfg v1.0.1
   Compiling proc-macro2 v1.0.26
...
   Compiling reqwest v0.11.3
    Finished test [unoptimized + debuginfo] target(s) in 27.70s
Jun 02 13:31:30.789  INFO cargo_tarpaulin::process_handling::linux: Launching test
Jun 02 13:31:30.789  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-cli/target/debug/deps/cli-1bc2ccfeadf46ce1

running 4 tests
test test_req_params_as_env_vars ... ok
test test_req_params ... ok
test test_help ... ok
test test_no_params ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

Jun 02 13:31:31.986  INFO cargo_tarpaulin::process_handling::linux: Launching test
Jun 02 13:31:31.986  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-cli/target/debug/deps/binance_cli-27b6fa3c5914ceca

running 41 tests
test binance_history::test::test_history_rec ... ok
test binance_signature::test::test_query_vec_u8_no_data ... ok
...
test binance_trade::test::test_convertcommission ... ok
test binance_trade::test::test_convert ... ok

test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s

Jun 02 13:31:37.136  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src/arg_matches.rs: 0/63
|| src/binance_account_info.rs: 27/113
|| src/binance_auto_buy.rs: 0/87
|| src/binance_auto_sell.rs: 0/120
|| src/binance_avg_price.rs: 0/16
|| src/binance_exchange_info.rs: 202/271
|| src/binance_get_klines_cmd.rs: 0/21
|| src/binance_history.rs: 1/63
|| src/binance_klines.rs: 59/129
|| src/binance_market_order_cmd.rs: 0/70
|| src/binance_my_trades.rs: 25/76
|| src/binance_order_response.rs: 134/225
|| src/binance_orders.rs: 0/64
|| src/binance_signature.rs: 114/114
|| src/binance_trade.rs: 55/185
|| src/binance_verify_order.rs: 59/116
|| src/common.rs: 100/173
|| src/configuration.rs: 114/176
|| src/de_string_or_number.rs: 60/75
|| src/main.rs: 0/158
|| 
41.04% coverage, 950/2315 lines covered
```

For subsequent runs you can use `--skip-clean` to save time:
```
wink@3900x:~/prgs/rust/projects/binance-cli (main)
$ cargo tarpaulin --skip-clean
Jun 02 13:32:45.299  INFO cargo_tarpaulin: Running Tarpaulin
Jun 02 13:32:45.299  INFO cargo_tarpaulin: Building project
    Finished test [unoptimized + debuginfo] target(s) in 0.05s
Jun 02 13:32:45.523  INFO cargo_tarpaulin::process_handling::linux: Launching test
Jun 02 13:32:45.523  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-cli/target/debug/deps/binance_cli-27b6fa3c5914ceca

running 41 tests
test binance_history::test::test_history_rec ... ok
test binance_signature::test::test_query_vec_u8_no_data ... ok
...
test binance_trade::test::test_convert ... ok
test binance_trade::test::test_convertcommission ... ok

test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s

Jun 02 13:32:50.679  INFO cargo_tarpaulin::process_handling::linux: Launching test
Jun 02 13:32:50.679  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-cli/target/debug/deps/cli-1bc2ccfeadf46ce1

running 4 tests
test test_req_params_as_env_vars ... ok
test test_req_params ... ok
test test_help ... ok
test test_no_params ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

Jun 02 13:32:51.852  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src/arg_matches.rs: 0/63 +0%
|| src/binance_account_info.rs: 27/113 +0%
...
|| src/configuration.rs: 114/176 +0%
|| src/de_string_or_number.rs: 60/75 +0%
|| src/main.rs: 0/158 +0%
|| 
41.04% coverage, 950/2315 lines covered, +0% change in coverage

```

**But**, as mentioned above, run `cargo build|run|test`
commands the first one will need to be preceeded by a `cargo clean`
otherwise you'll get an error from the linker:
```
wink@3900x:~/prgs/rust/projects/binance-cli (main)
$ cargo run
   Compiling libc v0.2.94
   Compiling proc-macro2 v1.0.26
   Compiling unicode-xid v0.2.2
...
          /home/wink/.cargo/registry/src/github.com-1ecc6299db9ec823/serde_json-1.0.64/src/de.rs:1725: undefined reference to `core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>'
          collect2: error: ld returned 1 exit status
          

error: aborting due to previous error

error: could not compile `binance-cli`

To learn more, run the command again with --verbose.

```

Here is what you need to do:
```
wink@3900x:~/prgs/rust/projects/binance-cli (main)
$ cargo clean ; cargo run
   Compiling libc v0.2.94
   Compiling proc-macro2 v1.0.26
   Compiling unicode-xid v0.2.2
...
   Compiling hyper-tls v0.5.0
   Compiling reqwest v0.11.3
    Finished dev [unoptimized + debuginfo] target(s) in 24.96s
     Running `target/debug/binance-cli`
Usage:   binance-cli help, --help or -h
app-ver: 0.2.0-a6cd35a
```

And, of course, subquent runs don't need the `cargo clean`:
```
wink@3900x:~/prgs/rust/projects/binance-cli (main)
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
```

## Before committing

If you like to submit a PR please Run `cargo pre-commit` before uploading.
It runs cargo check, fmt, test, tarpaulin and clippy.

> Note: to run `cargo pre-commit` you must have previously
> copied the `./cargo-pre-commit` to `~/.cargo/bin`.

```
$ cargo pre-commit
check
    Checking binance-cli v0.2.0 (/home/wink/prgs/rust/projects/binance-cli)
    Finished dev [unoptimized + debuginfo] target(s) in 0.81s
fmt
test
    Finished test [unoptimized + debuginfo] target(s) in 0.05s
     Running target/debug/deps/binance_cli-27b6fa3c5914ceca

running 41 tests
test binance_history::test::test_history_rec ... ok
test binance_my_trades::test::test_trade_rec ... ok
test binance_klines::test::test_kline_rec ... ok

...

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

tarpaulin
Jun 02 13:20:40.281  INFO cargo_tarpaulin: Running Tarpaulin
Jun 02 13:20:40.281  INFO cargo_tarpaulin: Building project
Jun 02 13:20:40.561  INFO cargo_tarpaulin::cargo: Cleaning project
   Compiling libc v0.2.94

...

test binance_trade::test::test_convert ... ok
test binance_trade::test::test_convertcommission ... ok

test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

Jun 02 13:21:14.481  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src/arg_matches.rs: 0/63
|| src/binance_account_info.rs: 27/113
...
|| src/configuration.rs: 114/176
|| src/de_string_or_number.rs: 60/75
|| src/main.rs: 0/158
|| 
41.04% coverage, 950/2315 lines covered
clean
clippy
   Compiling libc v0.2.94
   Compiling proc-macro2 v1.0.26
   Compiling unicode-xid v0.2.2

...

   Compiling binance-cli v0.2.0 (/home/wink/prgs/rust/projects/binance-cli)
    Checking hyper-tls v0.5.0
    Checking reqwest v0.11.3
   Compiling rust_decimal_macros v1.12.4
    Checking rusty-money v0.4.1
    Finished dev [unoptimized + debuginfo] target(s) in 18.03s
```

## Other things

In the root of the project are other scrpts that combine a series of command
to produce "processed" results. Currently there are two, `b.com.cttf.sh` and
`b.us.cttf.sh`. These two scripts take a single file as input, in this case
a "raw" binance.com trade history and binance.us distrubution history
respectively. These do not need to be sorted and can be just the concatenated
"raw" files, they will be sorted during the processing. Both of these scripts
produce a "consolidated TokenTax CSV file".

This is provided as an example, but allowed a set of 7M+ transactions to be
consolidated to 13K transactions.

Doing this type of thing produced
[Provide a way to validate transformations, Issue #11](https://github.com/winksaville/rust-binance-cli/issues/11)
which I'll hopefull solve at some point in the future.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
