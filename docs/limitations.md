# Scope, trust boundaries, and limitations

Stellaryn is a **public Soroban contract-specification comparator**. It reads `contractspecv0` from compiled WASM and compares the normalized functions, custom types (structs, numeric enums, tagged unions), error codes, and event specifications.

## What it can report

- The exact public-spec differences detected by implemented compatibility rules.
- Findings classified as `BREAKING`, `REVIEW_REQUIRED`, or `NON_BREAKING`.
- An aggregate `COMPATIBLE`, `REVIEW_REQUIRED`, or `INCOMPATIBLE` verdict based on those findings.
- Deterministic terminal/JSON reports suitable for developer review and CI policy gates.

## What it does **not** check

- Whether contract executable code implements equivalent behavior, even if the ABI is identical.
- Stored-state layout, migration routines, storage access semantics, or authorization correctness.
- Deployment transaction correctness or live chain upgrade behavior.
- Runtime memory/CPU behavior, gas budgets, or operational reliability.
- Security vulnerabilities, audit quality, or trustworthiness of external code.
- Contracts without a parseable, nonempty, unique `contractspecv0` section.
- Whether the comparison inputs are genuinely two historical versions of the **same** deployed contract (the caller supplies files/refs).

These gaps remain even if the verdict is `COMPATIBLE`. In particular, a self-diff is a parser/regression test, **not** evidence of upgrade safety.

## Evidence types

**Phase 10** uses synthetic XDR in minimal WebAssembly custom sections to test rules and fail-closed inputs. **Phase 11** adds six independently compiled, code-bearing WASM fixtures from pinned public GitHub repositories. The real compiled `add(i128)` and `add(u128)` variants show ABI-breaking differences but are not verified on-chain upgrades of one contract ID. Mainnet labels are inherited from an upstream dataset, not independently checked through chain RPC.

[Read the real-artifact validation evidence](real-world-validation.md) and [source hashes](../tests/fixtures/real/README.md).

## Policy choices

An exit of `0` means a comparison completed and passed the **chosen gate**, not that the upgrade is safe. In particular, `--fail-on never` can exit zero even when the verdict is `INCOMPATIBLE`. An extraction/analysis failure remains exit `1` under every policy.

A false positive, false negative, or unknown Soroban/XDR behavior should be treated as a **regression or research issue**, supported with pinned evidence and tests—not adjusted by guessing.
