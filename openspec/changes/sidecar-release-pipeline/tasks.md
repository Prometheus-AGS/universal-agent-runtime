## 0. Recording and operator gates (before any edit)

D4 falsifier 1: the tag workflow must produce a `uar-sidecar` archive for all six targets within 4 recorded working days of fixes per failing target. A target still failing after that is dropped from the first release and reported; a failing Windows target reopens D4. Days are recorded in the task log of child phase `the-boss-universal-agent-runtime` (prometheus-skills-mini repo; proposed file `effort-log.md`, shared with the D3 changes).

- [ ] 0.1 Write a dated start entry (`YYYY-MM-DD HH:MM start sidecar-release-pipeline`); verify it predates the first edit.
- [ ] 0.2 Decide design D1 (keep panic unwinding; deviates from D4's `panic = "abort"` because of the `catch_unwind` sites listed in `proposal.md`). Record the answer in `.prometheus/decisions.md`.
  Decision 2026-09-23 (technical decision by the coordinating session): accepted — `panic = "unwind"`. Abort is not adopted; the spec requirement "Release builds use the release profile" stands as written. Still open: the `.prometheus/decisions.md` entry, which the documents-only edit of this change did not write.
- [x] 0.3 Operator gate: approve extending `scripts/validate-github-actions-policy.mjs` with `sidecar-release.yml` (design D7) and the matching exception sentence in `docs/release-verification.md`. AGENTS.md forbids weakening the validator; without this approval, stop.
  Operator decision 2026-09-23: approved — add the tag-triggered sidecar release workflow to the policy validator's allowlist, and add to `docs/release-verification.md` the sentence that tag-triggered, test-disabled release builds for the sidecar are deployment, not development verification.
- [ ] 0.4 Run `pnpm github-actions-policy:validate` before editing any workflow, the validator, or a script a workflow calls; save the output to the evidence folder. A failure is a stop condition. (Planning-time baseline: `node scripts/validate-github-actions-policy.mjs` passed on 2026-09-23. `pnpm` was not run then: `package.json` pins `pnpm@11.15.0` and the installed pnpm was 10.34.5, so pnpm would have switched versions.)

## 1. Contract tests — written first, must fail before and pass after

All are static Node checks or local script runs; none builds the product and none runs in GitHub Actions.

- [ ] 1.1 `scripts/test-sidecar-release.mjs` — policy controls over fixture repositories, in the style of `scripts/test-documentation-publication.mjs`. Cases: (a) the valid `sidecar-release.yml` fixture passes `validateGitHubActionsPolicy`; each of these fails with a named failure: (b) adds `branches: [main]`, (c) adds `pull_request`, (d) adds `workflow_dispatch`, (e) adds `schedule`, (f) contains `cargo test`, (g) uses a feature string other than `minimal,local-models,document-intelligence,wasm-runtime`, (h) omits one of the six runner/target pairs, (i) omits `.sha256` generation, (j) executes the built `uar-sidecar` binary. Serves the spec requirements "runs only on a release tag" and "builds without testing". Fails before: case (a) is rejected as a prohibited non-deployment workflow.
- [ ] 1.2 `scripts/test-sidecar-release.mjs` — packaging control. Run the packaging script on a dummy file standing in for the binary and a temporary copy of the model assets. Assert the archive holds the binary, `models/` with its five files, `LICENSE` and `LICENSE-CC-BY-4.0.md`; the `.sha256` equals a digest computed independently in the test; `size-<target>.json` reports the dummy's byte size. Fails before: the script does not exist.
- [ ] 1.3 `scripts/validate-sidecar-release.mjs` — contract over the real repository. Assert `Cargo.toml` `[profile.release]` has `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"` and no `panic = "abort"` (task 0.2); `.github/workflows/sidecar-release.yml` exists with the tag-only trigger, the six pairs, and the exact build command from design D3. Serves "Release builds use the release profile". Fails before: no profile, no workflow.
- [ ] 1.4 D4 falsifier 1 evidence (not a code test): the prerelease tag run in group 8 publishes six archives and six `.sha256` files, and a digest recomputed from each downloaded archive matches its file. Recorded in `verification.md` with the run URL.

## 2. Release profile

- [ ] 2.1 Add `[profile.release]` to `Cargo.toml` per design D1 and task 0.2, with a comment naming the `catch_unwind` dependency. Verify with 1.3.

## 3. Packaging

- [ ] 3.1 Add the Node packaging script (design D4): stage binary, `models/`, licenses; write `.tar.gz` or `.zip`, `.sha256`, `size-<target>.json`. Verify with 1.2.

## 4. Policy validator

- [ ] 4.1 Extend `allowedWorkflows` with `sidecar-release.yml`, its markers and its prohibition list (design D7), leaving existing checks untouched. Verify with 1.1 and by diff review that no existing rule was removed or loosened.

## 5. Workflow

- [ ] 5.1 Write `.github/workflows/sidecar-release.yml`: `metadata` (tag equals package version), `build` matrix of six native runners with `fail-fast: false`, submodule checkout as in `deploy.yml`, the D3 build command, packaging, checksum recompute, artifact upload; `release` job with `contents: write`, size table in the notes body, `softprops/action-gh-release` pinned by commit SHA, `prerelease` for tags containing `-`. Re-check every pinned action SHA against its upstream tag at edit time. Verify with 1.1 and 1.3.

## 6. Documentation

- [ ] 6.1 Add to `docs/release-verification.md` the exception sentence approved in task 0.3 — tag-triggered, test-disabled release builds for the sidecar are deployment, not development verification — next to the rule it qualifies (`docs/release-verification.md:8-9`), and the local pre-tag guidance (design D6): the exact build command for the host target, the packaging command, and the dynamic-dependency check. Verify by comparing the documented command to the workflow's build line.

## 7. Post-edit validation and local build

- [ ] 7.1 Run `pnpm github-actions-policy:validate` after the edits; save the output. A failure is a stop condition.
- [ ] 7.2 Run `node scripts/test-sidecar-release.mjs` and `node scripts/validate-sidecar-release.mjs`; verify both pass and every test in group 1 that failed before now passes.
- [ ] 7.3 Maintainer local pre-tag build on the host target with the documented command (one build, one target directory). Record: date, target, binary bytes, archive bytes, dynamic dependencies. If ONNX Runtime is a shared library, add it to the packaging script and repeat 1.2.

## 8. Tagged run (operator actions)

- [ ] 8.1 Operator sets a prerelease package version and pushes `uar-sidecar-v<version>`; the agent does not push tags or publish. Record the run URL and per-target outcome in the task log.
- [ ] 8.2 For each failing target, record dated fix sessions against its 4-day limit; at the limit, drop the target from the matrix, note it in the release notes, and, if it is a Windows target, mark D4 reopened in the child phase.
- [ ] 8.3 Download the six archives, recompute their SHA-256 digests, compare them to the `.sha256` files, and copy the size table into `openspec/changes/sidecar-release-pipeline/verification.md`. State which targets were only built and never run.
- [ ] 8.4 `openspec validate sidecar-release-pipeline --strict` passes.
