# Stellaryn

**See exactly what changed before your Soroban contract upgrade ships.**

Stellaryn is a local-first Rust CLI for comparing Soroban contract interfaces and explaining compatibility changes before deployment.

> Passing Stellaryn is not a security audit and does not prove that a contract upgrade is safe to deploy.

## Status

**Phase 1 / pre-alpha foundation.** The workspace and CLI shell exist, but no contract comparison result is implemented yet. This is intentional: later phases will add verified Soroban/Wasm behavior rather than fake parsers or placeholder success paths.

## Phase 1 scope

- Rust workspace and crate boundaries
- CLI identity, help, version, and product metadata
- shared verdict/source primitives
- CI quality gates
- contribution/security documentation
- architecture and build-plan documentation

Not implemented in Phase 1:

- Soroban Wasm/spec extraction
- normalized contract interface model
- compatibility diff rules
- terminal/JSON reports
- Git-ref comparison

## Workspace

| Crate | Responsibility |
| --- | --- |
| `stellaryn-cli` | CLI entry point and orchestration |
| `stellaryn-core` | Shared domain types and compatibility primitives |
| `stellaryn-wasm` | Future verified Soroban Wasm/spec extraction |
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
