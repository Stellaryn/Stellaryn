//! Custom-type compatibility rules for normalized Soroban interfaces.
//!
//! Compare struct fields by name, numeric enum variants by name/discriminant,
//! and tagged union cases by name/payload. These findings are not a deployment
//! safety guarantee and do not constitute an overall verdict.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use stellaryn_core::{
    ContractInterface, EnumVariant, StructField, UserType, UserTypeKind,
};

use crate::{ChangeClassification, DiffError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeChangeId {
    TypeAdded,
    TypeRemoved,
    TypeKindChanged,
    StructFieldAdded,
    StructFieldRemoved,
    StructFieldTypeChanged,
    StructFieldReordered,
    EnumVariantAdded,
    EnumVariantRemoved,
    EnumDiscriminantChanged,
    UnionVariantAdded,
    UnionVariantRemoved,
    UnionPayloadCountChanged,
    UnionPayloadTypeChanged,
    UnionPayloadNameChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeChange {
    pub id: TypeChangeId,
    pub subject: String,
    pub classification: ChangeClassification,
    pub summary: String,
    pub before_evidence: Option<String>,
    pub after_evidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TypeDiff {
    pub changes: Vec<TypeChange>,
}

fn add(
    changes: &mut Vec<TypeChange>,
    id: TypeChangeId,
    subject: String,
    classification: ChangeClassification,
    summary: String,
    before: Option<String>,
    after: Option<String>,
) {
    changes.push(TypeChange {
        id,
        subject,
        classification,
        summary,
        before_evidence: before,
        after_evidence: after,
    });
}

fn priority(classification: ChangeClassification) -> u8 {
    match classification {
        ChangeClassification::Breaking => 0,
        ChangeClassification::ReviewRequired => 1,
        ChangeClassification::NonBreaking => 2,
    }
}

fn kind_name(kind: &UserTypeKind) -> &'static str {
    match kind {
        UserTypeKind::Struct { .. } => "struct",
        UserTypeKind::Enum { .. } => "numeric enum",
        UserTypeKind::Union { .. } => "tagged union",
    }
}

/// Reports type-specific differences; function, error, and event diffs are separate.
pub fn diff_types(
    before: &ContractInterface,
    after: &ContractInterface,
) -> Result<TypeDiff, DiffError> {
    before.validate().map_err(DiffError::InvalidBefore)?;
    after.validate().map_err(DiffError::InvalidAfter)?;

    let old: BTreeMap<&str, &UserType> = before
        .types
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let new: BTreeMap<&str, &UserType> = after
        .types
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let names: BTreeSet<&str> = old.keys().chain(new.keys()).copied().collect();
    let mut changes = Vec::new();

    for name in names {
        match (old.get(name), new.get(name)) {
            (Some(item), None) => add(
                &mut changes,
                TypeChangeId::TypeRemoved,
                item.stable_id(),
                ChangeClassification::Breaking,
                format!("Custom type '{name}' was removed."),
                Some(kind_name(&item.definition).to_owned()),
                None,
            ),
            (None, Some(item)) => add(
                &mut changes,
                TypeChangeId::TypeAdded,
                item.stable_id(),
                ChangeClassification::NonBreaking,
                format!("Custom type '{name}' was added."),
                None,
                Some(kind_name(&item.definition).to_owned()),
            ),
            (Some(old), Some(new)) => match (&old.definition, &new.definition) {
                (UserTypeKind::Struct { fields: old_fields }, UserTypeKind::Struct { fields: new_fields }) => {
                    diff_struct(name, old_fields, new_fields, &mut changes);
                }
                (UserTypeKind::Enum { variants: old_variants }, UserTypeKind::Enum { variants: new_variants }) => {
                    diff_variants(name, old_variants, new_variants, true, &mut changes);
                }
                (UserTypeKind::Union { variants: old_variants }, UserTypeKind::Union { variants: new_variants }) => {
                    diff_variants(name, old_variants, new_variants, false, &mut changes);
                }
                _ => add(
                    &mut changes,
                    TypeChangeId::TypeKindChanged,
                    old.stable_id(),
                    ChangeClassification::Breaking,
                    format!("Custom type '{name}' changed its definition kind."),
                    Some(kind_name(&old.definition).to_owned()),
                    Some(kind_name(&new.definition).to_owned()),
                ),
            },
            (None, None) => {}
        }
    }

    changes.sort_by(|a, b| {
        priority(a.classification)
            .cmp(&priority(b.classification))
            .then_with(|| a.id.cmp(&b.id))
            .then_with(|| a.subject.cmp(&b.subject))
            .then_with(|| a.summary.cmp(&b.summary))
    });
    Ok(TypeDiff { changes })
}

fn diff_struct(
    name: &str,
    before: &[StructField],
    after: &[StructField],
    changes: &mut Vec<TypeChange>,
) {
    let old: BTreeMap<&str, &StructField> = before
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect();
    let new: BTreeMap<&str, &StructField> = after
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect();

    for (field_name, field) in &old {
        let subject = format!("type:{name}::field:{field_name}");
        match new.get(field_name) {
            None => add(
                changes,
                TypeChangeId::StructFieldRemoved,
                subject,
                ChangeClassification::Breaking,
                format!("Field '{field_name}' was removed from struct '{name}'."),
                Some(field.type_ref.display_name()),
                None,
            ),
            Some(updated) if field.type_ref != updated.type_ref => add(
                changes,
                TypeChangeId::StructFieldTypeChanged,
                subject,
                ChangeClassification::Breaking,
                format!("Field '{field_name}' changed type in struct '{name}'."),
                Some(field.type_ref.display_name()),
                Some(updated.type_ref.display_name()),
            ),
            Some(_) => {}
        }
    }
    for (field_name, field) in &new {
        if !old.contains_key(field_name) {
            add(
                changes,
                TypeChangeId::StructFieldAdded,
                format!("type:{name}::field:{field_name}"),
                ChangeClassification::Breaking,
                format!("Field '{field_name}' was added to struct '{name}'."),
                None,
                Some(field.type_ref.display_name()),
            );
        }
    }

    let common_before: Vec<_> = before
        .iter()
        .filter(|field| new.contains_key(field.name.as_str()))
        .map(|field| field.name.as_str())
        .collect();
    let common_after: Vec<_> = after
        .iter()
        .filter(|field| old.contains_key(field.name.as_str()))
        .map(|field| field.name.as_str())
        .collect();
    if common_before != common_after {
        add(
            changes,
            TypeChangeId::StructFieldReordered,
            format!("type:{name}"),
            ChangeClassification::ReviewRequired,
            format!("Existing fields of struct '{name}' changed declaration order."),
            Some(common_before.join(", ")),
            Some(common_after.join(", ")),
        );
    }
}

fn diff_variants(
    name: &str,
    before: &[EnumVariant],
    after: &[EnumVariant],
    is_numeric: bool,
    changes: &mut Vec<TypeChange>,
) {
    let old: BTreeMap<&str, &EnumVariant> = before
        .iter()
        .map(|variant| (variant.name.as_str(), variant))
        .collect();
    let new: BTreeMap<&str, &EnumVariant> = after
        .iter()
        .map(|variant| (variant.name.as_str(), variant))
        .collect();

    for (variant_name, variant) in &old {
        let subject = format!("type:{name}::variant:{variant_name}");
        match new.get(variant_name) {
            None => add(
                changes,
                if is_numeric { TypeChangeId::EnumVariantRemoved } else { TypeChangeId::UnionVariantRemoved },
                subject,
                ChangeClassification::Breaking,
                format!("Variant '{variant_name}' was removed from '{name}'."),
                Some(variant_evidence(variant)),
                None,
            ),
            Some(updated) => {
                if is_numeric {
                    if variant.discriminant != updated.discriminant {
                        add(
                            changes,
                            TypeChangeId::EnumDiscriminantChanged,
                            subject,
                            ChangeClassification::Breaking,
                            format!("Numeric value of '{name}::{variant_name}' changed."),
                            variant.discriminant.map(|value| value.to_string()),
                            updated.discriminant.map(|value| value.to_string()),
                        );
                    }
                } else {
                    diff_payload(name, variant_name, variant, updated, changes);
                }
            }
        }
    }

    for (variant_name, variant) in &new {
        if !old.contains_key(variant_name) {
            add(
                changes,
                if is_numeric { TypeChangeId::EnumVariantAdded } else { TypeChangeId::UnionVariantAdded },
                format!("type:{name}::variant:{variant_name}"),
                ChangeClassification::ReviewRequired,
                format!("Variant '{variant_name}' was added to '{name}'."),
                None,
                Some(variant_evidence(variant)),
            );
        }
    }
}

fn variant_evidence(variant: &EnumVariant) -> String {
    if let Some(value) = variant.discriminant {
        return format!("{} = {value}", variant.name);
    }
    let types: Vec<_> = variant
        .fields
        .iter()
        .map(|field| field.type_ref.display_name())
        .collect();
    format!("{}({})", variant.name, types.join(", "))
}

fn diff_payload(
    name: &str,
    variant_name: &str,
    before: &EnumVariant,
    after: &EnumVariant,
    changes: &mut Vec<TypeChange>,
) {
    let subject = format!("type:{name}::variant:{variant_name}");
    if before.fields.len() != after.fields.len() {
        add(
            changes,
            TypeChangeId::UnionPayloadCountChanged,
            subject,
            ChangeClassification::Breaking,
            format!("Payload arity of '{name}::{variant_name}' changed."),
            Some(before.fields.len().to_string()),
            Some(after.fields.len().to_string()),
        );
        return;
    }

    for (index, (old, new)) in before.fields.iter().zip(after.fields.iter()).enumerate() {
        if old.type_ref != new.type_ref {
            add(
                changes,
                TypeChangeId::UnionPayloadTypeChanged,
                format!("{subject}::payload:{index}"),
                ChangeClassification::Breaking,
                format!("Payload type at index {index} changed for '{name}::{variant_name}'."),
                Some(old.type_ref.display_name()),
                Some(new.type_ref.display_name()),
            );
        }
        if old.name != new.name {
            add(
                changes,
                TypeChangeId::UnionPayloadNameChanged,
                format!("{subject}::payload:{index}"),
                ChangeClassification::ReviewRequired,
                format!("Payload field name at index {index} changed for '{name}::{variant_name}'."),
                old.name.clone(),
                new.name.clone(),
            );
        }
    }
}
