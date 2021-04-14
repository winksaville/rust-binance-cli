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

```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo test
    Finished test [unoptimized + debuginfo] target(s) in 0.01s
     Running target/debug/deps/binance_auto_sell-8b2f8d3614c3ece0

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running target/debug/deps/cli-595a799ae4fee77f

running 4 tests
test test_no_params ... ok
test test_req_params ... ok
test test_help ... ok
test test_req_params_as_env_vars ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Code coverage

> **There is a problem somewhere, if you've previously done a `cargo` `build`|`test`|`run`
> you must do a `cargo clean` before a `cargo tarpaulin` will succeed. And then you
> may get a `link` error if you subsequently do a `cargo build`. This too can be resolved
> by doing a `cargo clean` before the first subsequent `cargo build` :(**


Here is the error I'm seeing:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo tarpaulin --follow-exec
Apr 13 17:48:22.451  INFO cargo_tarpaulin: Running Tarpaulin
Apr 13 17:48:22.451  INFO cargo_tarpaulin: Building project
   Compiling autocfg v1.0.1
   Compiling proc-macro2 v1.0.26
...
   Compiling clap v3.0.0-beta.2
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
error: could not compile `binance-auto-sell`

To learn more, run the command again with --verbose.
warning: build failed, waiting for other jobs to finish...
thread 'main' panicked at 'already borrowed: BorrowMutError', src/tools/cargo/src/cargo/util/config/mod.rs:307:20
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Apr 13 17:48:30.124 ERROR cargo_tarpaulin: Failed to compile tests! Error: binance-auto-sell: linking with `cc` failed: exit code: 1
Error: "Failed to compile tests! Error: binance-auto-sell: linking with `cc` failed: exit code: 1"
```

After a `cargo clean` I see:
```
wink@3900x:~/prgs/rust/projects/binance-auto-sell (main)
$ cargo clean ; cargo tarpaulin --follow-exec
Apr 13 17:50:45.333  INFO cargo_tarpaulin: Running Tarpaulin
Apr 13 17:50:45.333  INFO cargo_tarpaulin: Building project
   Compiling autocfg v1.0.1
   Compiling proc-macro2 v1.0.26
   Compiling version_check v0.9.3
...
   Compiling clap v3.0.0-beta.2
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished test [unoptimized + debuginfo] target(s) in 8.25s
Apr 13 17:50:53.842  INFO cargo_tarpaulin::process_handling::linux: Launching test
Apr 13 17:50:53.842  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/binance_auto_sell-8b2f8d3614c3ece0

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

Apr 13 17:50:54.416  INFO cargo_tarpaulin::process_handling::linux: Launching test
Apr 13 17:50:54.416  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/cli-595a799ae4fee77f

running 4 tests
test test_no_params ... ok
test test_help ... ok
test test_req_params_as_env_vars ... ok
test test_req_params ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.30s

Apr 13 17:50:57.732  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src/main.rs: 6/6
|| 
100.00% coverage, 6/6 lines covered
```

## TODO:

Using nightly-2021-03-25 as that is the last rustfmt that works as
shown at https://rust-lang.github.io/rustup-components-history/.
Change this when rustfmt is fixed.

Fix problem with build issue of having to do `cargo cleans` before
and after doing `cargo tarpaulin`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
