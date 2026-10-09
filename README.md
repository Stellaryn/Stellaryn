# Stellaryn

**See exactly what changed before your Soroban contract upgrade ships.**

Stellaryn is a **local-first Rust CLI** that compares the public specifications embedded in compiled Soroban smart-contract WASM. It explains interface differences, classifies compatibility findings, and provides deterministic terminal and JSON reports for developers and CI.

> **Not a security audit.** A `COMPATIBLE` result means no breaking or review-required public-specification changes were detected by the implemented rules. It does **not** prove runtime behavior, stored-state migration, authorization or deployment safety.

## Get started

**Status:** Pre-release. Build from source using the pinned Rust 1.96.0 toolchain. Prebuilt binaries are not yet published as a public release.

```bash
git clone https://github.com/Stellaryn/Stellaryn.git
cd Stellaryn
cargo build --release -p stellaryn-cli
./target/release/stellaryn --help
```

On Windows the executable is `target/release/stellaryn.exe`. See [getting started](docs/getting-started.md) for OS requirements and usage notes.

### Compare two local contracts

```bash
stellaryn compare old.wasm new.wasm
stellaryn compare old.wasm new.wasm --format json
stellaryn compare old.wasm new.wasm --format json --fail-on review
```

Both inputs must be **compiled Soroban WASM** containing a valid, unambiguous `contractspecv0` section. Stellaryn does not build source contracts or download artifacts from the chain.

To try a *real, independently compiled* breaking ABI difference from the repository root:

```bash
cargo run -p stellaryn-cli -- compare \
  tests/fixtures/real/compiled_add_i128.wasm \
  tests/fixtures/real/compiled_add_u128.wasm \
  --format json --fail-on never
```

The result is `INCOMPATIBLE` with **two changed `add` parameters and one changed return type**. The `never` flag suppresses compatibility-policy failure **without changing the verdict**.

### Compare WASM committed in Git

```bash
stellaryn git --repo /path/to/contract-repo \
  --from v1.0.0 --to HEAD --wasm artifacts/token.wasm \
  --format json --fail-on review
```

Git mode reads committed artifact blobs without checking out revisions or changing the worktree. Use `--after-wasm` for a renamed path; each Git blob is limited to **32 MiB**. This example requires both Git revisions to contain the compiled WASM at their respective paths.

## Compatibility findings

Stellaryn currently analyzes:

- Public functions: additions, removals, parameters, ordering, names, input/output types.
- Custom types: struct fields, numeric enums and tagged unions, including payload changes.
- Contract errors: numeric codes, cases and names.
- Events: topic prefixes, data formats, parameters, locations and types.

Each finding is classified as `BREAKING`, `REVIEW_REQUIRED`, or `NON_BREAKING`. The aggregate verdict is `INCOMPATIBLE`, `REVIEW_REQUIRED`, or `COMPATIBLE`. Documentation-only specification edits are ignored, and results are deterministic.

| Exit code | Meaning |
| --- | --- |
| `0` | Analysis completed and passed the selected CI policy |
| `1` | Missing/invalid WASM, extraction failure, or analysis error |
| `2` | Completed analysis failed the selected CI policy; also used by Clap for invalid CLI arguments |

The default policy is `--fail-on breaking`; alternatives are `review` and `never`. A policy failure still emits the full report. An extraction error **never** emits a false-compatible JSON result.

## Documentation

[Documentation home](docs/README.md) · [Getting started](docs/getting-started.md) · [Worked examples](docs/examples.md) · [CI integration](docs/ci-integration.md) · [Troubleshooting](docs/troubleshooting.md) · [Compatibility rules](docs/rule-authoring.md) · [Scope and limitations](docs/limitations.md) · [Release packaging and verified platform builds](docs/releasing.md)

**Evidence:** [Synthetic spec test matrix](docs/fixture-matrix.md) · [Independently compiled WASM validation](docs/real-world-validation.md) · [Pinned third-party artifact sources](tests/fixtures/real/README.md)

## Build and test

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
python3 scripts/check_docs.py
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/real_world_probe.py
```

The workspace contains `stellaryn-cli`, `stellaryn-core`, `stellaryn-wasm`, `stellaryn-diff` and `stellaryn-report`. For the design see [architecture](docs/architecture.md). Release archives and checksums, when available in the repository's CI artifacts, are **test builds**, not published stable releases.

## Contributing and security

Contributions, especially **reproducible false-positive/false-negative findings and edge cases**, are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md), the [code of conduct](CODE_OF_CONDUCT.md), and [SECURITY.md](SECURITY.md) before filing public issues involving contract artifacts.

## License

[MIT](LICENSE). Third-party validation WASM remains attributed to its original sources, with provenance and licensing caveats documented [here](tests/fixtures/real/README.md).
