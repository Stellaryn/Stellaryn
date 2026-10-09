#![allow(clippy::unwrap_used)]

use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stellaryn"))
}

#[test]
fn help_describes_available_comparison_and_scope() {
    let output = binary().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compare Soroban contract WASM files"));
    assert!(stdout.contains("Passing Stellaryn is not a security audit"));
    assert!(stdout.contains("not a security audit"));
}

#[test]
fn version_is_available() {
    let output = binary().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn product_info_is_machine_friendly() {
    let output = binary().arg("--product-info").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("product=Stellaryn"));
    assert!(stdout.contains(&format!("version={}", env!("CARGO_PKG_VERSION"))));
    assert!(stdout.contains("status=initial-release"));
}
