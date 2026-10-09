# Git Revision Comparison (Phase 9)

Git mode compares **compiled Soroban WASM files committed in Git history**. It does not compile source code, guess which WASM contract to use, contact remotes, or change the index, HEAD, branches, or working tree.

```bash
stellaryn git --repo . --from v1.0.0 --to HEAD --wasm contracts/token.wasm
stellaryn git --repo . --from 'HEAD~1' --to HEAD --wasm contracts/token.wasm --format json
stellaryn git --repo /path/to/repo --from v1.0.0 --to main \
  --wasm artifacts/old.wasm --after-wasm deployment/token.wasm --fail-on review
```

`--repo` defaults to `.`. Both `--from` and `--to` are required. `--wasm` is always required: it identifies the artifact path **relative to the Git repository root**, not the process working directory. `--after-wasm` overrides the artifact path only for the target revision; otherwise both revisions use the same path.

## Object access and safety

The implementation launches a local `git` binary through Rust `Command` with **separate arguments**, never a shell. It first checks that each revision resolves to a commit (`git cat-file -t <ref>^{commit}`), queries the object's size (`git cat-file -s <ref>:<path>`), and reads the committed binary bytes (`git cat-file blob <ref>:<path>`). It supports branches, tags, HEAD, and ancestry expressions such as `HEAD~1`. Malformed refs and paths are rejected.

For safety and predictable memory usage, each Git-stored artifact is limited to **32 MiB**. Paths cannot be absolute, traverse parent directories, contain backslashes/colons or control characters, or include empty/`.` segments. Paths may contain spaces. A ref cannot start with `-` or contain a colon, whitespace, or control characters. Git errors are surfaced with bounded diagnostics.

No use of `checkout`, `reset`, `clean`, or `worktree` occurs. **Dirty or untracked local artifacts are not substituted for committed objects.** Comparison does not mutate Git refs, index, or working files.

## Existing analysis and reporting

Each binary is analyzed with `extract_interface_from_wasm` from the verified Soroban spec extractor, then both normalized models flow through `diff_contracts` and the Phase 8 renderers. Terminal and JSON outputs reflect the same finding categories, verdict, and policy. In reports, labels use `git:<revision>:<repository-relative-path>` to distinguish them from local-file comparisons.

The exit contract is unchanged:

| Exit | Meaning |
| --- | --- |
| 0 | Analysis completed; compatible with chosen policy |
| 1 | Git read, missing ref/artifact, Wasm extraction, or analysis failure |
| 2 | Successful comparison violates policy, or Clap rejects CLI usage |

Policy: `--fail-on breaking` (default), `review`, or `never`. A successful comparison emits a complete report even if its policy exits with 2; failures emit diagnostics on stderr, without false-compatible JSON.

## Limitations

Git mode **requires compiled WASM blobs committed at both revisions** and an installed Git executable. Projects that commit only Rust source must build their artifacts separately before using this mode. Auto-building contracts from Git refs or remote URLs is not implemented; Git revision comparisons do not establish runtime compatibility, storage migration correctness, or deployment safety.
