# Phase 11 Acceptance Criteria — Real Executable WASM Validation

Accept only after GitHub Actions succeeds against the exact final `main` commit on:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
python3 scripts/real_world_probe.py
cargo run -p stellaryn-cli -- --help
```

Required independent evidence:

- Six pinned upstream artifacts with executable code bodies, exactly one nonempty `contractspecv0` section, and unmodified Git blob SHA-1 fingerprints.
- Successful spec extraction, typed interface validation, and zero-finding self-diff for **each** compiled artifact.
- One verified public `increment(step: i64)` ABI and SDK `__constructor` ABI.
- Large AMM mainnet-dataset contract and smaller arb-bot artifact parsed, cross-compared and inspected; mainnet origin labels not falsely asserted to have been chain-verified.
- Signed 128-bit and unsigned 128-bit independently compiled `add` variants produce breaking parameter/output type changes and exit code 2.
- Truncated real compiled artifact fails closed; JSON stdout is empty on analysis errors.
- Complete JSON and policy semantics for cross-comparisons, with reproducible probe log.
- All Phase 2–10 regression tests remain green; no lint suppression.

**Evidence boundary:** These are independently compiled binaries, not independent proof that two files are historical upgrades of the same deployed contract. This phase validates real-WASM extraction and meaningful compiled-interface differences, but does not certify all upgrade-safety cases, on-chain behavior, or absence of false negatives.
