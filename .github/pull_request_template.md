## Summary

What changed and why? Link an issue where appropriate.

## Evidence and compatibility impact

- Which spec/domain and exact rule IDs changed?
- Before/after ABI or minimal reproduction (or note docs-only).
- Authoritative Stellar SDK/XDR evidence or pinned third-party artifact references.
- Compatibility/report schema changes, if any.
- Known limitations or uncertain assumptions.

## Tests and quality gates

Check only commands **actually run**; paste relevant results or a CI link.

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `python3 scripts/check_docs.py`
- [ ] `python3 -m unittest discover -s scripts -p 'test_*.py'`
- [ ] `python3 scripts/real_world_probe.py`
- [ ] `cargo run -p stellaryn-cli -- --help`

## Regression / safety checklist

- [ ] Added a positive test **and** a negative or edge-case test, or explained why not applicable.
- [ ] Invalid/missing contract specifications cannot return false compatibility.
- [ ] Deterministic ordering and typed rule IDs are preserved (or an explicit migration is documented).
- [ ] Changes do not claim runtime, storage migration, or deployment safety.
- [ ] Public fixtures have permission/provenance; no secrets or private contract source.
- [ ] Updated docs/README and help/examples where the behavior changed.
