# Error and Event Compatibility (Phase 6)

This phase adds comparisons for **public error definitions** and **Soroban event specifications**. These rules consume normalized `ContractInterface` values produced by the verified extractor. They do not execute contract code or claim deployment safety.

## Error rules

| Rule ID | Classification | Meaning |
| --- | --- | --- |
| `ERROR_DEFINITION_ADDED` | NON_BREAKING | New named error definition |
| `ERROR_DEFINITION_REMOVED` | BREAKING | Named error definition no longer exists |
| `ERROR_CASE_ADDED` | REVIEW_REQUIRED | New numeric error case may affect exhaustive consumers |
| `ERROR_CASE_REMOVED` | BREAKING | Previous error case no longer exists |
| `ERROR_CASE_RENAMED` | REVIEW_REQUIRED | Same numeric code, different name (unmatched names only) |
| `ERROR_CODE_CHANGED` | BREAKING | Existing named case changed numeric value |

Error cases are matched by name before checking numeric identities, so a numeric reassignment is not silently dismissed as a rename. A renamed case with a **stable numeric code** receives review; a renamed case with a **different code** generates an addition and removal rather than a guessed identity.

## Event rules

| Rule ID | Classification | Meaning |
| --- | --- | --- |
| `EVENT_ADDED` | NON_BREAKING | New named event specification |
| `EVENT_REMOVED` | BREAKING | Named event specification removed |
| `EVENT_PREFIX_TOPICS_CHANGED` | BREAKING | Ordered prefix topics changed |
| `EVENT_DATA_FORMAT_CHANGED` | BREAKING | Event data encoding shape changed |
| `EVENT_PARAMETER_ADDED` | BREAKING | Event parameter added |
| `EVENT_PARAMETER_REMOVED` | BREAKING | Event parameter removed |
| `EVENT_PARAMETER_RENAMED` | REVIEW_REQUIRED; BREAKING for Map data | Name changed, same positional type/location |
| `EVENT_PARAMETER_REORDERED` | BREAKING | Relative order of surviving named parameters changed |
| `EVENT_PARAMETER_TYPE_CHANGED` | BREAKING | Structured parameter type changed |
| `EVENT_PARAMETER_LOCATION_CHANGED` | BREAKING | Parameter moved between topic and data |

Names are matched first, including when positional order changes. Pure renames are inferred only for same-index, unmatched before/after names with identical type and location. A Map **data** key rename is conservatively breaking; other pure parameter renames require review rather than asserting a proven runtime break.

## Determinism and scope

The `diff_errors`, `diff_events`, and `diff_events_and_errors` APIs validate both interfaces before comparison. Findings are sorted by breaking/review/non-breaking priority, rule ID, subject, and summary. Documentation-only changes and error-case declaration reorderings are ignored.

The overall upgrade verdict/exit policy (Phase 7) and the human/JSON reports and end-user CLI (Phase 8) are **now implemented** elsewhere in the workspace. An empty event/error diff means no *error/event spec changes detected*, not a declaration that an upgrade is safe.
