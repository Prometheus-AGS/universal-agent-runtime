ASSESSMENT: fix-frontend-pnpm-lock-consistency
Project: universal-agent-runtime
Date: 2026-08-20
Codebase baseline: Commit 1274039a pins entity-management at 0352c83d under a separate frontend pnpm workspace, but its committed nested lock predates the pinned submodule manifests.
Cross-tool progress: none in this child; the parent screen certification stopped before browser execution when dependency preparation rewrote the nested lock.

IMPLEMENTATION STATUS
- nested frontend workspace declaration: DONE — frontend/pnpm-workspace.yaml declares ten projects and frontend/package.json pins pnpm 11.15.0.
- nested lock manifest alignment: MISSING — a clean frozen install rejects the committed lock with 17 added and 12 mismatched entity-management dependencies.
- reproducible candidate: PARTIAL — two independent clean lock-only regenerations produced SHA-256 0a7145d678283ac45de05ffd6773e1a3ba939ac915cd1c2673383c50242f472a.
- minimum-delta preservation: PARTIAL — restoring three noncausal common-snapshot mutations produced SHA-256 43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593; frozen metadata and empty-dependency-tree installs both accepted it without changing the lock.
- parent certification: PARTIAL — source commit 1274039a exists, but screen certification must not resume until the nested lock is committed and revalidated from a clean checkout.

CROSS-TOOL PROGRESS
- NONE — no child change was registered or implemented before this assessment.

SPEC GAP SUMMARY
- The canonical frontend-build-tooling contract does not explicitly require every independently active pnpm workspace root to carry a frozen-compatible lock.
- The prior root-lock child validated the repository-root workspace only. It did not cover frontend/pnpm-workspace.yaml, so its green result could not support nested frontend commands.
- The uncomfortable failure is procedural as well as technical: the parent certification reached dependency preparation before discovering that its own nested execution root was not reproducible.

BUILD HEALTH
- committed-lock frozen install: FAIL — `pnpm --dir frontend install --frozen-lockfile --ignore-scripts` exited 1 with `ERR_PNPM_OUTDATED_LOCKFILE`; the lock hash remained `a8dd7d07c43aadb2e9809b6c80ae22184d0a41093165cae60083530d7bd846e4`.
- experimental minimum-delta metadata check: PASS — `pnpm --dir frontend install --lockfile-only --frozen-lockfile --ignore-scripts` exited 0.
- experimental minimum-delta clean install: PASS — `pnpm --dir frontend install --frozen-lockfile --ignore-scripts` installed 1,191 packages from an empty `frontend/node_modules` tree and retained SHA-256 `43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593`.
- known violations: committed `frontend/pnpm-lock.yaml` is stale; the main worktree also contains an operator-owned lock candidate with a different hash and must not be adopted without causal comparison.
- test coverage: PARTIAL — the lock controls are observed; TypeScript, lint, focused unit, strict OpenSpec, and artifact integrity remain Execute-phase work.

CONSTRAINT CHECK
- AGENTS.md violations: the current committed nested lock violates evidence-over-assertion and frozen-install reproducibility for the frontend execution root.
- constraints.md violations: NONE observed within the child surface.

GOAL PROGRESS
- make the nested frontend lock reproducible under pnpm 11.15.0: PARTIAL — a tested scratch candidate exists but is not implemented.
- preserve unrelated dependency resolutions: PARTIAL — three noncausal common snapshots were identified and restored in scratch; the final HEAD-to-candidate audit remains required.
- return control to screen-by-screen-validation: NOT MET — parent certification remains correctly paused.

ASSESSMENT COMPLETE
