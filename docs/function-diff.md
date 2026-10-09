# Function Compatibility Diff

Phase 4 adds Stellaryn's first compatibility rule family.

The diff engine consumes normalized `ContractInterface` values only. It does not parse Wasm or XDR directly.

## Classifications

Each finding is one of:

- `BREAKING` — existing callers can no longer rely on the previous function shape;
- `REVIEW_REQUIRED` — a compatibility impact is plausible but not automatically provable as an on-chain break;
- `NON_BREAKING` — additive function-interface change.

Phase 4 originally introduced findings without an aggregate verdict; the later Phase 7 aggregator now combines these function findings with other domains to produce the current overall verdict.

## Function rules

| Rule ID | Classification | Meaning |
| --- | --- | --- |
| `FUNCTION_ADDED` | NON_BREAKING | A new public function exists. |
| `FUNCTION_REMOVED` | BREAKING | A previously public function no longer exists. |
| `FUNCTION_PARAMETER_ADDED` | BREAKING | The function requires an additional argument. |
| `FUNCTION_PARAMETER_REMOVED` | BREAKING | A previous argument is gone. |
| `FUNCTION_PARAMETER_REORDERED` | BREAKING | Existing arguments changed positional order. |
| `FUNCTION_PARAMETER_RENAMED` | REVIEW_REQUIRED | Type and position are unchanged but the spec name changed. |
| `FUNCTION_PARAMETER_TYPE_CHANGED` | BREAKING | An argument's structured type changed. |
| `FUNCTION_OUTPUT_COUNT_CHANGED` | BREAKING | The number of outputs changed. |
| `FUNCTION_OUTPUT_TYPE_CHANGED` | BREAKING | An output's structured type changed. |

## Parameter rename policy

Soroban function arguments are represented in an ordered input vector, but the contract specification also includes names and Stellar CLI uses those names when presenting/parsing invocation arguments.

A pure name change therefore receives `REVIEW_REQUIRED`: it is not treated as equivalent metadata, but Stellaryn does not claim that the positional host invocation itself is broken.

## Determinism

Functions are matched by name through ordered maps. Findings are sorted deterministically:

1. breaking;
2. review required;
3. non-breaking;
4. rule ID;
5. subject;
6. summary.

Documentation-only changes do not produce compatibility findings.

## Scope boundary

The function engine only evaluates public function specs. Separate, already implemented engines compare structs, numeric enums, tagged unions, error enums and events. The [overall verdict](verdict-and-exit-policy.md) aggregates those findings; this distinction does not limit the current CLI to functions.
