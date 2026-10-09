# Stellaryn

**See exactly what changed before your Soroban contract upgrade ships.**

Stellaryn is a local-first Rust CLI and library for comparing Soroban contract interfaces and explaining compatibility changes before deployment.

> Passing Stellaryn is not a security audit and does not prove that a contract upgrade is safe to deploy.

## Status

**Phase 4 / pre-alpha.** Stellaryn now has verified local Soroban Wasm extraction plus deterministic function-level compatibility findings. Custom-type, error, event, overall-verdict, and user-facing comparison reporting are intentionally still pending.

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

## Not implemented yet

- custom-type compatibility rules
- error/event compatibility rules
- overall verdict and exit policy
- terminal/JSON comparison reports
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
