# Architecture

Stellaryn is intentionally split so extraction, normalization, comparison, and presentation remain separate concerns.

```text
Wasm/spec input
     |
     v
stellaryn-wasm
     |
 normalized facts
     v
stellaryn-core
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

Phase 1 creates only these boundaries. It does not implement a Soroban parser or compatibility engine.
