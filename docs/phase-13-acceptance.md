# Phase 13 Acceptance — Release Hardening

This milestone is accepted only if **both workflows pass on the exact final main commit**:

- [Standard CI](../.github/workflows/ci.yml): README milestone safeguard and documentation tests, Rust formatting, warnings-as-errors Clippy, all Rust tests, independent compiled-WASM probes and CLI help.
- [Native release candidates](../.github/workflows/portability.yml): three completed jobs on `ubuntu-latest`, `windows-latest` and `macos-latest`. Each runs the full Rust tests **with Cargo.lock enforced**, builds the optimized binary, creates its native archive/checksum, verifies archive membership/checksum, runs **the packaged binary itself** against compiled WASM, then uploads only verified candidates.

## Required platform evidence

| Target (from `rustc -vV`) | Expected archive | Runtime checks |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `.tar.gz` + `.sha256` | version, help, valid JSON, 3 ABI-breaking rules, default exit 2, self-diff exit 0 |
| `x86_64-pc-windows-msvc` | `.zip` + `.sha256` | same gates |
| `aarch64-apple-darwin` | `.tar.gz` + `.sha256` | same gates |

The workflow must discover the triple at runtime, rather than making a false assumption about runner architecture. If GitHub changes the runner target, the actual target/arch must be reviewed and updated here with evidence.

## Packaging and supply-chain checks

- Pin Rust toolchain `1.96.0`, include a committed Cargo.lock, enforce `--locked` on CI builds/tests.
- Archive names include version and actual Rust host triple; no generic `stellaryn.zip` ambiguous across platforms.
- Archives normalize file ordering/timestamps, include binary, README, license and limitations, and emit a portable SHA-256 sidecar.
- Verifier rejects a changed archive checksum, unexpected zip paths, unsafe tar members (symlinks), wrong binary bytes, missing files and unexpected incompatibility verdicts.
- Tests require the packaged binary—not merely `target/release`—to execute the real compiled `i128 → u128` ABI differences correctly.
- CI artifacts are candidates with short retention. No automatic public GitHub Release or identity-based signature is claimed.

## Product-facing documentation

`README.md` is not a progress ledger: **no numbered phase references are allowed**. `scripts/check_docs.py` and its unit test enforce this while the internal milestone plan remains `docs/build-plan.md`.

## Boundaries

Native runner checks do not certify every supported operating-system version, CPU architecture or installer environment. Checksums are integrity checks, not cryptographic publisher signatures. Publishing a real v0.1.0 GitHub Release, signing/notarization decisions and wider compatibility targets remain explicitly deferred.
