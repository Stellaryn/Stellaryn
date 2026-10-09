use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const INTERFACE_SCHEMA_VERSION: &str = "1.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeRef {
    Primitive {
        name: String,
    },
    Named {
        name: String,
    },
    Option {
        value: Box<TypeRef>,
    },
    Result {
        ok: Box<TypeRef>,
        error: Box<TypeRef>,
    },
    Vector {
        element: Box<TypeRef>,
    },
    Map {
        key: Box<TypeRef>,
        value: Box<TypeRef>,
    },
    Tuple {
        elements: Vec<TypeRef>,
    },
    BytesN {
        length: u32,
    },
}

impl TypeRef {
    pub fn primitive(name: impl Into<String>) -> Self {
        Self::Primitive { name: name.into() }
    }

    pub fn named(name: impl Into<String>) -> Self {
        Self::Named { name: name.into() }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::Primitive { name } | Self::Named { name } => name.clone(),
            Self::Option { value } => format!("Option<{}>", value.display_name()),
            Self::Result { ok, error } => {
                format!("Result<{}, {}>", ok.display_name(), error.display_name())
            }
            Self::Vector { element } => format!("Vec<{}>", element.display_name()),
            Self::Map { key, value } => {
                format!("Map<{}, {}>", key.display_name(), value.display_name())
            }
            Self::Tuple { elements } => {
                let values: Vec<_> = elements.iter().map(Self::display_name).collect();
                format!("({})", values.join(", "))
            }
            Self::BytesN { length } => format!("BytesN<{length}>"),
        }
    }

    fn validate(&self, path: &str) -> Result<(), InterfaceValidationError> {
        match self {
            Self::Primitive { name } | Self::Named { name } => {
                require_name(path, name)?;
            }
            Self::Option { value } => value.validate(&format!("{path}.value"))?,
            Self::Result { ok, error } => {
                ok.validate(&format!("{path}.ok"))?;
                error.validate(&format!("{path}.error"))?;
            }
            Self::Vector { element } => element.validate(&format!("{path}.element"))?,
            Self::Map { key, value } => {
                key.validate(&format!("{path}.key"))?;
                value.validate(&format!("{path}.value"))?;
            }
            Self::Tuple { elements } => {
                for (index, element) in elements.iter().enumerate() {
                    element.validate(&format!("{path}.elements[{index}]"))?;
                }
            }
            Self::BytesN { .. } => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_ref: TypeRef,
    #[serde(default)]
    pub doc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(default)]
    pub doc: String,
    pub parameters: Vec<Parameter>,
    pub outputs: Vec<TypeRef>,
}

impl Function {
    pub fn stable_id(&self) -> String {
        format!("function:{}", self.name)
    }

    pub fn signature_display(&self) -> String {
        let params: Vec<_> = self
            .parameters
            .iter()
            .map(|param| format!("{}: {}", param.name, param.type_ref.display_name()))
            .collect();
        let output = match self.outputs.as_slice() {
            [] => String::new(),
            [single] => format!(" -> {}", single.display_name()),
            many => {
                let values: Vec<_> = many.iter().map(TypeRef::display_name).collect();
                format!(" -> ({})", values.join(", "))
            }
        };
        format!("fn {}({}){output}", self.name, params.join(", "))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub type_ref: TypeRef,
    #[serde(default)]
    pub doc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariantField {
    pub name: Option<String>,
    pub type_ref: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub discriminant: Option<u32>,
    #[serde(default)]
    pub doc: String,
    pub fields: Vec<VariantField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UserTypeKind {
    Struct { fields: Vec<StructField> },
    Enum { variants: Vec<EnumVariant> },
    Union { variants: Vec<EnumVariant> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserType {
    pub name: String,
    #[serde(default)]
    pub doc: String,
    pub definition: UserTypeKind,
}

impl UserType {
    pub fn stable_id(&self) -> String {
        format!("type:{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorCase {
    pub name: String,
    pub value: u32,
    #[serde(default)]
    pub doc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDefinition {
    pub name: String,
    #[serde(default)]
    pub doc: String,
    pub cases: Vec<ErrorCase>,
}

impl ErrorDefinition {
    pub fn stable_id(&self) -> String {
        format!("error:{}", self.name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventParameterLocation {
    Topic,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventDataFormat {
    SingleValue,
    Vec,
    Map,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventParameter {
    pub name: String,
    pub type_ref: TypeRef,
    pub location: EventParameterLocation,
    #[serde(default)]
    pub doc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventDefinition {
    pub name: String,
    #[serde(default)]
    pub doc: String,
    pub prefix_topics: Vec<String>,
    pub parameters: Vec<EventParameter>,
    pub data_format: EventDataFormat,
}

impl EventDefinition {
    pub fn stable_id(&self) -> String {
        format!("event:{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractInterface {
    pub schema_version: String,
    pub analysis_source: crate::AnalysisSource,
    pub functions: Vec<Function>,
    pub types: Vec<UserType>,
    pub errors: Vec<ErrorDefinition>,
    pub events: Vec<EventDefinition>,
}

impl ContractInterface {
    pub fn empty(analysis_source: crate::AnalysisSource) -> Self {
        Self {
            schema_version: INTERFACE_SCHEMA_VERSION.to_owned(),
            analysis_source,
            functions: Vec::new(),
            types: Vec::new(),
            errors: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), InterfaceValidationError> {
        if self.schema_version != INTERFACE_SCHEMA_VERSION {
            return Err(InterfaceValidationError::UnsupportedSchemaVersion {
                found: self.schema_version.clone(),
                expected: INTERFACE_SCHEMA_VERSION.to_owned(),
            });
        }

        validate_unique_top_level(
            "function",
            self.functions.iter().map(|item| item.name.as_str()),
        )?;
        validate_unique_top_level("type", self.types.iter().map(|item| item.name.as_str()))?;
        validate_unique_top_level("error", self.errors.iter().map(|item| item.name.as_str()))?;
        validate_unique_top_level("event", self.events.iter().map(|item| item.name.as_str()))?;

        for function in &self.functions {
            validate_function(function)?;
        }
        for user_type in &self.types {
            validate_user_type(user_type)?;
        }
        for error in &self.errors {
            validate_error(error)?;
        }
        for event in &self.events {
            validate_event(event)?;
        }

        Ok(())
    }

    pub fn normalized(mut self) -> Result<Self, InterfaceValidationError> {
        self.validate()?;

        self.functions
            .sort_by(|left, right| left.name.cmp(&right.name));
        self.types.sort_by(|left, right| left.name.cmp(&right.name));
        self.errors
            .sort_by(|left, right| left.name.cmp(&right.name));
        self.events
            .sort_by(|left, right| left.name.cmp(&right.name));

        for error in &mut self.errors {
            error
                .cases
                .sort_by(|left, right| (left.value, &left.name).cmp(&(right.value, &right.name)));
        }

        Ok(self)
    }

    pub fn stable_item_ids(&self) -> Vec<String> {
        let mut ids = Vec::with_capacity(
            self.functions.len() + self.types.len() + self.errors.len() + self.events.len(),
        );
        ids.extend(self.functions.iter().map(Function::stable_id));
        ids.extend(self.types.iter().map(UserType::stable_id));
        ids.extend(self.errors.iter().map(ErrorDefinition::stable_id));
        ids.extend(self.events.iter().map(EventDefinition::stable_id));
        ids.sort();
        ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InterfaceValidationError {
    #[error("unsupported interface schema version '{found}'; expected '{expected}'")]
    UnsupportedSchemaVersion { found: String, expected: String },

    #[error("empty name at {path}")]
    EmptyName { path: String },

    #[error("duplicate {kind} '{name}'")]
    DuplicateTopLevel { kind: String, name: String },

    #[error("duplicate {member_kind} '{name}' in {parent}")]
    DuplicateMember {
        parent: String,
        member_kind: String,
        name: String,
    },

    #[error("duplicate error value {value} in error '{error_name}'")]
    DuplicateErrorValue { error_name: String, value: u32 },

    #[error("invalid variant shape for '{variant}' in type '{type_name}'")]
    InvalidVariantShape { type_name: String, variant: String },

    #[error("duplicate enum discriminant {value} in type '{type_name}'")]
    DuplicateEnumDiscriminant { type_name: String, value: u32 },
}

fn require_name(path: &str, name: &str) -> Result<(), InterfaceValidationError> {
    if name.trim().is_empty() {
        return Err(InterfaceValidationError::EmptyName {
            path: path.to_owned(),
        });
    }
    Ok(())
}

fn validate_unique_top_level<'a>(
    kind: &str,
    names: impl Iterator<Item = &'a str>,
) -> Result<(), InterfaceValidationError> {
    let mut seen = BTreeSet::new();
    for name in names {
        require_name(kind, name)?;
        if !seen.insert(name) {
            return Err(InterfaceValidationError::DuplicateTopLevel {
                kind: kind.to_owned(),
                name: name.to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_named_members<'a>(
    parent: &str,
    member_kind: &str,
    names: impl Iterator<Item = &'a str>,
) -> Result<(), InterfaceValidationError> {
    let mut seen = BTreeSet::new();
    for name in names {
        require_name(&format!("{parent}.{member_kind}"), name)?;
        if !seen.insert(name) {
            return Err(InterfaceValidationError::DuplicateMember {
                parent: parent.to_owned(),
                member_kind: member_kind.to_owned(),
                name: name.to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_function(function: &Function) -> Result<(), InterfaceValidationError> {
    require_name("function", &function.name)?;
    validate_named_members(
        &format!("function:{}", function.name),
        "parameter",
        function
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str()),
    )?;

    for (index, parameter) in function.parameters.iter().enumerate() {
        parameter
            .type_ref
            .validate(&format!("function:{}:parameter:{index}", function.name))?;
    }
    for (index, output) in function.outputs.iter().enumerate() {
        output.validate(&format!("function:{}:output:{index}", function.name))?;
    }

    Ok(())
}

fn validate_user_type(user_type: &UserType) -> Result<(), InterfaceValidationError> {
    require_name("type", &user_type.name)?;

    match &user_type.definition {
        UserTypeKind::Struct { fields } => {
            validate_named_members(
                &format!("type:{}", user_type.name),
                "field",
                fields.iter().map(|field| field.name.as_str()),
            )?;
            for (index, field) in fields.iter().enumerate() {
                field
                    .type_ref
                    .validate(&format!("type:{}:field:{index}", user_type.name))?;
            }
        }
        UserTypeKind::Enum { variants } | UserTypeKind::Union { variants } => {
            validate_named_members(
                &format!("type:{}", user_type.name),
                "variant",
                variants.iter().map(|variant| variant.name.as_str()),
            )?;
            let mut discriminants = BTreeSet::new();
            for (variant_index, variant) in variants.iter().enumerate() {
                match &user_type.definition {
                    UserTypeKind::Enum { .. } => {
                        let value = variant.discriminant.ok_or_else(|| {
                            InterfaceValidationError::InvalidVariantShape {
                                type_name: user_type.name.clone(),
                                variant: variant.name.clone(),
                            }
                        })?;
                        if !variant.fields.is_empty() {
                            return Err(InterfaceValidationError::InvalidVariantShape {
                                type_name: user_type.name.clone(),
                                variant: variant.name.clone(),
                            });
                        }
                        if !discriminants.insert(value) {
                            return Err(InterfaceValidationError::DuplicateEnumDiscriminant {
                                type_name: user_type.name.clone(),
                                value,
                            });
                        }
                    }
                    UserTypeKind::Union { .. } => {
                        if variant.discriminant.is_some() {
                            return Err(InterfaceValidationError::InvalidVariantShape {
                                type_name: user_type.name.clone(),
                                variant: variant.name.clone(),
                            });
                        }
                    }
                    UserTypeKind::Struct { .. } => {}
                }
                let named_fields: Vec<_> = variant
                    .fields
                    .iter()
                    .filter_map(|field| field.name.as_deref())
                    .collect();
                validate_named_members(
                    &format!("type:{}:variant:{}", user_type.name, variant.name),
                    "field",
                    named_fields.into_iter(),
                )?;
                for (field_index, field) in variant.fields.iter().enumerate() {
                    if let Some(name) = &field.name {
                        require_name(
                            &format!(
                                "type:{}:variant:{variant_index}:field:{field_index}",
                                user_type.name
                            ),
                            name,
                        )?;
                    }
                    field.type_ref.validate(&format!(
                        "type:{}:variant:{variant_index}:field:{field_index}",
                        user_type.name
                    ))?;
                }
            }
        }
    }

    Ok(())
}

fn validate_error(error: &ErrorDefinition) -> Result<(), InterfaceValidationError> {
    require_name("error", &error.name)?;
    validate_named_members(
        &format!("error:{}", error.name),
        "case",
        error.cases.iter().map(|case| case.name.as_str()),
    )?;

    let mut values = BTreeMap::new();
    for case in &error.cases {
        if values.insert(case.value, case.name.as_str()).is_some() {
            return Err(InterfaceValidationError::DuplicateErrorValue {
                error_name: error.name.clone(),
                value: case.value,
            });
        }
    }

    Ok(())
}

fn validate_event(event: &EventDefinition) -> Result<(), InterfaceValidationError> {
    require_name("event", &event.name)?;
    validate_named_members(
        &format!("event:{}", event.name),
        "parameter",
        event
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str()),
    )?;

    for (index, parameter) in event.parameters.iter().enumerate() {
        parameter
            .type_ref
            .validate(&format!("event:{}:parameter:{index}", event.name))?;
    }

    Ok(())
}
