# Phase 4 Acceptance Criteria

Phase 4 is accepted only when CI proves:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo run -p stellaryn-cli -- --help`

The function diff suite must prove:

- identical functions produce no finding;
- function additions are non-breaking;
- function removals are breaking;
- parameter additions/removals are breaking;
- parameter reorder is breaking;
- pure parameter rename requires review;
- nested structured type changes are breaking;
- output count changes are breaking;
- output type changes are breaking;
- documentation changes are ignored;
- relative reorder is still found when the parameter set also changes;
- invalid normalized interfaces fail with typed errors;
- findings are deterministic and severity ordered.

No custom-type, error, event, or overall-verdict logic belongs in Phase 4.
