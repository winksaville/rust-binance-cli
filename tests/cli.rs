use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::process::Command;

const APP_NAME: &str = "binance-cli";

#[test]
#[cfg(not(tarpaulin_include))]
fn test_no_params() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(APP_NAME)?;

    cmd.assert()
        .code(predicate::eq(0))
        .stdout(predicate::str::starts_with("Usage: "));

    Ok(())
}

#[test]
#[cfg(not(tarpaulin_include))]
fn test_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(APP_NAME)?;
    cmd.arg("-h");

    cmd.assert()
        .code(predicate::eq(0))
        .stdout(predicate::str::contains("USAGE"));

    Ok(())
}

#[test]
#[cfg(not(tarpaulin_include))]
fn test_req_params() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(APP_NAME)?;
    cmd.arg("do-nothing");
    cmd.arg("--secret-key=secret_key");
    cmd.arg("--api-key").arg("api key");

    cmd.assert().code(predicate::eq(0));

    Ok(())
}
#[test]
#[cfg(not(tarpaulin_include))]
fn test_req_params_as_env_vars() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(APP_NAME)?;
    cmd.arg("do-nothing");
    cmd.env("SECRET_KEY", "secret key");
    cmd.env("API_KEY", "api key");

    cmd.assert().code(predicate::eq(0));

    Ok(())
}
