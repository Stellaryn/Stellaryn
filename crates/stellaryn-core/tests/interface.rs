#![allow(clippy::unwrap_used)]

use stellaryn_core::{
    AnalysisSource, ContractInterface, EnumVariant, ErrorCase, ErrorDefinition, EventDataFormat,
    EventDefinition, EventParameter, EventParameterLocation, Function, InterfaceValidationError,
    Parameter, StructField, TypeRef, UserType, UserTypeKind, INTERFACE_SCHEMA_VERSION,
};

fn sample_interface() -> ContractInterface {
    ContractInterface {
        schema_version: INTERFACE_SCHEMA_VERSION.to_owned(),
        analysis_source: AnalysisSource::WasmSpec,
        functions: vec![
            Function {
                name: "transfer".into(),
                doc: "Move funds.".into(),
                parameters: vec![
                    Parameter {
                        name: "to".into(),
                        type_ref: TypeRef::named("Address"),
                        doc: String::new(),
                    },
                    Parameter {
                        name: "amount".into(),
                        type_ref: TypeRef::primitive("i128"),
                        doc: String::new(),
                    },
                ],
                outputs: vec![TypeRef::primitive("bool")],
            },
            Function {
                name: "balance".into(),
                doc: String::new(),
                parameters: vec![Parameter {
                    name: "owner".into(),
                    type_ref: TypeRef::named("Address"),
                    doc: String::new(),
                }],
                outputs: vec![TypeRef::primitive("i128")],
            },
        ],
        types: vec![
            UserType {
                name: "Status".into(),
                doc: String::new(),
                definition: UserTypeKind::Enum {
                    variants: vec![
                        EnumVariant {
                            name: "Active".into(),
                            discriminant: Some(0),
                            doc: String::new(),
                            fields: Vec::new(),
                        },
                        EnumVariant {
                            name: "Paused".into(),
                            discriminant: Some(1),
                            doc: String::new(),
                            fields: Vec::new(),
                        },
                    ],
                },
            },
            UserType {
                name: "Account".into(),
                doc: String::new(),
                definition: UserTypeKind::Struct {
                    fields: vec![
                        StructField {
                            name: "owner".into(),
                            type_ref: TypeRef::named("Address"),
                            doc: String::new(),
                        },
                        StructField {
                            name: "flags".into(),
                            type_ref: TypeRef::Vector {
                                element: Box::new(TypeRef::primitive("u32")),
                            },
                            doc: String::new(),
                        },
                    ],
                },
            },
        ],
        errors: vec![ErrorDefinition {
            name: "ContractError".into(),
            doc: String::new(),
            cases: vec![
                ErrorCase {
                    name: "Unauthorized".into(),
                    value: 2,
                    doc: String::new(),
                },
                ErrorCase {
                    name: "InsufficientBalance".into(),
                    value: 1,
                    doc: String::new(),
                },
            ],
        }],
        events: vec![EventDefinition {
            name: "Transfer".into(),
            doc: String::new(),
            prefix_topics: vec!["transfer".into()],
            parameters: vec![
                EventParameter {
                    name: "from".into(),
                    type_ref: TypeRef::named("Address"),
                    location: EventParameterLocation::Topic,
                    doc: String::new(),
                },
                EventParameter {
                    name: "amount".into(),
                    type_ref: TypeRef::primitive("i128"),
                    location: EventParameterLocation::Data,
                    doc: String::new(),
                },
            ],
            data_format: EventDataFormat::SingleValue,
        }],
    }
}

#[test]
fn normalization_is_deterministic_for_top_level_items() {
    let left = sample_interface();
    let mut right = sample_interface();

    right.functions.reverse();
    right.types.reverse();

    let left = left.normalized().unwrap();
    let right = right.normalized().unwrap();

    assert_eq!(left, right);
    assert_eq!(left.functions[0].name, "balance");
    assert_eq!(left.functions[1].name, "transfer");
    assert_eq!(left.types[0].name, "Account");
    assert_eq!(left.types[1].name, "Status");

    left.validate().unwrap();
}

#[test]
fn normalization_preserves_semantic_member_order() {
    let normalized = sample_interface().normalized().unwrap();

    let transfer = normalized
        .functions
        .iter()
        .find(|function| function.name == "transfer")
        .unwrap();
    assert_eq!(transfer.parameters[0].name, "to");
    assert_eq!(transfer.parameters[1].name, "amount");

    let account = normalized
        .types
        .iter()
        .find(|user_type| user_type.name == "Account")
        .unwrap();
    assert!(matches!(account.definition, UserTypeKind::Struct { .. }));
    if let UserTypeKind::Struct { fields } = &account.definition {
        assert_eq!(fields[0].name, "owner");
        assert_eq!(fields[1].name, "flags");
    }

    let event = &normalized.events[0];
    assert_eq!(event.parameters[0].name, "from");
    assert_eq!(event.parameters[1].name, "amount");

    assert_eq!(normalized.errors[0].cases[0].value, 1);
    assert_eq!(normalized.errors[0].cases[1].value, 2);
}

#[test]
fn stable_ids_are_deterministic() {
    let normalized = sample_interface().normalized().unwrap();
    assert_eq!(
        normalized.stable_item_ids(),
        vec![
            "error:ContractError",
            "event:Transfer",
            "function:balance",
            "function:transfer",
            "type:Account",
            "type:Status",
        ]
    );
}

#[test]
fn function_signature_display_handles_multiple_outputs() {
    let function = Function {
        name: "lookup".into(),
        doc: String::new(),
        parameters: vec![Parameter {
            name: "id".into(),
            type_ref: TypeRef::primitive("u64"),
            doc: String::new(),
        }],
        outputs: vec![TypeRef::primitive("bool"), TypeRef::named("Account")],
    };

    assert_eq!(
        function.signature_display(),
        "fn lookup(id: u64) -> (bool, Account)"
    );
}

#[test]
fn type_ref_serialization_is_structured_and_round_trips() {
    let value = TypeRef::Map {
        key: Box::new(TypeRef::named("Address")),
        value: Box::new(TypeRef::Option {
            value: Box::new(TypeRef::Vector {
                element: Box::new(TypeRef::primitive("i128")),
            }),
        }),
    };

    let json = serde_json::to_string(&value).unwrap();
    assert!(json.contains("\"kind\":\"map\""));
    assert!(!json.contains("Map<Address"));

    let decoded: TypeRef = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(decoded.display_name(), "Map<Address, Option<Vec<i128>>>");

    let bytes = TypeRef::BytesN { length: 32 };
    assert_eq!(bytes.display_name(), "BytesN<32>");
    let encoded = serde_json::to_string(&bytes).unwrap();
    assert!(encoded.contains("\"kind\":\"bytes_n\""));
}

#[test]
fn normalized_interface_has_stable_json_round_trip() {
    let normalized = sample_interface().normalized().unwrap();
    let first = serde_json::to_string_pretty(&normalized).unwrap();
    let decoded: ContractInterface = serde_json::from_str(&first).unwrap();
    decoded.validate().unwrap();
    let second = serde_json::to_string_pretty(&decoded).unwrap();
    assert_eq!(first, second);
}

#[test]
fn duplicate_top_level_items_are_rejected() {
    let mut interface = sample_interface();
    interface.functions.push(interface.functions[0].clone());

    let error = interface.validate().unwrap_err();
    assert_eq!(
        error,
        InterfaceValidationError::DuplicateTopLevel {
            kind: "function".into(),
            name: "transfer".into(),
        }
    );
}

#[test]
fn duplicate_nested_members_are_rejected() {
    let mut interface = sample_interface();
    interface.functions[0].parameters[1].name = "to".into();

    let error = interface.validate().unwrap_err();
    assert_eq!(
        error,
        InterfaceValidationError::DuplicateMember {
            parent: "function:transfer".into(),
            member_kind: "parameter".into(),
            name: "to".into(),
        }
    );
}

#[test]
fn duplicate_error_values_are_rejected() {
    let mut interface = sample_interface();
    interface.errors[0].cases[1].value = 2;

    let error = interface.validate().unwrap_err();
    assert_eq!(
        error,
        InterfaceValidationError::DuplicateErrorValue {
            error_name: "ContractError".into(),
            value: 2,
        }
    );
}

#[test]
fn empty_nested_type_name_is_rejected() {
    let mut interface = sample_interface();
    interface.functions[0].parameters[0].type_ref = TypeRef::named(" ");

    let error = interface.validate().unwrap_err();
    assert!(matches!(error, InterfaceValidationError::EmptyName { .. }));
}
