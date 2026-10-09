//! CI gating for completed contract-spec comparisons.
//!
//! Nonzero analysis errors are independent of this policy. A permissive
//! policy never turns invalid input or failed extraction into a valid result.

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use stellaryn_core::Verdict;
use thiserror::Error;

use crate::ContractDiff;

/// Successful analysis without a policy violation.
pub const EXIT_SUCCESS: i32 = 0;
/// Reserved for invalid input, extraction, or comparison errors at the CLI.
pub const EXIT_ANALYSIS_ERROR: i32 = 1;
/// Successful analysis, but CI policy rejected its findings.
pub const EXIT_POLICY_VIOLATION: i32 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailOn {
    #[default]
    Breaking,
    Review,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid fail-on policy '{input}'; expected 'breaking', 'review', or 'never'")]
pub struct ParseFailOnError {
    pub input: String,
}

impl FromStr for FailOn {
    type Err = ParseFailOnError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "breaking" => Ok(Self::Breaking),
            "review" => Ok(Self::Review),
            "never" => Ok(Self::Never),
            _ => Err(ParseFailOnError {
                input: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExitPolicy {
    pub fail_on: FailOn,
}

impl ExitPolicy {
    #[must_use]
    pub fn should_fail(self, verdict: Verdict) -> bool {
        match self.fail_on {
            FailOn::Breaking => matches!(verdict, Verdict::Incompatible),
            FailOn::Review => {
                matches!(verdict, Verdict::Incompatible | Verdict::ReviewRequired)
            }
            FailOn::Never => false,
        }
    }

    /// Call only after successful analysis. Errors always exit 1 in the CLI.
    #[must_use]
    pub fn exit_code(self, result: &ContractDiff) -> i32 {
        if self.should_fail(result.verdict) {
            EXIT_POLICY_VIOLATION
        } else {
            EXIT_SUCCESS
        }
    }
}
