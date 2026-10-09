# Phase 11 — Independently Compiled Soroban Contract Validation

Phase 10 verified synthetic Soroban `ScSpecEntry` XDR and minimal WASM custom sections. Phase 11 adds **six external, independently compiled WASM binaries** whose bytes and sources are pinned in [tests/fixtures/real/README.md](../tests/fixtures/real/README.md). The suite exercises code-bearing contracts, varied SDK builds, the official SDK's constructor fixture, and a 44.5 KB complex mainnet-dataset AMM module.

## Acceptance evidence and reproducible commands

```bash
cargo test -p stellaryn-cli --test real_world
python3 scripts/real_world_probe.py

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p stellaryn-cli -- --help
```

The `real_world` integration tests verify:

1. Every checked-in binary starts with the WASM magic/version, has at least one compiled **code body** and exactly one nonempty `contractspecv0` section (so it is not just a Phase 10 XDR-only fixture).
2. `git hash-object` of every artifact matches its pinned upstream Git object SHA-1.
3. All six independently compiled specifications parse and validate without falling back to an empty interface. Every self-diff is `COMPATIBLE` with **zero findings**.
4. The independently built increment fixture contains `increment(step: i64)`; the official SDK fixture contains `__constructor`.
5. Large and small mainnet-dataset samples have distinct public interfaces and generate deterministic breaking findings when cross-compared.
6. Two compiled `add` variants produce `FUNCTION_PARAMETER_TYPE_CHANGED` and an `INCOMPATIBLE` verdict; the CLI policy exits with code 2.
7. JSON output remains parseable and complete, `--fail-on never` preserves the verdict, and truncated compiled WASM produces an **analysis error** with empty stdout.

All prior Phase 2–10 tests remain required, including the 44-case golden matrix and fail-closed malformed-input suite.

## Reviewed cross-contract comparison results

The `scripts/real_world_probe.py` script executes the actual CLI with JSON output and `--fail-on never`, validates verdicts and finding counts, and prints rule IDs for review. The following results were observed on the Phase 11 CI implementation run:

| Analysis | Verdict | Breaking | Review | Non-breaking | Notable rule(s) |
| --- | --- | ---: | ---: | ---: | --- |
| Every artifact compared with itself (six checks) | `COMPATIBLE` | 0 | 0 | 0 | None |
| Testnet increment → official SDK constructor fixture | `INCOMPATIBLE` | 1 | 0 | 3 | `FUNCTION_REMOVED` for `increment`; newly added constructor and `get_data` |
| Small arb-bot → complex AMM mainnet-dataset contract | `INCOMPATIBLE` | 6 | 0 | 90 | Five named function removals and one custom type removal; additive differences |
| Compiled `add(i128, i128) -> i128` variant → `add(u128, u128) -> u128` variant | `INCOMPATIBLE` | 3 | 0 | 0 | Two `FUNCTION_PARAMETER_TYPE_CHANGED` and one `FUNCTION_OUTPUT_TYPE_CHANGED` |

The cross-comparisons between **different products** deliberately reveal broad interface differences; they do not validate a real-life upgrade path. The two `add` fixtures compare *separately built type variants* and specifically validate a realistic ABI-breaking change. They are not known to be past/future revisions of one deployed contract.

## Findings and limitations

This phase found **no additional false-compatible extraction result** among these six successfully parsed fixtures. That is a limited observation, not proof that other contracts have no missing/incorrect findings. The Phase 10 hardening to reject empty and duplicate specification sections remains enforced.

**Unresolved validation gap:** There is not yet an independently verified, time-ordered pair of WASM revisions upgraded for the **same on-chain contract ID** with a trusted third-party compatibility oracle. Therefore no claim of comprehensive false-positive/false-negative measurement, stored-state migration safety, authorization equivalence, or deployment approval is made. A `COMPATIBLE` verdict is spec-level only; code changes that retain the same public spec can still change behavior dramatically.

The AMM and arb-bot mainnet labels are based on the upstream benchmark dataset, not a fresh chain RPC confirmation. In addition, this phase does **not** execute the contracts in a Soroban VM.

The detailed source and license/provenance caveats are in the fixture README. Phase 12 will make documentation and contributor workflow easier to follow; later work can add pinned same-contract real upgrade history and runtime migration checks as separate research tasks.
