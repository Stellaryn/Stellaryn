#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use stellar_xdr::{
    Limits, ScSpecEntry, ScSpecFunctionInputV0, ScSpecFunctionV0, ScSpecTypeDef, ScSymbol,
    WriteXdr,
};
use tempfile::TempDir;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stellaryn"))
}

fn leb_u32(bytes: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut item = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            item |= 0x80;
        }
        bytes.push(item);
        if value == 0 {
            break;
        }
    }
}

fn contract_wasm(parameter_name: &str, parameter_type: ScSpecTypeDef) -> Vec<u8> {
    let entry = ScSpecEntry::FunctionV0(ScSpecFunctionV0 {
        doc: "".try_into().unwrap(),
        name: ScSymbol("pay".try_into().unwrap()),
        inputs: vec![ScSpecFunctionInputV0 {
            doc: "".try_into().unwrap(),
            name: parameter_name.try_into().unwrap(),
            type_: parameter_type,
        }]
        .try_into()
        .unwrap(),
        outputs: vec![ScSpecTypeDef::Bool].try_into().unwrap(),
    });
    let section = b"contractspecv0";
    let xdr = entry.to_xdr(Limits::none()).unwrap();
    let mut payload = Vec::new();
    leb_u32(&mut payload, section.len() as u32);
    payload.extend_from_slice(section);
    payload.extend_from_slice(&xdr);

    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.push(0);
    leb_u32(&mut wasm, payload.len() as u32);
    wasm.extend_from_slice(&payload);
    wasm
}

fn write_fixture(folder: &Path, name: &str, parameter_name: &str, ty: ScSpecTypeDef) -> String {
    let path = folder.join(name);
    fs::write(&path, contract_wasm(parameter_name, ty)).unwrap();
    path.to_str().unwrap().to_owned()
}

fn invoke(before: &str, after: &str, additional: &[&str]) -> Output {
    let mut command = binary();
    command.arg("compare").arg(before).arg(after).args(additional);
    command.output().unwrap()
}

#[test]
fn compare_help_describes_formats_and_exit_policy() {
    let result = binary().args(["compare", "--help"]).output().unwrap();
    assert!(result.status.success());
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("--fail-on"));
    assert!(stdout.contains("BEFORE"));
    assert!(stdout.contains("AFTER"));
}

#[test]
fn identical_valid_wasm_returns_success_and_no_findings() {
    let folder = TempDir::new().unwrap();
    let path = write_fixture(folder.path(), "same.wasm", "amount", ScSpecTypeDef::I128);
    let result = invoke(&path, &path, &[]);
    assert_eq!(result.status.code(), Some(0));
    assert!(result.stderr.is_empty());
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stdout.contains("Overall verdict: COMPATIBLE"));
    assert!(stdout.contains("0 total"));
    assert!(stdout.contains("No public contract-spec compatibility changes detected."));
    assert!(stdout.contains("not a security audit"));
}

#[test]
fn breaking_wasm_change_returns_policy_exit_two_and_complete_report() {
    let folder = TempDir::new().unwrap();
    let before = write_fixture(folder.path(), "before.wasm", "amount", ScSpecTypeDef::I128);
    let after = write_fixture(folder.path(), "after.wasm", "amount", ScSpecTypeDef::U128);
    let result = invoke(&before, &after, &[]);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stderr.is_empty());
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stdout.contains("Overall verdict: INCOMPATIBLE"));
    assert!(stdout.contains("FUNCTION_PARAMETER_TYPE_CHANGED"));
    assert!(stdout.contains("i128"));
    assert!(stdout.contains("u128"));
}

#[test]
fn json_on_policy_violation_is_parseable_and_stdout_only() {
    let folder = TempDir::new().unwrap();
    let before = write_fixture(folder.path(), "old.wasm", "amount", ScSpecTypeDef::I128);
    let after = write_fixture(folder.path(), "new.wasm", "amount", ScSpecTypeDef::U128);
    let result = invoke(&before, &after, &["--format", "json"]);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stderr.is_empty());
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["schema_version"], "1.0");
    assert_eq!(json["before"], before);
    assert_eq!(json["after"], after);
    assert_eq!(json["analysis"]["verdict"], "INCOMPATIBLE");
    assert_eq!(json["analysis"]["totals"]["breaking"], 1);
    assert_eq!(
        json["analysis"]["findings"][0]["rule"]["id"],
        "FUNCTION_PARAMETER_TYPE_CHANGED"
    );
}

#[test]
fn json_output_is_deterministic_for_repeated_comparisons() {
    let folder = TempDir::new().unwrap();
    let before = write_fixture(folder.path(), "from.wasm", "amount", ScSpecTypeDef::I128);
    let after = write_fixture(folder.path(), "to.wasm", "amount", ScSpecTypeDef::U128);
    let first = invoke(&before, &after, &["--format", "json"]);
    let second = invoke(&before, &after, &["--format", "json"]);
    assert_eq!(first.status.code(), Some(2));
    assert_eq!(second.status.code(), Some(2));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn review_rename_passes_default_but_fails_on_review() {
    let folder = TempDir::new().unwrap();
    let before = write_fixture(folder.path(), "old.wasm", "amount", ScSpecTypeDef::I128);
    let after = write_fixture(folder.path(), "new.wasm", "value", ScSpecTypeDef::I128);
    let default = invoke(&before, &after, &["--format", "json"]);
    assert_eq!(default.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&default.stdout).unwrap();
    assert_eq!(json["analysis"]["verdict"], "REVIEW_REQUIRED");
    let strict = invoke(&before, &after, &["--fail-on", "review"]);
    assert_eq!(strict.status.code(), Some(2));
    assert!(String::from_utf8(strict.stdout).unwrap().contains("REVIEW_REQUIRED"));
}

#[test]
fn never_policy_allows_a_breaking_finding_without_changing_the_verdict() {
    let folder = TempDir::new().unwrap();
    let before = write_fixture(folder.path(), "old.wasm", "amount", ScSpecTypeDef::I128);
    let after = write_fixture(folder.path(), "new.wasm", "amount", ScSpecTypeDef::U128);
    let result = invoke(&before, &after, &["--fail-on", "never", "--format", "json"]);
    assert_eq!(result.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["analysis"]["verdict"], "INCOMPATIBLE");
}

#[test]
fn invalid_wasm_before_returns_error_without_json() {
    let folder = TempDir::new().unwrap();
    let before = folder.path().join("invalid.wasm");
    fs::write(&before, b"not-wasm").unwrap();
    let after = write_fixture(folder.path(), "new.wasm", "amount", ScSpecTypeDef::U128);
    let result = invoke(before.to_str().unwrap(), &after, &["--format", "json", "--fail-on", "never"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8(result.stderr)
        .unwrap()
        .contains("before-contract specification"));
}

#[test]
fn missing_new_wasm_returns_analysis_error_even_under_never() {
    let folder = TempDir::new().unwrap();
    let before = write_fixture(folder.path(), "old.wasm", "amount", ScSpecTypeDef::I128);
    let missing = folder.path().join("missing.wasm");
    let result = invoke(&before, missing.to_str().unwrap(), &["--fail-on", "never"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8(result.stderr)
        .unwrap()
        .contains("after-contract specification"));
}

#[test]
fn wasm_without_contract_spec_is_not_misclassified_compatible() {
    let folder = TempDir::new().unwrap();
    let empty = folder.path().join("empty.wasm");
    fs::write(&empty, b"\0asm\x01\0\0\0").unwrap();
    let after = write_fixture(folder.path(), "new.wasm", "amount", ScSpecTypeDef::I128);
    let result = invoke(empty.to_str().unwrap(), &after, &["--format", "json"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
}

#[test]
fn malformed_cli_format_and_policy_arguments_fail_without_results() {
    let folder = TempDir::new().unwrap();
    let path = write_fixture(folder.path(), "old.wasm", "amount", ScSpecTypeDef::I128);
    for args in [
        vec!["--format", "xml"],
        vec!["--fail-on", "anything"],
    ] {
        let result = invoke(&path, &path, &args);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
}

#[test]
fn filenames_with_spaces_are_supported_without_a_shell() {
    let folder = TempDir::new().unwrap();
    let before = write_fixture(folder.path(), "old contract.wasm", "amount", ScSpecTypeDef::I128);
    let after = write_fixture(folder.path(), "new contract.wasm", "amount", ScSpecTypeDef::I128);
    let result = invoke(&before, &after, &[]);
    assert_eq!(result.status.code(), Some(0));
}
