# Stellaryn v0.1.0 — verified publication record

> **Published 2026-10-09:** [Stellaryn v0.1.0 GitHub Release](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0). The release workflow produced native archives and matching `.sha256` integrity companions for Linux x86_64, Windows x86_64/MSVC and macOS Apple Silicon. This is not a security audit, publisher-identity signature or notarization.

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

For Windows, the binary filename is `stellaryn.exe`. Use the [actual published GitHub Release assets](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0) for each native target and its matching `.sha256` file. Check the SHA-256 against the trusted release before execution. The tool does not require RPC credentials or an indexer.

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

## Publication evidence (verified 2026-10-09)

- Tagged commit: [`17de3459ef24641c6544e845e35ba5a7fbcd0f9e`](https://github.com/Stellaryn/Stellaryn/commit/17de3459ef24641c6544e845e35ba5a7fbcd0f9e).
- Published tag/Release: [v0.1.0](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0); GitHub API confirms a published non-draft, non-prerelease release.
- [Final standard CI](https://github.com/Stellaryn/Stellaryn/actions/runs/37982084864): success on the tagged SHA.
- [Final native candidates](https://github.com/Stellaryn/Stellaryn/actions/runs/37982085083): all three supported OS runners successful on the tagged SHA.
- [Release publication](https://github.com/Stellaryn/Stellaryn/actions/runs/37982396203): all three native builds, checksum/asset verification and publishing job successful on the tagged SHA.
- Published assets: three native archives and three SHA-256 companion files, as verified on the GitHub Release page.
- Maintainer identity signatures, notarization or independent third-party deployment: **not claimed**.
