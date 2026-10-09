# Stellaryn development instructions (human and AI contributors)

**Start here:** [docs/PROJECT_HANDOFF.md](docs/PROJECT_HANDOFF.md), then inspect the *current* GitHub `main` branch and recent CI results. The handoff is a dated snapshot, never an excuse to skip fresh verification.

## Source-of-truth precedence

1. Current repository source files, manifests, tests, and workflow definitions at the exact Git commit being changed.
2. Current successful CI logs for that same commit, including all operating-system jobs relevant to the claim.
3. [docs/PROJECT_HANDOFF.md](docs/PROJECT_HANDOFF.md) and [docs/build-plan.md](docs/build-plan.md) for intentions and historical decisions.
4. Conversation summaries and assistant recollections are helpful navigation, **not evidence**.

When facts disagree, stop and reconcile them from code/CI. Do not invent successful builds, commands, source APIs, on-chain upgrades, releases, or test counts. Do not confuse a **GitHub branch** with ChatGPT's **Branch in new chat** feature.

## User-facing requirements

- **Never put numbered development phases or internal milestone progress in the root `README.md`.** Keep the README professional, product-focused and useful to visitors. Preserve historical progress in `docs/build-plan.md` and the handoff.
- Work on one verified milestone at a time; explicitly list what is and is not accepted, with exact commit SHA and CI URLs.
- Fail closed on ambiguous or invalid contract-spec evidence; a `COMPATIBLE` result is never a security audit or on-chain deployment approval.
- Add positive and negative/edge-case regressions for changes. Never weaken tests or lints to force a green result.
- For public third-party WASM fixtures, preserve upstream provenance, immutable refs and hashes, and do not claim independent on-chain verification.
- Do not call a release published before a GitHub Release actually exists.

## Existing workflows

Rust toolchain `1.96.0`; five workspace crates: `stellaryn-cli`, `stellaryn-core`, `stellaryn-wasm`, `stellaryn-diff`, `stellaryn-report`. Dependency graph is captured in `Cargo.lock`.

```sh
python3 scripts/check_docs.py
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 scripts/real_world_probe.py
cargo run --locked -p stellaryn-cli -- --help
```

Release candidate packaging and native Windows/Linux/macOS verification: [docs/releasing.md](docs/releasing.md) and `.github/workflows/portability.yml`. Verify native CI on each platform before describing it as working; **candidate CI artifacts are not published releases**.

If modifying source or workflows, commit with accurate descriptions and confirm all relevant GitHub Actions jobs on the new **final** SHA. If tools cannot retrieve the necessary repository/CI evidence, say so instead of guessing.
