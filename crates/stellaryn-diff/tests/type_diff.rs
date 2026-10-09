#![allow(clippy::unwrap_used)]

use stellaryn_core::{
    AnalysisSource, ContractInterface, EnumVariant, InterfaceValidationError, StructField, TypeRef,
    UserType, UserTypeKind, VariantField,
};
use stellaryn_diff::{diff_types, ChangeClassification, DiffError, TypeChangeId};

fn contract(types: Vec<UserType>) -> ContractInterface {
    let mut interface = ContractInterface::empty(AnalysisSource::WasmSpec);
    interface.types = types;
    interface
}

fn field(name: &str, kind: &str) -> StructField {
    StructField {
        name: name.into(),
        type_ref: TypeRef::primitive(kind),
        doc: String::new(),
    }
}

fn record(name: &str, fields: Vec<StructField>) -> UserType {
    UserType {
        name: name.into(),
        doc: String::new(),
        definition: UserTypeKind::Struct { fields },
    }
}

fn variant(name: &str, value: u32) -> EnumVariant {
    EnumVariant {
        name: name.into(),
        discriminant: Some(value),
        doc: String::new(),
        fields: Vec::new(),
    }
}

fn number_enum(name: &str, variants: Vec<EnumVariant>) -> UserType {
    UserType {
        name: name.into(),
        doc: String::new(),
        definition: UserTypeKind::Enum { variants },
    }
}

fn union_case(name: &str, kinds: &[&str]) -> EnumVariant {
    EnumVariant {
        name: name.into(),
        discriminant: None,
        doc: String::new(),
        fields: kinds
            .iter()
            .map(|kind| VariantField {
                name: None,
                type_ref: TypeRef::primitive(*kind),
            })
            .collect(),
    }
}

fn union(name: &str, variants: Vec<EnumVariant>) -> UserType {
    UserType {
        name: name.into(),
        doc: String::new(),
        definition: UserTypeKind::Union { variants },
    }
}

fn one(before: UserType, after: UserType) -> Vec<TypeChangeId> {
    diff_types(&contract(vec![before]), &contract(vec![after]))
        .unwrap()
        .changes
        .iter()
        .map(|finding| finding.id)
        .collect()
}

#[test]
fn same_type_and_doc_only_changes_produce_no_findings() {
    let before = record("Account", vec![field("owner", "Address")]);
    let mut after = before.clone();
    after.doc = "New docs".into();
    if let UserTypeKind::Struct { fields } = &mut after.definition {
        fields[0].doc = "Updated field docs".into();
    }
    assert!(one(before, after).is_empty());
}

#[test]
fn type_added_is_non_breaking_and_type_removed_is_breaking() {
    let item = record("Account", vec![]);
    let added = diff_types(&contract(vec![]), &contract(vec![item.clone()])).unwrap();
    let removed = diff_types(&contract(vec![item]), &contract(vec![])).unwrap();
    assert_eq!(added.changes[0].id, TypeChangeId::TypeAdded);
    assert_eq!(
        added.changes[0].classification,
        ChangeClassification::NonBreaking
    );
    assert_eq!(removed.changes[0].id, TypeChangeId::TypeRemoved);
    assert_eq!(
        removed.changes[0].classification,
        ChangeClassification::Breaking
    );
}

#[test]
fn struct_field_add_remove_and_type_change_are_breaking() {
    let old = record("Account", vec![field("owner", "Address")]);
    let new = record(
        "Account",
        vec![field("owner", "Bytes"), field("flags", "u32")],
    );
    let diff = diff_types(&contract(vec![old.clone()]), &contract(vec![new.clone()])).unwrap();
    assert_eq!(diff.changes.len(), 2);
    assert!(diff
        .changes
        .iter()
        .all(|f| f.classification == ChangeClassification::Breaking));
    assert!(diff
        .changes
        .iter()
        .any(|f| f.id == TypeChangeId::StructFieldAdded));
    assert!(diff
        .changes
        .iter()
        .any(|f| f.id == TypeChangeId::StructFieldTypeChanged));

    let reverse = one(new, old);
    assert!(reverse.contains(&TypeChangeId::StructFieldRemoved));
}

#[test]
fn struct_reorder_is_review_required_and_does_not_relabel_fields() {
    let before = record("Account", vec![field("a", "u32"), field("b", "u64")]);
    let after = record("Account", vec![field("b", "u64"), field("a", "u32")]);
    let diff = diff_types(&contract(vec![before]), &contract(vec![after])).unwrap();
    assert_eq!(diff.changes.len(), 1);
    assert_eq!(diff.changes[0].id, TypeChangeId::StructFieldReordered);
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::ReviewRequired
    );
}

#[test]
fn struct_field_changes_preserve_structured_nested_type_evidence() {
    let before = record("Account", vec![field("balance", "i128")]);
    let mut after = before.clone();
    if let UserTypeKind::Struct { fields } = &mut after.definition {
        fields[0].type_ref = TypeRef::Option {
            value: Box::new(TypeRef::primitive("i128")),
        };
    }
    let diff = diff_types(&contract(vec![before]), &contract(vec![after])).unwrap();
    assert_eq!(diff.changes[0].id, TypeChangeId::StructFieldTypeChanged);
    assert_eq!(
        diff.changes[0].after_evidence.as_deref(),
        Some("Option<i128>")
    );
}

#[test]
fn new_enum_variant_requires_review_and_removal_breaks() {
    let before = number_enum("Status", vec![variant("Active", 0)]);
    let after = number_enum("Status", vec![variant("Active", 0), variant("Paused", 1)]);
    let added = diff_types(
        &contract(vec![before.clone()]),
        &contract(vec![after.clone()]),
    )
    .unwrap();
    assert_eq!(added.changes[0].id, TypeChangeId::EnumVariantAdded);
    assert_eq!(
        added.changes[0].classification,
        ChangeClassification::ReviewRequired
    );
    let removed = diff_types(&contract(vec![after]), &contract(vec![before])).unwrap();
    assert_eq!(removed.changes[0].id, TypeChangeId::EnumVariantRemoved);
    assert_eq!(
        removed.changes[0].classification,
        ChangeClassification::Breaking
    );
}

#[test]
fn enum_discriminant_change_is_breaking() {
    let before = number_enum("Status", vec![variant("Active", 1)]);
    let after = number_enum("Status", vec![variant("Active", 2)]);
    let diff = diff_types(&contract(vec![before]), &contract(vec![after])).unwrap();
    assert_eq!(diff.changes[0].id, TypeChangeId::EnumDiscriminantChanged);
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::Breaking
    );
    assert_eq!(diff.changes[0].before_evidence.as_deref(), Some("1"));
    assert_eq!(diff.changes[0].after_evidence.as_deref(), Some("2"));
}

#[test]
fn enum_case_reorder_with_unchanged_discriminants_is_not_a_change() {
    let before = number_enum("Status", vec![variant("Active", 0), variant("Paused", 1)]);
    let after = number_enum("Status", vec![variant("Paused", 1), variant("Active", 0)]);
    assert!(one(before, after).is_empty());
}

#[test]
fn union_added_case_requires_review_and_removed_case_breaks() {
    let before = union("Action", vec![union_case("Pause", &[])]);
    let after = union(
        "Action",
        vec![union_case("Pause", &[]), union_case("Pay", &["u64"])],
    );
    let added = diff_types(
        &contract(vec![before.clone()]),
        &contract(vec![after.clone()]),
    )
    .unwrap();
    assert_eq!(added.changes[0].id, TypeChangeId::UnionVariantAdded);
    assert_eq!(
        added.changes[0].classification,
        ChangeClassification::ReviewRequired
    );
    let removed = one(after, before);
    assert_eq!(removed, vec![TypeChangeId::UnionVariantRemoved]);
}

#[test]
fn union_payload_arity_change_is_breaking() {
    let before = union("Action", vec![union_case("Pay", &["Address"])]);
    let after = union("Action", vec![union_case("Pay", &["Address", "u64"])]);
    let diff = diff_types(&contract(vec![before]), &contract(vec![after])).unwrap();
    assert_eq!(diff.changes[0].id, TypeChangeId::UnionPayloadCountChanged);
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::Breaking
    );
}

#[test]
fn union_payload_type_change_is_breaking() {
    let before = union("Action", vec![union_case("Pay", &["i128"])]);
    let after = union("Action", vec![union_case("Pay", &["u128"])]);
    let diff = diff_types(&contract(vec![before]), &contract(vec![after])).unwrap();
    assert_eq!(diff.changes[0].id, TypeChangeId::UnionPayloadTypeChanged);
    assert_eq!(diff.changes[0].before_evidence.as_deref(), Some("i128"));
}

#[test]
fn union_payload_field_name_change_requires_review() {
    let before = union("Action", vec![union_case("Pay", &["u64"])]);
    let mut after = before.clone();
    if let UserTypeKind::Union { variants } = &mut after.definition {
        variants[0].fields[0].name = Some("amount".into());
    }
    let diff = diff_types(&contract(vec![before]), &contract(vec![after])).unwrap();
    assert_eq!(diff.changes[0].id, TypeChangeId::UnionPayloadNameChanged);
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::ReviewRequired
    );
}

#[test]
fn type_kind_changes_are_breaking_including_enum_to_union() {
    let before = number_enum("Status", vec![variant("Active", 0)]);
    let after = union("Status", vec![union_case("Active", &[])]);
    let diff = diff_types(&contract(vec![before]), &contract(vec![after])).unwrap();
    assert_eq!(diff.changes.len(), 1);
    assert_eq!(diff.changes[0].id, TypeChangeId::TypeKindChanged);
    assert_eq!(
        diff.changes[0].classification,
        ChangeClassification::Breaking
    );
}

#[test]
fn findings_are_deterministic_under_unsorted_input_and_breaking_first() {
    let old = contract(vec![
        record("Zed", vec![field("value", "u32")]),
        number_enum("Status", vec![variant("Active", 0)]),
    ]);
    let new = contract(vec![
        number_enum("Status", vec![variant("Active", 0), variant("Paused", 1)]),
        record("Zed", vec![field("value", "u64")]),
    ]);
    let mut old_reversed = old.clone();
    old_reversed.types.reverse();
    let mut new_reversed = new.clone();
    new_reversed.types.reverse();

    let first = diff_types(&old, &new).unwrap();
    assert_eq!(first, diff_types(&old_reversed, &new_reversed).unwrap());
    assert_eq!(first.changes.len(), 2);
    assert_eq!(
        first.changes[0].classification,
        ChangeClassification::Breaking
    );
    assert_eq!(
        first.changes[1].classification,
        ChangeClassification::ReviewRequired
    );
}

#[test]
fn duplicate_enum_discriminants_are_rejected() {
    let invalid = number_enum("Status", vec![variant("A", 1), variant("B", 1)]);
    let error = diff_types(&contract(vec![invalid]), &contract(vec![])).unwrap_err();
    assert!(matches!(
        error,
        DiffError::InvalidBefore(InterfaceValidationError::DuplicateEnumDiscriminant { .. })
    ));
}

#[test]
fn wrong_enum_variant_shape_is_rejected() {
    let invalid = number_enum("Status", vec![union_case("A", &["u32"])]);
    let error = diff_types(&contract(vec![invalid]), &contract(vec![])).unwrap_err();
    assert!(matches!(
        error,
        DiffError::InvalidBefore(InterfaceValidationError::InvalidVariantShape { .. })
    ));
}

#[test]
fn wrong_union_discriminant_is_rejected() {
    let invalid = union("Status", vec![variant("A", 1)]);
    let error = diff_types(&contract(vec![invalid]), &contract(vec![])).unwrap_err();
    assert!(matches!(
        error,
        DiffError::InvalidBefore(InterfaceValidationError::InvalidVariantShape { .. })
    ));
}

#[test]
fn serialized_findings_have_stable_rule_ids() {
    let before = record("Account", vec![field("balance", "i128")]);
    let after = record("Account", vec![field("balance", "u128")]);
    let diff = diff_types(&contract(vec![before]), &contract(vec![after])).unwrap();
    let encoded = serde_json::to_string(&diff).unwrap();
    assert!(encoded.contains("STRUCT_FIELD_TYPE_CHANGED"));
    let decoded: stellaryn_diff::TypeDiff = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, diff);
}
