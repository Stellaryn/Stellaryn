# Published v0.1.0 release, native builds and checksums

**Stellaryn v0.1.0 is published:** [download from its verified GitHub Release](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0). GitHub published it on 2026-10-09, with six assets (three native archives and three corresponding `.sha256` files). Its tag resolves to commit [`17de3459ef24641c6544e845e35ba5a7fbcd0f9e`](https://github.com/Stellaryn/Stellaryn/commit/17de3459ef24641c6544e845e35ba5a7fbcd0f9e). The [publishing workflow](https://github.com/Stellaryn/Stellaryn/actions/runs/37982396203) completed successfully, including native builds and executable smoke tests on Linux, Windows and macOS, and verification that all six assets were present and their checksums matched.

The regular native-platform workflow **still produces temporary CI release candidates**, independently of the publicly published release. The archives have SHA-256 integrity companions; they are not signed or notarized and have not been certified for every CPU or OS version.

## Supported CI targets

The [Native release candidates workflow](../.github/workflows/portability.yml) runs on pushes to `main`, pull requests, and explicit dispatches. It compiles and tests **natively** on three GitHub-hosted runners:

| Runner | Artifact format | Executable |
| --- | --- | --- |
| `ubuntu-latest` | `.tar.gz` | `stellaryn` |
| `windows-latest` | `.zip` | `stellaryn.exe` |
| `macos-latest` | `.tar.gz` | `stellaryn` |

The artifact name includes `rustc -vV`'s *actual host triple*, rather than assuming architecture from the runner label. This is native CI, **not** proof of every possible OS version, CPU target or installation configuration.

## Download and verify the published binaries

At the [v0.1.0 Release](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0), select the matching archive and `.sha256` companion:

| Platform | Archive filename |
| --- | --- |
| Linux x86_64 | `stellaryn-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` |
| Windows x86_64/MSVC | `stellaryn-v0.1.0-x86_64-pc-windows-msvc.zip` |
| macOS Apple Silicon | `stellaryn-v0.1.0-aarch64-apple-darwin.tar.gz` |

Download the archive **and** its `.sha256` file to the same directory. On Linux, run `sha256sum -c ARCHIVE_NAME.sha256`. On macOS, run `shasum -a 256 -c ARCHIVE_NAME.sha256`. Replace `ARCHIVE_NAME` with the exact filename above (including `.tar.gz` or `.zip`). Both commands should print `OK`; stop if verification fails. On Windows, run `Get-FileHash -Algorithm SHA256 -Path .\\ARCHIVE_NAME.zip` in PowerShell and compare the full 64-character hash with the first field of the matching `.sha256` file. Do not execute the binary if they differ.

Each archive contains the platform's executable, README, license and scope limitations. Extract it and run `stellaryn --version` (Windows: `stellaryn.exe --version`). All assets in this release were individually verified by the [publishing workflow](https://github.com/Stellaryn/Stellaryn/actions/runs/37982396203); there are no publisher-identity signatures or notarization claims.

## Reproduce packages from source

Install Rust `1.96.0` and Python `3.11+`, then from the repository root:

```sh
cargo test --locked --workspace --all-features
cargo build --locked --release -p stellaryn-cli
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/package_release.py build --dist dist
python3 scripts/package_release.py verify --dist dist
```

On Windows, use `python` in place of `python3`. Python's `tomllib` library reads the release-candidate version from the workspace manifest. The script detects the native target via `rustc -vV` and packages the existing compiled executable: **it never downloads or builds code from an untrusted URL**.

## What's inside each archive

```text
stellaryn-v<workspace-version>-<rustc-host-triple>/
  stellaryn[.exe]
  README.md
  LICENSE
  docs/limitations.md
```

The output directory contains one archive and one matching `.sha256` file using standard `<hex>  <filename>` notation. Archives normalize member ordering, archive timestamps, uid/gid and modes to produce deterministic output from identical inputs.

```sh
# Linux/macOS, substitute the actual archive name from dist/
sha256sum -c dist/stellaryn-v0.1.0-<target>.tar.gz.sha256
```

For native Windows verification, use `python scripts/package_release.py verify --dist dist`; it checks the exact checksum and archive contents.

## What the verifier proves

The release-candidate workflow checks:

1. Full Rust workspace test suite on each OS.
2. Native `cargo build --release -p stellaryn-cli` with pinned Rust `1.96.0`.
3. Deterministic packaging and archive/companion SHA-256 integrity.
4. Archive contains **only** an executable, README, LICENSE, and scope limits (no surprising or traversal paths).
5. Extracted executable bytes match the native optimized binary.
6. The **packaged binary itself** answers `--version` and `--help`.
7. The packaged binary emits valid JSON and all **three expected breaking rules** for the independently compiled `add(i128)` vs `add(u128)` WASM pair.
8. Default policy returns exit `2` on that breaking comparison; `--fail-on never` returns exit `0` without changing its `INCOMPATIBLE` verdict.
9. Comparing the same compiled WASM on both sides returns `COMPATIBLE` and exit `0`.

All tests intentionally use **real compiled fixtures** already pinned in the repository. The [fixture provenance](../tests/fixtures/real/README.md) and [scope limitations](limitations.md) remain relevant.

## Temporary CI release candidates

Open [GitHub Actions](https://github.com/Stellaryn/Stellaryn/actions/workflows/portability.yml), select a completed successful run, and find its `stellaryn-<OS>-<commit>` artifacts. Each upload contains the archive and `.sha256`; checksum validation is also executed before upload.

These are **temporary GitHub Actions artifacts**, not public release downloads. They may require GitHub authentication and expire according to workflow retention settings.

## Known limitations and next release steps

- A successful CI run is a necessary portability signal, not a formal security audit or OS compatibility certification.
- The current native workflow builds each target separately; it does not claim byte-identical binaries across OSes.
- The published archives and the temporary CI candidates are neither notarized nor cryptographically signed with a release-maintainer identity. SHA-256 detects modifications when compared with a trusted checksum, but is **not** a signature.
- The Rust dependency graph should be locked and verified for a reproducible official release. A committed `Cargo.lock` pins the resolution graph; dependency/toolchain changes must be tested and documented.
- The distinct [publish-v0.1.0 workflow](../.github/workflows/publish-v0.1.0.yml) produced the existing public tag and GitHub Release; the regular candidate workflow does not tag or publish. Future releases require a new reviewed publication process.
