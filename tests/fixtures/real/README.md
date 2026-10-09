# Independently Compiled Soroban WASM Test Artifacts

These binaries are copied byte-for-byte from **external GitHub repositories**, pinned to immutable upstream commits. They contain executable WASM code sections and embedded `contractspecv0` specifications; unlike the Phase 10 fixtures, they are **not synthetic XDR-only modules**.

## Immutable sources

| Local artifact | Bytes | Upstream repo and path | Upstream commit | Git object SHA-1 |
| --- | ---: | --- | --- | --- |
| `testnet_increment.wasm` | 4,742 | [Stellar-Cost-Labs/soroban-cost-estimator — tests/fixtures/contract.wasm](https://github.com/Stellar-Cost-Labs/soroban-cost-estimator/blob/9627919775dcc671d6286b39cab1bee64e502686/tests/fixtures/contract.wasm) | `9627919775dcc671d6286b39cab1bee64e502686` | `2844edf0215995794b1e27f94c6c88cc9a7f92b5` |
| `sdk_constructor.wasm` | 2,434 | [stellar/rs-soroban-sdk — soroban-sdk/doctest_fixtures/contract_with_constructor.wasm](https://github.com/stellar/rs-soroban-sdk/blob/8e5b425dce5fa1dcd5a3fe1d3f3fcd18431586dd/soroban-sdk/doctest_fixtures/contract_with_constructor.wasm) | `8e5b425dce5fa1dcd5a3fe1d3f3fcd18431586dd` | `e20801812b85a8803bcabd4fe938ca2d32ca0041` |
| `mainnet_arb_bot.wasm` | 7,083 | [Inferara/soroban-ret — benchmark-data/mainnet/arb-bot-contract-CCBVCCNP.wasm](https://github.com/Inferara/soroban-ret/blob/1b6a5aecd526daf305c4324384d0592ed7608cf4/benchmark-data/mainnet/arb-bot-contract-CCBVCCNP.wasm) | `1b6a5aecd526daf305c4324384d0592ed7608cf4` | `482a061aa69ffe0039bf2f2dda071ffd07d5250a` |
| `mainnet_aqua_amm.wasm` | 44,514 | [Inferara/soroban-ret — benchmark-data/mainnet/aqua-amm-CBQDHNBF.wasm](https://github.com/Inferara/soroban-ret/blob/1b6a5aecd526daf305c4324384d0592ed7608cf4/benchmark-data/mainnet/aqua-amm-CBQDHNBF.wasm) | `1b6a5aecd526daf305c4324384d0592ed7608cf4` | `be4442d68e11b19efa4e5f3a4462ceeaee71a3bd` |
| `compiled_add_i128.wasm` | 787 | [Inferara/soroban-ret — tests/fixtures/test_add_i128.wasm](https://github.com/Inferara/soroban-ret/blob/1b6a5aecd526daf305c4324384d0592ed7608cf4/tests/fixtures/test_add_i128.wasm) | `1b6a5aecd526daf305c4324384d0592ed7608cf4` | `ec29ca8d1273d85b75213cab073c15b4c9454d23` |
| `compiled_add_u128.wasm` | 769 | [Inferara/soroban-ret — tests/fixtures/test_add_u128.wasm](https://github.com/Inferara/soroban-ret/blob/1b6a5aecd526daf305c4324384d0592ed7608cf4/tests/fixtures/test_add_u128.wasm) | `1b6a5aecd526daf305c4324384d0592ed7608cf4` | `9e15c73b755bbfb95cc67552907005c7f063fa17` |

Each upstream Git object SHA-1 was verified **before importing** its bytes into Stellaryn. The Phase 11 `real_world` test suite reruns `git hash-object` on every checked-in binary and asserts its hash matches this list.

## Provenance interpretation

- The `soroban-cost-estimator` project documents the 4,742-byte increment fixture as a compiled release artifact used in its own Stellar testnet exercise. **Stellaryn has not independently replayed that deployment.**
- The constructor fixture is from the official Stellar Soroban SDK repository.
- The mainnet-named AMM and arb-bot fixtures are from `soroban-ret`'s **public mainnet benchmark dataset**. Their chain deployment attribution comes from that dataset and was **not independently verified by Stellaryn**.
- The two compiled `add` fixtures are **independent variants** changing the `add` interface from signed to unsigned 128-bit arguments and result. They demonstrate a realistic ABI-breaking difference, **not a proven before/after on-chain upgrade of the same contract ID**.

The upstream repositories currently report Apache-2.0 licensing; retain source attribution and inspect the upstream repositories for full licensing details. For mainnet-sourced compiled binaries, underlying original-contract ownership and licensing have not been independently established. Inclusion as test data does not claim authorship, on-chain deployment verification, or security audit.

## Verify locally

```sh
for f in tests/fixtures/real/*.wasm; do git hash-object "$f"; done
cargo test -p stellaryn-cli --test real_world
python3 scripts/real_world_probe.py
```

Do not replace these fixtures with regenerated minimal WASM modules. The value of this suite is independently compiled **executable code plus real embedded specifications**.
