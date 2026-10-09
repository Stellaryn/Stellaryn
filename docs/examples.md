# Worked examples

These examples use **checked-in, upstream-built WASM binaries** so they can be copied verbatim from the repository root. The exact artifacts and original Git hashes are in [fixture provenance](../tests/fixtures/real/README.md).

## A. No change, no findings

```bash
cargo run -p stellaryn-cli -- compare \
  tests/fixtures/real/sdk_constructor.wasm \
  tests/fixtures/real/sdk_constructor.wasm
```

Expected: `COMPATIBLE`, zero findings, **exit 0**. This is a self-comparison only; it does not mean the code is safe to deploy.

## B. Breaking numeric-type changes

```bash
cargo run -p stellaryn-cli -- compare \
  tests/fixtures/real/compiled_add_i128.wasm \
  tests/fixtures/real/compiled_add_u128.wasm \
  --format json
```

Expected: `INCOMPATIBLE`, **3 breaking findings**, process **exit 2** under the default `breaking` threshold:

| Rule ID | Subject |
| --- | --- |
| `FUNCTION_PARAMETER_TYPE_CHANGED` | `function:add::parameter:a` |
| `FUNCTION_PARAMETER_TYPE_CHANGED` | `function:add::parameter:b` |
| `FUNCTION_OUTPUT_TYPE_CHANGED` | `function:add::output:0` |

To obtain an exit-0 example while preserving the real `INCOMPATIBLE` verdict, add `--fail-on never`. **Do not** interpret that as making the upgrade compatible.

## C. JSON output

```bash
cargo run -p stellaryn-cli -- compare \
  tests/fixtures/real/compiled_add_i128.wasm \
  tests/fixtures/real/compiled_add_u128.wasm \
  --format json --fail-on never
```

The JSON envelope contains `schema_version: "1.0"`, `before`, `after`, `analysis` (verdict, counts by domain, typed rule IDs and evidence), and a safety disclaimer. Output is deterministic for the same paths and contents; it does not include timestamps.

## D. Missing or corrupt artifact

```bash
cargo run -p stellaryn-cli -- compare missing.wasm \
  tests/fixtures/real/sdk_constructor.wasm \
  --format json --fail-on never
```

Expected: **exit 1**, empty stdout, and a diagnostic on stderr. A permissive policy **cannot** convert extraction failure into a valid compatibility result.

## E. Git history mode

```bash
stellaryn git --repo /path/to/contract-repo \
  --from v1.0.0 --to main \
  --wasm artifacts/token.wasm \
  --after-wasm deployment/token.wasm \
  --format json --fail-on review
```

**Adapt both revisions and repository-relative paths** to a repository that actually commits the built WASM files. This mode does not compile from source, fetch remotes, or checkout revisions.

## Understanding results

`BREAKING` means an implemented specification rule identifies a compatibility-impacting change; `REVIEW_REQUIRED` means a potential impact needs human review; `NON_BREAKING` is additive **under the implemented spec rules**, not a deployment guarantee. Read the [rule policy](rule-authoring.md) and [limitations](limitations.md).
