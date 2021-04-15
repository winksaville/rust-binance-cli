# Binance auto sell
[![codecov](https://codecov.io/gh/winksaville/rust-binance-auto-sell/branch/main/graph/badge.svg?token=5l3L7yVGTj)](https://codecov.io/gh/winksaville/rust-binance-auto-sell)

Automatically sell all assets owned by the user on binance.us
except USD, USDT plus a minimum of BNB is kept.

Currently this is just the `cargo new` "Hello, World!" program:
```
$ cargo run
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished dev [unoptimized + debuginfo] target(s) in 0.18s
     Running `target/debug/binance-auto-sell`
Hello, world!
```

## Prerequisites

Along with the normal rust tools installed via `rustup` and `cargo install`.
(Add more specific docs here)

You must install a version >= `0.18.0-alpha1`, currently I'm installing with:
```
$ cargo install --git https://github.com/xd009642/tarpaulin.git --branch develop cargo-tarpaulin
```

You should verify `follow-exec` is in the help:
```
$ cargo tarpaulin --help | grep 'follow-exe'
        --follow-exec            Follow executed processes capturing coverage information if they're part of your
```

## Building and run

```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s

wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo run -- -s the-secret-key -a the-api-key
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/binance-auto-sell -s the-secret-key -a the-api-key`
sec_key=secret key is never displayed api_key=the-api-key
```

## Test

Test using `cargo test`:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo test
    Finished test [unoptimized + debuginfo] target(s) in 0.01s
     Running target/debug/deps/binance_auto_sell-049bc8b772cabc65

running 6 tests
test de_string_or_number::tests::test_de_sting_or_number_to_f64_errors ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_i64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_strings ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_u64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_numbers ... ok
test exchange_info::test::test_exchange_info ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running target/debug/deps/cli-ce0518d68c450e5d

running 4 tests
test test_req_params ... ok
test test_no_params ... ok
test test_help ... ok
test test_req_params_as_env_vars ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Code coverage

> Note: tarpaulin is being used for code coverage, but you must use 0.18.0+.
> Also, to run tarpaulin you need to clean the repo first otherwise
> you may see problems described in [issue #1 in this repo](https://github.com/winksaville/rust-binance-auto-sell/issues/1)
> and [issue #736 in Tarpaulin repo](https://github.com/xd009642/tarpaulin/issues/736). 

So the first time run `cargo clean && cargo tarpaulin`:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo clean ; cargo tarpaulin
Apr 15 09:37:21.936  INFO cargo_tarpaulin: Running config all
Apr 15 09:37:21.936  INFO cargo_tarpaulin: Running Tarpaulin
Apr 15 09:37:21.936  INFO cargo_tarpaulin: Building project
   Compiling proc-macro2 v1.0.26
   Compiling unicode-xid v0.2.1
...
   Compiling clap v3.0.0-beta.2
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished test [unoptimized + debuginfo] target(s) in 12.37s
Apr 15 09:37:34.553  INFO cargo_tarpaulin::process_handling::linux: Launching test
Apr 15 09:37:34.553  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/cli-ce0518d68c450e5d

running 4 tests
test test_req_params_as_env_vars ... ok
test test_req_params ... ok
test test_no_params ... ok
test test_help ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.83s

Apr 15 09:37:38.409  INFO cargo_tarpaulin::process_handling::linux: Launching test
Apr 15 09:37:38.409  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/binance_auto_sell-049bc8b772cabc65

running 6 tests
test de_string_or_number::tests::test_de_sting_or_number_to_u64_errors ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_i64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_strings ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_f64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_numbers ... ok
test exchange_info::test::test_exchange_info ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

Apr 15 09:37:39.234  INFO cargo_tarpaulin::report: Coverage Results:
|| Uncovered Lines:
|| Tested/Total Lines:
|| src/de_string_or_number.rs: 18/18
|| src/main.rs: 6/6
|| 
100.00% coverage, 24/24 lines covered
```

Subsequent runs clean is not necessary:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo tarpaulin
Apr 15 09:37:50.434  INFO cargo_tarpaulin: Running config all
Apr 15 09:37:50.434  INFO cargo_tarpaulin: Running Tarpaulin
Apr 15 09:37:50.434  INFO cargo_tarpaulin: Building project
    Finished test [unoptimized + debuginfo] target(s) in 0.01s
Apr 15 09:37:50.499  INFO cargo_tarpaulin::process_handling::linux: Launching test
Apr 15 09:37:50.499  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/cli-ce0518d68c450e5d

running 4 tests
test test_no_params ... ok
test test_help ... ok
test test_req_params_as_env_vars ... ok
test test_req_params ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.82s

Apr 15 09:37:54.346  INFO cargo_tarpaulin::process_handling::linux: Launching test
Apr 15 09:37:54.346  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/binance_auto_sell-049bc8b772cabc65

running 6 tests
test de_string_or_number::tests::test_de_sting_or_number_to_i64_errors ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_u64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_strings ... ok
test de_string_or_number::tests::test_de_sting_or_number_to_f64_errors ... ok
test de_string_or_number::tests::test_de_string_or_number_from_numbers ... ok
test exchange_info::test::test_exchange_info ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

Apr 15 09:37:55.164  INFO cargo_tarpaulin::report: Coverage Results:
|| Uncovered Lines:
|| Tested/Total Lines:
|| src/de_string_or_number.rs: 18/18 +0%
|| src/main.rs: 6/6 +0%
|| 
100.00% coverage, 24/24 lines covered, +0% change in coverage
```

**But**, when you want to run any other of the `cargo build|run|test`
commands the first one will also need to be preceeded by a `cargo clean`.
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo clean ; cargo run -- -s mysecretkey -a myapikey
   Compiling proc-macro2 v1.0.26
   Compiling unicode-xid v0.2.1
...
   Compiling clap v3.0.0-beta.2
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished dev [unoptimized + debuginfo] target(s) in 10.88s
     Running `target/debug/binance-auto-sell -s mysecretkey -a myapikey`
sec_key=secret key is never displayed api_key=myapikey
```

For subseqent `build|run|test`'s it is not necessary:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
```

## TODO:

Using nightly-2021-03-25 as that is the last rustfmt that works as
shown at https://rust-lang.github.io/rustup-components-history/.
Change this when rustfmt is fixed.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
