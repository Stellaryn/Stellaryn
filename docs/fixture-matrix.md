# Reproducible Soroban Fixture Matrix (Phase 10)

The objective of Phase 10 is to detect compatibility-rule regressions **and false-compatible results** using a broad, deterministic baseline. Every matrix row passes through the **real Stellaryn CLI**, official Soroban spec decoder, normalized model, all four diff engines, and the JSON report.

## Fixture provenance and limitations

The baseline is constructed directly from `stellar-xdr 28.0.0` `ScSpecEntry` structures, serialized as XDR and embedded into the `contractspecv0` custom section of a valid **minimal WebAssembly module**. The six baseline entry categories are:

| Spec category | Baseline |
| --- | --- |
| Function | `transfer(to: Address, amount: i128) -> bool` |
| Struct | `Account { owner: Address, hash: BytesN<32> }` |
| Numeric enum | `Status { Active = 0, Paused = 1 }` |
| Tagged union | `Action { Stop, Pay(Address, i128) }` |
| Error enum | `ContractError { Unauthorized = 1, Insufficient = 2 }` |
| Event | `Transfer` with prefix `transfer`, address topic and amount data |

**These are synthetic spec fixtures, not deployable compiled contract binaries.** They are high-signal for XDR extraction, normalization, rule behavior, reports, and error handling, but cannot establish that real compiled contracts behave equivalently. Phase 11 must validate independently compiled public Soroban artifacts and investigate false-positive/negative findings before a release-readiness claim.

## 44 golden outcome scenarios

Each scenario starts from the same baseline and compares a single intentionally altered spec (except the mixed-severity scenario). The test asserts expected overall verdict, **exact number of findings**, an expected stable rule ID, total-count consistency and successful JSON/exit handling.

- No effective change (5): identical, documentation-only, reordered top-level XDR entries, reordered numeric-enum declarations, reordered error-code declarations.
- Functions (11): added/removed/renamed function, renamed/reordered/added/removed/retagged input, nested map/option/vector input, output type change, output count change.
- Custom types (14): added struct, added/removed/retyped/reordered struct fields, fixed-length bytes size change, numeric-enum variant addition/removal/discriminant change, numeric-enum to union kind change, union case addition/removal/payload arity/type.
- Errors (5): error definition addition, case addition/removal, stable-code rename, numeric code change.
- Events (8): addition/removal, changed prefix topics, data format, data type, topic-vs-data location, parameter rename, parameter reorder.
- Mixed severity (1): breaking function type change plus review-needed error addition plus additive event. The report must retain all three and place breaking before review before non-breaking.

Additional assertion: multi-byte LEB128 section-length encoding must be exercised (the baseline payload is larger than 127 bytes).

## Negative / fail-closed matrix

`invalid_fixtures.rs` covers these categories:

- invalid WASM magic, truncated WASM, corrupt trailing data;
- missing `contractspecv0`, **empty** `contractspecv0`, duplicate `contractspecv0` sections;
- malformed XDR bytes;
- duplicate function names, duplicate struct field names, duplicate enum numeric discriminants, duplicate error codes, duplicate event parameter names;
- invalid *before* input (not just after);
- irrelevant custom sections with no spec (reject), and a valid spec plus unrelated custom metadata (accept);
- mixed metadata and real breaking type changes (the breaking change must survive);
- `--fail-on never` must **not** turn malformed input into success.

For invalid input the exit code must be `1`, **stdout empty** (especially no misleading JSON verdict), and **stderr diagnostic**. The standard `--fail-on` policy is evaluated only for successfully extracted, validated and compared interfaces.

## Extractor hardening discovered in Phase 10

The official `soroban_spec::read::from_wasm` returns an empty list for an empty custom section and returns after finding the first `contractspecv0` section. To avoid false confidence, Stellaryn now uses the matching `wasmparser 0.116.1` to **validate the complete WASM module**, rejects multiple matching sections, and rejects zero extracted `ScSpecEntry` values.

This is an intentional conservative limitation: a module that contains no public contract specification is an **analysis error**, not evidence of a safe upgrade.

## Reproduce

```sh
cargo test -p stellaryn-cli --test fixture_matrix
cargo test -p stellaryn-cli --test invalid_fixtures
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

The golden suite contains 44 data-driven scenarios inside two integration tests. The negative suite contains seven integration tests with multiple adverse cases inside some tests. Counts of test functions and covered scenarios are different metrics; neither should be inflated.
