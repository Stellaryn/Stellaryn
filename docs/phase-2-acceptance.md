# Phase 2 Acceptance Criteria

Phase 2 is accepted only when CI proves:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo run -p stellaryn-cli -- --help`

And the normalized interface layer demonstrates:

- structured type references;
- functions with ordered parameters and outputs;
- structs and enums with ordered members;
- errors with numeric cases;
- events with topic/data parameter locations;
- stable top-level identifiers;
- deterministic top-level normalization;
- stable JSON round-trip behavior;
- validation of duplicate/empty identifiers;
- preservation of semantically meaningful member order.

Phase 2 must not include Wasm/XDR/Stellar CLI parsing or compatibility classifications.
