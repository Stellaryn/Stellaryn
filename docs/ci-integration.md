# CI integration

Stellaryn returns a compatibility verdict **and** a process exit code. CI must distinguish a comparison-policy failure from a file/extraction failure.

| Exit code | Meaning | Action |
| --- | --- | --- |
| `0` | Completed comparison passed chosen policy | Archive/review report |
| `1` | Missing, invalid, or unreadable WASM; analysis failed | Fix evidence/input; **do not treat as compatible** |
| `2` | Valid comparison violates policy | Review changes and block as configured |

Clap also returns `2` for **invalid CLI arguments** (before a report exists). Check whether a JSON report was produced when diagnosing exit 2.

## Fail-on thresholds

| Verdict | `breaking` (default) | `review` | `never` |
| --- | --- | --- | --- |
| `COMPATIBLE` | 0 | 0 | 0 |
| `REVIEW_REQUIRED` | 0 | 2 | 0 |
| `INCOMPATIBLE` | 2 | 2 | 0 |

`never` is useful for reporting-only jobs, **not** for accepting upgrades as safe.

## GitHub Actions example

After your pipeline generates two **local compiled WASM** files, run:

```yaml
- name: Compare contract specifications
  shell: bash
  run: |
    set +e
    cargo run -p stellaryn-cli -- compare \
      build/old.wasm build/new.wasm \
      --format json --fail-on review > stellaryn-report.json
    status=$?
    set -e

    if [ "$status" -eq 1 ]; then
      echo "Stellaryn could not analyze the WASM files" >&2
    elif [ "$status" -eq 2 ]; then
      echo "Stellaryn rejected a compatibility change or CLI arguments" >&2
    fi
    exit "$status"

- name: Upload comparison report
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: stellaryn-report
    path: stellaryn-report.json
    if-no-files-found: ignore
```

The example assumes Rust/Cargo and Stellaryn source are available in the runner. The report is written **before** a policy exit, so it remains available for review if compatibility fails. On an extraction error stdout remains empty; inspect stderr logs instead.

## Comparing Git revisions

```bash
stellaryn git --repo . --from 'HEAD~1' --to HEAD \
  --wasm artifacts/token.wasm --format json --fail-on review
```

This requires that the compiled binary is committed **at both refs**. Git mode does not build historical Rust source and does not touch a dirty worktree.

## Determinism and trust

Machine consumers should use stable **rule IDs** and the documented JSON envelope, not parse prose or ANSI output. The report schema is `1.0`, separate from the normalized interface schema `1.1`. See [CLI report format](cli-reports.md).

A `COMPATIBLE` verdict cannot establish code behavior, authorization, stored-data migration, or security safety; read [limitations](limitations.md).
