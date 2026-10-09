# Architecture

Stellaryn is intentionally split so extraction, normalization, comparison, and presentation remain separate concerns.

```text
Wasm/spec input
     |
     v
stellaryn-wasm
     |
 verified extraction
     v
stellaryn-core
  ContractInterface
  TypeRef
  validation + canonical ordering
     |
     v
stellaryn-diff
     |
 compatibility result
     v
stellaryn-report
     |
     v
stellaryn-cli
```

## Current state

Phase 1 established the workspace and CLI foundation.

Phase 2 adds the normalized interface contract in `stellaryn-core`. It still does not parse Wasm or make compatibility decisions.

Phase 3 will be the first layer allowed to understand concrete Soroban contract specification formats, after the exact Stellar/Soroban APIs and data shapes are verified.
