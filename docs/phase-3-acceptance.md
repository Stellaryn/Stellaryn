# Phase 3 Acceptance Criteria

Phase 3 is accepted only when CI proves:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo run -p stellaryn-cli -- --help`

And the extraction layer demonstrates:

- direct parsing through `soroban_spec::read::from_wasm`;
- exact dependencies on `soroban-spec 28.0.0` and `stellar-xdr 28.0.0`;
- handling of every current `ScSpecEntry` category;
- structured mapping of nested and parameterized XDR types;
- union payload preservation;
- numeric enum/error values;
- event prefix topics, parameter locations, and data format;
- deterministic normalization through `stellaryn-core`;
- explicit errors for invalid Wasm and missing contract specifications;
- an end-to-end test that embeds XDR entries in a real `contractspecv0` Wasm custom section and extracts them with the official reader.

Compatibility classification remains out of scope until Phase 4.
