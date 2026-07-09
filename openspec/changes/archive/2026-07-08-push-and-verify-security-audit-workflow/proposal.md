## Why

`.github/workflows/security-audit.yml` was added in
`uar-dependabot-remediation-2026-07` but has never actually run — it only
exists in the local git history, never pushed to `origin/main`, so
GitHub Actions has no way to discover or fire its `schedule`/
`workflow_dispatch` triggers. This phase's own assessment confirmed the
root cause (`gh run list --workflow=security-audit.yml` → 404; `git
status` → 16+ commits ahead of `origin/main`). Pushing and dispatching a
real run is the only way to close this loop for real, rather than relying
on local simulation again.

## What Changes

- `git push origin main` (with explicit user confirmation obtained via
  `AskUserQuestion` before running it).
- **Unplanned but necessary**: the push was initially rejected —
  `origin/main` had moved (8 new commits, 4 merged Dependabot PRs)
  while this phase's work was in progress locally. One of those PRs
  (#69) had already merged `vite` 7.3.3→8.1.3 in `frontend/` — the exact
  major-version jump `frontend-npm-remediation` (prior phase) had
  deliberately avoided. Surfaced this conflict to the user directly
  before proceeding; they chose to accept origin's vite 8.1.3 as the new
  baseline. Merging then required:
  - Resolving 3 real conflicts (`Cargo.lock`, `frontend/package.json`,
    `frontend/pnpm-lock.yaml`), all rooted in the vite version disagreement
    plus a stale `bollard`/`bollard-stubs` entry left over from before
    `direct-network-facing-vulns` (prior phase) had removed
    `testcontainers` — `origin/main` predates that removal.
  - Rebuilding `frontend/pnpm-lock.yaml` by starting from *this session's*
    already-fixed lockfile (with `undici`/`js-yaml` already patched) and
    running `pnpm update vite` on top — naively taking `origin/main`'s
    lockfile wholesale would have silently reverted those fixes (caught
    via a `pnpm audit` re-run showing 8 vulnerabilities reappear).
  - Fixing two real Vite 7→8 regressions surfaced by `cargo check`
    (which invokes the frontend build via `build.rs`):
    1. Vite 8 (Rolldown) removed the object-form `build.rollupOptions
       .output.manualChunks` — converted `frontend/vite.config.ts` to the
       (deprecated but supported) function form. This didn't take effect
       until a stale, git-tracked duplicate `frontend/vite.config.js`
       (untouched since the original React/Vite migration, referenced by
       no script or config) was deleted — it was silently shadowing the
       `.ts` config.
    2. The newer `lightningcss` bundled with Vite 8 now strictly rejects
       Tailwind v4's `--spacing()` theme function — 6 occurrences across
       `sidebar.tsx`, `calendar.tsx`, `combobox.tsx`, and
       `toggle-group.tsx` (shadcn-ui components using v4 syntax despite
       this project pinning Tailwind v3) were replaced with the literal
       `calc()`/`rem` equivalent (`spacing(N)` = `N × 0.25rem`). These
       classes were already producing invalid/no-op CSS under Tailwind
       v3 — the old `lightningcss` silently tolerated the invalid syntax;
       the new one errors instead.
- `gh workflow run security-audit.yml` (manual `workflow_dispatch`
  dispatch, verified rather than waiting for the Monday cron).

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "CI Trigger Actually Fires"
  requirement (a new or modified CI trigger is not considered verified
  until observed firing on the actual CI platform, not just locally
  simulated).

## Impact

- **Affected code**: `Cargo.lock`, `frontend/pnpm-lock.yaml`,
  `frontend/package.json` (merge conflict resolution), plus
  `frontend/vite.config.ts` (manualChunks fix), deletion of
  `frontend/vite.config.js` (stale duplicate), and 4 component files'
  Tailwind class fixes (`sidebar.tsx`, `calendar.tsx`, `combobox.tsx`,
  `toggle-group.tsx`).
- **Runtime UX / provider compatibility / realtime state**: none expected
  — the CSS fixes replace invalid/no-op arbitrary values with their
  correct literal equivalent (same visual result); the manualChunks fix
  preserves the same vendor-chunk groupings.
- **KBD workflow state**: `progress.json` for
  `uar-post-dependabot-followup-2026-07` updated to DONE for this change
  once the workflow run is confirmed — this is the phase's 4th and final
  change; the whole phase closes out (ready for `/kbd-reflect`) once
  archived.
