#![allow(clippy::unwrap_used)]

// These fixtures are genuine ScSpecEntry XDR in minimal WASM custom sections.
// They are intentionally NOT described as full deployable Soroban contracts.
#[path = "fixtures/mod.rs"]
mod fixtures;

use fixtures::*;
use std::{fs, process::Command};
use stellar_xdr::{
    ScSpecEntry, ScSpecEventDataFormat, ScSpecEventParamLocationV0, ScSpecTypeBytesN,
    ScSpecTypeDef, ScSpecTypeMap, ScSpecTypeOption, ScSpecTypeVec, ScSpecUdtUnionCaseV0,
};
use tempfile::TempDir;

struct Case {
    name: &'static str,
    mutation: &'static str,
    verdict: &'static str,
    rule: Option<&'static str>,
    findings: usize,
}

fn replace_args(entries: &mut [ScSpecEntry], args: Vec<stellar_xdr::ScSpecFunctionInputV0>) {
    function_mut(entries).inputs = args.try_into().unwrap();
}

fn apply(change: &str, entries: &mut Vec<ScSpecEntry>) {
    match change {
        "unchanged" => {}
        "metadata_only" => {
            function_mut(entries).doc = "new documentation only".try_into().unwrap();
            struct_mut(entries).doc = "struct docs".try_into().unwrap();
            event_mut(entries).doc = "event docs".try_into().unwrap();
        }
        "entry_order" => entries.reverse(),
        "enum_order" => {
            let item = enum_mut(entries);
            let mut values: Vec<_> = item.cases.iter().cloned().collect();
            values.reverse();
            item.cases = values.try_into().unwrap();
        }
        "error_order" => {
            let item = error_mut(entries);
            let mut values: Vec<_> = item.cases.iter().cloned().collect();
            values.reverse();
            item.cases = values.try_into().unwrap();
        }
        "function_added" => entries.push(function("balance", vec![
            argument("owner", ScSpecTypeDef::Address),
        ])),
        "function_removed" => entries.retain(|e| !matches!(e, ScSpecEntry::FunctionV0(_))),
        "function_renamed" => function_mut(entries).name = symbol("pay"),
        "parameter_renamed" => {
            let mut args: Vec<_> = function_mut(entries).inputs.iter().cloned().collect();
            args[1].name = "value".try_into().unwrap();
            replace_args(entries, args);
        }
        "parameter_reordered" => {
            let mut args: Vec<_> = function_mut(entries).inputs.iter().cloned().collect();
            args.reverse();
            replace_args(entries, args);
        }
        "parameter_added" => {
            let mut args: Vec<_> = function_mut(entries).inputs.iter().cloned().collect();
            args.push(argument("memo", ScSpecTypeDef::String));
            replace_args(entries, args);
        }
        "parameter_removed" => {
            let mut args: Vec<_> = function_mut(entries).inputs.iter().cloned().collect();
            args.pop();
            replace_args(entries, args);
        }
        "parameter_type" => {
            let mut args: Vec<_> = function_mut(entries).inputs.iter().cloned().collect();
            args[1].type_ = ScSpecTypeDef::U128;
            replace_args(entries, args);
        }
        "nested_type" => {
            let mut args: Vec<_> = function_mut(entries).inputs.iter().cloned().collect();
            args[1].type_ = ScSpecTypeDef::Map(Box::new(ScSpecTypeMap {
                key_type: Box::new(ScSpecTypeDef::Address),
                value_type: Box::new(ScSpecTypeDef::Option(Box::new(ScSpecTypeOption {
                    value_type: Box::new(ScSpecTypeDef::Vec(Box::new(ScSpecTypeVec {
                        element_type: Box::new(ScSpecTypeDef::I128),
                    }))),
                }))),
            }));
            replace_args(entries, args);
        }
        "output_type" => function_mut(entries).outputs = vec![ScSpecTypeDef::U32].try_into().unwrap(),
        "output_count" => function_mut(entries).outputs = Vec::new().try_into().unwrap(),
        "struct_added" => {
            let mut item = struct_mut(entries).clone();
            item.name = "Wallet".try_into().unwrap();
            entries.push(ScSpecEntry::UdtStructV0(item));
        }
        "struct_field_added" => {
            let item = struct_mut(entries);
            let mut fields: Vec<_> = item.fields.iter().cloned().collect();
            fields.push(struct_field("balance", ScSpecTypeDef::I128));
            item.fields = fields.try_into().unwrap();
        }
        "struct_field_removed" => {
            let item = struct_mut(entries);
            let mut fields: Vec<_> = item.fields.iter().cloned().collect();
            fields.remove(1);
            item.fields = fields.try_into().unwrap();
        }
        "struct_field_type" => {
            let item = struct_mut(entries);
            let mut fields: Vec<_> = item.fields.iter().cloned().collect();
            fields[0].type_ = ScSpecTypeDef::Bytes;
            item.fields = fields.try_into().unwrap();
        }
        "bytes_n_length" => {
            let item = struct_mut(entries);
            let mut fields: Vec<_> = item.fields.iter().cloned().collect();
            fields[1].type_ = ScSpecTypeDef::BytesN(ScSpecTypeBytesN { n: 64 });
            item.fields = fields.try_into().unwrap();
        }
        "struct_field_order" => {
            let item = struct_mut(entries);
            let mut fields: Vec<_> = item.fields.iter().cloned().collect();
            fields.reverse();
            item.fields = fields.try_into().unwrap();
        }
        "enum_added" => {
            let item = enum_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            cases.push(enum_case("Frozen", 2));
            item.cases = cases.try_into().unwrap();
        }
        "enum_removed" => {
            let item = enum_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            cases.pop();
            item.cases = cases.try_into().unwrap();
        }
        "enum_discriminant" => {
            let item = enum_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            cases[1].value = 9;
            item.cases = cases.try_into().unwrap();
        }
        "type_kind_changed" => {
            let mut item = union_mut(entries).clone();
            item.name = "Status".try_into().unwrap();
            let pos = entries.iter().position(|e| matches!(e, ScSpecEntry::UdtEnumV0(_))).unwrap();
            entries[pos] = ScSpecEntry::UdtUnionV0(item);
        }
        "union_added" => {
            let item = union_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            if let ScSpecUdtUnionCaseV0::VoidV0(value) = &mut cases[0] {
                value.name = "Resume".try_into().unwrap();
            }
            let new = cases[0].clone();
            cases.push(new);
            item.cases = cases.try_into().unwrap();
        }
        "union_removed" => {
            let item = union_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            cases.remove(0);
            item.cases = cases.try_into().unwrap();
        }
        "union_payload_type" => {
            let item = union_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            if let ScSpecUdtUnionCaseV0::TupleV0(value) = &mut cases[1] {
                value.type_ = vec![ScSpecTypeDef::Address, ScSpecTypeDef::U128].try_into().unwrap();
            }
            item.cases = cases.try_into().unwrap();
        }
        "union_payload_count" => {
            let item = union_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            if let ScSpecUdtUnionCaseV0::TupleV0(value) = &mut cases[1] {
                value.type_ = vec![ScSpecTypeDef::Address].try_into().unwrap();
            }
            item.cases = cases.try_into().unwrap();
        }
        "error_definition_added" => {
            let mut item = error_mut(entries).clone();
            item.name = "OtherError".try_into().unwrap();
            entries.push(ScSpecEntry::UdtErrorEnumV0(item));
        }
        "error_case_added" => {
            let item = error_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            cases.push(error_case("Expired", 3));
            item.cases = cases.try_into().unwrap();
        }
        "error_case_removed" => {
            let item = error_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            cases.pop();
            item.cases = cases.try_into().unwrap();
        }
        "error_case_renamed" => {
            let item = error_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            cases[0].name = "Denied".try_into().unwrap();
            item.cases = cases.try_into().unwrap();
        }
        "error_code_changed" => {
            let item = error_mut(entries);
            let mut cases: Vec<_> = item.cases.iter().cloned().collect();
            cases[0].value = 27;
            item.cases = cases.try_into().unwrap();
        }
        "event_added" => {
            let mut item = event_mut(entries).clone();
            item.name = symbol("Minted");
            entries.push(ScSpecEntry::EventV0(item));
        }
        "event_removed" => entries.retain(|e| !matches!(e, ScSpecEntry::EventV0(_))),
        "event_prefix" => event_mut(entries).prefix_topics = vec![symbol("transfer_v2")].try_into().unwrap(),
        "event_format" => event_mut(entries).data_format = ScSpecEventDataFormat::Map,
        "event_param_type" => {
            let item = event_mut(entries);
            let mut fields: Vec<_> = item.params.iter().cloned().collect();
            fields[1].type_ = ScSpecTypeDef::U128;
            item.params = fields.try_into().unwrap();
        }
        "event_location" => {
            let item = event_mut(entries);
            let mut fields: Vec<_> = item.params.iter().cloned().collect();
            fields[0].location = ScSpecEventParamLocationV0::Data;
            item.params = fields.try_into().unwrap();
        }
        "event_param_renamed" => {
            let item = event_mut(entries);
            let mut fields: Vec<_> = item.params.iter().cloned().collect();
            fields[1].name = "value".try_into().unwrap();
            item.params = fields.try_into().unwrap();
        }
        "event_param_order" => {
            let item = event_mut(entries);
            let mut fields: Vec<_> = item.params.iter().cloned().collect();
            fields.reverse();
            item.params = fields.try_into().unwrap();
        }
        "mixed_severity" => {
            apply("parameter_type", entries);
            apply("error_case_added", entries);
            apply("event_added", entries);
        }
        other => assert!(false, "unknown fixture mutation {other}"),
    }
}

fn run(before: &[u8], after: &[u8]) -> std::process::Output {
    let dir = TempDir::new().unwrap();
    let left = dir.path().join("before.wasm");
    let right = dir.path().join("after.wasm");
    fs::write(&left, before).unwrap();
    fs::write(&right, after).unwrap();
    Command::new(env!("CARGO_BIN_EXE_stellaryn"))
        .arg("compare")
        .arg(&left)
        .arg(&right)
        .args(["--format", "json", "--fail-on", "never"])
        .output()
        .unwrap()
}

#[test]
fn golden_soroban_spec_matrix_covers_every_contract_category() {
    let cases = [
        Case { name:"unchanged", mutation:"unchanged", verdict:"COMPATIBLE", rule:None, findings:0 },
        Case { name:"docs only", mutation:"metadata_only", verdict:"COMPATIBLE", rule:None, findings:0 },
        Case { name:"top-level XDR reorder", mutation:"entry_order", verdict:"COMPATIBLE", rule:None, findings:0 },
        Case { name:"numeric enum declaration reorder", mutation:"enum_order", verdict:"COMPATIBLE", rule:None, findings:0 },
        Case { name:"error declaration reorder", mutation:"error_order", verdict:"COMPATIBLE", rule:None, findings:0 },
        Case { name:"add function", mutation:"function_added", verdict:"COMPATIBLE", rule:Some("FUNCTION_ADDED"), findings:1 },
        Case { name:"remove function", mutation:"function_removed", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_REMOVED"), findings:1 },
        Case { name:"rename function", mutation:"function_renamed", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_REMOVED"), findings:2 },
        Case { name:"rename input", mutation:"parameter_renamed", verdict:"REVIEW_REQUIRED", rule:Some("FUNCTION_PARAMETER_RENAMED"), findings:1 },
        Case { name:"reorder inputs", mutation:"parameter_reordered", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_PARAMETER_REORDERED"), findings:1 },
        Case { name:"add input", mutation:"parameter_added", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_PARAMETER_ADDED"), findings:1 },
        Case { name:"remove input", mutation:"parameter_removed", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_PARAMETER_REMOVED"), findings:1 },
        Case { name:"change input type", mutation:"parameter_type", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_PARAMETER_TYPE_CHANGED"), findings:1 },
        Case { name:"change to nested map/option/vec", mutation:"nested_type", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_PARAMETER_TYPE_CHANGED"), findings:1 },
        Case { name:"change output type", mutation:"output_type", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_OUTPUT_TYPE_CHANGED"), findings:1 },
        Case { name:"change output count", mutation:"output_count", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_OUTPUT_COUNT_CHANGED"), findings:1 },
        Case { name:"add struct", mutation:"struct_added", verdict:"COMPATIBLE", rule:Some("TYPE_ADDED"), findings:1 },
        Case { name:"add struct field", mutation:"struct_field_added", verdict:"INCOMPATIBLE", rule:Some("STRUCT_FIELD_ADDED"), findings:1 },
        Case { name:"remove struct field", mutation:"struct_field_removed", verdict:"INCOMPATIBLE", rule:Some("STRUCT_FIELD_REMOVED"), findings:1 },
        Case { name:"struct field type", mutation:"struct_field_type", verdict:"INCOMPATIBLE", rule:Some("STRUCT_FIELD_TYPE_CHANGED"), findings:1 },
        Case { name:"BytesN size", mutation:"bytes_n_length", verdict:"INCOMPATIBLE", rule:Some("STRUCT_FIELD_TYPE_CHANGED"), findings:1 },
        Case { name:"struct field reorder", mutation:"struct_field_order", verdict:"REVIEW_REQUIRED", rule:Some("STRUCT_FIELD_REORDERED"), findings:1 },
        Case { name:"add numeric enum variant", mutation:"enum_added", verdict:"REVIEW_REQUIRED", rule:Some("ENUM_VARIANT_ADDED"), findings:1 },
        Case { name:"remove numeric enum variant", mutation:"enum_removed", verdict:"INCOMPATIBLE", rule:Some("ENUM_VARIANT_REMOVED"), findings:1 },
        Case { name:"enum discriminant changes", mutation:"enum_discriminant", verdict:"INCOMPATIBLE", rule:Some("ENUM_DISCRIMINANT_CHANGED"), findings:1 },
        Case { name:"numeric enum changes to tagged union", mutation:"type_kind_changed", verdict:"INCOMPATIBLE", rule:Some("TYPE_KIND_CHANGED"), findings:1 },
        Case { name:"add union case", mutation:"union_added", verdict:"REVIEW_REQUIRED", rule:Some("UNION_VARIANT_ADDED"), findings:1 },
        Case { name:"remove union case", mutation:"union_removed", verdict:"INCOMPATIBLE", rule:Some("UNION_VARIANT_REMOVED"), findings:1 },
        Case { name:"union payload type", mutation:"union_payload_type", verdict:"INCOMPATIBLE", rule:Some("UNION_PAYLOAD_TYPE_CHANGED"), findings:1 },
        Case { name:"union payload arity", mutation:"union_payload_count", verdict:"INCOMPATIBLE", rule:Some("UNION_PAYLOAD_COUNT_CHANGED"), findings:1 },
        Case { name:"add error enum", mutation:"error_definition_added", verdict:"COMPATIBLE", rule:Some("ERROR_DEFINITION_ADDED"), findings:1 },
        Case { name:"add error case", mutation:"error_case_added", verdict:"REVIEW_REQUIRED", rule:Some("ERROR_CASE_ADDED"), findings:1 },
        Case { name:"remove error case", mutation:"error_case_removed", verdict:"INCOMPATIBLE", rule:Some("ERROR_CASE_REMOVED"), findings:1 },
        Case { name:"rename error case stable code", mutation:"error_case_renamed", verdict:"REVIEW_REQUIRED", rule:Some("ERROR_CASE_RENAMED"), findings:1 },
        Case { name:"change error code", mutation:"error_code_changed", verdict:"INCOMPATIBLE", rule:Some("ERROR_CODE_CHANGED"), findings:1 },
        Case { name:"add event", mutation:"event_added", verdict:"COMPATIBLE", rule:Some("EVENT_ADDED"), findings:1 },
        Case { name:"remove event", mutation:"event_removed", verdict:"INCOMPATIBLE", rule:Some("EVENT_REMOVED"), findings:1 },
        Case { name:"event prefix change", mutation:"event_prefix", verdict:"INCOMPATIBLE", rule:Some("EVENT_PREFIX_TOPICS_CHANGED"), findings:1 },
        Case { name:"event format change", mutation:"event_format", verdict:"INCOMPATIBLE", rule:Some("EVENT_DATA_FORMAT_CHANGED"), findings:1 },
        Case { name:"event data type change", mutation:"event_param_type", verdict:"INCOMPATIBLE", rule:Some("EVENT_PARAMETER_TYPE_CHANGED"), findings:1 },
        Case { name:"event topic/data location", mutation:"event_location", verdict:"INCOMPATIBLE", rule:Some("EVENT_PARAMETER_LOCATION_CHANGED"), findings:1 },
        Case { name:"rename event data parameter", mutation:"event_param_renamed", verdict:"REVIEW_REQUIRED", rule:Some("EVENT_PARAMETER_RENAMED"), findings:1 },
        Case { name:"event param reorder", mutation:"event_param_order", verdict:"INCOMPATIBLE", rule:Some("EVENT_PARAMETER_REORDERED"), findings:1 },
        Case { name:"mixed breaking/review/additive", mutation:"mixed_severity", verdict:"INCOMPATIBLE", rule:Some("FUNCTION_PARAMETER_TYPE_CHANGED"), findings:3 },
    ];
    let base = baseline();
    let bytes = wasm(&base);
    assert!(bytes.len() > 127, "fixture must exercise multi-byte WASM LEB encoding");
    for case in cases {
        let mut upgraded = base.clone();
        apply(case.mutation, &mut upgraded);
        let output = run(&bytes, &wasm(&upgraded));
        assert_eq!(output.status.code(), Some(0), "{} stderr: {}", case.name, String::from_utf8_lossy(&output.stderr));
        assert!(output.stderr.is_empty(), "{}", case.name);
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(json["analysis"]["verdict"], case.verdict, "{}", case.name);
        let findings = json["analysis"]["findings"].as_array().unwrap();
        assert_eq!(findings.len(), case.findings, "{}", case.name);
        if let Some(rule) = case.rule {
            assert!(findings.iter().any(|f| f["rule"]["id"] == rule), "fixture {} missing expected rule {}", case.name, rule);
        }
        let total = json["analysis"]["totals"]["breaking"].as_u64().unwrap()
            + json["analysis"]["totals"]["review_required"].as_u64().unwrap()
            + json["analysis"]["totals"]["non_breaking"].as_u64().unwrap();
        assert_eq!(total as usize, findings.len(), "{} counts", case.name);
    }
}

#[test]
fn mixed_finding_order_is_severity_first_and_never_drops_review_findings() {
    let base = baseline();
    let mut after = base.clone();
    apply("mixed_severity", &mut after);
    let out = run(&wasm(&base), &wasm(&after));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let changes = json["analysis"]["findings"].as_array().unwrap();
    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0]["classification"], "BREAKING");
    assert_eq!(changes[1]["classification"], "REVIEW_REQUIRED");
    assert_eq!(changes[2]["classification"], "NON_BREAKING");
    assert_eq!(json["analysis"]["totals"]["breaking"], 1);
    assert_eq!(json["analysis"]["totals"]["review_required"], 1);
    assert_eq!(json["analysis"]["totals"]["non_breaking"], 1);
}
