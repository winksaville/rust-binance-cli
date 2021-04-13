# Binance auto sell

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

## Building and run

```
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s

$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/binance-auto-sell`
Hello, world!
```

## Test

```
$ cargo test
   Compiling binance-auto-sell v0.1.0 (/home/wink/prgs/rust/projects/binance-auto-sell)
    Finished test [unoptimized + debuginfo] target(s) in 0.39s
     Running target/debug/deps/binance_auto_sell-95189b5df5f0e54d

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running target/debug/deps/cli-957434e122fb0001

running 1 test
test test_hello_world ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s```
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
