//! Deterministic compatibility comparison for normalized Soroban interfaces.
//!
//! Phases 4–6 cover functions, custom types, errors, and events.
//! Overall verdict aggregation and process exit policy are reserved for Phase 7.

mod events_errors;
mod function;
mod types;

pub use function::{
    diff_functions, ChangeClassification, DiffError, FunctionChange, FunctionChangeId, FunctionDiff,
};

pub use types::{diff_types, TypeChange, TypeChangeId, TypeDiff};
pub use events_errors::{
    diff_errors, diff_events, diff_events_and_errors, EventErrorChange, EventErrorChangeId,
    EventErrorDiff,
};

pub const COMPONENT: &str = "stellaryn-diff";
