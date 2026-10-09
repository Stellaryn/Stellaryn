# Phase 12 Acceptance Criteria — Documentation and Contributor Readiness

**Status:** acceptance is based on the final GitHub Actions run for this exact commit. Repository-hosted Markdown is the primary public documentation. `.gitbook.yaml` and `docs/SUMMARY.md` prepare for optional GitBook Git Sync; **no new Stellaryn GitBook site is claimed as live**.

## Required checks

```sh
python3 scripts/check_docs.py
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
python3 scripts/real_world_probe.py
cargo run -p stellaryn-cli -- --help
```

## Coverage

- Working public GitHub documentation index, GitBook-compatible root/README/SUMMARY.
- Accurate build/start commands with two pinned compiled fixtures (`compiled_add_i128.wasm` and `compiled_add_u128.wasm`).
- Expected verdict and rule IDs, deterministic JSON contract, `--fail-on` policy matrix and exit code distinctions, including Clap usage exit 2.
- Git history mode's exact `--repo`, `--from`, `--to`, `--wasm`, `--after-wasm` semantics and limitation that WASM blobs must be committed at both revisions.
- Troubleshooting, explicit deployment-safety limitations and linked real-world artifact provenance.
- Evidence-driven rule-writing guide pointing to actual source modules and tests.
- Contributor setup / PR checklist / code of conduct; separate issue forms for bug, compatibility misclassification and feature requests.
- CI gate catching broken local Markdown links, duplicated GitBook navigation references, missing required pages and improperly configured docs root; regression unit tests for that checker.
- Existing Rust and real compiled-WASM regression gates remain intact; no quality checks weakened.

## Boundaries

The docs themselves are public on GitHub. A **hosted GitBook site** requires configuring/authorizing GitBook Git Sync and publishing a new Stellaryn site; existing GitBook sites are left untouched. These docs do not claim a release binary, native Windows validation, a security audit or on-chain upgrade verification. Phase 13 handles release hardening.
