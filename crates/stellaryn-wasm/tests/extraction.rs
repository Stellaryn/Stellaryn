#![allow(clippy::unwrap_used)]

use soroban_spec::read::FromWasmError;
use stellar_xdr::{
    Limits, ScSpecEntry, ScSpecEventDataFormat, ScSpecEventParamLocationV0, ScSpecEventParamV0,
    ScSpecEventV0, ScSpecFunctionInputV0, ScSpecFunctionV0, ScSpecTypeBytesN, ScSpecTypeDef,
    ScSpecTypeMap, ScSpecTypeOption, ScSpecTypeUdt, ScSpecUdtEnumCaseV0, ScSpecUdtEnumV0,
    ScSpecUdtErrorEnumCaseV0, ScSpecUdtErrorEnumV0, ScSpecUdtStructFieldV0, ScSpecUdtStructV0,
    ScSpecUdtUnionCaseTupleV0, ScSpecUdtUnionCaseV0, ScSpecUdtUnionCaseVoidV0, ScSpecUdtUnionV0,
    ScSymbol, WriteXdr,
};
use stellaryn_core::{EventDataFormat, EventParameterLocation, TypeRef, UserTypeKind};
use stellaryn_wasm::{
    extract_interface_from_wasm, normalize_spec_entries, normalize_type_ref, WasmError,
};

fn symbol(value: &str) -> ScSymbol {
    ScSymbol(value.try_into().unwrap())
}

fn entries() -> Vec<ScSpecEntry> {
    vec![
        ScSpecEntry::FunctionV0(ScSpecFunctionV0 {
            doc: "Transfer funds.".try_into().unwrap(),
            name: symbol("transfer"),
            inputs: vec![
                ScSpecFunctionInputV0 {
                    doc: "".try_into().unwrap(),
                    name: "to".try_into().unwrap(),
                    type_: ScSpecTypeDef::Address,
                },
                ScSpecFunctionInputV0 {
                    doc: "Amount.".try_into().unwrap(),
                    name: "amount".try_into().unwrap(),
                    type_: ScSpecTypeDef::I128,
                },
                ScSpecFunctionInputV0 {
                    doc: "".try_into().unwrap(),
                    name: "memo".try_into().unwrap(),
                    type_: ScSpecTypeDef::Option(Box::new(ScSpecTypeOption {
                        value_type: Box::new(ScSpecTypeDef::String),
                    })),
                },
            ]
            .try_into()
            .unwrap(),
            outputs: vec![ScSpecTypeDef::Bool].try_into().unwrap(),
        }),
        ScSpecEntry::UdtStructV0(ScSpecUdtStructV0 {
            doc: "Account data.".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: "Account".try_into().unwrap(),
            fields: vec![
                ScSpecUdtStructFieldV0 {
                    doc: "".try_into().unwrap(),
                    name: "owner".try_into().unwrap(),
                    type_: ScSpecTypeDef::Address,
                },
                ScSpecUdtStructFieldV0 {
                    doc: "".try_into().unwrap(),
                    name: "hash".try_into().unwrap(),
                    type_: ScSpecTypeDef::BytesN(ScSpecTypeBytesN { n: 32 }),
                },
            ]
            .try_into()
            .unwrap(),
        }),
        ScSpecEntry::UdtUnionV0(ScSpecUdtUnionV0 {
            doc: "Action.".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: "Action".try_into().unwrap(),
            cases: vec![
                ScSpecUdtUnionCaseV0::VoidV0(ScSpecUdtUnionCaseVoidV0 {
                    doc: "".try_into().unwrap(),
                    name: "Pause".try_into().unwrap(),
                }),
                ScSpecUdtUnionCaseV0::TupleV0(ScSpecUdtUnionCaseTupleV0 {
                    doc: "".try_into().unwrap(),
                    name: "Transfer".try_into().unwrap(),
                    type_: vec![ScSpecTypeDef::Address, ScSpecTypeDef::I128]
                        .try_into()
                        .unwrap(),
                }),
            ]
            .try_into()
            .unwrap(),
        }),
        ScSpecEntry::UdtEnumV0(ScSpecUdtEnumV0 {
            doc: "Status.".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: "Status".try_into().unwrap(),
            cases: vec![
                ScSpecUdtEnumCaseV0 {
                    doc: "".try_into().unwrap(),
                    name: "Active".try_into().unwrap(),
                    value: 0,
                },
                ScSpecUdtEnumCaseV0 {
                    doc: "".try_into().unwrap(),
                    name: "Paused".try_into().unwrap(),
                    value: 1,
                },
            ]
            .try_into()
            .unwrap(),
        }),
        ScSpecEntry::UdtErrorEnumV0(ScSpecUdtErrorEnumV0 {
            doc: "Contract failures.".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: "ContractError".try_into().unwrap(),
            cases: vec![
                ScSpecUdtErrorEnumCaseV0 {
                    doc: "".try_into().unwrap(),
                    name: "Unauthorized".try_into().unwrap(),
                    value: 2,
                },
                ScSpecUdtErrorEnumCaseV0 {
                    doc: "".try_into().unwrap(),
                    name: "InsufficientBalance".try_into().unwrap(),
                    value: 1,
                },
            ]
            .try_into()
            .unwrap(),
        }),
        ScSpecEntry::EventV0(ScSpecEventV0 {
            doc: "Transfer event.".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: symbol("Transfer"),
            prefix_topics: vec![symbol("transfer")].try_into().unwrap(),
            params: vec![
                ScSpecEventParamV0 {
                    doc: "".try_into().unwrap(),
                    name: "from".try_into().unwrap(),
                    type_: ScSpecTypeDef::Address,
                    location: ScSpecEventParamLocationV0::TopicList,
                },
                ScSpecEventParamV0 {
                    doc: "".try_into().unwrap(),
                    name: "amount".try_into().unwrap(),
                    type_: ScSpecTypeDef::I128,
                    location: ScSpecEventParamLocationV0::Data,
                },
            ]
            .try_into()
            .unwrap(),
            data_format: ScSpecEventDataFormat::SingleValue,
        }),
    ]
}

fn push_leb_u32(output: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn wasm_with_spec(entries: &[ScSpecEntry]) -> Vec<u8> {
    let mut xdr = Vec::new();
    for entry in entries {
        xdr.extend(entry.to_xdr(Limits::none()).unwrap());
    }

    let section_name = b"contractspecv0";
    let mut payload = Vec::new();
    push_leb_u32(&mut payload, section_name.len() as u32);
    payload.extend_from_slice(section_name);
    payload.extend_from_slice(&xdr);

    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.push(0);
    push_leb_u32(&mut wasm, payload.len() as u32);
    wasm.extend_from_slice(&payload);
    wasm
}

#[test]
fn maps_all_current_spec_entry_categories() {
    let interface = normalize_spec_entries(&entries()).unwrap();

    assert_eq!(interface.functions.len(), 1);
    assert_eq!(interface.types.len(), 3);
    assert_eq!(interface.errors.len(), 1);
    assert_eq!(interface.events.len(), 1);

    let function = &interface.functions[0];
    assert_eq!(function.name, "transfer");
    assert_eq!(
        function.parameters[0].type_ref,
        TypeRef::primitive("Address")
    );
    assert_eq!(
        function.parameters[2].type_ref.display_name(),
        "Option<String>"
    );

    let account = interface
        .types
        .iter()
        .find(|item| item.name == "Account")
        .unwrap();
    assert!(matches!(account.definition, UserTypeKind::Struct { .. }));
    if let UserTypeKind::Struct { fields } = &account.definition {
        assert_eq!(fields[1].type_ref, TypeRef::BytesN { length: 32 });
    }

    let action = interface
        .types
        .iter()
        .find(|item| item.name == "Action")
        .unwrap();
    assert!(matches!(action.definition, UserTypeKind::Enum { .. }));
    if let UserTypeKind::Enum { variants } = &action.definition {
        assert_eq!(variants[1].fields.len(), 2);
        assert_eq!(variants[1].fields[0].name, None);
    }

    assert_eq!(interface.errors[0].cases[0].value, 1);
    assert_eq!(interface.errors[0].cases[1].value, 2);

    let event = &interface.events[0];
    assert_eq!(event.prefix_topics, vec!["transfer"]);
    assert_eq!(event.parameters[0].location, EventParameterLocation::Topic);
    assert_eq!(event.parameters[1].location, EventParameterLocation::Data);
    assert_eq!(event.data_format, EventDataFormat::SingleValue);
}

#[test]
fn official_soroban_spec_reader_extracts_the_normalized_interface() {
    let expected = normalize_spec_entries(&entries()).unwrap();
    let wasm = wasm_with_spec(&entries());

    let extracted = extract_interface_from_wasm(&wasm).unwrap();

    assert_eq!(extracted, expected);
}

#[test]
fn missing_contract_spec_is_not_treated_as_success() {
    let minimal_wasm = b"\0asm\x01\0\0\0";
    let error = extract_interface_from_wasm(minimal_wasm).unwrap_err();

    assert!(matches!(
        error,
        WasmError::ContractSpec(FromWasmError::NotFound)
    ));
}

#[test]
fn invalid_wasm_is_not_treated_as_success() {
    let error = extract_interface_from_wasm(b"not wasm").unwrap_err();
    assert!(matches!(
        error,
        WasmError::ContractSpec(FromWasmError::Read(_))
    ));
}

#[test]
fn parameterized_and_nested_xdr_types_remain_structured() {
    let nested = ScSpecTypeDef::Map(Box::new(ScSpecTypeMap {
        key_type: Box::new(ScSpecTypeDef::Address),
        value_type: Box::new(ScSpecTypeDef::Option(Box::new(ScSpecTypeOption {
            value_type: Box::new(ScSpecTypeDef::Udt(ScSpecTypeUdt {
                name: "Account".try_into().unwrap(),
            })),
        }))),
    }));

    let normalized = normalize_type_ref(&nested);
    assert_eq!(normalized.display_name(), "Map<Address, Option<Account>>");
    assert!(matches!(normalized, TypeRef::Map { .. }));

    let bytes = normalize_type_ref(&ScSpecTypeDef::BytesN(ScSpecTypeBytesN { n: 64 }));
    assert_eq!(bytes, TypeRef::BytesN { length: 64 });
}
