#![allow(clippy::unwrap_used)]

#[path = "fixtures/mod.rs"]
mod fixtures;

use fixtures::*;
use std::{fs, process::Command};
use stellar_xdr::ScSpecTypeDef;
use tempfile::TempDir;

fn run(before: &[u8], after: &[u8]) -> std::process::Output {
    let dir = TempDir::new().unwrap();
    let old = dir.path().join("before.wasm");
    let new = dir.path().join("after.wasm");
    fs::write(&old, before).unwrap();
    fs::write(&new, after).unwrap();
    Command::new(env!("CARGO_BIN_EXE_stellaryn"))
        .arg("compare")
        .arg(&old)
        .arg(&new)
        .args(["--format", "json", "--fail-on", "never"])
        .output()
        .unwrap()
}

fn assert_failure(base: &[u8], bad: &[u8], expected: &str) {
    let result = run(base, bad);
    assert_eq!(
        result.status.code(),
        Some(1),
        "expected analysis failure for {expected}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        result.stdout.is_empty(),
        "invalid WASM must not return a compatibility verdict"
    );
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains(expected),
        "expected '{expected}' in diagnostic: {stderr}"
    );
}

#[test]
fn malformed_and_ambiguous_spec_sections_fail_closed() {
    let base = wasm(&baseline());
    let invalid_binary = b"not a wasm module".to_vec();
    let no_spec = missing_spec_wasm();
    let empty_section = wasm_raw_spec(&[]);
    let invalid_xdr = wasm_raw_spec(&[0xff, 0xff, 0xff, 0xff]);

    let mut truncated = base.clone();
    truncated.truncate(truncated.len() - 3);

    let mut bad_suffix = base.clone();
    bad_suffix.push(0xff);

    let mut duplicate_section = base.clone();
    let another = wasm(&baseline());
    duplicate_section.extend_from_slice(&another[8..]);

    for (name, artifact, expected) in [
        ("bad magic", invalid_binary, "reading wasm"),
        ("missing spec section", no_spec, "contract spec not found"),
        (
            "empty spec section",
            empty_section,
            "contains no contract specification entries",
        ),
        ("invalid XDR", invalid_xdr, "parsing contract spec"),
        ("truncated WASM", truncated, "reading wasm"),
        ("corrupt trailing section", bad_suffix, "reading wasm"),
        (
            "duplicate contract spec sections",
            duplicate_section,
            "multiple contractspecv0 sections",
        ),
    ] {
        let result = run(&base, &artifact);
        assert_eq!(
            result.status.code(),
            Some(1),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty(), "{name} yielded false confidence");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{name} yielded unexpected diagnostic: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn invalid_normalized_spec_entries_fail_closed() {
    let base_entries = baseline();
    let base = wasm(&base_entries);

    let mut duplicate_functions = base_entries.clone();
    duplicate_functions.push(base_entries[0].clone());
    assert_failure(&base, &wasm(&duplicate_functions), "duplicate function");

    let mut duplicate_fields = base_entries.clone();
    let item = struct_mut(&mut duplicate_fields);
    let mut fields: Vec<_> = item.fields.iter().cloned().collect();
    fields.push(struct_field("owner", ScSpecTypeDef::Address));
    item.fields = fields.try_into().unwrap();
    assert_failure(&base, &wasm(&duplicate_fields), "duplicate field");

    let mut duplicate_enum_numeric_values = base_entries.clone();
    let item = enum_mut(&mut duplicate_enum_numeric_values);
    let mut cases: Vec<_> = item.cases.iter().cloned().collect();
    cases[1].value = cases[0].value;
    item.cases = cases.try_into().unwrap();
    assert_failure(
        &base,
        &wasm(&duplicate_enum_numeric_values),
        "duplicate enum discriminant",
    );

    let mut duplicate_error_codes = base_entries.clone();
    let item = error_mut(&mut duplicate_error_codes);
    let mut cases: Vec<_> = item.cases.iter().cloned().collect();
    cases[1].value = cases[0].value;
    item.cases = cases.try_into().unwrap();
    assert_failure(
        &base,
        &wasm(&duplicate_error_codes),
        "duplicate error value",
    );

    let mut duplicate_event_parameters = base_entries.clone();
    let item = event_mut(&mut duplicate_event_parameters);
    let mut params: Vec<_> = item.params.iter().cloned().collect();
    params[1].name = "from".try_into().unwrap();
    item.params = params.try_into().unwrap();
    assert_failure(
        &base,
        &wasm(&duplicate_event_parameters),
        "duplicate parameter",
    );
}

#[test]
fn invalid_before_contract_never_emits_valid_after_analysis() {
    let base = wasm(&baseline());
    let empty = wasm_raw_spec(&[]);
    let result = run(&empty, &base);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("before-contract"));
}

#[test]
fn metadata_only_mutations_do_not_hide_real_type_changes() {
    let base = baseline();
    let mut changed = base.clone();
    function_mut(&mut changed).doc = "documentation-only alteration".try_into().unwrap();
    let mut args: Vec<_> = function_mut(&mut changed).inputs.iter().cloned().collect();
    args[1].type_ = ScSpecTypeDef::U128;
    function_mut(&mut changed).inputs = args.try_into().unwrap();
    let outcome = run(&wasm(&base), &wasm(&changed));
    assert_eq!(outcome.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&outcome.stdout).unwrap();
    assert_eq!(json["analysis"]["verdict"], "INCOMPATIBLE");
    assert_eq!(json["analysis"]["totals"]["breaking"], 1);
}

#[test]
fn empty_public_interface_is_not_mistaken_for_a_safe_upgrade() {
    let empty = wasm_raw_spec(&[]);
    let out = run(&empty, &empty);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
}

#[test]
fn only_unknown_custom_sections_do_not_count_as_a_contract_spec() {
    let mut wasm_module = missing_spec_wasm();
    let mut section = Vec::new();
    leb_u32(&mut section, 5);
    section.extend_from_slice(b"dummy");
    section.extend_from_slice(b"abc");
    wasm_module.push(0);
    leb_u32(&mut wasm_module, section.len() as u32);
    wasm_module.extend(section);
    let valid = wasm(&baseline());
    let out = run(&valid, &wasm_module);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
}

#[test]
fn same_valid_spec_with_extra_irrelevant_custom_section_is_compatible() {
    let input = wasm(&baseline());
    let mut extra = input.clone();
    let mut payload = Vec::new();
    leb_u32(&mut payload, 5);
    payload.extend_from_slice(b"dummy");
    payload.push(2);
    extra.push(0);
    leb_u32(&mut extra, payload.len() as u32);
    extra.extend(payload);

    let out = run(&input, &extra);
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["analysis"]["verdict"], "COMPATIBLE");
    assert_eq!(json["analysis"]["totals"]["breaking"], 0);
}
