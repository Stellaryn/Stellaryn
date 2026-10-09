use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use stellaryn_core::{ContractInterface, Function, InterfaceValidationError, Parameter};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeClassification {
    NonBreaking,
    ReviewRequired,
    Breaking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FunctionChangeId {
    FunctionAdded,
    FunctionRemoved,
    FunctionParameterAdded,
    FunctionParameterRemoved,
    FunctionParameterReordered,
    FunctionParameterRenamed,
    FunctionParameterTypeChanged,
    FunctionOutputCountChanged,
    FunctionOutputTypeChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionChange {
    pub id: FunctionChangeId,
    pub subject: String,
    pub classification: ChangeClassification,
    pub summary: String,
    pub before_evidence: Option<String>,
    pub after_evidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FunctionDiff {
    pub changes: Vec<FunctionChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DiffError {
    #[error("before interface is invalid: {0}")]
    InvalidBefore(InterfaceValidationError),

    #[error("after interface is invalid: {0}")]
    InvalidAfter(InterfaceValidationError),
}

pub fn diff_functions(
    before: &ContractInterface,
    after: &ContractInterface,
) -> Result<FunctionDiff, DiffError> {
    before.validate().map_err(DiffError::InvalidBefore)?;
    after.validate().map_err(DiffError::InvalidAfter)?;

    let before_functions: BTreeMap<&str, &Function> = before
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect();
    let after_functions: BTreeMap<&str, &Function> = after
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect();

    let names: BTreeSet<&str> = before_functions
        .keys()
        .chain(after_functions.keys())
        .copied()
        .collect();

    let mut changes = Vec::new();

    for name in names {
        match (before_functions.get(name), after_functions.get(name)) {
            (Some(old), None) => changes.push(FunctionChange {
                id: FunctionChangeId::FunctionRemoved,
                subject: old.stable_id(),
                classification: ChangeClassification::Breaking,
                summary: format!("Public function '{name}' was removed."),
                before_evidence: Some(old.signature_display()),
                after_evidence: None,
            }),
            (None, Some(new)) => changes.push(FunctionChange {
                id: FunctionChangeId::FunctionAdded,
                subject: new.stable_id(),
                classification: ChangeClassification::NonBreaking,
                summary: format!("Public function '{name}' was added."),
                before_evidence: None,
                after_evidence: Some(new.signature_display()),
            }),
            (Some(old), Some(new)) => {
                diff_parameters(old, new, &mut changes);
                diff_outputs(old, new, &mut changes);
            }
            (None, None) => {}
        }
    }

    changes.sort_by(|left, right| {
        classification_rank(left.classification)
            .cmp(&classification_rank(right.classification))
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.subject.cmp(&right.subject))
            .then_with(|| left.summary.cmp(&right.summary))
    });

    Ok(FunctionDiff { changes })
}

fn classification_rank(classification: ChangeClassification) -> u8 {
    match classification {
        ChangeClassification::Breaking => 0,
        ChangeClassification::ReviewRequired => 1,
        ChangeClassification::NonBreaking => 2,
    }
}

fn diff_parameters(before: &Function, after: &Function, changes: &mut Vec<FunctionChange>) {
    if same_parameter_shapes(&before.parameters, &after.parameters) {
        return;
    }

    if is_pure_reorder(&before.parameters, &after.parameters) {
        changes.push(FunctionChange {
            id: FunctionChangeId::FunctionParameterReordered,
            subject: before.stable_id(),
            classification: ChangeClassification::Breaking,
            summary: format!("Parameters for function '{}' were reordered.", before.name),
            before_evidence: Some(before.signature_display()),
            after_evidence: Some(after.signature_display()),
        });
        return;
    }

    if before.parameters.len() == after.parameters.len() {
        for (index, (old, new)) in before
            .parameters
            .iter()
            .zip(after.parameters.iter())
            .enumerate()
        {
            match (old.name == new.name, old.type_ref == new.type_ref) {
                (true, true) => {}
                (true, false) => push_parameter_type_change(before, after, old, new, changes),
                (false, true) => changes.push(FunctionChange {
                    id: FunctionChangeId::FunctionParameterRenamed,
                    subject: format!("{}::parameter:{index}", before.stable_id()),
                    classification: ChangeClassification::ReviewRequired,
                    summary: format!(
                        "Parameter at index {index} in function '{}' was renamed from '{}' to '{}'.",
                        before.name, old.name, new.name
                    ),
                    before_evidence: Some(format!(
                        "{}: {}",
                        old.name,
                        old.type_ref.display_name()
                    )),
                    after_evidence: Some(format!(
                        "{}: {}",
                        new.name,
                        new.type_ref.display_name()
                    )),
                }),
                (false, false) => {
                    push_parameter_type_change(before, after, old, new, changes);
                }
            }
        }
        return;
    }

    let before_by_name: BTreeMap<&str, (&Parameter, usize)> = before
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.name.as_str(), (parameter, index)))
        .collect();
    let after_by_name: BTreeMap<&str, (&Parameter, usize)> = after
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.name.as_str(), (parameter, index)))
        .collect();

    for (name, (old, index)) in &before_by_name {
        match after_by_name.get(name) {
            None => changes.push(FunctionChange {
                id: FunctionChangeId::FunctionParameterRemoved,
                subject: format!("{}::parameter:{name}", before.stable_id()),
                classification: ChangeClassification::Breaking,
                summary: format!(
                    "Parameter '{}' at index {} was removed from function '{}'.",
                    old.name, index, before.name
                ),
                before_evidence: Some(format!("{}: {}", old.name, old.type_ref.display_name())),
                after_evidence: None,
            }),
            Some((new, _)) if old.type_ref != new.type_ref => {
                push_parameter_type_change(before, after, old, new, changes);
            }
            Some(_) => {}
        }
    }

    for (name, (new, index)) in &after_by_name {
        if !before_by_name.contains_key(name) {
            changes.push(FunctionChange {
                id: FunctionChangeId::FunctionParameterAdded,
                subject: format!("{}::parameter:{name}", after.stable_id()),
                classification: ChangeClassification::Breaking,
                summary: format!(
                    "Parameter '{}' was added at index {} to function '{}'.",
                    new.name, index, after.name
                ),
                before_evidence: None,
                after_evidence: Some(format!("{}: {}", new.name, new.type_ref.display_name())),
            });
        }
    }

    let shared_before: Vec<_> = before
        .parameters
        .iter()
        .filter(|parameter| after_by_name.contains_key(parameter.name.as_str()))
        .map(|parameter| parameter.name.as_str())
        .collect();
    let shared_after: Vec<_> = after
        .parameters
        .iter()
        .filter(|parameter| before_by_name.contains_key(parameter.name.as_str()))
        .map(|parameter| parameter.name.as_str())
        .collect();

    if shared_before != shared_after {
        changes.push(FunctionChange {
            id: FunctionChangeId::FunctionParameterReordered,
            subject: before.stable_id(),
            classification: ChangeClassification::Breaking,
            summary: format!(
                "The relative order of existing parameters in function '{}' changed.",
                before.name
            ),
            before_evidence: Some(before.signature_display()),
            after_evidence: Some(after.signature_display()),
        });
    }
}

fn same_parameter_shapes(before: &[Parameter], after: &[Parameter]) -> bool {
    before.len() == after.len()
        && before
            .iter()
            .zip(after)
            .all(|(old, new)| old.name == new.name && old.type_ref == new.type_ref)
}

fn is_pure_reorder(before: &[Parameter], after: &[Parameter]) -> bool {
    if before.len() != after.len() || same_parameter_shapes(before, after) {
        return false;
    }

    before.iter().all(|old| {
        after
            .iter()
            .any(|new| old.name == new.name && old.type_ref == new.type_ref)
    })
}

fn push_parameter_type_change(
    before: &Function,
    after: &Function,
    old: &Parameter,
    new: &Parameter,
    changes: &mut Vec<FunctionChange>,
) {
    changes.push(FunctionChange {
        id: FunctionChangeId::FunctionParameterTypeChanged,
        subject: format!("{}::parameter:{}", before.stable_id(), old.name),
        classification: ChangeClassification::Breaking,
        summary: format!(
            "Parameter '{}' in function '{}' changed type from '{}' to '{}'.",
            old.name,
            before.name,
            old.type_ref.display_name(),
            new.type_ref.display_name()
        ),
        before_evidence: Some(before.signature_display()),
        after_evidence: Some(after.signature_display()),
    });
}

fn diff_outputs(before: &Function, after: &Function, changes: &mut Vec<FunctionChange>) {
    if before.outputs.len() != after.outputs.len() {
        changes.push(FunctionChange {
            id: FunctionChangeId::FunctionOutputCountChanged,
            subject: before.stable_id(),
            classification: ChangeClassification::Breaking,
            summary: format!(
                "Function '{}' changed output count from {} to {}.",
                before.name,
                before.outputs.len(),
                after.outputs.len()
            ),
            before_evidence: Some(before.signature_display()),
            after_evidence: Some(after.signature_display()),
        });
        return;
    }

    for (index, (old, new)) in before.outputs.iter().zip(after.outputs.iter()).enumerate() {
        if old != new {
            changes.push(FunctionChange {
                id: FunctionChangeId::FunctionOutputTypeChanged,
                subject: format!("{}::output:{index}", before.stable_id()),
                classification: ChangeClassification::Breaking,
                summary: format!(
                    "Output {index} of function '{}' changed type from '{}' to '{}'.",
                    before.name,
                    old.display_name(),
                    new.display_name()
                ),
                before_evidence: Some(old.display_name()),
                after_evidence: Some(new.display_name()),
            });
        }
    }
}
