# Troubleshooting

## I get exit code 1 instead of a JSON result

The extractor or comparison failed. Confirm that both inputs exist, are readable compiled Soroban WASM, and contain a **nonempty, unique** `contractspecv0` custom section. Arbitrary WASM modules or raw contract source cannot be compared.

Stellaryn deliberately rejects truncated/malformed trailing WASM, duplicate contract spec sections, empty contract specs, and invalid normalized interfaces rather than inventing a `COMPATIBLE` result.

## My `INCOMPATIBLE` command exited with 2

This is usually the expected `--fail-on breaking` behavior. The complete comparison report is still written to stdout. To inspect the same findings without a compatibility-policy failure, add `--fail-on never`. This does **not** change the actual verdict.

Clap also uses exit 2 for invalid command-line arguments. If there is **no JSON report** and stderr shows a usage error, inspect `stellaryn compare --help` or `stellaryn git --help`.

## A parameter rename is marked REVIEW_REQUIRED

Names can matter to tools and generated clients. Some changes cannot be proven safe or breaking from the XDR spec alone. Review the consumers manually; see [function rules](function-diff.md).

## Why is an added struct field BREAKING?

The current rules conservatively treat changes to an existing struct's public shape as breaking, and distinguish this from adding an entirely new public type. See [custom-type rules](type-diff.md).

## Git revision or artifact not found

`git` mode requires a **local Git repository**, a valid commit-ish on each side, and compiled WASM bytes already committed at the requested path. The `--wasm` and `--after-wasm` paths are **relative to the Git repository root**. `HEAD~1` only exists when your branch has a parent commit. Worktree changes are ignored. Blobs over 32 MiB are rejected.

```bash
git -C /path/to/repo rev-parse 'HEAD^{commit}'
git -C /path/to/repo cat-file -s 'HEAD:artifacts/token.wasm'
```

These are **diagnostic** commands, not steps run by Stellaryn to change Git state.

## Does COMPATIBLE mean upgrade safe?

**No.** Stellaryn compares the available public specification and only applies its currently implemented rules. Runtime behavior can change while the public specification stays unchanged. Storage layout/migrations, contract authorization, deployment steps, on-chain state, and security are not verified. See [scope and limitations](limitations.md).

## How do I report a surprising finding?

Use the [compatibility report issue template](../.github/ISSUE_TEMPLATE/compatibility.yml) with the precise two artifact versions, expected and actual rule IDs, the CLI arguments, and environment. Share a **minimal public repro**; do not upload proprietary contracts or secret keys. For security-sensitive findings, follow the [security policy](../SECURITY.md).
