//! Deterministic Soroban XDR test fixtures embedded in valid minimal WASM.
//! These modules exercise contractspecv0; they are NOT deployable contracts.

#![allow(clippy::unwrap_used)]

use stellar_xdr::{
    Limits, ScSpecEntry, ScSpecEventDataFormat, ScSpecEventParamLocationV0, ScSpecEventParamV0,
    ScSpecEventV0, ScSpecFunctionInputV0, ScSpecFunctionV0, ScSpecTypeBytesN, ScSpecTypeDef,
    ScSpecUdtEnumCaseV0, ScSpecUdtEnumV0, ScSpecUdtErrorEnumCaseV0, ScSpecUdtErrorEnumV0,
    ScSpecUdtStructFieldV0, ScSpecUdtStructV0, ScSpecUdtUnionCaseTupleV0, ScSpecUdtUnionCaseV0,
    ScSpecUdtUnionCaseVoidV0, ScSpecUdtUnionV0, ScSymbol, WriteXdr,
};

pub fn symbol(value: &str) -> ScSymbol {
    ScSymbol(value.try_into().unwrap())
}

pub fn argument(name: &str, ty: ScSpecTypeDef) -> ScSpecFunctionInputV0 {
    ScSpecFunctionInputV0 {
        doc: "".try_into().unwrap(),
        name: name.try_into().unwrap(),
        type_: ty,
    }
}

pub fn function(name: &str, args: Vec<ScSpecFunctionInputV0>) -> ScSpecEntry {
    ScSpecEntry::FunctionV0(ScSpecFunctionV0 {
        doc: "".try_into().unwrap(),
        name: symbol(name),
        inputs: args.try_into().unwrap(),
        outputs: vec![ScSpecTypeDef::Bool].try_into().unwrap(),
    })
}

pub fn struct_field(name: &str, ty: ScSpecTypeDef) -> ScSpecUdtStructFieldV0 {
    ScSpecUdtStructFieldV0 {
        doc: "".try_into().unwrap(),
        name: name.try_into().unwrap(),
        type_: ty,
    }
}

pub fn enum_case(name: &str, value: u32) -> ScSpecUdtEnumCaseV0 {
    ScSpecUdtEnumCaseV0 {
        doc: "".try_into().unwrap(),
        name: name.try_into().unwrap(),
        value,
    }
}

pub fn error_case(name: &str, value: u32) -> ScSpecUdtErrorEnumCaseV0 {
    ScSpecUdtErrorEnumCaseV0 {
        doc: "".try_into().unwrap(),
        name: name.try_into().unwrap(),
        value,
    }
}

pub fn topic(name: &str, ty: ScSpecTypeDef) -> ScSpecEventParamV0 {
    ScSpecEventParamV0 {
        doc: "".try_into().unwrap(),
        name: name.try_into().unwrap(),
        type_: ty,
        location: ScSpecEventParamLocationV0::TopicList,
    }
}

pub fn data(name: &str, ty: ScSpecTypeDef) -> ScSpecEventParamV0 {
    ScSpecEventParamV0 {
        doc: "".try_into().unwrap(),
        name: name.try_into().unwrap(),
        type_: ty,
        location: ScSpecEventParamLocationV0::Data,
    }
}

pub fn baseline() -> Vec<ScSpecEntry> {
    vec![
        function(
            "transfer",
            vec![
                argument("to", ScSpecTypeDef::Address),
                argument("amount", ScSpecTypeDef::I128),
            ],
        ),
        ScSpecEntry::UdtStructV0(ScSpecUdtStructV0 {
            doc: "".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: "Account".try_into().unwrap(),
            fields: vec![
                struct_field("owner", ScSpecTypeDef::Address),
                struct_field("hash", ScSpecTypeDef::BytesN(ScSpecTypeBytesN { n: 32 })),
            ]
            .try_into()
            .unwrap(),
        }),
        ScSpecEntry::UdtEnumV0(ScSpecUdtEnumV0 {
            doc: "".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: "Status".try_into().unwrap(),
            cases: vec![enum_case("Active", 0), enum_case("Paused", 1)]
                .try_into()
                .unwrap(),
        }),
        ScSpecEntry::UdtUnionV0(ScSpecUdtUnionV0 {
            doc: "".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: "Action".try_into().unwrap(),
            cases: vec![
                ScSpecUdtUnionCaseV0::VoidV0(ScSpecUdtUnionCaseVoidV0 {
                    doc: "".try_into().unwrap(),
                    name: "Stop".try_into().unwrap(),
                }),
                ScSpecUdtUnionCaseV0::TupleV0(ScSpecUdtUnionCaseTupleV0 {
                    doc: "".try_into().unwrap(),
                    name: "Pay".try_into().unwrap(),
                    type_: vec![ScSpecTypeDef::Address, ScSpecTypeDef::I128]
                        .try_into()
                        .unwrap(),
                }),
            ]
            .try_into()
            .unwrap(),
        }),
        ScSpecEntry::UdtErrorEnumV0(ScSpecUdtErrorEnumV0 {
            doc: "".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: "ContractError".try_into().unwrap(),
            cases: vec![error_case("Unauthorized", 1), error_case("Insufficient", 2)]
                .try_into()
                .unwrap(),
        }),
        ScSpecEntry::EventV0(ScSpecEventV0 {
            doc: "".try_into().unwrap(),
            lib: "".try_into().unwrap(),
            name: symbol("Transfer"),
            prefix_topics: vec![symbol("transfer")].try_into().unwrap(),
            params: vec![
                topic("from", ScSpecTypeDef::Address),
                data("amount", ScSpecTypeDef::I128),
            ]
            .try_into()
            .unwrap(),
            data_format: ScSpecEventDataFormat::Vec,
        }),
    ]
}

pub fn function_mut(entries: &mut [ScSpecEntry]) -> &mut ScSpecFunctionV0 {
    entries.iter_mut().find_map(|entry| {
        if let ScSpecEntry::FunctionV0(value) = entry {
            Some(value)
        } else {
            None
        }
    }).unwrap()
}

pub fn struct_mut(entries: &mut [ScSpecEntry]) -> &mut ScSpecUdtStructV0 {
    entries.iter_mut().find_map(|entry| {
        if let ScSpecEntry::UdtStructV0(value) = entry {
            Some(value)
        } else {
            None
        }
    }).unwrap()
}

pub fn enum_mut(entries: &mut [ScSpecEntry]) -> &mut ScSpecUdtEnumV0 {
    entries.iter_mut().find_map(|entry| {
        if let ScSpecEntry::UdtEnumV0(value) = entry {
            Some(value)
        } else {
            None
        }
    }).unwrap()
}

pub fn union_mut(entries: &mut [ScSpecEntry]) -> &mut ScSpecUdtUnionV0 {
    entries.iter_mut().find_map(|entry| {
        if let ScSpecEntry::UdtUnionV0(value) = entry {
            Some(value)
        } else {
            None
        }
    }).unwrap()
}

pub fn error_mut(entries: &mut [ScSpecEntry]) -> &mut ScSpecUdtErrorEnumV0 {
    entries.iter_mut().find_map(|entry| {
        if let ScSpecEntry::UdtErrorEnumV0(value) = entry {
            Some(value)
        } else {
            None
        }
    }).unwrap()
}

pub fn event_mut(entries: &mut [ScSpecEntry]) -> &mut ScSpecEventV0 {
    entries.iter_mut().find_map(|entry| {
        if let ScSpecEntry::EventV0(value) = entry {
            Some(value)
        } else {
            None
        }
    }).unwrap()
}

pub fn leb_u32(output: &mut Vec<u8>, mut value: u32) {
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

/// Build a real WebAssembly custom section with official XDR entries.
pub fn wasm(entries: &[ScSpecEntry]) -> Vec<u8> {
    let mut xdr = Vec::new();
    for item in entries {
        xdr.extend(item.to_xdr(Limits::none()).unwrap());
    }
    wasm_raw_spec(&xdr)
}

pub fn wasm_raw_spec(raw: &[u8]) -> Vec<u8> {
    let mut payload = Vec::new();
    let section = b"contractspecv0";
    leb_u32(&mut payload, section.len() as u32);
    payload.extend_from_slice(section);
    payload.extend_from_slice(raw);
    let mut module = b"\0asm\x01\0\0\0".to_vec();
    module.push(0);
    leb_u32(&mut module, payload.len() as u32);
    module.extend_from_slice(&payload);
    module
}

pub fn missing_spec_wasm() -> Vec<u8> {
    b"\0asm\x01\0\0\0".to_vec()
}
