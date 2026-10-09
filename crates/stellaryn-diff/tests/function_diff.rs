#![allow(clippy::unwrap_used)]

use stellaryn_core::{
    AnalysisSource, ContractInterface, Function, InterfaceValidationError, Parameter, TypeRef,
};
use stellaryn_diff::{
    diff_functions, ChangeClassification, DiffError, FunctionChangeId,
};

fn interface(functions: Vec<Function>) -> ContractInterface {
    ContractInterface {
        schema_version: stellaryn_core::INTERFACE_SCHEMA_VERSION.to_owned(),
        analysis_source: AnalysisSource::WasmSpec,
        functions,
        types: Vec::new(),
        errors: Vec::new(),
        events: Vec::new(),
    }
}

fn function(name: &str, parameters: Vec<(&str, TypeRef)>, outputs: Vec<TypeRef>) -> Function {
    Function {
        name: name.into(),
        doc: String::new(),
        parameters: parameters
            .into_iter()
            .map(|(name, type_ref)| Parameter {
                name: name.into(),
                type_ref,
                doc: String::new(),
            })
            .collect(),
        outputs,
    }
}

#[test]
fn identical_functions_produce_no_changes() {
    let before = interface(vec![function(
        "balance",
        vec![("owner", TypeRef::primitive("Address"))],
        vec![TypeRef::primitive("i128")],
    )]);
    let after = before.clone();

    let diff = diff_functions(&before, &after).unwrap();
    assert!(diff.changes.is_empty());
}

#[test]
fn added_function_is_non_breaking() {
    let before = interface(Vec::new());
    let after = interface(vec![function("version", Vec::new(), vec![TypeRef::primitive("u32")])]);

    let diff = diff_functions(&before, &after).unwrap();

    assert_eq!(diff.changes.len(), 1);
    assert_eq!(diff.changes[0].id, FunctionChangeId::FunctionAdded);
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::NonBreaking
    );
}

#[test]
fn removed_function_is_breaking() {
    let before = interface(vec![function("version", Vec::new(), Vec::new())]);
    let after = interface(Vec::new());

    let diff = diff_functions(&before, &after).unwrap();

    assert_eq!(diff.changes[0].id, FunctionChangeId::FunctionRemoved);
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::Breaking
    );
}

#[test]
fn pure_parameter_reorder_is_one_breaking_change() {
    let before = interface(vec![function(
        "transfer",
        vec![
            ("to", TypeRef::primitive("Address")),
            ("amount", TypeRef::primitive("i128")),
        ],
        Vec::new(),
    )]);
    let after = interface(vec![function(
        "transfer",
        vec![
            ("amount", TypeRef::primitive("i128")),
            ("to", TypeRef::primitive("Address")),
        ],
        Vec::new(),
    )]);

    let diff = diff_functions(&before, &after).unwrap();

    assert_eq!(diff.changes.len(), 1);
    assert_eq!(
        diff.changes[0].id,
        FunctionChangeId::FunctionParameterReordered
    );
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::Breaking
    );
}

#[test]
fn pure_parameter_rename_requires_review() {
    let before = interface(vec![function(
        "balance",
        vec![("owner", TypeRef::primitive("Address"))],
        Vec::new(),
    )]);
    let after = interface(vec![function(
        "balance",
        vec![("account", TypeRef::primitive("Address"))],
        Vec::new(),
    )]);

    let diff = diff_functions(&before, &after).unwrap();

    assert_eq!(
        diff.changes[0].id,
        FunctionChangeId::FunctionParameterRenamed
    );
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::ReviewRequired
    );
}

#[test]
fn parameter_type_change_is_breaking_and_structural() {
    let before = interface(vec![function(
        "set",
        vec![(
            "value",
            TypeRef::Option {
                value: Box::new(TypeRef::primitive("i128")),
            },
        )],
        Vec::new(),
    )]);
    let after = interface(vec![function(
        "set",
        vec![(
            "value",
            TypeRef::Option {
                value: Box::new(TypeRef::primitive("u128")),
            },
        )],
        Vec::new(),
    )]);

    let diff = diff_functions(&before, &after).unwrap();

    assert_eq!(
        diff.changes[0].id,
        FunctionChangeId::FunctionParameterTypeChanged
    );
    assert!(diff.changes[0].summary.contains("Option<i128>"));
    assert!(diff.changes[0].summary.contains("Option<u128>"));
}

#[test]
fn added_parameter_is_breaking() {
    let before = interface(vec![function(
        "transfer",
        vec![("to", TypeRef::primitive("Address"))],
        Vec::new(),
    )]);
    let after = interface(vec![function(
        "transfer",
        vec![
            ("to", TypeRef::primitive("Address")),
            ("amount", TypeRef::primitive("i128")),
        ],
        Vec::new(),
    )]);

    let diff = diff_functions(&before, &after).unwrap();

    assert_eq!(
        diff.changes[0].id,
        FunctionChangeId::FunctionParameterAdded
    );
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::Breaking
    );
}

#[test]
fn removed_parameter_is_breaking() {
    let before = interface(vec![function(
        "transfer",
        vec![
            ("to", TypeRef::primitive("Address")),
            ("amount", TypeRef::primitive("i128")),
        ],
        Vec::new(),
    )]);
    let after = interface(vec![function(
        "transfer",
        vec![("to", TypeRef::primitive("Address"))],
        Vec::new(),
    )]);

    let diff = diff_functions(&before, &after).unwrap();

    assert_eq!(
        diff.changes[0].id,
        FunctionChangeId::FunctionParameterRemoved
    );
}

#[test]
fn output_count_change_is_breaking() {
    let before = interface(vec![function("version", Vec::new(), Vec::new())]);
    let after = interface(vec![function(
        "version",
        Vec::new(),
        vec![TypeRef::primitive("u32")],
    )]);

    let diff = diff_functions(&before, &after).unwrap();

    assert_eq!(
        diff.changes[0].id,
        FunctionChangeId::FunctionOutputCountChanged
    );
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::Breaking
    );
}

#[test]
fn output_type_change_is_breaking() {
    let before = interface(vec![function(
        "version",
        Vec::new(),
        vec![TypeRef::primitive("u32")],
    )]);
    let after = interface(vec![function(
        "version",
        Vec::new(),
        vec![TypeRef::primitive("u64")],
    )]);

    let diff = diff_functions(&before, &after).unwrap();

    assert_eq!(
        diff.changes[0].id,
        FunctionChangeId::FunctionOutputTypeChanged
    );
}

#[test]
fn documentation_changes_are_ignored() {
    let mut old = function(
        "balance",
        vec![("owner", TypeRef::primitive("Address"))],
        vec![TypeRef::primitive("i128")],
    );
    let mut new = old.clone();
    old.doc = "Old function doc".into();
    old.parameters[0].doc = "Old parameter doc".into();
    new.doc = "New function doc".into();
    new.parameters[0].doc = "New parameter doc".into();

    let diff = diff_functions(&interface(vec![old]), &interface(vec![new])).unwrap();
    assert!(diff.changes.is_empty());
}

#[test]
fn findings_are_deterministic_and_breaking_first() {
    let before = interface(vec![
        function("removed", Vec::new(), Vec::new()),
        function(
            "changed",
            vec![("value", TypeRef::primitive("u32"))],
            Vec::new(),
        ),
    ]);
    let after = interface(vec![
        function("added", Vec::new(), Vec::new()),
        function(
            "changed",
            vec![("value", TypeRef::primitive("u64"))],
            Vec::new(),
        ),
    ]);

    let first = diff_functions(&before, &after).unwrap();
    let second = diff_functions(&before, &after).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.changes.len(), 3);
    assert_eq!(
        first.changes[0].classification,
        ChangeClassification::Breaking
    );
    assert_eq!(
        first.changes[1].classification,
        ChangeClassification::Breaking
    );
    assert_eq!(
        first.changes[2].classification,
        ChangeClassification::NonBreaking
    );
}

#[test]
fn relative_reorder_is_detected_when_parameter_count_also_changes() {
    let before = interface(vec![function(
        "call",
        vec![
            ("a", TypeRef::primitive("u32")),
            ("b", TypeRef::primitive("u32")),
            ("c", TypeRef::primitive("u32")),
        ],
        Vec::new(),
    )]);
    let after = interface(vec![function(
        "call",
        vec![
            ("c", TypeRef::primitive("u32")),
            ("a", TypeRef::primitive("u32")),
            ("d", TypeRef::primitive("u32")),
        ],
        Vec::new(),
    )]);

    let diff = diff_functions(&before, &after).unwrap();
    let ids: Vec<_> = diff.changes.iter().map(|change| change.id).collect();

    assert!(ids.contains(&FunctionChangeId::FunctionParameterRemoved));
    assert!(ids.contains(&FunctionChangeId::FunctionParameterAdded));
    assert!(ids.contains(&FunctionChangeId::FunctionParameterReordered));
}

#[test]
fn invalid_before_interface_returns_typed_error() {
    let mut before = interface(vec![function("f", Vec::new(), Vec::new())]);
    before.functions.push(before.functions[0].clone());
    let after = interface(Vec::new());

    let error = diff_functions(&before, &after).unwrap_err();

    assert!(matches!(
        error,
        DiffError::InvalidBefore(InterfaceValidationError::DuplicateTopLevel { .. })
    ));
}
