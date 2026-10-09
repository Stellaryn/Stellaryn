# Phase 5 Acceptance

Accept this phase only after the final HEAD passes GitHub Actions:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

Required evidence: tests for unchanged/docs-only types; addition/removal; kind transitions; struct fields and reordering; numeric enum variant additions/removals/discriminants and order independence; tagged union cases and payload arity/types/names; model validation failures; deterministic serialization/sorting; and previous Phase 2–4 regression suites.

No full upgrade verdict, CLI comparison, error/event diff, or security-audit claim is introduced in this phase.
