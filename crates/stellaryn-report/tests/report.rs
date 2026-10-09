#![allow(clippy::unwrap_used)]

use stellaryn_core::{AnalysisSource, ContractInterface, Function, Parameter, TypeRef};
use stellaryn_diff::diff_contracts;
use stellaryn_report::{render_json, render_terminal, REPORT_SCHEMA_VERSION};

fn result() -> stellaryn_diff::ContractDiff {
    let before = ContractInterface::empty(AnalysisSource::WasmSpec);
    let mut after = before.clone();
    after.functions.push(Function {
        name: "balance".into(),
        doc: String::new(),
        parameters: vec![Parameter {
            name: "owner".into(),
            type_ref: TypeRef::primitive("Address"),
            doc: String::new(),
        }],
        outputs: vec![TypeRef::primitive("i128")],
    });
    diff_contracts(&before, &after).unwrap()
}

#[test]
fn terminal_renders_no_ansi_and_all_counts() {
    let output = render_terminal(&result(), "old.wasm", "new.wasm");
    assert!(output.starts_with("Stellaryn"));
    assert!(output.contains("Overall verdict: COMPATIBLE"));
    assert!(output.contains("1 total (0 breaking, 0 review required, 1 non-breaking)"));
    assert!(output.contains("function FUNCTION_ADDED"));
    assert!(output.contains("function:balance"));
    assert!(!output.contains("\u{1b}["));
    assert!(output.contains("not a security audit"));
}

#[test]
fn terminal_unchanged_is_explicit_without_claiming_deployment_safety() {
    let empty = ContractInterface::empty(AnalysisSource::WasmSpec);
    let analysis = diff_contracts(&empty, &empty).unwrap();
    let output = render_terminal(&analysis, "a.wasm", "a.wasm");
    assert!(output.contains("No public contract-spec compatibility changes detected."));
    assert!(output.contains("not a security audit"));
}

#[test]
fn json_is_parseable_deterministic_and_has_complete_analysis() {
    let analysis = result();
    let first = render_json(&analysis, "old.wasm", "new.wasm").unwrap();
    let second = render_json(&analysis, "old.wasm", "new.wasm").unwrap();
    assert_eq!(first, second);
    let value: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(value["schema_version"], REPORT_SCHEMA_VERSION);
    assert_eq!(value["before"], "old.wasm");
    assert_eq!(value["after"], "new.wasm");
    assert_eq!(value["analysis"]["verdict"], "COMPATIBLE");
    assert_eq!(value["analysis"]["totals"]["non_breaking"], 1);
    assert_eq!(
        value["analysis"]["findings"][0]["rule"]["domain"],
        "function"
    );
    assert_eq!(
        value["analysis"]["findings"][0]["rule"]["id"],
        "FUNCTION_ADDED"
    );
    assert!(value["disclaimer"]
        .as_str()
        .unwrap()
        .contains("not a security audit"));
}

#[test]
fn json_escapes_untrusted_labels_without_corruption() {
    let json = render_json(&result(), "old\"\\newline.wasm", "new\nfile.wasm").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["before"], "old\"\\newline.wasm");
    assert_eq!(parsed["after"], "new\nfile.wasm");
}
