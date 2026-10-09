# Overall Verdict and CI Exit Policy (Phase 7)

Phase 7 joins every existing normalized interface comparison into a single typed result. The API accepts two `ContractInterface` instances and returns `Result<ContractDiff, DiffError>`:

```rust
let diff = stellaryn_diff::diff_contracts(&before, &after)?;
let status = stellaryn_diff::ExitPolicy::default().exit_code(&diff);
```

`ContractDiff` includes `verdict`, `totals`, `by_domain`, and `findings`. Each finding has a strongly typed `CompatibilityRule` (`Function`, `CustomType`, `Event`, `Error`), stable subject, classification, summary, and available before/after evidence. Findings are sorted **breaking first**, then review, then non-breaking, followed by typed rule, subject, summary, and evidence. Domain counts are derived from exactly that list so no rule is counted twice.

## Verdict precedence

| Findings detected | Overall `Verdict` |
| --- | --- |
| Any breaking finding | `INCOMPATIBLE` |
| No breaking findings, but one or more review-required findings | `REVIEW_REQUIRED` |
| Only non-breaking findings, or no findings | `COMPATIBLE` |

**Scope:** `COMPATIBLE` means no known breaking or review-required findings within the **normalized public contract specification** and implemented rules. It **does not prove deployment safety, runtime behavior equivalence, storage migration safety, or security**. Missing/malformed input is an error, never an empty successful result.

## Exit policy for CI

`ExitPolicy { fail_on: FailOn::... }` evaluates a **successful** comparison. The supported policy strings for eventual CLI wiring are `breaking`, `review`, and `never`.

| Verdict | `breaking` (default) | `review` | `never` |
| --- | --- | --- | --- |
| `COMPATIBLE` | 0 | 0 | 0 |
| `REVIEW_REQUIRED` | 0 | 2 | 0 |
| `INCOMPATIBLE` | 2 | 2 | 0 |

Exit code `0` means comparison completed without violating the selected policy, **not** a safety guarantee. Code `2` means the findings violated the policy. Exit code `1` is reserved for extraction, validation, or analysis errors, regardless of `fail_on`. `never` **cannot** silence malformed or missing contract evidence.

Phase 7 implements the Rust library result and exit policy, not the end-user `stellaryn compare` subcommand. Phase 8 will wire those values into the CLI with terminal/JSON reports and exercise process-level exits.

## Acceptance

On the exact final commit, the repository CI must pass formatting, strict Clippy, all workspace tests, and CLI help. Regression tests cover pure additions, pure reviews, breaking changes, mixed severity precedence, deterministic global ordering, domain counts, JSON round-trips, policy matrix, parse errors, and failure for invalid before/after input. 
