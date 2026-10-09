# Getting started

## Install from the published release

[Stellaryn v0.1.0](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0) provides verified archives for Linux x86_64 (`.tar.gz`), Windows x86_64/MSVC (`.zip`), and macOS Apple Silicon (`.tar.gz`), each with a matching `.sha256` file. Download both files for your platform from that release, verify the checksum **before** extracting or running the binary, and consult [release and checksum instructions](releasing.md). Not all operating system versions or CPU architectures have been tested.

## Prerequisites for building from source

- Git, to clone the repository and (optionally) compare committed WASM.
- The pinned **Rust 1.96.0** toolchain; `rust-toolchain.toml` records the repository's version.
- Cargo; commands below are from the repository root.
- Two **compiled Soroban contract WASM binaries** with nonempty `contractspecv0` sections. Stellaryn does not compile source or fetch chain artifacts automatically.

## Build

```bash
git clone https://github.com/Stellaryn/Stellaryn.git
cd Stellaryn
rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy
cargo build --locked --release -p stellaryn-cli
./target/release/stellaryn --help
```

On Windows, the executable is `target/release/stellaryn.exe`. The [native release-candidate workflow](https://github.com/Stellaryn/Stellaryn/actions/workflows/portability.yml) separately builds, tests, packages and smoke-tests Linux, Windows and macOS artifacts. These temporary CI artifacts are distinct from the [published v0.1.0 GitHub Release](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0); neither proves compatibility with every possible machine or OS version.

## First comparison with checked-in real WASM

The repository includes two **independently compiled**, upgrade-like `add` variants. They are not verified chronological upgrades of the same deployed contract:

```bash
cargo run -p stellaryn-cli -- compare \
  tests/fixtures/real/compiled_add_i128.wasm \
  tests/fixtures/real/compiled_add_u128.wasm \
  --format terminal
```

This reports `INCOMPATIBLE`: two input parameter types and one output type changed from `i128` to `u128`. Because the default failure threshold is `breaking`, **the command deliberately exits with code 2**. That exit is a policy result, not an extraction failure.

To inspect the same findings without a nonzero policy exit:

```bash
cargo run -p stellaryn-cli -- compare \
  tests/fixtures/real/compiled_add_i128.wasm \
  tests/fixtures/real/compiled_add_u128.wasm \
  --format json --fail-on never
```

For a successful zero-change comparison, use the same compiled file twice:

```bash
cargo run -p stellaryn-cli -- compare \
  tests/fixtures/real/sdk_constructor.wasm \
  tests/fixtures/real/sdk_constructor.wasm
```

## Your own contracts

After building your own **old** and **new** Soroban contract WASM artifacts with a verified SDK/toolchain:

```bash
./target/release/stellaryn compare old.wasm new.wasm --format json --fail-on review
```

Use `--help` and `compare --help` for exact flag syntax. No path discovery is performed. A missing, unreadable, invalid, empty-spec or ambiguous-spec WASM **fails with exit 1 and no JSON success report**.

## Compare committed Git blobs

```bash
cargo run -p stellaryn-cli -- git \
  --repo . --from 'HEAD~1' --to HEAD \
  --wasm contracts/token.wasm --format json
```

This example assumes `contracts/token.wasm` exists **in both commits**. It is not a runnable demonstration against the Stellaryn source repository itself. For a renamed path, use `--after-wasm`; see [Git revision comparison](git-revision-comparison.md). Git mode reads committed objects without changing your worktree and limits each blob to **32 MiB**.

## What next?

[Worked examples](examples.md) · [Using Stellaryn in CI](ci-integration.md) · [Troubleshooting](troubleshooting.md).
