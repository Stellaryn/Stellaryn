//! Verified Soroban Wasm contract-spec extraction.
//!
//! Stellaryn reads the same `contractspecv0` section consumed by Stellar CLI
//! through `soroban_spec::read::from_wasm`, then maps the typed XDR entries
//! into the comparison-first model in `stellaryn-core`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use stellar_xdr::{
    ScSpecEntry, ScSpecEventDataFormat, ScSpecEventParamLocationV0, ScSpecTypeDef,
    ScSpecUdtUnionCaseV0,
};
use stellaryn_core::{
    AnalysisSource, ContractInterface, EnumVariant, ErrorCase, ErrorDefinition, EventDataFormat,
    EventDefinition, EventParameter, EventParameterLocation, Function, InterfaceValidationError,
    Parameter, StructField, TypeRef, UserType, UserTypeKind, VariantField,
};
use thiserror::Error;

pub const COMPONENT: &str = "stellaryn-wasm";
pub const SOROBAN_SPEC_VERSION: &str = "28.0.0";
pub const STELLAR_XDR_VERSION: &str = "28.0.0";

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("failed to read Wasm file '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to read Soroban contract specification from Wasm: {0}")]
    ContractSpec(#[from] soroban_spec::read::FromWasmError),

    #[error("normalized contract interface is invalid: {0}")]
    InvalidInterface(#[from] InterfaceValidationError),
}

pub fn extract_interface_from_path(path: impl AsRef<Path>) -> Result<ContractInterface, WasmError> {
    let path = path.as_ref();
    let wasm = fs::read(path).map_err(|source| WasmError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    extract_interface_from_wasm(&wasm)
}

pub fn extract_interface_from_wasm(wasm: &[u8]) -> Result<ContractInterface, WasmError> {
    let entries = soroban_spec::read::from_wasm(wasm)?;
    normalize_spec_entries(&entries)
}

pub fn normalize_spec_entries(entries: &[ScSpecEntry]) -> Result<ContractInterface, WasmError> {
    let mut interface = ContractInterface::empty(AnalysisSource::WasmSpec);

    for entry in entries {
        match entry {
            ScSpecEntry::FunctionV0(function) => {
                interface.functions.push(Function {
                    name: function.name.to_utf8_string_lossy(),
                    doc: function.doc.to_utf8_string_lossy(),
                    parameters: function
                        .inputs
                        .iter()
                        .map(|input| Parameter {
                            name: input.name.to_utf8_string_lossy(),
                            type_ref: normalize_type_ref(&input.type_),
                            doc: input.doc.to_utf8_string_lossy(),
                        })
                        .collect(),
                    outputs: function.outputs.iter().map(normalize_type_ref).collect(),
                });
            }
            ScSpecEntry::UdtStructV0(user_type) => {
                interface.types.push(UserType {
                    name: user_type.name.to_utf8_string_lossy(),
                    doc: user_type.doc.to_utf8_string_lossy(),
                    definition: UserTypeKind::Struct {
                        fields: user_type
                            .fields
                            .iter()
                            .map(|field| StructField {
                                name: field.name.to_utf8_string_lossy(),
                                type_ref: normalize_type_ref(&field.type_),
                                doc: field.doc.to_utf8_string_lossy(),
                            })
                            .collect(),
                    },
                });
            }
            ScSpecEntry::UdtUnionV0(user_type) => {
                let variants = user_type
                    .cases
                    .iter()
                    .map(|case| match case {
                        ScSpecUdtUnionCaseV0::VoidV0(variant) => EnumVariant {
                            name: variant.name.to_utf8_string_lossy(),
                            discriminant: None,
                            doc: variant.doc.to_utf8_string_lossy(),
                            fields: Vec::new(),
                        },
                        ScSpecUdtUnionCaseV0::TupleV0(variant) => EnumVariant {
                            name: variant.name.to_utf8_string_lossy(),
                            discriminant: None,
                            doc: variant.doc.to_utf8_string_lossy(),
                            fields: variant
                                .type_
                                .iter()
                                .map(|type_ref| VariantField {
                                    name: None,
                                    type_ref: normalize_type_ref(type_ref),
                                })
                                .collect(),
                        },
                    })
                    .collect();

                interface.types.push(UserType {
                    name: user_type.name.to_utf8_string_lossy(),
                    doc: user_type.doc.to_utf8_string_lossy(),
                    definition: UserTypeKind::Enum { variants },
                });
            }
            ScSpecEntry::UdtEnumV0(user_type) => {
                interface.types.push(UserType {
                    name: user_type.name.to_utf8_string_lossy(),
                    doc: user_type.doc.to_utf8_string_lossy(),
                    definition: UserTypeKind::Enum {
                        variants: user_type
                            .cases
                            .iter()
                            .map(|variant| EnumVariant {
                                name: variant.name.to_utf8_string_lossy(),
                                discriminant: Some(variant.value),
                                doc: variant.doc.to_utf8_string_lossy(),
                                fields: Vec::new(),
                            })
                            .collect(),
                    },
                });
            }
            ScSpecEntry::UdtErrorEnumV0(error) => {
                interface.errors.push(ErrorDefinition {
                    name: error.name.to_utf8_string_lossy(),
                    doc: error.doc.to_utf8_string_lossy(),
                    cases: error
                        .cases
                        .iter()
                        .map(|case| ErrorCase {
                            name: case.name.to_utf8_string_lossy(),
                            value: case.value,
                            doc: case.doc.to_utf8_string_lossy(),
                        })
                        .collect(),
                });
            }
            ScSpecEntry::EventV0(event) => {
                interface.events.push(EventDefinition {
                    name: event.name.to_utf8_string_lossy(),
                    doc: event.doc.to_utf8_string_lossy(),
                    prefix_topics: event
                        .prefix_topics
                        .iter()
                        .map(|topic| topic.to_utf8_string_lossy())
                        .collect(),
                    parameters: event
                        .params
                        .iter()
                        .map(|parameter| EventParameter {
                            name: parameter.name.to_utf8_string_lossy(),
                            type_ref: normalize_type_ref(&parameter.type_),
                            location: normalize_event_location(parameter.location),
                            doc: parameter.doc.to_utf8_string_lossy(),
                        })
                        .collect(),
                    data_format: normalize_event_data_format(event.data_format),
                });
            }
        }
    }

    interface.normalized().map_err(WasmError::from)
}

#[must_use]
pub fn normalize_type_ref(type_ref: &ScSpecTypeDef) -> TypeRef {
    match type_ref {
        ScSpecTypeDef::Val => TypeRef::primitive("Val"),
        ScSpecTypeDef::Bool => TypeRef::primitive("bool"),
        ScSpecTypeDef::Void => TypeRef::primitive("void"),
        ScSpecTypeDef::Error => TypeRef::primitive("Error"),
        ScSpecTypeDef::U32 => TypeRef::primitive("u32"),
        ScSpecTypeDef::I32 => TypeRef::primitive("i32"),
        ScSpecTypeDef::U64 => TypeRef::primitive("u64"),
        ScSpecTypeDef::I64 => TypeRef::primitive("i64"),
        ScSpecTypeDef::Timepoint => TypeRef::primitive("Timepoint"),
        ScSpecTypeDef::Duration => TypeRef::primitive("Duration"),
        ScSpecTypeDef::U128 => TypeRef::primitive("u128"),
        ScSpecTypeDef::I128 => TypeRef::primitive("i128"),
        ScSpecTypeDef::U256 => TypeRef::primitive("U256"),
        ScSpecTypeDef::I256 => TypeRef::primitive("I256"),
        ScSpecTypeDef::Bytes => TypeRef::primitive("Bytes"),
        ScSpecTypeDef::String => TypeRef::primitive("String"),
        ScSpecTypeDef::Symbol => TypeRef::primitive("Symbol"),
        ScSpecTypeDef::Address => TypeRef::primitive("Address"),
        ScSpecTypeDef::MuxedAddress => TypeRef::primitive("MuxedAddress"),
        ScSpecTypeDef::Option(option) => TypeRef::Option {
            value: Box::new(normalize_type_ref(option.value_type.as_ref())),
        },
        ScSpecTypeDef::Result(result) => TypeRef::Result {
            ok: Box::new(normalize_type_ref(result.ok_type.as_ref())),
            error: Box::new(normalize_type_ref(result.error_type.as_ref())),
        },
        ScSpecTypeDef::Vec(vector) => TypeRef::Vector {
            element: Box::new(normalize_type_ref(vector.element_type.as_ref())),
        },
        ScSpecTypeDef::Map(map) => TypeRef::Map {
            key: Box::new(normalize_type_ref(map.key_type.as_ref())),
            value: Box::new(normalize_type_ref(map.value_type.as_ref())),
        },
        ScSpecTypeDef::Tuple(tuple) => TypeRef::Tuple {
            elements: tuple.value_types.iter().map(normalize_type_ref).collect(),
        },
        ScSpecTypeDef::BytesN(bytes) => TypeRef::BytesN { length: bytes.n },
        ScSpecTypeDef::Udt(user_type) => TypeRef::named(user_type.name.to_utf8_string_lossy()),
    }
}

const fn normalize_event_location(
    location: ScSpecEventParamLocationV0,
) -> EventParameterLocation {
    match location {
        ScSpecEventParamLocationV0::Data => EventParameterLocation::Data,
        ScSpecEventParamLocationV0::TopicList => EventParameterLocation::Topic,
    }
}

const fn normalize_event_data_format(format: ScSpecEventDataFormat) -> EventDataFormat {
    match format {
        ScSpecEventDataFormat::SingleValue => EventDataFormat::SingleValue,
        ScSpecEventDataFormat::Vec => EventDataFormat::Vec,
        ScSpecEventDataFormat::Map => EventDataFormat::Map,
    }
}
