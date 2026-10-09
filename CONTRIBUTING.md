# Contributing to Stellaryn

Stellaryn is built with an evidence-first rule: do not invent Stellar/Soroban behavior.

Before implementing an unfamiliar Stellar CLI command, Soroban SDK API, XDR type, contract-spec format, or Wasm convention, inspect authoritative documentation, exact crate source/docs, or verified CLI output and record the evidence.

## Required gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

## Engineering rules

- No production `todo!()`, `unimplemented!()`, or fake-success parser paths.
- Use typed Rust errors.
- Avoid `unwrap()`/`expect()` in production code.
- Pass subprocess arguments separately; never construct shell commands from user input.
- Add positive and negative tests for compatibility rules.
- Keep output deterministic where it forms part of the machine-facing contract.
- Do not claim a test passed unless it was actually run.
- Keep shared Git history additive; do not rewrite already-shared commits.
