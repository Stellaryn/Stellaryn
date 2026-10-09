# Terminal / JSON Reports and CLI Compare (Phase 8)

Run directly against two local compiled Soroban contract WASM artifacts:

```bash
stellaryn compare old.wasm new.wasm
stellaryn compare old.wasm new.wasm --format json
stellaryn compare old.wasm new.wasm --fail-on review
stellaryn compare old.wasm new.wasm --format json --fail-on never
```

When building from source, prefix with `cargo run -p stellaryn-cli --`.

## Comparison pipeline

```text
old.wasm, new.wasm
  -> soroban_spec::read::from_wasm
  -> verified ScSpecEntry mapping
  -> normalized ContractInterface
  -> diff_contracts
  -> complete ContractDiff (all four categories)
  -> terminal or JSON serializer
  -> ExitPolicy
```

The output is generated **before** evaluating the policy exit code. Thus breaking findings still produce a complete report. Filenames are subprocess arguments (never interpolated into a shell command), and human-readable output has no ANSI escapes.

## JSON contract

`--format json` writes exactly one pretty-printed JSON object to stdout, with `schema_version` (report schema `1.0`), `before`, `after`, `analysis` (`ContractDiff`: verdict, totals, domain totals, typed findings/evidence), and `disclaimer`. Paths are echoed as supplied; the same inputs and file content yield stable output with no timestamps. `stderr` remains empty for successful analyses, including policy violations.

The report schema `1.0` is independent from the normalized interface schema `1.1`.

## Exit code policy

| Outcome | Code |
| --- | --- |
| Successful completed comparison passing policy | 0 |
| Missing, malformed, or unreadable WASM; invalid spec or analysis | 1 |
| Successful comparison that violates `--fail-on` | 2 |
| Invalid command arguments (Clap usage error) | 2 |

The default `--fail-on breaking` blocks `INCOMPATIBLE`. `--fail-on review` blocks both `INCOMPATIBLE` and `REVIEW_REQUIRED`; `--fail-on never` blocks neither. A policy never conceals an extraction error. On such errors stdout stays empty; a diagnostic is written to stderr.

## Trust boundary

`COMPATIBLE` means **no known breaking/review-required public specification changes found by implemented rules**. It does not prove identical runtime behavior, contract-state migration compatibility, on-chain authorization correctness, or deployment safety. No security-audit claim is made.

## Phase 8 acceptance

GitHub CI at the final commit must pass fmt, Clippy warnings-as-errors, all tests, and CLI help. Tests must exercise actual `contractspecv0` WASM generated from typed `ScSpecEntry` XDR to prove output format, deterministic rendering, valid/missing/corrupt input handling, `--fail-on` combinations, stdout/stderr separation, and process exit codes. Git-ref comparison was implemented in Phase 9. Later phases expanded the compiled-contract evidence and developer documentation.
