# Phase 6 Acceptance Criteria

Accept Phase 6 only when the latest HEAD passes all CI quality gates:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

Required regressions: error definitions, case add/remove, name/code changes, stable-code rename, declaration order independence; event add/remove, prefix topics, data format, parameter additions/removals/reorders/renames, structured type changes, topic/data location changes, Map data field-name handling; deterministic ordering/JSON, malformed interface failures, and all previous phases' tests.

Scope: spec-level error/event compatibility only; no runtime behavior guarantees, no deployment safety claims, and no overall verdict or user-facing reporting yet.
