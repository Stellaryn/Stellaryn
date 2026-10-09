# Stellaryn v0.1.0 — prepublication checklist

> **Working release gate, reviewed 2026-10-09.** This is an engineering checklist, not a published-release announcement or a statement of Drips eligibility. Recheck the current `main` SHA and live CI before accepting any item. Do not create a Git tag, publish a GitHub Release, or submit an application without the owner's explicit authorization.

## Verified release-hardening baseline

- Baseline `main` SHA on 2026-10-09: [`615af5df1de4c2f78be8a77cc49af2b2560fd87a`](https://github.com/Stellaryn/Stellaryn/commit/615af5df1de4c2f78be8a77cc49af2b2560fd87a).
- [Standard CI run 37970023694](https://github.com/Stellaryn/Stellaryn/actions/runs/37970023694): successful quality job on that exact SHA (documentation, formatting, warnings-as-errors Clippy, workspace tests, real compiled-WASM probe, CLI help).
- [Native release-candidate run 37970023679](https://github.com/Stellaryn/Stellaryn/actions/runs/37970023679): three successful native jobs on that same SHA. The job logs show package verification, executable smoke testing and artifact uploads:
  - Linux `x86_64-unknown-linux-gnu`: `.tar.gz` plus SHA-256 file.
  - Windows `x86_64-pc-windows-msvc`: `.zip` plus SHA-256 file.
  - macOS `aarch64-apple-darwin`: `.tar.gz` plus SHA-256 file.
- Workspace manifest version at this checkpoint: `0.1.0-alpha.1`. Native artifacts are ephemeral, **not** a stable release. Their CI retention was 14 days.
- Release candidate integrity uses SHA-256, **not** a publisher signature or identity attestation.
- See [release documentation](releasing.md), [acceptance evidence](phase-13-acceptance.md), [limitations](limitations.md), and [canonical project handoff](PROJECT_HANDOFF.md).

## Documentation publication (GitBook)

- [x] Maintain repository-owned documentation in `docs/` with `.gitbook.yaml` at the repository root.
- [x] Create an empty GitBook site named **Stellaryn Documentation** in the connected organization (site ID `site_V9vjJ` on 2026-10-09).
- [ ] Configure Git Sync **in the GitBook app** against `Stellaryn/Stellaryn`, branch `main`. This connection cannot be completed through the available GitBook integration.
- [ ] For the **initial import**, choose **GitHub → GitBook**. Never choose GitBook → GitHub when importing into the empty site, since that could overwrite existing repository documentation.
- [ ] Verify imported navigation, code blocks, internal links, commands, images, limitation wording, and that internal milestones do not appear in the public root README.
- [ ] Publish the populated site and check its actual public URL. Add that URL to the public README only after it resolves.

## Release preparation (no publishing without approval)

- [ ] Decide whether to release as `v0.1.0` or retain a prerelease label; record version semantics and intended support targets.
- [ ] Update workspace version, lockfile and related version-sensitive docs/scripts in a reviewable change; run strict quality gates and test CLI `--version`.
- [ ] Verify the final release candidate on the **same final commit** with [standard CI](https://github.com/Stellaryn/Stellaryn/actions/workflows/ci.yml) and [native release-candidate CI](https://github.com/Stellaryn/Stellaryn/actions/workflows/portability.yml).
- [ ] Confirm each target's actual `rustc -vV` host triple, package filename, SHA-256 sidecar, binary identity, real WASM verdict and expected exit behavior. Keep negative/tampering tests green.
- [ ] Write release notes, installation and verification instructions, limitations, and a decision on signing/provenance; do not imply signing unless actually implemented and verified.
- [ ] Decide on the release/tag commit and rollback/correction procedure. Obtain the owner's explicit approval **before** creating any tag or GitHub Release.
- [ ] After authorized publication, inspect the live GitHub Release, downloadable assets and matching published checksums; update README and `docs/releasing.md` to reflect the observed state.

## Contributor and application readiness

- [ ] Curate a small number of genuine, independently actionable contributor issues with owners, reproducible context, acceptance checks and test expectations. Do not generate superficial issue volume.
- [ ] Prepare an authentic CLI walkthrough/demo showing real compiled WASM and incompatibility findings; keep tool scope and limitations explicit.
- [ ] Confirm current official Stellar Wave/Drips program rules and applicable dates from primary sources before asserting eligibility; this checklist does **not** establish approval.
- [ ] Assemble evidence: repo, license, exact SHA, same-commit CI links, docs, public release, contributor materials, maintainer contact/ownership and real usage signals (if any). Mark missing evidence as unknown.
- [ ] Obtain authorization before submitting any application.

## Change control

The public root `README.md` is a product overview, **not** an internal phase log. The release checklist is intentionally under `docs/`. No workflow should auto-tag or auto-publish from this checklist. On each meaningful milestone, update the canonical [project handoff](PROJECT_HANDOFF.md) with a fresh verified SHA, links and open blockers.
