# Contributing to Stellaryn

Thanks for helping improve this local-first Soroban compatibility analyzer. Contributors can start with docs, edge-case fixtures, rule accuracy, extraction behavior, tests, or portability.

**Trust boundary:** Stellaryn compares public contract *specifications*. Its results are not security audits or proofs that a contract upgrade is safe.

## Getting set up

```bash
git clone https://github.com/Stellaryn/Stellaryn.git
cd Stellaryn
rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy
cargo test --locked --workspace --all-features
cargo run --locked -p stellaryn-cli -- --help
```

The supported CI baseline is Ubuntu with pinned Rust **1.96.0**. A native Windows release has not yet been validated.

Read [getting started](docs/getting-started.md), [architecture](docs/architecture.md), [rules](docs/rule-authoring.md), and [limitations](docs/limitations.md). For tests with *independently compiled* contracts, see [provenance](tests/fixtures/real/README.md).

## Choose a contribution

- **Report a reproducible bug** with environment, two public WASM artifacts (or minimal fixture), command, exit code, stdout/stderr, and expected versus actual behavior.
- **Challenge a rule** with an exact `ScSpecEntry` / SDK source or reproducible compiled contract pair and expected rule IDs. Consider false positives **and** false negatives.
- **Add a rule** by following [rule-authoring](docs/rule-authoring.md); keep extraction, normalization, diff, report, and CLI responsibilities separate.
- **Improve docs or examples** with verifiable commands and local relative links.

Before starting a major rule-policy or serialized-schema change, open an issue describing the evidence and proposed behavior to avoid wasted work.

## Development flow

1. Fork or branch from up-to-date `main`; use a narrowly scoped branch such as `fix/error-code-collision`.
2. Add tests for both a positive and an adjacent negative/edge case. Keep deterministic ordering, typed errors, stable JSON field names and report schema in mind.
3. Implement the smallest change consistent with evidence. No invented Soroban SDK/XDR/CLI behavior. Cite exact SDK/XDR source, verified CLI output, pinned artifact, or documentation.
4. Re-run the quality gates **in full**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   python3 scripts/check_docs.py
   python3 -m unittest discover -s scripts -p 'test_*.py'
   python3 scripts/real_world_probe.py
   cargo run -p stellaryn-cli -- --help
   ```
5. Open a pull request using the checklist. Link issues and list *actual* commands/results. For docs-only patches, explain any gates that were not run.
6. Expect review on backwards compatibility, source evidence, deterministic output, edge cases and fail-closed behavior. Do not force-push a shared release branch or weaken CI to make it green.

## Engineering constraints

- No production `todo!()`, `unimplemented!()`, fake success, or speculative parser paths.
- No `unwrap()`, `expect()` or `panic!()` in production code; use typed errors.
- Execute Git through argument-separated subprocesses; never interpolate untrusted paths into shell commands.
- Invalid, missing or ambiguous WASM evidence must **never** be reported as `COMPATIBLE`.
- Keep JSON shape, typed rule IDs, and deterministic ordering stable. Document any planned migrations.
- Maintain an additive, reviewable commit history; do not rewrite already-shared `main` commits.
- Do **not** introduce unlicensed or private third-party WASM in public fixtures. Attribute immutable original refs/hashes and describe provenance limits.
- Do not claim tested, audited, or deployed results that have not actually been verified.

## Community and security

Follow [our code of conduct](CODE_OF_CONDUCT.md). Public GitHub issues/PRs are for non-sensitive findings. Do not post secrets, proprietary contract source, or exploitation details; follow [SECURITY.md](SECURITY.md) for private reporting guidance. Maintainers will review contributions as capacity permits; no response or merge SLA is promised.

## Helpful commands

```bash
cargo test -p stellaryn-cli --test fixture_matrix
cargo test -p stellaryn-cli --test invalid_fixtures
cargo test -p stellaryn-cli --test real_world
cargo run -p stellaryn-cli -- compare \
  tests/fixtures/real/compiled_add_i128.wasm \
  tests/fixtures/real/compiled_add_u128.wasm --format json --fail-on never
```
