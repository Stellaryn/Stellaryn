# Stellaryn

**See exactly what changed before your Soroban contract upgrade ships.**

Stellaryn is a local-first Rust CLI and library for comparing Soroban contract interfaces and explaining compatibility changes before deployment.

> Passing Stellaryn is not a security audit and does not prove that a contract upgrade is safe to deploy.

## Status

**Phase 7 / pre-alpha.** Stellaryn now has a library-level overall contract-spec verdict and CI exit policy built on verified local Soroban WASM extraction and deterministic findings for functions, custom types, errors, and events. The user-facing compare CLI and reporting are still pending.

## Implemented

- Rust workspace and strict CI gates
- deterministic normalized interface model
- functions, structs, unions/enums, error enums, and events
- structured Soroban type references
- direct `contractspecv0` extraction through `soroban-spec 28.0.0`
- typed mapping from `stellar-xdr 28.0.0`
- explicit failure for invalid Wasm or missing contract specifications
- deterministic function compatibility diff
- breaking/non-breaking/review-required function findings
- distinct struct, numeric-enum, and union kinds with custom-type compatibility rules
- numeric error-code compatibility findings
- event topic, format, and parameter compatibility findings
- unified, deterministic findings with counts by domain
- overall spec-level compatibility verdict and configurable CI failure threshold

## Function compatibility rules

Stellaryn currently detects:

- added and removed functions;
- added and removed parameters;
- parameter reordering;
- parameter renames;
- structured parameter type changes;
- output count changes;
- structured output type changes.

Documentation-only edits are ignored.

## Programmatic comparison and CI policy (Phase 7)

Rust library consumers can now use `stellaryn_diff::diff_contracts(&before, &after)` to produce a typed `ContractDiff` containing a `COMPATIBLE`, `REVIEW_REQUIRED`, or `INCOMPATIBLE` verdict, aggregate counts, per-domain counts, and findings. `ExitPolicy { fail_on: FailOn::Breaking }` is the default and produces exit code 2 on a breaking finding. `FailOn::Review` also blocks review-required changes; `FailOn::Never` does not block completed analyses. Analysis errors are reserved for exit code 1 regardless of policy. The CLI does not expose these comparison options yet; that is Phase 8.

**Important:** A `COMPATIBLE` verdict means no known breaking or review-required changes were found **in the extracted public contract specification**. It is not a security audit and does not establish runtime or storage-upgrade safety.

## Not implemented yet

- terminal/JSON comparison reports and CLI wiring for the exit policy
- Git-ref comparison
- release packaging

## Workspace

| Crate | Responsibility |
| --- | --- |
| `stellaryn-cli` | CLI entry point and orchestration |
| `stellaryn-core` | Normalized interface types, validation, and stable ordering |
| `stellaryn-wasm` | Verified Soroban Wasm/spec extraction |
| `stellaryn-diff` | Deterministic compatibility findings |
| `stellaryn-report` | Future terminal and machine-readable reporting |

## Quality gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

## License

MIT
