use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::process::Command;

const APP_NAME: &str = "binance-cli";

// For some reason these tests are unreliable on tarpaulin when
// executed via github Actions. The cfg(not(tarpaulin)) causes
// them to be skipped.

#[test]
#[cfg(not(tarpaulin))]
fn test_no_params() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(APP_NAME)?;

    cmd.assert()
        .code(predicate::eq(0))
        .stdout(predicate::str::starts_with("Usage: "));

    Ok(())
}

// For some reason this is unreliable on tarpaulin, this should completely skip it.
#[test]
#[cfg(not(tarpaulin))]
fn test_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(APP_NAME)?;
    cmd.arg("-h");

    cmd.assert()
        .code(predicate::eq(0))
        .stdout(predicate::str::contains("USAGE"));

    Ok(())
}

#[test]
#[cfg(not(tarpaulin))]
fn test_params() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(APP_NAME)?;
    cmd.arg("check-params");
    cmd.arg("--secret-key=secret-key");
    cmd.arg("--api-key").arg("api key");

    //let p = cmd.output();
    //println!("{p:#?}");

    cmd.assert().code(predicate::eq(0));

    Ok(())
}

#[test]
#[cfg(not(tarpaulin))]
fn test_params_as_env_vars() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(APP_NAME)?;
    cmd.arg("check-params");
    cmd.env("BINANCE_SECRET_KEY", "secret-key");
    cmd.env("BINANCE_API_KEY", "api key");

    //let p = cmd.output();
    //println!("{p:#?}");

    cmd.assert().code(predicate::eq(0));

    Ok(())
}

#[test]
#[cfg(not(tarpaulin))]
fn test_params_failure() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(APP_NAME)?;
    cmd.arg("check-params");
    cmd.arg("--secret-key="); // Previously the default for keys was that they were None.
                              // Now we'll force it to be empty and check the error.
    cmd.arg("--api-key").arg("api key");

    //let p = cmd.output();
    //println!("{p:#?}");

    // Should fail because --secret-key is missing
    cmd.assert().code(predicate::eq(1));

    Ok(())
}
