//! Deterministic contract error and event compatibility analysis.
//!
//! Findings are spec-level observations, not a security audit or a
//! deployment-safety verdict.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use stellaryn_core::{
    ContractInterface, ErrorCase, ErrorDefinition, EventDataFormat, EventDefinition,
    EventParameter, EventParameterLocation,
};

use crate::{ChangeClassification, DiffError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventErrorChangeId {
    ErrorDefinitionAdded,
    ErrorDefinitionRemoved,
    ErrorCaseAdded,
    ErrorCaseRemoved,
    ErrorCaseRenamed,
    ErrorCodeChanged,
    EventAdded,
    EventRemoved,
    EventPrefixTopicsChanged,
    EventDataFormatChanged,
    EventParameterAdded,
    EventParameterRemoved,
    EventParameterRenamed,
    EventParameterReordered,
    EventParameterTypeChanged,
    EventParameterLocationChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventErrorChange {
    pub id: EventErrorChangeId,
    pub subject: String,
    pub classification: ChangeClassification,
    pub summary: String,
    pub before_evidence: Option<String>,
    pub after_evidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EventErrorDiff {
    pub changes: Vec<EventErrorChange>,
}

fn push_change(
    changes: &mut Vec<EventErrorChange>,
    id: EventErrorChangeId,
    subject: String,
    classification: ChangeClassification,
    summary: String,
    before_evidence: Option<String>,
    after_evidence: Option<String>,
) {
    changes.push(EventErrorChange {
        id,
        subject,
        classification,
        summary,
        before_evidence,
        after_evidence,
    });
}

fn rank(classification: ChangeClassification) -> u8 {
    match classification {
        ChangeClassification::Breaking => 0,
        ChangeClassification::ReviewRequired => 1,
        ChangeClassification::NonBreaking => 2,
    }
}

fn finalize(mut changes: Vec<EventErrorChange>) -> EventErrorDiff {
    changes.sort_by(|left, right| {
        rank(left.classification)
            .cmp(&rank(right.classification))
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.subject.cmp(&right.subject))
            .then_with(|| left.summary.cmp(&right.summary))
    });
    EventErrorDiff { changes }
}

fn validated(before: &ContractInterface, after: &ContractInterface) -> Result<(), DiffError> {
    before.validate().map_err(DiffError::InvalidBefore)?;
    after.validate().map_err(DiffError::InvalidAfter)
}

/// Compare both public error codes and event specifications.
pub fn diff_events_and_errors(
    before: &ContractInterface,
    after: &ContractInterface,
) -> Result<EventErrorDiff, DiffError> {
    validated(before, after)?;
    let mut changes = Vec::new();
    compare_errors(before, after, &mut changes);
    compare_events(before, after, &mut changes);
    Ok(finalize(changes))
}

/// Compare only error definitions and their numeric cases.
pub fn diff_errors(
    before: &ContractInterface,
    after: &ContractInterface,
) -> Result<EventErrorDiff, DiffError> {
    validated(before, after)?;
    let mut changes = Vec::new();
    compare_errors(before, after, &mut changes);
    Ok(finalize(changes))
}

/// Compare only event definitions.
pub fn diff_events(
    before: &ContractInterface,
    after: &ContractInterface,
) -> Result<EventErrorDiff, DiffError> {
    validated(before, after)?;
    let mut changes = Vec::new();
    compare_events(before, after, &mut changes);
    Ok(finalize(changes))
}

fn compare_errors(
    before: &ContractInterface,
    after: &ContractInterface,
    changes: &mut Vec<EventErrorChange>,
) {
    let old: BTreeMap<&str, &ErrorDefinition> = before
        .errors
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let new: BTreeMap<&str, &ErrorDefinition> = after
        .errors
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let names: BTreeSet<&str> = old.keys().chain(new.keys()).copied().collect();

    for name in names {
        match (old.get(name), new.get(name)) {
            (Some(item), None) => push_change(
                changes,
                EventErrorChangeId::ErrorDefinitionRemoved,
                item.stable_id(),
                ChangeClassification::Breaking,
                format!("Error definition '{name}' was removed."),
                Some(error_cases_display(item)),
                None,
            ),
            (None, Some(item)) => push_change(
                changes,
                EventErrorChangeId::ErrorDefinitionAdded,
                item.stable_id(),
                ChangeClassification::NonBreaking,
                format!("Error definition '{name}' was added."),
                None,
                Some(error_cases_display(item)),
            ),
            (Some(old_item), Some(new_item)) => {
                compare_error_cases(name, &old_item.cases, &new_item.cases, changes);
            }
            (None, None) => {}
        }
    }
}

fn error_cases_display(item: &ErrorDefinition) -> String {
    let mut cases: Vec<_> = item
        .cases
        .iter()
        .map(|case| (case.value, case.name.as_str()))
        .collect();
    cases.sort();
    cases
        .iter()
        .map(|(value, name)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn compare_error_cases(
    error_name: &str,
    before: &[ErrorCase],
    after: &[ErrorCase],
    changes: &mut Vec<EventErrorChange>,
) {
    let old: BTreeMap<&str, &ErrorCase> = before
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let new: BTreeMap<&str, &ErrorCase> = after
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();

    for (name, item) in &old {
        if let Some(updated) = new.get(name) {
            if item.value != updated.value {
                push_change(
                    changes,
                    EventErrorChangeId::ErrorCodeChanged,
                    format!("error:{error_name}::case:{name}"),
                    ChangeClassification::Breaking,
                    format!("Numeric error code of '{error_name}::{name}' changed."),
                    Some(item.value.to_string()),
                    Some(updated.value.to_string()),
                );
            }
        }
    }

    // A case absent by name on both sides may be the same numeric error
    // under a new identifier. Match only unmatched names to avoid
    // misclassifying genuine code reassignment as a rename.
    let newly_named_by_code: BTreeMap<u32, &ErrorCase> = after
        .iter()
        .filter(|item| !old.contains_key(item.name.as_str()))
        .map(|item| (item.value, item))
        .collect();
    let mut renamed_new = BTreeSet::new();

    for (name, old_case) in &old {
        if new.contains_key(name) {
            continue;
        }
        if let Some(renamed) = newly_named_by_code.get(&old_case.value) {
            // The numeric code may also be claimed by a surviving case;
            // treat such a collision as a removal/addition rather than
            // inferring an unproven rename.
            let claimed_by_survivor = old.iter().any(|(surviving_name, item)| {
                new.contains_key(surviving_name) && item.value == renamed.value
            });
            if !claimed_by_survivor {
                renamed_new.insert(renamed.name.as_str());
                push_change(
                    changes,
                    EventErrorChangeId::ErrorCaseRenamed,
                    format!("error:{error_name}::code:{}", old_case.value),
                    ChangeClassification::ReviewRequired,
                    format!(
                        "Error code {} changed name from '{}' to '{}'.",
                        old_case.value, old_case.name, renamed.name
                    ),
                    Some(old_case.name.clone()),
                    Some(renamed.name.clone()),
                );
                continue;
            }
        }
        push_change(
            changes,
            EventErrorChangeId::ErrorCaseRemoved,
            format!("error:{error_name}::case:{name}"),
            ChangeClassification::Breaking,
            format!("Error case '{error_name}::{name}' was removed."),
            Some(old_case.value.to_string()),
            None,
        );
    }

    for (name, new_case) in &new {
        if !old.contains_key(name) && !renamed_new.contains(name) {
            push_change(
                changes,
                EventErrorChangeId::ErrorCaseAdded,
                format!("error:{error_name}::case:{name}"),
                ChangeClassification::ReviewRequired,
                format!("Error case '{error_name}::{name}' was added."),
                None,
                Some(new_case.value.to_string()),
            );
        }
    }
}

fn compare_events(
    before: &ContractInterface,
    after: &ContractInterface,
    changes: &mut Vec<EventErrorChange>,
) {
    let old: BTreeMap<&str, &EventDefinition> = before
        .events
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let new: BTreeMap<&str, &EventDefinition> = after
        .events
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let names: BTreeSet<&str> = old.keys().chain(new.keys()).copied().collect();

    for name in names {
        match (old.get(name), new.get(name)) {
            (Some(item), None) => push_change(
                changes,
                EventErrorChangeId::EventRemoved,
                item.stable_id(),
                ChangeClassification::Breaking,
                format!("Event '{name}' was removed."),
                Some(event_evidence(item)),
                None,
            ),
            (None, Some(item)) => push_change(
                changes,
                EventErrorChangeId::EventAdded,
                item.stable_id(),
                ChangeClassification::NonBreaking,
                format!("Event '{name}' was added."),
                None,
                Some(event_evidence(item)),
            ),
            (Some(old_event), Some(new_event)) => {
                if old_event.prefix_topics != new_event.prefix_topics {
                    push_change(
                        changes,
                        EventErrorChangeId::EventPrefixTopicsChanged,
                        old_event.stable_id(),
                        ChangeClassification::Breaking,
                        format!("Event '{name}' changed its prefix topics."),
                        Some(format!("{:?}", old_event.prefix_topics)),
                        Some(format!("{:?}", new_event.prefix_topics)),
                    );
                }
                if old_event.data_format != new_event.data_format {
                    push_change(
                        changes,
                        EventErrorChangeId::EventDataFormatChanged,
                        old_event.stable_id(),
                        ChangeClassification::Breaking,
                        format!("Event '{name}' changed its data format."),
                        Some(format!("{:?}", old_event.data_format)),
                        Some(format!("{:?}", new_event.data_format)),
                    );
                }
                compare_event_parameters(old_event, new_event, changes);
            }
            (None, None) => {}
        }
    }
}

fn event_evidence(event: &EventDefinition) -> String {
    format!(
        "prefix_topics={:?}, data_format={:?}, parameters={}",
        event.prefix_topics,
        event.data_format,
        event
            .parameters
            .iter()
            .map(event_parameter_evidence)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn event_parameter_evidence(parameter: &EventParameter) -> String {
    format!(
        "{}: {} ({:?})",
        parameter.name,
        parameter.type_ref.display_name(),
        parameter.location
    )
}

fn compare_event_parameters(
    before: &EventDefinition,
    after: &EventDefinition,
    changes: &mut Vec<EventErrorChange>,
) {
    let old: BTreeMap<&str, (&EventParameter, usize)> = before
        .parameters
        .iter()
        .enumerate()
        .map(|(index, param)| (param.name.as_str(), (param, index)))
        .collect();
    let new: BTreeMap<&str, (&EventParameter, usize)> = after
        .parameters
        .iter()
        .enumerate()
        .map(|(index, param)| (param.name.as_str(), (param, index)))
        .collect();

    for (name, (item, _)) in &old {
        if let Some((updated, _)) = new.get(name) {
            let subject = format!("{}::parameter:{name}", before.stable_id());
            if item.type_ref != updated.type_ref {
                push_change(
                    changes,
                    EventErrorChangeId::EventParameterTypeChanged,
                    subject.clone(),
                    ChangeClassification::Breaking,
                    format!("Type of event parameter '{name}' changed in '{}'.", before.name),
                    Some(item.type_ref.display_name()),
                    Some(updated.type_ref.display_name()),
                );
            }
            if item.location != updated.location {
                push_change(
                    changes,
                    EventErrorChangeId::EventParameterLocationChanged,
                    subject,
                    ChangeClassification::Breaking,
                    format!("Location of event parameter '{name}' changed in '{}'.", before.name),
                    Some(format!("{:?}", item.location)),
                    Some(format!("{:?}", updated.location)),
                );
            }
        }
    }

    let surviving_before: Vec<_> = before
        .parameters
        .iter()
        .filter(|item| new.contains_key(item.name.as_str()))
        .map(|item| item.name.as_str())
        .collect();
    let surviving_after: Vec<_> = after
        .parameters
        .iter()
        .filter(|item| old.contains_key(item.name.as_str()))
        .map(|item| item.name.as_str())
        .collect();
    if surviving_before != surviving_after {
        push_change(
            changes,
            EventErrorChangeId::EventParameterReordered,
            before.stable_id(),
            ChangeClassification::Breaking,
            format!("Existing event parameters changed order in '{}'.", before.name),
            Some(surviving_before.join(", ")),
            Some(surviving_after.join(", ")),
        );
    }

    let mut renamed_old = BTreeSet::new();
    let mut renamed_new = BTreeSet::new();
    if before.parameters.len() == after.parameters.len() {
        for (index, (item, updated)) in before
            .parameters
            .iter()
            .zip(after.parameters.iter())
            .enumerate()
        {
            if !new.contains_key(item.name.as_str())
                && !old.contains_key(updated.name.as_str())
                && item.type_ref == updated.type_ref
                && item.location == updated.location
            {
                renamed_old.insert(item.name.as_str());
                renamed_new.insert(updated.name.as_str());
                let map_data = (before.data_format == EventDataFormat::Map
                    || after.data_format == EventDataFormat::Map)
                    && item.location == EventParameterLocation::Data;
                push_change(
                    changes,
                    EventErrorChangeId::EventParameterRenamed,
                    format!("{}::parameter:{index}", before.stable_id()),
                    if map_data {
                        ChangeClassification::Breaking
                    } else {
                        ChangeClassification::ReviewRequired
                    },
                    format!(
                        "Event parameter at index {index} in '{}' changed name from '{}' to '{}'.",
                        before.name, item.name, updated.name
                    ),
                    Some(item.name.clone()),
                    Some(updated.name.clone()),
                );
            }
        }
    }

    for (name, (item, index)) in &old {
        if !new.contains_key(name) && !renamed_old.contains(name) {
            push_change(
                changes,
                EventErrorChangeId::EventParameterRemoved,
                format!("{}::parameter:{name}", before.stable_id()),
                ChangeClassification::Breaking,
                format!("Event parameter '{name}' at index {index} was removed from '{}'.", before.name),
                Some(event_parameter_evidence(item)),
                None,
            );
        }
    }

    for (name, (item, index)) in &new {
        if !old.contains_key(name) && !renamed_new.contains(name) {
            push_change(
                changes,
                EventErrorChangeId::EventParameterAdded,
                format!("{}::parameter:{name}", after.stable_id()),
                ChangeClassification::Breaking,
                format!("Event parameter '{name}' was added at index {index} in '{}'.", after.name),
                None,
                Some(event_parameter_evidence(item)),
            );
        }
    }
}
