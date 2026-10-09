# Native builds, release candidates and artifact checksums

Stellaryn is **pre-release**. The native-platform workflow prepares reviewable, downloadable **release-candidate CI artifacts**; it does not publish a GitHub Release or claim a stable v0.1.0 release. Publishing tags, provenance attestations, signing keys, long-term support and a public release announcement remain separate release decisions.

## Supported CI targets

The [Native release candidates workflow](../.github/workflows/portability.yml) runs on pushes to `main`, pull requests, and explicit dispatches. It compiles and tests **natively** on three GitHub-hosted runners:

| Runner | Artifact format | Executable |
| --- | --- | --- |
| `ubuntu-latest` | `.tar.gz` | `stellaryn` |
| `windows-latest` | `.zip` | `stellaryn.exe` |
| `macos-latest` | `.tar.gz` | `stellaryn` |

The artifact name includes `rustc -vV`'s *actual host triple*, rather than assuming architecture from the runner label. This is native CI, **not** proof of every possible OS version, CPU target or installation configuration.

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
sha256sum -c dist/stellaryn-v0.1.0-alpha.1-<target>.tar.gz.sha256
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

## Download CI candidates

Open [GitHub Actions](https://github.com/Stellaryn/Stellaryn/actions/workflows/portability.yml), select a completed successful run, and find its `stellaryn-<OS>-<commit>` artifacts. Each upload contains the archive and `.sha256`; checksum validation is also executed before upload.

These are **temporary GitHub Actions artifacts**, not public release downloads. They may require GitHub authentication and expire according to workflow retention settings.

## Known limitations and next release steps

- A successful CI run is a necessary portability signal, not a formal security audit or OS compatibility certification.
- The current native workflow builds each target separately; it does not claim byte-identical binaries across OSes.
- These candidates are neither notarized nor cryptographically signed with a release-maintainer identity. SHA-256 detects accidental changes when downloaded from a trusted source, but is **not** a signature.
- The Rust dependency graph should be locked and verified for a reproducible official release. A committed `Cargo.lock` pins the resolution graph; dependency/toolchain changes must be tested and documented.
- A public tag, consolidated checksums across targets, and a stable published GitHub Release are **not** part of this workflow.
