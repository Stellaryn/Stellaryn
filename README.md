# Stellaryn

**See exactly what changed before your Soroban contract upgrade ships.**

Stellaryn is a local-first Rust CLI and library for comparing Soroban contract interfaces and explaining compatibility changes before deployment.

> Passing Stellaryn is not a security audit and does not prove that a contract upgrade is safe to deploy.

## Status

**Phase 3 / pre-alpha.** Stellaryn now has a normalized contract-interface model and verified local Soroban Wasm specification extraction. Compatibility classification and user-facing comparison reports are intentionally not implemented yet.

## Implemented

- Rust workspace and strict CI gates
- deterministic normalized interface model
- functions, structs, unions/enums, error enums, and events
- structured Soroban type references
- direct `contractspecv0` extraction through `soroban-spec 28.0.0`
- typed mapping from `stellar-xdr 28.0.0`
- explicit failure for invalid Wasm or missing contract specifications

## Not implemented yet

- compatibility diff rules
- terminal/JSON comparison reports
- Git-ref comparison
- release packaging

## Workspace

| Crate | Responsibility |
| --- | --- |
| `stellaryn-cli` | CLI entry point and orchestration |
| `stellaryn-core` | Normalized interface types, validation, and stable ordering |
| `stellaryn-wasm` | Verified Soroban Wasm/spec extraction |
| `stellaryn-diff` | Future deterministic compatibility comparison |
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
