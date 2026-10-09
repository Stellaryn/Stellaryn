# Stellaryn documentation

**See exactly what changed before your Soroban contract upgrade ships.**

Stellaryn is a local-first Rust CLI for inspecting the **public specification** of two compiled Soroban contract WASM files and reporting compatibility findings. It can also compare WASM files already committed at two Git revisions.

> A `COMPATIBLE` finding is **not** a security audit, runtime correctness proof, storage-migration check, or deployment approval.

## Choose your path

- [Getting started](getting-started.md) — build and run the CLI with verified test artifacts.
- [Worked examples](examples.md) — compatible and breaking checks with expected results.
- [CI integration](ci-integration.md) — automation, JSON reports, and exit-code policy.
- [Troubleshooting](troubleshooting.md) — missing specs, Git errors, malformed WASM, and confusing verdicts.
- [Compatibility rules](rule-authoring.md) — how rules work and how to propose or implement changes.
- [Scope and limitations](limitations.md) — evidence boundaries and what is not analyzed.
- [Published v0.1.0 release and packages](releasing.md) — downloads, archive layout, SHA-256 verification and native platform evidence.
- [Contributing](../CONTRIBUTING.md) — developer setup and PR checklist.
- [Security policy](../SECURITY.md) — reporting sensitive findings.

## Technical references

[Architecture](architecture.md) · [Interface model](interface-model.md) · [WASM extraction](wasm-extraction.md) · [Function rules](function-diff.md) · [Custom-type rules](type-diff.md) · [Event/error rules](error-event-diff.md) · [Verdict/exit policy](verdict-and-exit-policy.md) · [CLI JSON specification](cli-reports.md) · [Git revision comparison](git-revision-comparison.md).

## Evidence and verification

The [44-case synthetic fixture matrix](fixture-matrix.md) tests the normalized rules with generated Soroban XDR. The [real-world validation report](real-world-validation.md) tests **six independently compiled** contract WASM files from pinned third-party sources, with [byte-level provenance](../tests/fixtures/real/README.md). Neither establishes historical on-chain upgrade safety.

The [internal build plan](build-plan.md) records implementation history. The [first public binary release](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0) is verified on Linux, Windows and macOS CI runners.

**Published docs:** [Stellaryn Documentation on GitBook](https://oobayemi.gitbook.io/stellaryn-documentation/), with active one-way GitHub-to-GitBook import verified on 2026-10-09. The authoritative source remains this repository's `docs/` directory. `gitbook-docs.yaml` maps site-wide content to `./docs`, and `.gitbook.yaml` retains the original space settings. Edit source via normal reviewed GitHub pull requests to keep website content synchronized.
