//! Shared domain types for Stellaryn.

mod interface;

pub use interface::{
    ContractInterface, EnumVariant, ErrorCase, ErrorDefinition, EventDefinition,
    EventParameter, EventParameterLocation, Function, InterfaceValidationError, Parameter,
    StructField, TypeRef, UserType, UserTypeKind, VariantField, INTERFACE_SCHEMA_VERSION,
};

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub const PRODUCT_NAME: &str = "Stellaryn";
pub const DISCLAIMER: &str = "Passing Stellaryn is not a security audit and does not prove that a contract upgrade is safe to deploy.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Verdict {
    Compatible,
    ReviewRequired,
    Incompatible,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Compatible => "COMPATIBLE",
            Self::ReviewRequired => "REVIEW_REQUIRED",
            Self::Incompatible => "INCOMPATIBLE",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisSource {
    WasmSpec,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn verdict_display_is_stable() {
        assert_eq!(Verdict::Compatible.to_string(), "COMPATIBLE");
        assert_eq!(Verdict::ReviewRequired.to_string(), "REVIEW_REQUIRED");
        assert_eq!(Verdict::Incompatible.to_string(), "INCOMPATIBLE");
    }

    #[test]
    fn verdict_serialization_is_stable() {
        let value = serde_json::to_string(&Verdict::ReviewRequired).unwrap();
        assert_eq!(value, "\"REVIEW_REQUIRED\"");
    }
}
