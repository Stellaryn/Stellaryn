#![allow(clippy::unwrap_used)]

use std::str::FromStr;

use stellaryn_core::{
    AnalysisSource, ContractInterface, EnumVariant, ErrorCase, ErrorDefinition, EventDataFormat,
    EventDefinition, EventParameter, EventParameterLocation, Function, InterfaceValidationError,
    Parameter, StructField, TypeRef, UserType, UserTypeKind, Verdict,
};
use stellaryn_diff::{
    diff_contracts, ChangeClassification, CompatibilityRule, DiffError, DomainCounts,
    EventErrorChangeId, ExitPolicy, FailOn, FunctionChangeId, TypeChangeId, EXIT_ANALYSIS_ERROR,
    EXIT_POLICY_VIOLATION, EXIT_SUCCESS,
};

fn empty() -> ContractInterface {
    ContractInterface::empty(AnalysisSource::WasmSpec)
}

fn function(name: &str, params: Vec<(&str, &str)>) -> Function {
    Function {
        name: name.into(),
        doc: String::new(),
        parameters: params
            .into_iter()
            .map(|(name, type_name)| Parameter {
                name: name.into(),
                type_ref: TypeRef::primitive(type_name),
                doc: String::new(),
            })
            .collect(),
        outputs: Vec::new(),
    }
}

fn record(name: &str, type_name: &str) -> UserType {
    UserType {
        name: name.into(),
        doc: String::new(),
        definition: UserTypeKind::Struct {
            fields: vec![StructField {
                name: "value".into(),
                type_ref: TypeRef::primitive(type_name),
                doc: String::new(),
            }],
        },
    }
}

fn enum_type(name: &str, values: &[(&str, u32)]) -> UserType {
    UserType {
        name: name.into(),
        doc: String::new(),
        definition: UserTypeKind::Enum {
            variants: values
                .iter()
                .map(|(name, value)| EnumVariant {
                    name: (*name).into(),
                    discriminant: Some(*value),
                    doc: String::new(),
                    fields: Vec::new(),
                })
                .collect(),
        },
    }
}

fn error(cases: Vec<(&str, u32)>) -> ErrorDefinition {
    ErrorDefinition {
        name: "ContractError".into(),
        doc: String::new(),
        cases: cases
            .into_iter()
            .map(|(name, value)| ErrorCase {
                name: name.into(),
                value,
                doc: String::new(),
            })
            .collect(),
    }
}

fn event(name: &str, parameters: Vec<(&str, &str)>) -> EventDefinition {
    EventDefinition {
        name: name.into(),
        doc: String::new(),
        prefix_topics: vec!["transfer".into()],
        parameters: parameters
            .into_iter()
            .map(|(name, type_name)| EventParameter {
                name: name.into(),
                type_ref: TypeRef::primitive(type_name),
                location: EventParameterLocation::Data,
                doc: String::new(),
            })
            .collect(),
        data_format: EventDataFormat::Vec,
    }
}

fn review_inputs() -> (ContractInterface, ContractInterface) {
    let mut before = empty();
    before.functions.push(function("balance", vec![("owner", "Address")]));
    before.types.push(enum_type("Status", &[("Active", 0)]));
    before.events.push(event("Transfer", vec![("amount", "u64")]));
    before.errors.push(error(vec![("Denied", 1)]));
    let mut after = before.clone();
    after.functions[0].parameters[0].name = "account".into();
    if let UserTypeKind::Enum { variants } = &mut after.types[0].definition {
        variants.push(EnumVariant {
            name: "Paused".into(),
            discriminant: Some(1),
            doc: String::new(),
            fields: Vec::new(),
        });
    }
    after.events[0].parameters[0].name = "value".into();
    after.errors[0].cases.push(ErrorCase {
        name: "Expired".into(),
        value: 2,
        doc: String::new(),
    });
    (before, after)
}

fn breaking_inputs() -> (ContractInterface, ContractInterface) {
    let mut before = empty();
    before.functions.push(function("transfer", vec![("amount", "i128")]));
    before.types.push(record("Account", "i128"));
    before.events.push(event("Transfer", vec![("amount", "i128")]));
    before.errors.push(error(vec![("Denied", 1)]));
    let mut after = before.clone();
    after.functions[0].parameters[0].type_ref = TypeRef::primitive("u128");
    if let UserTypeKind::Struct { fields } = &mut after.types[0].definition {
        fields[0].type_ref = TypeRef::primitive("u128");
    }
    after.events[0].prefix_topics = vec!["transfer_v2".into()];
    after.errors[0].cases[0].value = 9;
    (before, after)
}

#[test]
fn unchanged_interfaces_are_compatible_with_zero_findings() {
    let before = empty();
    let after = before.clone();
    let result = diff_contracts(&before, &after).unwrap();
    assert_eq!(result.verdict, Verdict::Compatible);
    assert_eq!(result.totals.total(), 0);
    assert!(result.findings.is_empty());
    assert_eq!(result.by_domain, DomainCounts::default());
}

#[test]
fn metadata_only_changes_produce_no_findings() {
    let (before, _) = breaking_inputs();
    let mut after = before.clone();
    after.functions[0].doc = "New docs".into();
    after.types[0].doc = "Revised docs".into();
    after.events[0].doc = "Event docs".into();
    after.errors[0].doc = "Error docs".into();
    assert_eq!(diff_contracts(&before, &after).unwrap().totals.total(), 0);
}

#[test]
fn additions_from_every_domain_remain_compatible_spec_level() {
    let before = empty();
    let mut after = empty();
    after.functions.push(function("version", Vec::new()));
    after.types.push(record("Account", "u32"));
    after.events.push(event("Created", Vec::new()));
    after.errors.push(error(vec![("Denied", 1)]));
    let result = diff_contracts(&before, &after).unwrap();

    assert_eq!(result.verdict, Verdict::Compatible);
    assert_eq!(result.totals.non_breaking, 4);
    assert_eq!(result.totals.breaking, 0);
    assert_eq!(result.by_domain.functions.non_breaking, 1);
    assert_eq!(result.by_domain.custom_types.non_breaking, 1);
    assert_eq!(result.by_domain.events.non_breaking, 1);
    assert_eq!(result.by_domain.errors.non_breaking, 1);
    assert_eq!(result.findings.len(), 4);
}

#[test]
fn review_from_every_domain_produces_review_verdict() {
    let (before, after) = review_inputs();
    let result = diff_contracts(&before, &after).unwrap();
    assert_eq!(result.verdict, Verdict::ReviewRequired);
    assert_eq!(result.totals.review_required, 4);
    assert_eq!(result.totals.breaking, 0);
    assert_eq!(result.by_domain.functions.review_required, 1);
    assert_eq!(result.by_domain.custom_types.review_required, 1);
    assert_eq!(result.by_domain.events.review_required, 1);
    assert_eq!(result.by_domain.errors.review_required, 1);
    assert!(result
        .findings
        .iter()
        .all(|finding| finding.classification == ChangeClassification::ReviewRequired));
}

#[test]
fn breaking_from_all_domains_produces_incompatible_verdict() {
    let (before, after) = breaking_inputs();
    let result = diff_contracts(&before, &after).unwrap();
    assert_eq!(result.verdict, Verdict::Incompatible);
    assert_eq!(result.totals.breaking, 4);
    assert_eq!(result.totals.review_required, 0);
    assert_eq!(result.totals.non_breaking, 0);
    assert_eq!(result.by_domain.functions.breaking, 1);
    assert_eq!(result.by_domain.custom_types.breaking, 1);
    assert_eq!(result.by_domain.events.breaking, 1);
    assert_eq!(result.by_domain.errors.breaking, 1);
}

#[test]
fn breaking_takes_precedence_over_review_and_additions() {
    let (before, mut after) = review_inputs();
    after.functions.push(function("version", Vec::new()));
    after.functions[0].outputs.push(TypeRef::primitive("u32"));
    let result = diff_contracts(&before, &after).unwrap();
    assert_eq!(result.verdict, Verdict::Incompatible);
    assert_eq!(result.totals.breaking, 1);
    assert_eq!(result.totals.review_required, 4);
    assert_eq!(result.totals.non_breaking, 1);
    assert_eq!(result.findings[0].classification, ChangeClassification::Breaking);
    assert_eq!(result.findings.last().unwrap().classification, ChangeClassification::NonBreaking);
}

#[test]
fn rule_ids_remain_typed_and_preserve_original_evidence() {
    let (before, after) = breaking_inputs();
    let result = diff_contracts(&before, &after).unwrap();
    assert!(result.findings.iter().any(|f| matches!(
        f.rule, CompatibilityRule::Function(FunctionChangeId::FunctionParameterTypeChanged)
    )));
    assert!(result.findings.iter().any(|f| matches!(
        f.rule, CompatibilityRule::CustomType(TypeChangeId::StructFieldTypeChanged)
    )));
    assert!(result.findings.iter().any(|f| matches!(
        f.rule, CompatibilityRule::Event(EventErrorChangeId::EventPrefixTopicsChanged)
    )));
    assert!(result.findings.iter().any(|f| matches!(
        f.rule, CompatibilityRule::Error(EventErrorChangeId::ErrorCodeChanged)
    )));
    let type_finding = result
        .findings
        .iter()
        .find(|f| matches!(f.rule, CompatibilityRule::CustomType(_)))
        .unwrap();
    assert_eq!(type_finding.before_evidence.as_deref(), Some("i128"));
    assert_eq!(type_finding.after_evidence.as_deref(), Some("u128"));
}

#[test]
fn result_order_does_not_depend_on_input_collection_order() {
    let (mut before, mut after) = breaking_inputs();
    before.functions.push(function("old_only", Vec::new()));
    after.functions.push(function("new_only", Vec::new()));
    before.types.push(record("Zed", "u32"));
    after.types.push(record("Zed", "u32"));
    before.events.push(event("Zed", Vec::new()));
    after.events.push(event("Zed", Vec::new()));
    let first = diff_contracts(&before, &after).unwrap();

    before.functions.reverse();
    before.types.reverse();
    before.events.reverse();
    after.functions.reverse();
    after.types.reverse();
    after.events.reverse();

    let second = diff_contracts(&before, &after).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.verdict, Verdict::Incompatible);
}

#[test]
fn finding_order_is_severity_first_then_stable_rule_and_subject() {
    let (before, mut after) = review_inputs();
    after.functions.push(function("zebra", Vec::new()));
    after.functions.push(function("alpha", Vec::new()));
    let result = diff_contracts(&before, &after).unwrap();
    for pair in result.findings.windows(2) {
        assert!(
            pair[0].classification != ChangeClassification::NonBreaking
                || pair[1].classification == ChangeClassification::NonBreaking
        );
    }
    let new_functions: Vec<_> = result
        .findings
        .iter()
        .filter(|item| {
            matches!(item.rule, CompatibilityRule::Function(FunctionChangeId::FunctionAdded))
        })
        .map(|item| item.subject.as_str())
        .collect();
    assert_eq!(new_functions, vec!["function:alpha", "function:zebra"]);
}

#[test]
fn result_json_round_trip_keeps_verdict_counts_and_typed_rules() {
    let (before, after) = breaking_inputs();
    let result = diff_contracts(&before, &after).unwrap();
    let encoded = serde_json::to_string_pretty(&result).unwrap();
    assert!(encoded.contains("\"INCOMPATIBLE\""));
    assert!(encoded.contains("\"domain\": \"function\""));
    assert!(encoded.contains("FUNCTION_PARAMETER_TYPE_CHANGED"));
    let decoded: stellaryn_diff::ContractDiff = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, result);
}

#[test]
fn all_domain_totals_equal_summary_count() {
    let (before, mut after) = review_inputs();
    after.events.push(event("NewEvent", Vec::new()));
    let result = diff_contracts(&before, &after).unwrap();
    let by = &result.by_domain;
    let sum = by.functions.total()
        + by.custom_types.total()
        + by.events.total()
        + by.errors.total();
    assert_eq!(sum, result.totals.total());
    assert_eq!(sum, result.findings.len());
}

#[test]
fn malformed_before_input_cannot_produce_compatible_verdict() {
    let mut before = empty();
    before.functions.push(function("same", Vec::new()));
    before.functions.push(function("same", Vec::new()));
    let result = diff_contracts(&before, &empty());
    assert!(matches!(
        result,
        Err(DiffError::InvalidBefore(
            InterfaceValidationError::DuplicateTopLevel { .. }
        ))
    ));
}

#[test]
fn malformed_after_input_cannot_produce_compatible_verdict() {
    let mut after = empty();
    after.types.push(record("Account", "u32"));
    after.types.push(record("Account", "u64"));
    let result = diff_contracts(&empty(), &after);
    assert!(matches!(
        result,
        Err(DiffError::InvalidAfter(
            InterfaceValidationError::DuplicateTopLevel { .. }
        ))
    ));
}

#[test]
fn default_policy_blocks_breaking_but_allows_review() {
    let policy = ExitPolicy::default();
    assert_eq!(policy.fail_on, FailOn::Breaking);
    let (before, after) = review_inputs();
    let review = diff_contracts(&before, &after).unwrap();
    assert_eq!(policy.exit_code(&review), EXIT_SUCCESS);

    let (before, after) = breaking_inputs();
    let breaking = diff_contracts(&before, &after).unwrap();
    assert_eq!(policy.exit_code(&breaking), EXIT_POLICY_VIOLATION);
}

#[test]
fn review_policy_blocks_review_and_breaking() {
    let policy = ExitPolicy {
        fail_on: FailOn::Review,
    };
    assert!(policy.should_fail(Verdict::ReviewRequired));
    assert!(policy.should_fail(Verdict::Incompatible));
    assert!(!policy.should_fail(Verdict::Compatible));
    let (before, after) = review_inputs();
    assert_eq!(
        policy.exit_code(&diff_contracts(&before, &after).unwrap()),
        EXIT_POLICY_VIOLATION
    );
    let (before, after) = breaking_inputs();
    assert_eq!(
        policy.exit_code(&diff_contracts(&before, &after).unwrap()),
        EXIT_POLICY_VIOLATION
    );
}

#[test]
fn never_policy_allows_any_successful_analysis_verdict() {
    let policy = ExitPolicy {
        fail_on: FailOn::Never,
    };
    for verdict in [
        Verdict::Compatible,
        Verdict::ReviewRequired,
        Verdict::Incompatible,
    ] {
        assert!(!policy.should_fail(verdict));
    }
    let (before, after) = breaking_inputs();
    assert_eq!(
        policy.exit_code(&diff_contracts(&before, &after).unwrap()),
        EXIT_SUCCESS
    );
}

#[test]
fn never_policy_does_not_suppress_invalid_analysis_input() {
    let mut before = empty();
    before.errors.push(error(vec![("One", 1), ("Two", 1)]));
    assert!(matches!(
        diff_contracts(&before, &empty()),
        Err(DiffError::InvalidBefore(
            InterfaceValidationError::DuplicateErrorValue { .. }
        ))
    ));
    assert_ne!(EXIT_ANALYSIS_ERROR, EXIT_SUCCESS);
    assert_ne!(EXIT_ANALYSIS_ERROR, EXIT_POLICY_VIOLATION);
}

#[test]
fn fail_on_strings_have_explicit_parse_contract() {
    assert_eq!(FailOn::from_str("breaking").unwrap(), FailOn::Breaking);
    assert_eq!(FailOn::from_str("review").unwrap(), FailOn::Review);
    assert_eq!(FailOn::from_str("never").unwrap(), FailOn::Never);
    let error = FailOn::from_str("review_required").unwrap_err();
    assert!(error.to_string().contains("expected 'breaking', 'review', or 'never'"));
    assert_eq!(error.input, "review_required");
}

#[test]
fn exit_policy_json_is_explicit_and_stable() {
    let policy = ExitPolicy {
        fail_on: FailOn::Review,
    };
    let encoded = serde_json::to_string(&policy).unwrap();
    assert_eq!(encoded, r#"{"fail_on":"review"}"#);
    let decoded: ExitPolicy = serde_json::from_str(&encoded).unwrap();
    assert_eq!(policy, decoded);
}

#[test]
fn policy_matrix_including_nonbreaking_changes() {
    let empty = empty();
    let mut added = empty();
    added.functions.push(function("version", Vec::new()));
    let compatible = diff_contracts(&empty, &added).unwrap();
    let (review_before, review_after) = review_inputs();
    let review = diff_contracts(&review_before, &review_after).unwrap();
    let (breaking_before, breaking_after) = breaking_inputs();
    let breaking = diff_contracts(&breaking_before, &breaking_after).unwrap();

    let policies = [FailOn::Breaking, FailOn::Review, FailOn::Never];
    let expected = [
        [EXIT_SUCCESS, EXIT_SUCCESS, EXIT_POLICY_VIOLATION],
        [EXIT_SUCCESS, EXIT_POLICY_VIOLATION, EXIT_POLICY_VIOLATION],
        [EXIT_SUCCESS, EXIT_SUCCESS, EXIT_SUCCESS],
    ];
    for (i, policy) in policies.into_iter().enumerate() {
        let gate = ExitPolicy { fail_on: policy };
        assert_eq!(gate.exit_code(&compatible), expected[i][0]);
        assert_eq!(gate.exit_code(&review), expected[i][1]);
        assert_eq!(gate.exit_code(&breaking), expected[i][2]);
    }
}
