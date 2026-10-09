#![allow(clippy::unwrap_used)]

// Real upstream-compiled Soroban contract binaries, not generated XDR-only
// modules. Provenance, immutable source revisions and upstream object hashes
// are documented in tests/fixtures/real/README.md.

use std::path::Path;
use std::process::{Command, Output};

use stellaryn_core::Verdict;
use stellaryn_diff::{diff_contracts, CompatibilityRule, FunctionChangeId};
use stellaryn_wasm::extract_interface_from_wasm;
use wasmparser::{Parser, Payload};

struct Artifact {
    name: &'static str,
    bytes: &'static [u8],
    git_blob: &'static str,
}

fn real_artifacts() -> [Artifact; 6] {
    [
        Artifact {
            name: "testnet_increment.wasm",
            bytes: include_bytes!("../fixtures/real/testnet_increment.wasm"),
            git_blob: "2844edf0215995794b1e27f94c6c88cc9a7f92b5",
        },
        Artifact {
            name: "sdk_constructor.wasm",
            bytes: include_bytes!("../fixtures/real/sdk_constructor.wasm"),
            git_blob: "e20801812b85a8803bcabd4fe938ca2d32ca0041",
        },
        Artifact {
            name: "mainnet_arb_bot.wasm",
            bytes: include_bytes!("../fixtures/real/mainnet_arb_bot.wasm"),
            git_blob: "482a061aa69ffe0039bf2f2dda071ffd07d5250a",
        },
        Artifact {
            name: "mainnet_aqua_amm.wasm",
            bytes: include_bytes!("../fixtures/real/mainnet_aqua_amm.wasm"),
            git_blob: "be4442d68e11b19efa4e5f3a4462ceeaee71a3bd",
        },
        Artifact {
            name: "compiled_add_i128.wasm",
            bytes: include_bytes!("../fixtures/real/compiled_add_i128.wasm"),
            git_blob: "ec29ca8d1273d85b75213cab073c15b4c9454d23",
        },
        Artifact {
            name: "compiled_add_u128.wasm",
            bytes: include_bytes!("../fixtures/real/compiled_add_u128.wasm"),
            git_blob: "9e15c73b755bbfb95cc67552907005c7f063fa17",
        },
    ]
}

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/real")
        .join(name)
}

fn compare(left: &str, right: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_stellaryn"))
        .arg("compare")
        .arg(fixture_path(left))
        .arg(fixture_path(right))
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn independently_compiled_artifacts_have_executable_code_and_contract_specifications() {
    for artifact in real_artifacts() {
        assert!(
            artifact.bytes.starts_with(b"\0asm\x01\0\0\0"),
            "{}",
            artifact.name
        );
        assert!(artifact.bytes.len() > 500, "{}", artifact.name);
        let mut spec_sections = 0;
        let mut code_bodies = 0;
        for entry in Parser::new(0).parse_all(artifact.bytes) {
            match entry.unwrap() {
                Payload::CustomSection(section) if section.name() == "contractspecv0" => {
                    assert!(!section.data().is_empty(), "{}", artifact.name);
                    spec_sections += 1;
                }
                Payload::CodeSectionEntry(_) => code_bodies += 1,
                _ => {}
            }
        }
        assert_eq!(
            spec_sections, 1,
            "{} has ambiguous contract spec",
            artifact.name
        );
        assert!(
            code_bodies > 0,
            "{} is not executable compiled WASM",
            artifact.name
        );
    }
}

#[test]
fn fixture_git_object_hashes_match_the_pinned_upstream_sources() {
    for artifact in real_artifacts() {
        let output = Command::new("git")
            .arg("hash-object")
            .arg(fixture_path(artifact.name))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Git hash-object failed for {}",
            artifact.name
        );
        let sha = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            sha.trim(),
            artifact.git_blob,
            "{} bytes changed",
            artifact.name
        );
    }
}

#[test]
fn all_six_compiled_contract_specs_extract_without_empty_fallbacks() {
    for artifact in real_artifacts() {
        let interface = extract_interface_from_wasm(artifact.bytes).unwrap();
        assert!(
            !interface.functions.is_empty(),
            "{} has no functions",
            artifact.name
        );
        interface.validate().unwrap();
        let identical = diff_contracts(&interface, &interface).unwrap();
        assert_eq!(identical.verdict, Verdict::Compatible, "{}", artifact.name);
        assert_eq!(identical.findings.len(), 0, "{}", artifact.name);
    }
}

#[test]
fn independent_testnet_increment_signature_is_recognized() {
    let interface =
        extract_interface_from_wasm(include_bytes!("../fixtures/real/testnet_increment.wasm"))
            .unwrap();
    let inc = interface
        .functions
        .iter()
        .find(|function| function.name == "increment")
        .unwrap();
    assert!(!inc.parameters.is_empty());
    assert_eq!(inc.parameters[0].name, "step");
    assert_eq!(inc.parameters[0].type_ref.display_name(), "i64");
}

#[test]
fn official_sdk_constructor_artifact_has_a_constructor_function() {
    let interface =
        extract_interface_from_wasm(include_bytes!("../fixtures/real/sdk_constructor.wasm"))
            .unwrap();
    assert!(interface
        .functions
        .iter()
        .any(|func| func.name == "__constructor"));
}

#[test]
fn public_mainnet_dataset_contracts_have_distinct_structured_interfaces() {
    let small =
        extract_interface_from_wasm(include_bytes!("../fixtures/real/mainnet_arb_bot.wasm"))
            .unwrap();
    let large =
        extract_interface_from_wasm(include_bytes!("../fixtures/real/mainnet_aqua_amm.wasm"))
            .unwrap();
    assert!(
        large.functions.len() >= 5,
        "complex AMM fixture lost functions"
    );
    assert_ne!(small, large);
    let report = diff_contracts(&small, &large).unwrap();
    assert_eq!(report.verdict, Verdict::Incompatible);
    assert!(report.totals.breaking > 0);
    assert!(report.findings.iter().any(|finding| matches!(
        finding.rule,
        CompatibilityRule::Function(FunctionChangeId::FunctionRemoved)
            | CompatibilityRule::Function(FunctionChangeId::FunctionAdded)
    )));
}

#[test]
fn real_wasm_self_comparison_preserves_cli_json_and_exit_zero() {
    for artifact in real_artifacts() {
        let output = compare(artifact.name, artifact.name, &["--format", "json"]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}: {}",
            artifact.name,
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty(), "{}", artifact.name);
        let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            data["analysis"]["verdict"], "COMPATIBLE",
            "{}",
            artifact.name
        );
        assert_eq!(data["analysis"]["totals"]["breaking"], 0);
        assert_eq!(data["analysis"]["findings"].as_array().unwrap().len(), 0);
    }
}

#[test]
fn comparing_different_compiled_contracts_produces_findings_and_policy_exit_two() {
    let output = compare(
        "testnet_increment.wasm",
        "sdk_constructor.wasm",
        &["--format", "json"],
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["analysis"]["verdict"], "INCOMPATIBLE");
    assert!(report["analysis"]["totals"]["breaking"].as_u64().unwrap() > 0);
    assert!(!report["analysis"]["findings"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn never_policy_preserves_real_contract_incompatibility() {
    let output = compare(
        "mainnet_arb_bot.wasm",
        "mainnet_aqua_amm.wasm",
        &["--format", "json", "--fail-on", "never"],
    );
    assert_eq!(output.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["analysis"]["verdict"], "INCOMPATIBLE");
}

#[test]
fn truncation_of_real_compiled_artifact_fails_instead_of_reporting_compatible() {
    let dir = tempfile::TempDir::new().unwrap();
    let truncated = dir.path().join("truncated.wasm");
    let bytes = include_bytes!("../fixtures/real/testnet_increment.wasm");
    std::fs::write(&truncated, &bytes[..bytes.len() - 1]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_stellaryn"))
        .arg("compare")
        .arg(fixture_path("sdk_constructor.wasm"))
        .arg(truncated)
        .args(["--format", "json", "--fail-on", "never"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn independently_compiled_add_variants_detect_real_parameter_type_break() {
    let signed =
        extract_interface_from_wasm(include_bytes!("../fixtures/real/compiled_add_i128.wasm"))
            .unwrap();
    let unsigned =
        extract_interface_from_wasm(include_bytes!("../fixtures/real/compiled_add_u128.wasm"))
            .unwrap();

    let signed_add = signed.functions.iter().find(|f| f.name == "add").unwrap();
    let unsigned_add = unsigned.functions.iter().find(|f| f.name == "add").unwrap();
    assert!(!signed_add.parameters.is_empty());
    assert_eq!(signed_add.parameters.len(), unsigned_add.parameters.len());
    assert!(signed_add
        .parameters
        .iter()
        .zip(unsigned_add.parameters.iter())
        .any(|(old, new)| old.type_ref != new.type_ref));

    let diff = diff_contracts(&signed, &unsigned).unwrap();
    assert_eq!(diff.verdict, Verdict::Incompatible);
    assert!(diff.findings.iter().any(|finding| matches!(
        finding.rule,
        CompatibilityRule::Function(FunctionChangeId::FunctionParameterTypeChanged)
    )));

    let cli = compare(
        "compiled_add_i128.wasm",
        "compiled_add_u128.wasm",
        &["--format", "json"],
    );
    assert_eq!(cli.status.code(), Some(2));
    assert!(cli.stderr.is_empty());
    let json: serde_json::Value = serde_json::from_slice(&cli.stdout).unwrap();
    assert_eq!(json["analysis"]["verdict"], "INCOMPATIBLE");
    assert!(json["analysis"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["rule"]["id"] == "FUNCTION_PARAMETER_TYPE_CHANGED"));
}
