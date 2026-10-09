//! Deterministic compatibility comparison for normalized Soroban interfaces.
//!
//! Phases 4 and 5 cover functions and custom types respectively.
//! Error and event compatibility remain for Phase 6.

mod function;
mod types;

pub use function::{
    diff_functions, ChangeClassification, DiffError, FunctionChange, FunctionChangeId, FunctionDiff,
};

pub use types::{diff_types, TypeChange, TypeChangeId, TypeDiff};

pub const COMPONENT: &str = "stellaryn-diff";
