//! Deterministic compatibility comparison for normalized Soroban interfaces.
//!
//! Phases 4–6 cover functions, custom types, errors, and events.
//! Phase 7 aggregates these findings into a spec-level verdict and CI exit policy.

mod aggregate;
mod events_errors;
mod function;
mod policy;
mod types;

pub use function::{
    diff_functions, ChangeClassification, DiffError, FunctionChange, FunctionChangeId, FunctionDiff,
};

pub use events_errors::{
    diff_errors, diff_events, diff_events_and_errors, EventErrorChange, EventErrorChangeId,
    EventErrorDiff,
};
pub use types::{diff_types, TypeChange, TypeChangeId, TypeDiff};

pub use aggregate::{
    diff_contracts, CompatibilityFinding, CompatibilityRule, ContractDiff, DomainCounts,
    FindingCounts,
};
pub use policy::{
    ExitPolicy, FailOn, ParseFailOnError, EXIT_ANALYSIS_ERROR, EXIT_POLICY_VIOLATION, EXIT_SUCCESS,
};

pub const COMPONENT: &str = "stellaryn-diff";
