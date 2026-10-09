#![allow(clippy::unwrap_used)]

use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stellaryn"))
}

#[test]
fn help_is_honest_about_phase_one_scope() {
    let output = binary().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Stellaryn compares Soroban contract interfaces"));
    assert!(stdout.contains("No compatibility result is produced in Phase 1"));
    assert!(stdout.contains("not a security audit"));
}

#[test]
fn version_is_available() {
    let output = binary().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("0.1.0-alpha.1"));
}

#[test]
fn product_info_is_machine_friendly() {
    let output = binary().arg("--product-info").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("product=Stellaryn"));
    assert!(stdout.contains("status=foundation"));
}
