# Phase 9 Acceptance Criteria

Phase 9 is accepted only if the final GitHub commit passes:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

Acceptance tests must cover:

- comparison of two commits, a tag and HEAD, and an ancestry revision;
- typed breaking compatibility findings and deterministic JSON output for Git blobs;
- a **dirty working tree** and unchanged HEAD before/after Git analysis;
- compatible same-revision comparison;
- artifact moves/renames using distinct --wasm and --after-wasm paths;
- missing/invalid references, missing committed artifacts, traversal paths, and corrupt spec data returning an analysis error;
- correct `--fail-on never` and `review` thresholds;
- preservation of all Phase 2–8 tests.

Scope exclusions: compiling source at arbitrary Git refs, remote Git fetch, deleting/cleaning worktrees, or bypassing validation. CLI may report that Git is not installed, but must not misclassify an error as compatible.
