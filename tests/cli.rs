use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::process::Command;

#[test]
fn test_hello_world() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("binance-auto-sell")?;

    cmd.assert()
        .stdout(predicate::str::contains("Hello, world!"));

    Ok(())
}
