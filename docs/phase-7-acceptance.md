# Phase 7 Acceptance Criteria

Phase 7 is accepted **only** when GitHub Actions proves all gates at final HEAD:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

The `diff_contracts` regression suite must establish:

- identical and docs-only changes give a zero-finding spec-level `COMPATIBLE` result;
- additive changes in all four domains do not imply a breaking verdict;
- review-only changes produce `REVIEW_REQUIRED`;
- breaking changes from each domain produce `INCOMPATIBLE` and override reviews;
- all findings preserve typed rule identities/evidence, and have stable ordering/counts;
- JSON serialization round-trips;
- malformed before/after interfaces return typed errors;
- `breaking`, `review`, `never` policy thresholds map to reserved exit codes;
- `never` does not turn failed analysis into success;
- all prior Phase 2–6 tests remain green.

The API is a **library** feature in Phase 7. End-user compare CLI, terminal/JSON reports, and true process exit integration are reserved for Phase 8. A `COMPATIBLE` verdict is not a contract security audit.
