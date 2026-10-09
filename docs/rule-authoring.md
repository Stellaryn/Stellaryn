# Compatibility rule authoring and review

This guide is for contributors changing Stellaryn's contract-specification compatibility classification. Do not infer unverified Soroban ABI/runtime properties.

## Where rules live

| Domain | Code | Regression suites |
| --- | --- | --- |
| Public functions | `crates/stellaryn-diff/src/function.rs` | `crates/stellaryn-diff/tests/function_diff.rs` |
| Structs, numeric enums, tagged unions | `crates/stellaryn-diff/src/types.rs` | `crates/stellaryn-diff/tests/type_diff.rs` |
| Events and error enums | `crates/stellaryn-diff/src/events_errors.rs` | `crates/stellaryn-diff/tests/event_error_diff.rs` |
| Overall finding aggregation and verdict | `crates/stellaryn-diff/src/aggregate.rs` | `crates/stellaryn-diff/tests/contract_diff.rs` |
| Exit policy | `crates/stellaryn-diff/src/policy.rs` | `crates/stellaryn-diff/tests/contract_diff.rs` |
| Soroban XDR parsing and normalization | `crates/stellaryn-wasm/src/lib.rs` | `crates/stellaryn-wasm/tests/extraction.rs` |
| CLI/report output | `crates/stellaryn-cli/src/main.rs` and `crates/stellaryn-report/src/lib.rs` | `tests/cli/compare.rs` and `crates/stellaryn-report/tests/report.rs` |

There are further end-to-end tests in `tests/cli/fixture_matrix.rs`, `tests/cli/invalid_fixtures.rs`, and `tests/cli/real_world.rs`.

## Classification semantics

- `BREAKING`: existing clients depending on the old public spec may fail under an established conservative rule.
- `REVIEW_REQUIRED`: likely client/tooling impact, but spec evidence does not justify an automatic breaking verdict.
- `NON_BREAKING`: additive under the current public-spec rules, **not** guaranteed to be runtime safe.

The highest severity determines the contract verdict: breaking wins over review; review wins over non-breaking. The overall verdict is implemented in `aggregate.rs`. The CLI policy is separate. Do not change exit codes by modifying individual rule logic.

## Workflow for changing a rule

1. **Describe the contract**: which XDR field, original spec and new spec, affected clients, and the precise claim. Cite a versioned SDK/XDR spec, Stellar CLI output, or an independently compiled artifact. No invented APIs or binary encodings.
2. **Decide the correct domain**: Soroban-specific decoding belongs in `stellaryn-wasm`; normalization invariants belong in `stellaryn-core`; compatibility logic belongs in `stellaryn-diff`; presentation belongs in `stellaryn-report`.
3. **Keep rule identity stable**: add a typed `ChangeId` only when needed; preserve `subject`, classification, summary and available before/after evidence. If changing meaning of an existing rule, document the migration and consider downstream JSON users.
4. **Provide both kinds of tests**: a positive case proving the rule is emitted, and a negative case proving a harmless or ambiguous neighbor does **not** incorrectly emit it. Test missing entries, duplicate names/codes, reorderings, nested types and mixed severity as appropriate.
5. **Use realistic fixtures**: start with normalized unit tests; add generated XDR-backed end-to-end fixtures when mapping is relevant. Use pinned independently compiled WASM for an externally observed behavior, and preserve its origin/hash.
6. **Record the policy**: update the relevant `docs/*-diff.md` page, examples or limitations if the classification changes. Identify exactly what is known, inferred, or still unverified.
7. **Run all quality gates** shown in [CONTRIBUTING](../CONTRIBUTING.md), not only the new test. CI rejects lint warnings and formatted-code discrepancies.

## Example review question

Suppose an enum gains a numeric case. Current `ENUM_VARIANT_ADDED` is `REVIEW_REQUIRED` because exhaustive downstream consumers may need a change even though existing cases retain their numeric values. Do **not** silently reclassify it as `NON_BREAKING` merely because one test client tolerated it. Attach specific consumer behavior and contract-spec evidence before changing the policy.

## Required PR evidence

The pull request should state: motivation; before/after ABI; expected rule ID, subject and severity; positive/negative test cases; source links or pinned hashes; schema/report compatibility consequences; actual `cargo fmt`, `cargo clippy`, `cargo test` output; and safety limits.

[Browse all rule definitions](function-diff.md) · [Custom-type rules](type-diff.md) · [Event/error rules](error-event-diff.md) · [Report contract](cli-reports.md).
