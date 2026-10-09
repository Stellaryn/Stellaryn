//! Deterministic compatibility comparison for normalized Soroban interfaces.
//!
//! Phase 4 implements function-level compatibility only. Custom types, errors,
//! and events are intentionally deferred to later phases.

mod function;

pub use function::{
    diff_functions, ChangeClassification, DiffError, FunctionChange, FunctionChangeId, FunctionDiff,
};

pub const COMPONENT: &str = "stellaryn-diff";
