//! Human-friendly and stable JSON output for completed Soroban contract diffs.
//!
//! Renderers are deterministic: they never add timestamps, colors, or
//! filesystem reads. Caller-supplied input labels are echoed unchanged.

use serde::Serialize;
use stellaryn_core::{Verdict, DISCLAIMER};
use stellaryn_diff::{ChangeClassification, CompatibilityRule, ContractDiff};
use thiserror::Error;

pub const COMPONENT: &str = "stellaryn-report";
pub const REPORT_SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("could not serialize compatibility report: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Serialize)]
struct JsonReport<'a> {
    schema_version: &'static str,
    before: &'a str,
    after: &'a str,
    analysis: &'a ContractDiff,
    disclaimer: &'static str,
}

/// Deterministic, machine-readable analysis output.
///
/// A policy violation still produces the entire report; the CLI selects
/// the process exit code only after writing it.
pub fn render_json(
    analysis: &ContractDiff,
    before: &str,
    after: &str,
) -> Result<String, ReportError> {
    Ok(serde_json::to_string_pretty(&JsonReport {
        schema_version: REPORT_SCHEMA_VERSION,
        before,
        after,
        analysis,
        disclaimer: DISCLAIMER,
    })?)
}

/// Plain-text, non-ANSI terminal output suitable for logs and redirection.
#[must_use]
pub fn render_terminal(analysis: &ContractDiff, before: &str, after: &str) -> String {
    let mut result = String::new();
    result.push_str("Stellaryn — Soroban contract interface comparison\n");
    result.push_str(&format!("Before: {before}\nAfter:  {after}\n\n"));
    result.push_str(&format!("Overall verdict: {}\n", analysis.verdict));
    result.push_str(&format!(
        "Findings: {} total ({} breaking, {} review required, {} non-breaking)\n",
        analysis.totals.total(),
        analysis.totals.breaking,
        analysis.totals.review_required,
        analysis.totals.non_breaking,
    ));
    result.push_str(&format!(
        "By domain: functions {}, custom types {}, events {}, errors {}\n",
        analysis.by_domain.functions.total(),
        analysis.by_domain.custom_types.total(),
        analysis.by_domain.events.total(),
        analysis.by_domain.errors.total(),
    ));

    if analysis.findings.is_empty() {
        result.push_str("\nNo public contract-spec compatibility changes detected.\n");
    } else {
        result.push_str("\nCompatibility findings:\n");
        for finding in &analysis.findings {
            let classification = match finding.classification {
                ChangeClassification::Breaking => "BREAKING",
                ChangeClassification::ReviewRequired => "REVIEW_REQUIRED",
                ChangeClassification::NonBreaking => "NON_BREAKING",
            };
            let (domain, id) = match finding.rule {
                CompatibilityRule::Function(id) => ("function", format!("{id:?}")),
                CompatibilityRule::CustomType(id) => ("custom_type", format!("{id:?}")),
                CompatibilityRule::Event(id) => ("event", format!("{id:?}")),
                CompatibilityRule::Error(id) => ("error", format!("{id:?}")),
            };
            result.push_str(&format!(
                "\n  [{classification}] {} {}\n",
                domain,
                to_upper_snake(&id),
            ));
            result.push_str(&format!("  {}\n", finding.subject));
            result.push_str(&format!("  {}\n", finding.summary));
            if let Some(before) = &finding.before_evidence {
                result.push_str(&format!("  Before: {before}\n"));
            }
            if let Some(after) = &finding.after_evidence {
                result.push_str(&format!("  After:  {after}\n"));
            }
        }
    }
    result.push_str("\n");
    result.push_str(DISCLAIMER);
    result.push('\n');
    result
}

fn to_upper_snake(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_uppercase() && !output.is_empty() {
            output.push('_');
        }
        output.extend(character.to_uppercase());
    }
    output
}

/// Safe for users to read, but not a deployment approval.
#[must_use]
pub fn verdict_is_incompatible(analysis: &ContractDiff) -> bool {
    analysis.verdict == Verdict::Incompatible
}
