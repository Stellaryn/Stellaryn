# Phase 8 Acceptance Criteria

Accept Phase 8 only after GitHub Actions on **final HEAD** passes all four gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

And Phase 8's tests demonstrate that:

- `stellaryn compare BEFORE.wasm AFTER.wasm` works with actual Soroban-spec custom sections;
- identical WASM => `COMPATIBLE`, complete terminal output, exit 0;
- breaking change => `INCOMPATIBLE`, full report, default exit 2;
- pure rename => `REVIEW_REQUIRED` (default exit 0, strict review exit 2);
- `--fail-on never` preserves verdict and returns exit 0 on findings;
- missing/corrupt/no-spec WASM => error exit 1, empty stdout, stderr diagnostic;
- JSON remains valid, deterministic, and machine-readable even on policy violation;
- terminal report is ANSI-free and contains verdict/counts/rule evidence;
- CLI options/filenames with spaces are correctly parsed;
- all previous Phase 2–7 tests pass without weakening quality checks.

Git-ref comparison, external commands, deployment-safety claims, and SARIF remain out of scope.
