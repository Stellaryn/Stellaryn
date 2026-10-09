# Stellaryn v0.1.0 — release and remaining readiness checklist

> **Verified release update, 2026-10-09:** [v0.1.0 was published](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0) from tag SHA `17de3459ef24641c6544e845e35ba5a7fbcd0f9e`, with six platform/checksum assets. [Release workflow](https://github.com/Stellaryn/Stellaryn/actions/runs/37982396203) completed successfully. This checklist also preserves the previous historical prepublication baseline and tracks completed GitBook publication and still-open application work; it is not evidence of Drips eligibility.

## Verified public release (2026-10-09)

- [x] Workspace and lockfile version `0.1.0`; CLI version and product metadata tested on final release commit.
- [x] Final baseline same-commit [CI](https://github.com/Stellaryn/Stellaryn/actions/runs/37982084864), [Linux/Windows/macOS candidates](https://github.com/Stellaryn/Stellaryn/actions/runs/37982085083), and [publication workflow](https://github.com/Stellaryn/Stellaryn/actions/runs/37982396203) successful.
- [x] [Public v0.1.0 Release](https://github.com/Stellaryn/Stellaryn/releases/tag/v0.1.0) verified: tag, six uploaded archive/`.sha256` assets, linked release notes, and tag SHA `17de3459ef24641c6544e845e35ba5a7fbcd0f9e`.
- [x] [Public GitBook documentation](https://oobayemi.gitbook.io/stellaryn-documentation/) verified through the GitBook publishing API (`published=true`), after successful GitHub → GitBook import and inspection of the populated navigation and key pages.
- [ ] Stellar Wave/Drips eligibility, application and approval remain unverified.

## Historical verified release-hardening baseline

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
- [x] GitBook site Git Sync is **active**, linked to `Stellaryn/Stellaryn` `main`, with a successful GitHub → GitBook import of `./docs` on 2026-10-09.
- [x] Initial import used **GitHub → GitBook**, verified in GitBook operation metadata. Future updates should preserve source-of-truth discipline and avoid reversing sync unintentionally.
- [x] Inspect imported navigation and representative pages covering setup commands, examples, release checksums, limitations and README safeguards. GitBook converted internal doc links to site-relative paths; automated GitHub documentation checks remain required. No independent browser crawl or full visual review is claimed.
- [x] Publish the populated site and confirm GitBook reports the public URL [https://oobayemi.gitbook.io/stellaryn-documentation/](https://oobayemi.gitbook.io/stellaryn-documentation/). Add the URL to public README via reviewed PR. A separate browser-load verification remains advisable because public browser access was unavailable from the assistant's environment.

## Release preparation (no publishing without approval)

- [x] Decide release version `v0.1.0`; record verified target triples and scope.
- [x] Update workspace version, lockfile and version-sensitive docs/tests through verified PR #2.
- [x] Verify final release commit on [standard CI](https://github.com/Stellaryn/Stellaryn/actions/runs/37982084864) and [native release-candidate CI](https://github.com/Stellaryn/Stellaryn/actions/runs/37982085083).
- [x] Confirm target triples, release asset names, sidecars, packaged executable behavior, known real-WASM verdict and negative/tampering checks with [publishing workflow](https://github.com/Stellaryn/Stellaryn/actions/runs/37982396203).
- [x] Publish reviewed release notes and limitations, and clearly document SHA-256 integrity **without** any identity-signing or notarization claim.
- [x] Owner authorized publication in this conversation, and the workflow created the tag only from the exact verified `main` commit. Never overwrite the tag; issue a new version for corrections.
- [x] Inspect the live public GitHub Release with its six downloadable assets; reconcile root README and `docs/releasing.md` through a separate reviewed docs PR.

## Contributor and application readiness

- [x] Curate three genuine, independently actionable contributor issues [#3](https://github.com/Stellaryn/Stellaryn/issues/3), [#4](https://github.com/Stellaryn/Stellaryn/issues/4), [#5](https://github.com/Stellaryn/Stellaryn/issues/5); ownership/assignment remains to be confirmed.
- [ ] Prepare an authentic CLI walkthrough/demo showing real compiled WASM and incompatibility findings; keep tool scope and limitations explicit.
- [ ] Confirm current official Stellar Wave/Drips program rules and applicable dates from primary sources before asserting eligibility; this checklist does **not** establish approval.
- [ ] Assemble evidence: repo, license, exact SHA, same-commit CI links, docs, public release, contributor materials, maintainer contact/ownership and real usage signals (if any). Mark missing evidence as unknown.
- [ ] Obtain authorization before submitting any application.

## Change control

The public root `README.md` is a product overview, **not** an internal phase log. The release checklist is intentionally under `docs/`. No workflow should auto-tag or auto-publish from this checklist. On each meaningful milestone, update the canonical [project handoff](PROJECT_HANDOFF.md) with a fresh verified SHA, links and open blockers.
