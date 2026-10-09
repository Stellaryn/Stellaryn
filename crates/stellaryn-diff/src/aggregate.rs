//! Aggregate every compatibility rule family into one deterministic result.
//!
//! COMPATIBLE is a *spec-level* result, not a security-audit result, and does
//! not prove runtime behavior, stored-state safety, or a safe deployment.

use serde::{Deserialize, Serialize};
use stellaryn_core::{ContractInterface, Verdict};

use crate::{
    diff_errors, diff_events, diff_functions, diff_types, ChangeClassification, DiffError,
    EventErrorChange, EventErrorChangeId, FunctionChange, FunctionChangeId, TypeChange,
    TypeChangeId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "domain", content = "id", rename_all = "snake_case")]
pub enum CompatibilityRule {
    Function(FunctionChangeId),
    CustomType(TypeChangeId),
    Event(EventErrorChangeId),
    Error(EventErrorChangeId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityFinding {
    pub rule: CompatibilityRule,
    pub subject: String,
    pub classification: ChangeClassification,
    pub summary: String,
    pub before_evidence: Option<String>,
    pub after_evidence: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingCounts {
    pub breaking: usize,
    pub review_required: usize,
    pub non_breaking: usize,
}

impl FindingCounts {
    #[must_use]
    pub fn total(self) -> usize {
        self.breaking + self.review_required + self.non_breaking
    }

    fn record(&mut self, classification: ChangeClassification) {
        match classification {
            ChangeClassification::Breaking => self.breaking += 1,
            ChangeClassification::ReviewRequired => self.review_required += 1,
            ChangeClassification::NonBreaking => self.non_breaking += 1,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainCounts {
    pub functions: FindingCounts,
    pub custom_types: FindingCounts,
    pub events: FindingCounts,
    pub errors: FindingCounts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractDiff {
    pub verdict: Verdict,
    pub totals: FindingCounts,
    pub by_domain: DomainCounts,
    pub findings: Vec<CompatibilityFinding>,
}

fn severity(classification: ChangeClassification) -> u8 {
    match classification {
        ChangeClassification::Breaking => 0,
        ChangeClassification::ReviewRequired => 1,
        ChangeClassification::NonBreaking => 2,
    }
}

fn from_function(item: FunctionChange) -> CompatibilityFinding {
    CompatibilityFinding {
        rule: CompatibilityRule::Function(item.id),
        subject: item.subject,
        classification: item.classification,
        summary: item.summary,
        before_evidence: item.before_evidence,
        after_evidence: item.after_evidence,
    }
}

fn from_type(item: TypeChange) -> CompatibilityFinding {
    CompatibilityFinding {
        rule: CompatibilityRule::CustomType(item.id),
        subject: item.subject,
        classification: item.classification,
        summary: item.summary,
        before_evidence: item.before_evidence,
        after_evidence: item.after_evidence,
    }
}

fn from_event(item: EventErrorChange) -> CompatibilityFinding {
    from_event_or_error(item, false)
}

fn from_error(item: EventErrorChange) -> CompatibilityFinding {
    from_event_or_error(item, true)
}

fn from_event_or_error(item: EventErrorChange, error: bool) -> CompatibilityFinding {
    CompatibilityFinding {
        rule: if error {
            CompatibilityRule::Error(item.id)
        } else {
            CompatibilityRule::Event(item.id)
        },
        subject: item.subject,
        classification: item.classification,
        summary: item.summary,
        before_evidence: item.before_evidence,
        after_evidence: item.after_evidence,
    }
}

/// Compare every supported public contract-spec category.
///
/// The individual engines validate the inputs; malformed interfaces return
/// typed errors rather than producing a COMPATIBLE verdict.
pub fn diff_contracts(
    before: &ContractInterface,
    after: &ContractInterface,
) -> Result<ContractDiff, DiffError> {
    let mut findings = Vec::new();
    findings.extend(diff_functions(before, after)?.changes.into_iter().map(from_function));
    findings.extend(diff_types(before, after)?.changes.into_iter().map(from_type));
    findings.extend(diff_events(before, after)?.changes.into_iter().map(from_event));
    findings.extend(diff_errors(before, after)?.changes.into_iter().map(from_error));

    findings.sort_by(|left, right| {
        severity(left.classification)
            .cmp(&severity(right.classification))
            .then_with(|| left.rule.cmp(&right.rule))
            .then_with(|| left.subject.cmp(&right.subject))
            .then_with(|| left.summary.cmp(&right.summary))
            .then_with(|| left.before_evidence.cmp(&right.before_evidence))
            .then_with(|| left.after_evidence.cmp(&right.after_evidence))
    });

    let mut totals = FindingCounts::default();
    let mut by_domain = DomainCounts::default();
    for finding in &findings {
        totals.record(finding.classification);
        let counts = match finding.rule {
            CompatibilityRule::Function(_) => &mut by_domain.functions,
            CompatibilityRule::CustomType(_) => &mut by_domain.custom_types,
            CompatibilityRule::Event(_) => &mut by_domain.events,
            CompatibilityRule::Error(_) => &mut by_domain.errors,
        };
        counts.record(finding.classification);
    }

    let verdict = if totals.breaking > 0 {
        Verdict::Incompatible
    } else if totals.review_required > 0 {
        Verdict::ReviewRequired
    } else {
        Verdict::Compatible
    };

    Ok(ContractDiff {
        verdict,
        totals,
        by_domain,
        findings,
    })
}
