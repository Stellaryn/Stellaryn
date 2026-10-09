# Stellaryn v0.1.0 — release notes (publication draft)

> Draft for the first public release. **Do not link to a release tag, claim download URLs, or describe this as published until the GitHub Release and artifacts are verified.** The final commit SHA, GitHub Actions runs, target triples, assets and checksums must be recorded at publication.

## What Stellaryn does

Stellaryn is a local-first Rust command-line tool that compares public specifications extracted from **already compiled Soroban contract WASM**. It reports deterministic `BREAKING`, `REVIEW_REQUIRED`, and `NON_BREAKING` findings, together with an aggregate compatibility verdict.

- Compare two local WASM files with `stellaryn compare BEFORE.wasm AFTER.wasm`.
- Compare already committed WASM files across two Git revisions using `stellaryn git --repo PATH --from REF --to REF --wasm PATH` (and `--after-wasm` when the artifact's repository-relative path changes).
- Read terminal or JSON reports (`--format json`) and choose CI failure behavior (`--fail-on breaking|review|never`).
- Detect public function, type, error-code and event specification changes within the implemented compatibility rules.

## Installation and verification

Rust source builds use pinned **Rust 1.96.0**:

```sh
git clone https://github.com/Stellaryn/Stellaryn.git
cd Stellaryn
cargo build --locked --release -p stellaryn-cli
./target/release/stellaryn --version
```

For Windows, the binary filename is `stellaryn.exe`. After publication, use the **actual** GitHub Release assets for each native target and its matching `.sha256` file. Check the SHA-256 against a trusted release page before execution. The tool does not require RPC credentials or an indexer.

## Try the compiled-WASM fixtures

```sh
cargo run --locked -p stellaryn-cli -- compare \
  tests/fixtures/real/compiled_add_i128.wasm \
  tests/fixtures/real/compiled_add_u128.wasm \
  --format json --fail-on never
```

The pinned, independently compiled fixture pair yields `INCOMPATIBLE` with three function signature type-change findings. These are related variants, **not** evidence of a verified on-chain upgrade of the same contract.

## Security and scope boundaries

**A `COMPATIBLE` verdict is not a security audit or upgrade safety certificate.** Stellaryn does not verify implementation behavior, storage migration, authorization correctness, on-chain contract provenance or deployment outcomes. Analysis errors fail closed; `--fail-on never` only changes CI exit policy, not the reported verdict.

Documentation: [getting started](getting-started.md), [full limitations](limitations.md), [CI integration](ci-integration.md), [compiled-WASM validation](real-world-validation.md), and [release packaging](releasing.md).

## Publication evidence (to fill only after verification)

- Tagged commit: **NOT YET VERIFIED**
- Tag and GitHub Release URL: **NOT YET PUBLISHED**
- Final standard CI URL, success and head SHA: **PENDING**
- Final native Linux, Windows and macOS jobs: **PENDING**
- Published packages and independent SHA-256 verification: **PENDING**
- Maintainer signatures/attestations: **NOT CLAIMED**
