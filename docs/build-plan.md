# Stellaryn Build Plan

## Phase 1 — Foundation

Workspace, CLI shell, CI, shared primitives, docs, and tests. No fake comparison implementation.

## Phase 2 — Normalized contract interface

Model functions, parameters, return values, structs, enums, errors, events, and stable identifiers with deterministic serialization.

## Phase 3 — Verified Soroban Wasm extraction

Inspect exact Soroban SDK/XDR/Stellar CLI behavior before implementing extraction. Prefer compiled contract specification evidence.

## Phase 4 — Function compatibility diff

Classify removals, additions, parameter order/type changes, and return-type changes.

## Phase 5 — Custom type compatibility diff

Struct and enum compatibility rules with explicit review semantics for uncertain additive cases.

## Phase 6 — Events and errors

Deterministic comparison for public errors and event shapes.

## Phase 7 — Verdict and exit policy

Stable compatible/review/incompatible policy and process exit semantics.

## Phase 8 — Reports

Terminal and JSON first. Add other formats only with a clear integration need.

## Phase 9 — Git revision comparison

Compare committed refs without mutating the active worktree.

## Phase 10 — Fixture matrix

Healthy and intentionally breaking Wasm/spec fixtures with stable expected results.

## Phase 11 — Real-world validation

Pinned public Soroban repositories, manual finding review, and false-positive regression fixes.

## Phase 12 — Documentation and contributor readiness

GitBook/public docs, issue templates, contributor workflow, and rule-authoring guidance.

## Phase 13 — Release hardening

Cross-platform release binaries, packaging, checksums, and portability testing.

## Phase 14 — v0.1.0 and Stellar Wave readiness

Public release, scoped contributor backlog, and maintainer application materials.
