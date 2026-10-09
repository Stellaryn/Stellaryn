# Stellaryn Build Plan

## Phase 1 — Foundation

Workspace, CLI shell, CI, shared primitives, docs, and tests. No fake comparison implementation.

## Phase 2 — Normalized contract interface

Model functions, parameters, return values, structs, enums, errors, events, and stable identifiers with deterministic serialization.

## Phase 3 — Verified Soroban Wasm extraction

Inspect exact Soroban SDK/XDR/Stellar CLI behavior before implementing extraction. Prefer compiled contract specification evidence.

## Phase 4 — Function compatibility diff

Implemented: deterministic function additions/removals, parameter count/order/name/type changes, and output count/type changes.

## Phase 5 — Custom type compatibility diff

Implemented: struct field changes/reordering, numeric enum discriminants and variants, tagged union variants/payloads, type-kind changes, stable ordering, and explicit review semantics for uncertain additive changes.

## Phase 6 — Events and errors

Implemented: numeric error-code changes, renames and additions/removals; event prefixes, data format, parameter location/type/name/order and additions/removals, with deterministic classification.

## Phase 7 — Verdict and exit policy

Implemented: typed aggregation across functions, custom types, events, and errors; deterministic finding ordering and per-domain counts; compatible/review-required/incompatible spec-level verdict; configurable CI failure thresholds and reserved exit codes. End-user CLI wiring is Phase 8.

## Phase 8 — Reports

Implemented: `stellaryn compare BEFORE.wasm AFTER.wasm` with deterministic terminal and JSON output, configurable `--fail-on` policy and process exit codes; generated-XDR WASM CLI smoke tests validate the behavior.

## Phase 9 — Git revision comparison

Implemented: `stellaryn git --repo . --from REF --to REF --wasm PATH`, optionally `--after-wasm PATH` for renamed artifacts. Reads bounded compiled WASM blobs directly from Git objects with no checkout or worktree mutation. Keeps the existing report and CI policy contracts.

## Phase 10 — Fixture matrix

Implemented: 44 deterministic XDR-backed golden compatibility scenarios across all six Soroban contract-spec categories, plus malformed/ambiguous WASM and normalization regression tests. Hardened the extractor to reject empty/duplicate contractspecv0 sections and trailing malformed WASM. These are synthetic spec-section WASM fixtures, not independently compiled deployment artifacts; that separate validation is Phase 11.

## Phase 11 — Real-world validation

Implemented: six pinned, independently compiled Soroban contract WASM binaries from three upstream public repositories, real executable-code and Git-object-hash provenance tests, spec extraction and self-comparison, compiled i128/u128 ABI-breaking pair, complex AMM/arb-bot cross-comparisons, and a CI probe printing inspectable findings. These checks do not independently verify on-chain deployment history or stored-state upgrade safety.

## Phase 12 — Documentation and contributor readiness

Implemented: GitHub-hosted Markdown documentation hub and GitBook Git Sync-ready navigation, public reproducible usage/CI examples, troubleshooting/scope guides, compatibility-rule authoring, evidence-first contribution workflow, issue/PR templates, code of conduct and CI checks for local links/navigation. **A separate public GitBook site has not yet been created or connected; repository docs are publicly readable on GitHub.**

## Phase 13 — Release hardening

Cross-platform release binaries, packaging, checksums, and portability testing.

## Phase 14 — v0.1.0 and Stellar Wave readiness

Public release, scoped contributor backlog, and maintainer application materials.
