# Phase 10 Acceptance Criteria

Phase 10 is accepted only when the exact final `main` commit passes:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

Required evidence:

- 44 data-driven expected-verdict compatibility scenarios across functions, structs, numeric enums, unions, errors, events, and mixed verdicts;
- assertion of exact finding counts, stable rule IDs, and count-total consistency;
- no findings for metadata-only edits, top-level ordering, and harmless enum/error declaration ordering;
- structured nested type and fixed-length BytesN changes;
- malformed WASM, malformed XDR, missing/empty/duplicate contract specs, invalid trailing bytes;
- duplicate or ambiguous normalized identifiers/codes rejected;
- analysis failures return exit 1 with no false-compatible JSON, even under `--fail-on never`;
- preservation of all Phase 2–9 tests and CLI smoke checks;
- documented fixture provenance and limitations, without calling the minimal custom-section modules deployable contracts.

**Not within Phase 10**: compiled real-world contract validation, security auditing, actual on-chain upgrade simulation or deployment approval. Independent compiled artifacts are the Phase 11 milestone.
