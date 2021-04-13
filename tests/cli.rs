use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::process::Command;

#[test]
fn test_no_params() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("binance-auto-sell")?;

    cmd.assert()
        .code(predicate::eq(2))
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ));

    Ok(())
}

#[test]
fn test_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("binance-auto-sell")?;
    cmd.arg("-h");

    cmd.assert()
        .code(predicate::eq(0))
        .stdout(predicate::str::contains("USAGE"));

    Ok(())
}

#[test]
fn test_req_params() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("binance-auto-sell")?;
    cmd.arg("-s=secret_key");
    cmd.arg("-a").arg("api key");

    cmd.assert()
        .code(predicate::eq(0))
        .stdout(predicate::str::contains(
            "sec_key=secret key is never displayed",
        ))
        .stdout(predicate::str::contains("api_key=api key"));

    Ok(())
}
#[test]
fn test_req_params_as_env_vars() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("binance-auto-sell")?;
    cmd.env("SECRET_KEY", "secret key");
    cmd.env("API_KEY", "api key");

    cmd.assert()
        .code(predicate::eq(0))
        .stdout(predicate::str::contains(
            "sec_key=secret key is never displayed",
        ))
        .stdout(predicate::str::contains("api_key=api key"));

    Ok(())
}
