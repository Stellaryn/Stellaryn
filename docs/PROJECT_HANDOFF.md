# Stellaryn — Canonical Project Handoff for New Chats

> **Snapshot created: 2026-10-09.** This file helps a fresh ChatGPT/Codex session continue faithfully without relying on long-chat memory. **Always verify live `main` and current CI first:** this is a dated checkpoint, not live status.

## Latest independently verified checkpoint — 2026-10-09 (supersedes historical pre-release snapshot below)

- **Published release:** [Stellaryn v0.1.0](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0), GitHub Release `draft=false`, `prerelease=false`, published 2026-10-09T19:48:37Z.
- **Release tag `v0.1.0` and `main` at publication:** `17de3459ef24641c6544e845e35ba5a7fbcd0f9e`. Verify live `main` separately on every future visit; documentation-only commits may move it beyond the release tag.
- **Final release CI:** [standard quality checks](https://github.com/Stellaryn/Stellaryn/actions/runs/37982084864) successful; [native release candidates](https://github.com/Stellaryn/Stellaryn/actions/runs/37982085083) successful on all three runners; [public release publication workflow](https://github.com/Stellaryn/Stellaryn/actions/runs/37982396203) successful on release tag SHA.
- **Published assets:** three platform archives plus three matching SHA-256 companions for Linux `x86_64-unknown-linux-gnu`, Windows `x86_64-pc-windows-msvc`, macOS `aarch64-apple-darwin`. These are verified integrity checks, **not** signatures or notarization.
- **Development evidence:** [GitBook Git Sync config and readiness PR #1](https://github.com/Stellaryn/Stellaryn/pull/1), [version/documentation PR #2](https://github.com/Stellaryn/Stellaryn/pull/2), and [release pipeline PR #6](https://github.com/Stellaryn/Stellaryn/pull/6), all merged after same-commit standard/native CI passed.
- **Contributor opportunities:** [verified on-chain-upgrade fixture research #3](https://github.com/Stellaryn/Stellaryn/issues/3), [adversarial WASM/XDR regression tests #4](https://github.com/Stellaryn/Stellaryn/issues/4), [Windows PowerShell quickstart #5](https://github.com/Stellaryn/Stellaryn/issues/5). These issues do not assert completed work or Drips acceptance.
- **GitBook publication blocker:** site `site_V9vjJ`, space `SnnJHT7ASMqWHFbG2v84`, linked GitBook source config `gitbook-docs.yaml` in repository root maps `./docs`. At last GitBook verification the connection was **pending**, the site had **zero pages**, and it was **not published**. Complete/authorize initial **GitHub → GitBook** sync in GitBook UI, recheck imported pages and navigation, then publish. Never send an empty site live or choose GitBook → GitHub for initial import.
- **Program application:** official current Drips Wave acceptance, active eligibility requirements, submission and approvals must still be checked. A public binary release does **not** prove Drips program eligibility or approval. Site/landing page is not an excuse to invent adoption data.
- **Remaining tasks:** postpublication README/docs reconciliation, GitBook successful import + verified public URL, genuine CLI demo, maintainer/application evidence. Keep public root README free of phase numbers; test all edits.

**History note:** The older section below is a dated *historical pre-release* snapshot. Do not treat its `0.1.0-alpha.1` or “no public release” statements as current state.

## 1. Project and links

- **Project:** Stellaryn — local-first Soroban smart-contract **public-interface compatibility analyzer**.
- **Tagline:** “See exactly what changed before your Soroban contract upgrade ships.”
- **Canonical GitHub repository:** [Stellaryn/Stellaryn](https://github.com/Stellaryn/Stellaryn), default branch `main`.
- **Docs home:** [docs/README.md](README.md).
- **14-milestone development roadmap:** [docs/build-plan.md](build-plan.md).
- **Current developer setup:** [docs/getting-started.md](getting-started.md).
- **Release candidate status:** [docs/releasing.md](releasing.md).
- **Real WASM test provenance:** [tests/fixtures/real/README.md](../tests/fixtures/real/README.md).

**User requirement:** Keep the public root [README.md](../README.md) **free of numbered phases or internal milestone-status paragraphs**. It should describe product value, setup, actual capabilities, commands, limitations, documentation and contributions. Record progress here and in `docs/build-plan.md` instead.

## 2. Status: verified evidence, not assumptions

As checked during this snapshot:

- `main` HEAD: **`3a60c3a694c2966fc6a9f457e1492a5290b258cc`**.
- [CI run #37969098702](https://github.com/Stellaryn/Stellaryn/actions/runs/37969098702): **success**, on that exact commit.
- [Native release candidates run #37969098774](https://github.com/Stellaryn/Stellaryn/actions/runs/37969098774): **success**, on the same commit; three completed successful jobs on **`ubuntu-latest`**, **`windows-latest`**, and **`macos-latest`**.
- The source has a committed **`Cargo.lock`**, a reproducible archive/checksum packager, packaging tests, and a [native portability workflow](../.github/workflows/portability.yml).
- **No stable `v0.1.0` publication has been verified.** Workspace package version is still `0.1.0-alpha.1`; native Actions packages are **release-candidate CI artifacts**, not a published GitHub Release.
- Root README has been cleaned of internal phase numbering. Verify it stays that way on every future edit.

**Milestone interpretation:** Historical phases 1–12 were accepted with passing CI in the originating development conversation. **Phase 13 (release hardening)** now has implemented release packaging and verified green native cross-platform CI on the snapshot HEAD; check current `docs/build-plan.md`, CI and open release work before deciding whether to formally accept it. **Phase 14 (public v0.1.0 and Stellar Wave readiness)** is not yet verified as complete. Never mark a phase accepted solely from this snapshot.

**Important:** `main` can move after this file was written. The *next* chat must fetch the latest HEAD/commit history, the real workflow YAML and CI run/job conclusions before acting. Do not assume the snapshot SHA is still the tip.

## 3. Exact product behavior

Two supported CLI workflows:

```sh
stellaryn compare old.wasm new.wasm --format json --fail-on review

stellaryn git --repo /path/to/git/repository \
  --from v1.0.0 --to HEAD \
  --wasm artifacts/token.wasm \
  --after-wasm moved/token.wasm \
  --format json --fail-on breaking
```

- Direct mode compares **existing locally compiled Soroban WASM** with parseable, nonempty `contractspecv0` sections.
- Git mode reads **compiled WASM blobs already committed at both refs**, without checkout or modification of the current worktree. Paths are repository-relative; `--after-wasm` is optional. Max **32 MiB per Git blob**. Does **not** rebuild historical Rust source or fetch contract bytes from a chain.
- `--format` is `terminal` (default) or `json`. JSON report schema version **`1.0`**, deterministic typed findings and totals; no ANSI color in terminal report.
- Classifications: `BREAKING`, `REVIEW_REQUIRED`, `NON_BREAKING`.
- Aggregate verdict: `INCOMPATIBLE` if any breaking, else `REVIEW_REQUIRED` if any review, else `COMPATIBLE`.
- `--fail-on breaking` (default), `review`, `never`. Completed comparison exits `0` if policy passes, `2` for policy violation. Extraction/analysis errors exit `1` **regardless of policy**. Clap also uses `2` for usage mistakes.
- There is no proof of runtime behavior equivalence, on-chain contract identity, stored-state migration compatibility, authorization correctness, deploy safety or security. Public spec compatibility is the product's deliberately limited scope.

## 4. Architecture and key source files

Five Rust crates in the `Cargo.toml` workspace, pinned toolchain **Rust 1.96.0**, currently pinned `soroban-spec = 28.0.0`, `stellar-xdr = 28.0.0`, `wasmparser = 0.116.1`:

| Crate | Responsibility |
| --- | --- |
| `stellaryn-cli` | Clap commands, file and Git artifact paths, errors and actual exit behavior |
| `stellaryn-core` | Normalized `ContractInterface`, Soroban type model, invariants and validation |
| `stellaryn-wasm` | Verified Soroban `contractspecv0` extraction, typed XDR mapping, fail-closed Wasm validation |
| `stellaryn-diff` | Functions, custom types, error/event diff rules, `diff_contracts`, typed findings/verdict and `ExitPolicy` |
| `stellaryn-report` | Stable terminal text and JSON rendering |

Read source, rather than relying on names in this file, before editing. The important files are `crates/stellaryn-diff/src/{function,types,events_errors,aggregate,policy}.rs`, `crates/stellaryn-wasm/src/lib.rs`, `crates/stellaryn-cli/src/{main,git}.rs`, and `crates/stellaryn-report/src/lib.rs`. Official Soroban SDK/XDR behavior must be checked before implementing any unverified new mapping.

## 5. Testing and edge cases — do not regress

- A deterministic **44-scenario** synthetic Soroban XDR/spec-only WASM matrix covering functions, structs, numeric enums, unions, error enums, events, metadata-only changes, mixed severities and nested types. It is **not** deployable contract bytecode.
- Negative tests for invalid/truncated/trailing-garbage WASM, missing/empty/duplicate `contractspecv0`, malformed XDR, duplicate names or discriminants, and failure under `--fail-on never` when evidence is invalid.
- **Six pinned independently compiled, code-bearing** contract WASM fixtures from public upstream projects, with Git blob hashes and source links in [fixture provenance](../tests/fixtures/real/README.md).
- An independent compiled `add(i128, i128) -> i128` vs `add(u128, u128) -> u128` pair produces exactly **three breaking** function parameter/output type findings. These are related *variants*, **not a verified on-chain upgrade of one contract ID**.
- The `mainnet` AMM/arb-bot labels came from a third-party dataset; on-chain attribution has not independently been checked.
- The 11-test `real_world` suite and `scripts/real_world_probe.py` exercise independent compiled binaries; a green parser alone does not prove upgrade safety.
- Tests and reports must remain deterministic, and invalid input must never be converted into a false `COMPATIBLE` output.

## 6. Current release-hardening work and remaining questions

On the snapshot commit, release support includes:

- committed `Cargo.lock`; `--locked` in relevant build/test commands;
- `scripts/package_release.py` with deterministic native archives and SHA-256 companion checks;
- `scripts/test_package_release.py` with tampered/traversal/invalid archive negative checks;
- `.github/workflows/portability.yml` doing native builds, workspace tests, packaging and executable smoke tests across GitHub-hosted Linux/Windows/macOS;
- `docs/releasing.md` distinguishing ephemeral release candidates from an official public release.

**Do not invent additional platform support.** Native GitHub Actions jobs provide evidence for the three runner operating systems, not every CPU, OS version or desktop installation. Archives/checksums are not signatures. Check actual job steps and artifact existence if recommending download.

Before declaring final release readiness, examine tags, GitHub Releases, archive provenance/signing, artifact checksums, release docs and any version/title inconsistencies. A public v0.1.0 release should be a separate explicitly verified action.

## 7. Verification protocol for every future chat

1. Open GitHub repository and fetch **live `main` SHA**, recent commits, `Cargo.toml`, `Cargo.lock`, `.github/workflows/{ci,portability}.yml`, `docs/releasing.md` and relevant source/tests.
2. Fetch GitHub Actions **runs AND individual jobs**. A workflow with one successful job does not prove all OS jobs passed. Compare workflow `head_sha` to current commit.
3. State what is already implemented, what remains unverified, and the next small task. Do **not** begin by rebuilding the entire repository or inventing missing code.
4. Make minimal, reviewable, traceable commits; preserve green tests and strict lint rules. For rule work, add **positive and negative** regressions with source evidence.
5. On the new final SHA, verify at least:

```sh
python3 scripts/check_docs.py
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 scripts/real_world_probe.py
cargo run --locked -p stellaryn-cli -- --help
```

6. For release tasks, also inspect native `windows-latest`, `ubuntu-latest`, `macos-latest` jobs in the portability workflow and packaging tests.
7. Report precise commit SHAs, run URLs, failures or acceptance outcomes. **Never say CI passed unless observed on the stated SHA.** Keep internal milestone numbers **out of the root README**.
8. **Update this handoff after meaningful milestones** so another fresh chat can continue without guessing.

## 8. Next conversation instructions

The user is moving the long Stellaryn development conversation to a **new ChatGPT chat** and wants continuity without hallucinations. The GitHub integration may be available. Begin by reading this file, `AGENTS.md`, the live repository, build plan and current workflow results. Confirm whether Phase 13 release hardening is now accepted, and then continue only at the user's direction. If asked to start the public release phase, prepare a truthful release-readiness checklist, verify actual versions, archives, tags and GitHub Releases, and obtain any necessary authorization before irreversible publishing actions.

This file is a navigational aid. **GitHub source/CI beats past assistant assertions, including assertions in this file.**
