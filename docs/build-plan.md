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

Terminal and JSON first. Add other formats only with a clear integration need.

## Phase 9 — Git revision comparison

Implemented: `stellaryn git --repo . --from REF --to REF --wasm PATH`, optionally `--after-wasm PATH` for renamed artifacts. Reads bounded compiled WASM blobs directly from Git objects with no checkout or worktree mutation. Keeps the existing report and CI policy contracts.

## Phase 10 — Fixture matrix

Implemented: 44 deterministic XDR-backed golden compatibility scenarios across all six Soroban contract-spec categories, plus malformed/ambiguous WASM and normalization regression tests. Hardened the extractor to reject empty/duplicate contractspecv0 sections and trailing malformed WASM. These are synthetic spec-section WASM fixtures, not independently compiled deployment artifacts; that separate validation is Phase 11.

## Phase 11 — Real-world validation

Pinned public Soroban repositories, manual finding review, and false-positive regression fixes.

## Phase 12 — Documentation and contributor readiness

GitBook/public docs, issue templates, contributor workflow, and rule-authoring guidance.

## Phase 13 — Release hardening

Cross-platform release binaries, packaging, checksums, and portability testing.

## Phase 14 — v0.1.0 and Stellar Wave readiness

Public release, scoped contributor backlog, and maintainer application materials.
