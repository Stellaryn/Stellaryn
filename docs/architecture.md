# Architecture

Stellaryn separates extraction, normalization, comparison, and presentation.

```text
Soroban Wasm bytes
     |
     v
soroban_spec::read::from_wasm
     |
 Vec<ScSpecEntry>
     |
     v
stellaryn-wasm
  exact XDR -> normalized mapping
     |
     v
stellaryn-core
  ContractInterface
  TypeRef
  validation + canonical top-level ordering
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

## Phase boundaries

Phase 1 established the workspace and CLI foundation.

Phase 2 established the protocol-independent normalized interface contract.

Phase 3 is the first protocol-aware layer. It is pinned to verified `soroban-spec 28.0.0` and `stellar-xdr 28.0.0` behavior and parses local Wasm without requiring a Stellar CLI subprocess.

Phase 4 consumes only the normalized interface to detect function changes. Phase 5 adds the distinct numeric-enum and tagged-union model (schema 1.1), validates their shapes, and compares custom types without parsing XDR. Phase 6 compares error definitions/codes and event specifications using the same normalized interface and stable finding policy. Phase 7 aggregates the four finding domains into a `ContractDiff` containing a spec-level verdict and deterministic totals, plus a separate policy engine for CI exit behavior. CLI wiring and reports are reserved for Phase 8.
