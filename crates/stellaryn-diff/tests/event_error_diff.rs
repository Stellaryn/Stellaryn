#![allow(clippy::unwrap_used)]

use stellaryn_core::{
    AnalysisSource, ContractInterface, ErrorCase, ErrorDefinition, EventDataFormat, EventDefinition,
    EventParameter, EventParameterLocation, InterfaceValidationError, TypeRef,
};
use stellaryn_diff::{
    diff_errors, diff_events, diff_events_and_errors, ChangeClassification, DiffError,
    EventErrorChangeId, EventErrorDiff,
};

fn contract(errors: Vec<ErrorDefinition>, events: Vec<EventDefinition>) -> ContractInterface {
    let mut result = ContractInterface::empty(AnalysisSource::WasmSpec);
    result.errors = errors;
    result.events = events;
    result
}

fn case(name: &str, value: u32) -> ErrorCase {
    ErrorCase {
        name: name.into(),
        value,
        doc: String::new(),
    }
}

fn error(name: &str, cases: Vec<ErrorCase>) -> ErrorDefinition {
    ErrorDefinition {
        name: name.into(),
        doc: String::new(),
        cases,
    }
}

fn param(name: &str, ty: TypeRef, location: EventParameterLocation) -> EventParameter {
    EventParameter {
        name: name.into(),
        type_ref: ty,
        location,
        doc: String::new(),
    }
}

fn data(name: &str, ty: &str) -> EventParameter {
    param(name, TypeRef::primitive(ty), EventParameterLocation::Data)
}

fn topic(name: &str, ty: &str) -> EventParameter {
    param(name, TypeRef::primitive(ty), EventParameterLocation::Topic)
}

fn event(name: &str, params: Vec<EventParameter>) -> EventDefinition {
    EventDefinition {
        name: name.into(),
        doc: String::new(),
        prefix_topics: vec!["transfer".into()],
        parameters: params,
        data_format: EventDataFormat::Vec,
    }
}

fn ids(diff: &EventErrorDiff) -> Vec<EventErrorChangeId> {
    diff.changes.iter().map(|item| item.id).collect()
}

#[test]
fn identical_interfaces_produce_no_changes() {
    let c = contract(
        vec![error("ContractError", vec![case("Denied", 1)])],
        vec![event("Transfer", vec![topic("to", "Address"), data("amount", "i128")])],
    );
    assert!(diff_events_and_errors(&c, &c).unwrap().changes.is_empty());
}

#[test]
fn documentation_changes_are_ignored() {
    let a = contract(
        vec![error("ContractError", vec![case("Denied", 1)])],
        vec![event("Transfer", vec![data("amount", "i128")])],
    );
    let mut b = a.clone();
    b.errors[0].doc = "new error docs".into();
    b.errors[0].cases[0].doc = "new case docs".into();
    b.events[0].doc = "new event docs".into();
    b.events[0].parameters[0].doc = "new parameter docs".into();
    assert!(diff_events_and_errors(&a, &b).unwrap().changes.is_empty());
}

#[test]
fn error_definition_addition_is_nonbreaking_and_removal_breaks() {
    let empty = contract(vec![], vec![]);
    let full = contract(vec![error("ContractError", vec![case("Denied", 1)])], vec![]);
    let added = diff_errors(&empty, &full).unwrap();
    let removed = diff_errors(&full, &empty).unwrap();
    assert_eq!(ids(&added), vec![EventErrorChangeId::ErrorDefinitionAdded]);
    assert_eq!(added.changes[0].classification, ChangeClassification::NonBreaking);
    assert_eq!(ids(&removed), vec![EventErrorChangeId::ErrorDefinitionRemoved]);
    assert_eq!(removed.changes[0].classification, ChangeClassification::Breaking);
}

#[test]
fn error_case_addition_requires_review_and_removal_breaks() {
    let before = contract(vec![error("ContractError", vec![case("Denied", 1)])], vec![]);
    let after = contract(
        vec![error("ContractError", vec![case("Denied", 1), case("Expired", 2)])],
        vec![],
    );
    let added = diff_errors(&before, &after).unwrap();
    let removed = diff_errors(&after, &before).unwrap();
    assert_eq!(ids(&added), vec![EventErrorChangeId::ErrorCaseAdded]);
    assert_eq!(added.changes[0].classification, ChangeClassification::ReviewRequired);
    assert_eq!(ids(&removed), vec![EventErrorChangeId::ErrorCaseRemoved]);
    assert_eq!(removed.changes[0].classification, ChangeClassification::Breaking);
}

#[test]
fn stable_error_code_with_changed_name_requires_review() {
    let before = contract(vec![error("ContractError", vec![case("Denied", 12)])], vec![]);
    let after = contract(vec![error("ContractError", vec![case("Unauthorized", 12)])], vec![]);
    let findings = diff_errors(&before, &after).unwrap();
    assert_eq!(ids(&findings), vec![EventErrorChangeId::ErrorCaseRenamed]);
    assert_eq!(findings.changes[0].classification, ChangeClassification::ReviewRequired);
    assert_eq!(findings.changes[0].subject, "error:ContractError::code:12");
}

#[test]
fn same_error_name_with_changed_numeric_code_breaks() {
    let before = contract(vec![error("ContractError", vec![case("Denied", 1)])], vec![]);
    let after = contract(vec![error("ContractError", vec![case("Denied", 42)])], vec![]);
    let findings = diff_errors(&before, &after).unwrap();
    assert_eq!(ids(&findings), vec![EventErrorChangeId::ErrorCodeChanged]);
    assert_eq!(findings.changes[0].classification, ChangeClassification::Breaking);
    assert_eq!(findings.changes[0].before_evidence.as_deref(), Some("1"));
    assert_eq!(findings.changes[0].after_evidence.as_deref(), Some("42"));
}

#[test]
fn renamed_error_with_reassigned_code_is_not_assumed_to_be_same_case() {
    let before = contract(vec![error("ContractError", vec![case("Denied", 1)])], vec![]);
    let after = contract(vec![error("ContractError", vec![case("Unauthorized", 2)])], vec![]);
    let findings = diff_errors(&before, &after).unwrap();
    assert!(ids(&findings).contains(&EventErrorChangeId::ErrorCaseRemoved));
    assert!(ids(&findings).contains(&EventErrorChangeId::ErrorCaseAdded));
    assert!(!ids(&findings).contains(&EventErrorChangeId::ErrorCaseRenamed));
}

#[test]
fn error_case_declaration_reordering_does_not_change_codes() {
    let before = contract(
        vec![error("ContractError", vec![case("Denied", 3), case("Expired", 8)])],
        vec![],
    );
    let after = contract(
        vec![error("ContractError", vec![case("Expired", 8), case("Denied", 3)])],
        vec![],
    );
    assert!(diff_errors(&before, &after).unwrap().changes.is_empty());
}

#[test]
fn event_addition_is_nonbreaking_and_removal_breaks() {
    let empty = contract(vec![], vec![]);
    let full = contract(vec![], vec![event("Transfer", vec![data("amount", "i128")])]);
    let added = diff_events(&empty, &full).unwrap();
    let removed = diff_events(&full, &empty).unwrap();
    assert_eq!(ids(&added), vec![EventErrorChangeId::EventAdded]);
    assert_eq!(added.changes[0].classification, ChangeClassification::NonBreaking);
    assert_eq!(ids(&removed), vec![EventErrorChangeId::EventRemoved]);
    assert_eq!(removed.changes[0].classification, ChangeClassification::Breaking);
}

#[test]
fn event_prefix_topic_value_and_order_changes_break() {
    let before = contract(vec![], vec![event("Transfer", vec![])]);
    let mut after = before.clone();
    after.events[0].prefix_topics = vec!["transfer".into(), "v2".into()];
    let diff = diff_events(&before, &after).unwrap();
    assert_eq!(ids(&diff), vec![EventErrorChangeId::EventPrefixTopicsChanged]);
    assert_eq!(diff.changes[0].classification, ChangeClassification::Breaking);
    after.events[0].prefix_topics = vec!["v2".into(), "transfer".into()];
    let reordered = diff_events(
        &contract(vec![], vec![EventDefinition {
            prefix_topics: vec!["transfer".into(), "v2".into()],
            ..before.events[0].clone()
        }]),
        &after,
    )
    .unwrap();
    assert_eq!(ids(&reordered), vec![EventErrorChangeId::EventPrefixTopicsChanged]);
}

#[test]
fn event_data_format_change_breaks() {
    let before = contract(vec![], vec![event("Transfer", vec![data("amount", "i128")])]);
    let mut after = before.clone();
    after.events[0].data_format = EventDataFormat::Map;
    let diff = diff_events(&before, &after).unwrap();
    assert_eq!(ids(&diff), vec![EventErrorChangeId::EventDataFormatChanged]);
    assert_eq!(diff.changes[0].classification, ChangeClassification::Breaking);
}

#[test]
fn event_parameter_add_and_remove_break() {
    let before = contract(vec![], vec![event("Transfer", vec![data("amount", "i128")])]);
    let after = contract(vec![], vec![event(
        "Transfer",
        vec![data("amount", "i128"), topic("to", "Address")],
    )]);
    let added = diff_events(&before, &after).unwrap();
    let removed = diff_events(&after, &before).unwrap();
    assert_eq!(ids(&added), vec![EventErrorChangeId::EventParameterAdded]);
    assert_eq!(ids(&removed), vec![EventErrorChangeId::EventParameterRemoved]);
    assert_eq!(added.changes[0].classification, ChangeClassification::Breaking);
    assert_eq!(removed.changes[0].classification, ChangeClassification::Breaking);
}

#[test]
fn event_parameter_nested_type_change_breaks() {
    let before = contract(vec![], vec![event("Transfer", vec![data("amount", "i128")])]);
    let mut after = before.clone();
    after.events[0].parameters[0].type_ref = TypeRef::Option {
        value: Box::new(TypeRef::primitive("u128")),
    };
    let diff = diff_events(&before, &after).unwrap();
    assert_eq!(ids(&diff), vec![EventErrorChangeId::EventParameterTypeChanged]);
    assert_eq!(diff.changes[0].after_evidence.as_deref(), Some("Option<u128>"));
}

#[test]
fn event_parameter_topic_data_location_change_breaks() {
    let before = contract(vec![], vec![event("Transfer", vec![topic("to", "Address")])]);
    let after = contract(vec![], vec![event("Transfer", vec![data("to", "Address")])]);
    let diff = diff_events(&before, &after).unwrap();
    assert_eq!(ids(&diff), vec![EventErrorChangeId::EventParameterLocationChanged]);
    assert_eq!(diff.changes[0].classification, ChangeClassification::Breaking);
}

#[test]
fn pure_event_parameter_reorder_is_single_breaking_finding() {
    let before = contract(vec![], vec![event(
        "Transfer",
        vec![topic("to", "Address"), data("amount", "i128")],
    )]);
    let after = contract(vec![], vec![event(
        "Transfer",
        vec![data("amount", "i128"), topic("to", "Address")],
    )]);
    let diff = diff_events(&before, &after).unwrap();
    assert_eq!(ids(&diff), vec![EventErrorChangeId::EventParameterReordered]);
    assert_eq!(diff.changes[0].classification, ChangeClassification::Breaking);
}

#[test]
fn event_parameter_rename_requires_review_when_not_map_data() {
    let before = contract(vec![], vec![event("Transfer", vec![topic("recipient", "Address")])]);
    let after = contract(vec![], vec![event("Transfer", vec![topic("to", "Address")])]);
    let diff = diff_events(&before, &after).unwrap();
    assert_eq!(ids(&diff), vec![EventErrorChangeId::EventParameterRenamed]);
    assert_eq!(diff.changes[0].classification, ChangeClassification::ReviewRequired);
}

#[test]
fn map_data_parameter_name_change_breaks() {
    let mut before = event("Transfer", vec![data("amount", "i128")]);
    before.data_format = EventDataFormat::Map;
    let mut after = before.clone();
    after.parameters[0].name = "value".into();
    let diff = diff_events(&contract(vec![], vec![before]), &contract(vec![], vec![after])).unwrap();
    assert_eq!(ids(&diff), vec![EventErrorChangeId::EventParameterRenamed]);
    assert_eq!(diff.changes[0].classification, ChangeClassification::Breaking);
}

#[test]
fn vec_data_parameter_rename_requires_review() {
    let before = contract(vec![], vec![event("Transfer", vec![data("amount", "i128")])]);
    let after = contract(vec![], vec![event("Transfer", vec![data("value", "i128")])]);
    let diff = diff_events(&before, &after).unwrap();
    assert_eq!(ids(&diff), vec![EventErrorChangeId::EventParameterRenamed]);
    assert_eq!(diff.changes[0].classification, ChangeClassification::ReviewRequired);
}

#[test]
fn reordered_and_replaced_event_parameters_are_not_misidentified_as_renames() {
    let before = contract(vec![], vec![event(
        "Transfer",
        vec![data("a", "u32"), data("b", "u32"), data("c", "u32")],
    )]);
    let after = contract(vec![], vec![event(
        "Transfer",
        vec![data("c", "u32"), data("a", "u32"), data("d", "u32")],
    )]);
    let diff = diff_events(&before, &after).unwrap();
    assert!(ids(&diff).contains(&EventErrorChangeId::EventParameterReordered));
    assert!(ids(&diff).contains(&EventErrorChangeId::EventParameterAdded));
    assert!(ids(&diff).contains(&EventErrorChangeId::EventParameterRemoved));
    assert!(!ids(&diff).contains(&EventErrorChangeId::EventParameterRenamed));
}

#[test]
fn both_type_and_location_change_are_reported() {
    let before = contract(vec![], vec![event("Transfer", vec![topic("item", "u32")])]);
    let after = contract(vec![], vec![event("Transfer", vec![data("item", "u64")])]);
    let diff = diff_events(&before, &after).unwrap();
    assert_eq!(diff.changes.len(), 2);
    assert!(ids(&diff).contains(&EventErrorChangeId::EventParameterTypeChanged));
    assert!(ids(&diff).contains(&EventErrorChangeId::EventParameterLocationChanged));
}

#[test]
fn event_only_and_error_only_api_do_not_mix_domains() {
    let old = contract(vec![], vec![]);
    let new = contract(vec![error("ContractError", vec![case("Denied", 1)])], vec![event("Transfer", vec![])]);
    assert_eq!(ids(&diff_errors(&old, &new).unwrap()), vec![EventErrorChangeId::ErrorDefinitionAdded]);
    assert_eq!(ids(&diff_events(&old, &new).unwrap()), vec![EventErrorChangeId::EventAdded]);
    assert_eq!(diff_events_and_errors(&old, &new).unwrap().changes.len(), 2);
}

#[test]
fn malformed_before_and_after_interfaces_return_typed_errors() {
    let mut invalid_before = contract(vec![error("ContractError", vec![case("One", 1), case("Two", 1)])], vec![]);
    let valid = contract(vec![], vec![]);
    assert!(matches!(
        diff_events_and_errors(&invalid_before, &valid).unwrap_err(),
        DiffError::InvalidBefore(InterfaceValidationError::DuplicateErrorValue { .. })
    ));
    invalid_before.errors.clear();
    let mut invalid_after = valid.clone();
    invalid_after.events.push(event("Transfer", vec![topic("x", "Address"), topic("x", "Address")]));
    assert!(matches!(
        diff_events_and_errors(&valid, &invalid_after).unwrap_err(),
        DiffError::InvalidAfter(InterfaceValidationError::DuplicateMember { .. })
    ));
}

#[test]
fn results_are_deterministic_breaking_first_and_round_trip_as_json() {
    let old = contract(
        vec![error("ContractError", vec![case("Denied", 1)])],
        vec![event("Transfer", vec![topic("to", "Address")])],
    );
    let new = contract(
        vec![error("ContractError", vec![case("Denied", 1), case("Expired", 3)])],
        vec![event("Transfer", vec![data("to", "Address")]), event("NewEvent", vec![])],
    );
    let mut old_reordered = old.clone();
    old_reordered.errors.reverse();
    let mut new_reordered = new.clone();
    new_reordered.events.reverse();

    let first = diff_events_and_errors(&old, &new).unwrap();
    assert_eq!(first, diff_events_and_errors(&old_reordered, &new_reordered).unwrap());
    assert_eq!(first.changes.len(), 3);
    assert_eq!(first.changes[0].classification, ChangeClassification::Breaking);
    assert_eq!(first.changes[1].classification, ChangeClassification::ReviewRequired);
    assert_eq!(first.changes[2].classification, ChangeClassification::NonBreaking);

    let json = serde_json::to_string(&first).unwrap();
    assert!(json.contains("EVENT_PARAMETER_LOCATION_CHANGED"));
    assert!(json.contains("ERROR_CASE_ADDED"));
    let decoded: EventErrorDiff = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, first);
}
