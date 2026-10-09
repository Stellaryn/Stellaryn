# Phase 1 Acceptance Criteria

Phase 1 is accepted only when CI proves all of the following on the repository:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo run -p stellaryn-cli -- --help`
- CLI help clearly states that comparison logic is not yet implemented.
- No production `todo!()`, `unimplemented!()`, or dummy compatibility success path exists.
- The five workspace crates compile together.
- Documentation describes the current phase honestly.

Local execution was not available in the build environment used to assemble this scaffold, so no local test result should be inferred. GitHub Actions is the required evidence gate once the repository is created.
