# Stellaryn

**See exactly what changed before your Soroban contract upgrade ships.**

Stellaryn is a local-first Rust CLI and library for comparing Soroban contract interfaces and explaining compatibility changes before deployment.

> Passing Stellaryn is not a security audit and does not prove that a contract upgrade is safe to deploy.

## Status

**Phase 9 / pre-alpha.** Stellaryn compares local Soroban WASM files or compiled contract artifacts committed at two Git revisions. Both modes use identical compatibility rules, JSON/terminal reporting, and CI exit policies. Automated builds from Git sources and release packaging remain future work.

## Implemented

- Rust workspace and strict CI gates
- deterministic normalized interface model
- functions, structs, unions/enums, error enums, and events
- structured Soroban type references
- direct `contractspecv0` extraction through `soroban-spec 28.0.0`
- typed mapping from `stellar-xdr 28.0.0`
- explicit failure for invalid Wasm or missing contract specifications
- deterministic function compatibility diff
- breaking/non-breaking/review-required function findings
- distinct struct, numeric-enum, and union kinds with custom-type compatibility rules
- numeric error-code compatibility findings
- event topic, format, and parameter compatibility findings
- unified, deterministic findings with counts by domain
- overall spec-level compatibility verdict and configurable CI failure threshold
- non-mutating Git revision comparison for committed WASM artifacts

## Function compatibility rules

Stellaryn currently detects:

- added and removed functions;
- added and removed parameters;
- parameter reordering;
- parameter renames;
- structured parameter type changes;
- output count changes;
- structured output type changes.

Documentation-only edits are ignored.

## Compare local Soroban WASM files

```bash
# Readable terminal report; default --fail-on breaking
cargo run -p stellaryn-cli -- compare old.wasm new.wasm

# Pretty-printed JSON for automation
cargo run -p stellaryn-cli -- compare old.wasm new.wasm --format json

# Fail CI on review-required or breaking findings
cargo run -p stellaryn-cli -- compare old.wasm new.wasm --fail-on review

# Report even breaking findings without failing the policy gate
cargo run -p stellaryn-cli -- compare old.wasm new.wasm --format json --fail-on never
```

Both inputs must be local compiled Soroban WASM files with a readable `contractspecv0` section. Successful comparisons print complete reports on stdout; extraction failures print diagnostics on stderr instead of claiming compatibility.

Exit codes: `0` = successful analysis passing the selected policy, `1` = extraction/analysis failure, `2` = completed comparison violating policy. Invalid CLI arguments also use `2` (Clap's usage error). Terminal output has no ANSI escapes; `--format json` emits deterministic JSON with report schema `1.0` and the full typed analysis.

## Compare committed Git revisions (Phase 9)

```bash
# The --wasm path is relative to the repository root and must exist in both revisions.
cargo run -p stellaryn-cli -- git --repo . --from v1.0.0 --to HEAD --wasm contracts/token.wasm

# Compare a contract artifact moved or renamed between commits.
cargo run -p stellaryn-cli -- git --repo . --from v1.0.0 --to HEAD \
  --wasm contracts/old-token.wasm --after-wasm deployments/token.wasm \
  --format json --fail-on review

# Compare a parent commit to HEAD without changing the working tree.
cargo run -p stellaryn-cli -- git --repo . --from 'HEAD~1' --to HEAD \
  --wasm contracts/token.wasm --fail-on never
```

Git mode requires a local Git installation and **WASM artifacts already committed at both revisions**. It does not compile contracts from source or guess an artifact path. It reads Git objects directly with `git cat-file`; no checkout, reset, staging, or worktree changes are made. Repository-relative paths with spaces are supported. Missing refs/artifacts, malformed WASM, and oversize artifacts fail as analysis errors.

**Git artifact size limit:** 32 MiB per input. Use the standard `compare` command for larger local artifacts. The same `--format`, `--fail-on`, and exit-code behavior applies to both comparison modes.

## Programmatic comparison and CI policy (Phase 7)

Rust library consumers can now use `stellaryn_diff::diff_contracts(&before, &after)` to produce a typed `ContractDiff` containing a `COMPATIBLE`, `REVIEW_REQUIRED`, or `INCOMPATIBLE` verdict, aggregate counts, per-domain counts, and findings. `ExitPolicy { fail_on: FailOn::Breaking }` is the default and produces exit code 2 on a breaking finding. `FailOn::Review` also blocks review-required changes; `FailOn::Never` does not block completed analyses. Analysis errors use exit code 1 regardless of policy. Phase 8 now exposes these policies through the CLI.

**Important:** A `COMPATIBLE` verdict means no known breaking or review-required changes were found **in the extracted public contract specification**. It is not a security audit and does not establish runtime or storage-upgrade safety.

## Not implemented yet

- release packaging

## Workspace

| Crate | Responsibility |
| --- | --- |
| `stellaryn-cli` | CLI entry point and orchestration |
| `stellaryn-core` | Normalized interface types, validation, and stable ordering |
| `stellaryn-wasm` | Verified Soroban Wasm/spec extraction |
| `stellaryn-diff` | Deterministic compatibility findings |
| `stellaryn-report` | Deterministic terminal and machine-readable reporting |

## Quality gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

## License

MIT
