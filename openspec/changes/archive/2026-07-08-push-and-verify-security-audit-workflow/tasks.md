## 1. Push and merge reconciliation

- [x] 1.1 Confirm with the user before pushing (`AskUserQuestion`) — approved.
- [x] 1.2 `git push origin main` — rejected, `origin/main` had moved (8
      commits, 4 merged Dependabot PRs).
- [x] 1.3 `git fetch origin` + inspect what changed — found PR #69
      already merged `vite` 7.3.3→8.1.3, directly conflicting with
      `frontend-npm-remediation`'s (prior phase) deliberate `^7.3.6` pin.
- [x] 1.4 Surfaced the conflict to the user via `AskUserQuestion` before
      resolving — chose to accept origin's vite 8.1.3 as the new baseline.
- [x] 1.5 `git merge origin/main --no-commit --no-ff` — 3 real conflicts:
      `Cargo.lock`, `frontend/package.json`, `frontend/pnpm-lock.yaml`.
- [x] 1.6 Resolved `Cargo.lock`: removed a stale `bollard`/`bollard-stubs`
      entry (origin/main predates `direct-network-facing-vulns`'s
      `testcontainers` removal), regenerated via `cargo check`.
- [x] 1.7 Resolved `frontend/package.json`: took origin's `vite ^8.1.3`.
- [x] 1.8 Resolved `frontend/pnpm-lock.yaml`: rebuilt from THIS SESSION's
      already-fixed lockfile (undici/js-yaml patched) + `pnpm update vite`
      — not origin's lockfile wholesale, which would have silently
      reverted those fixes (caught via a `pnpm audit` re-run showing 8
      vulnerabilities reappear, corrected before proceeding).

## 2. Fix Vite 7→8 regressions surfaced by the merge

- [x] 2.1 `cargo check` (invokes the frontend build via `build.rs`)
      failed: Vite 8/Rolldown removed the object-form `manualChunks`.
      Converted `frontend/vite.config.ts` to the (deprecated but
      supported) function form.
- [x] 2.2 Fix didn't take effect — found a stale, git-tracked duplicate
      `frontend/vite.config.js` (untouched since the original React/Vite
      migration, referenced by no script/config) silently shadowing the
      `.ts` config. Deleted it.
- [x] 2.3 Frontend build then failed on a `lightningcss` (bundled with
      Vite 8) syntax error: Tailwind v4's `--spacing()` theme function,
      used in `toggle-group.tsx`. Found 6 total occurrences across
      `sidebar.tsx` (×2), `calendar.tsx`, `combobox.tsx` (×3), and
      `toggle-group.tsx` (×1) via `grep -rln -- "--spacing(" src/`.
      Replaced each with the literal `calc()`/`rem` equivalent
      (`spacing(N)` = `N × 0.25rem`) — these were already producing
      invalid/no-op CSS under this project's actual Tailwind v3 pin; the
      old `lightningcss` silently tolerated it, the new one errors.
- [x] 2.4 Frontend build succeeds; `cargo check --lib --tests` passes.

## 3. Verify the merge

- [x] 3.1 `cargo check --lib --tests` — clean.
- [x] 3.2 `cargo test --lib` — 387/388 pass (1 pre-existing ignore), no
      regression.
- [x] 3.3 `cargo clippy --lib` — 499 warnings, same as baseline.
- [x] 3.4 `cargo audit` — 11 vulnerabilities unchanged (all pre-existing,
      disclosed); warnings dropped 8→7 (`scc` incidentally resolved by
      one of Dependabot's merged patch-group bumps).
- [x] 3.5 `pnpm audit` (frontend) — 0 vulnerabilities.

## 4. Push, dispatch, and verify the workflow fires

- [x] 4.1 Commit the merge (`git commit`, standard merge commit with 2
      parents).
- [x] 4.2 `git push origin main` — succeeded.
- [x] 4.3 `gh workflow run security-audit.yml` — dispatched a real
      `workflow_dispatch` run.
- [x] 4.4 `gh run watch` — confirmed all 4 jobs (`rust-audit`,
      `npm-root-audit`, `frontend-audit`, `sdk-typescript-audit`) passed.
      Only informational annotations (Node.js 20 deprecation notices on
      the underlying GitHub Actions, harmless).

## 5. Unplanned: reconcile the GitHub-reported 50-alert count

- [x] 5.1 GitHub's push output reported "50 vulnerabilities" — much
      higher than local tooling's 11. Checked
      `gh api repos/.../dependabot/alerts?state=open` directly — found
      only 4 currently open (the "50" was a stale pre-scan count from
      before this push completed processing).
- [x] 5.2 Investigated the 4 open alerts: 2 already-known/disclosed
      `hickory-proto` GHSA IDs (not previously tracked by `cargo audit`'s
      RustSec db — same not-reachable disposition confirmed via
      `cargo tree -i`), plus 2 new, real, reachable, patch-available
      CVEs: `cmov` (`CVE-2026-50185`) and `opentelemetry_sdk`
      (`CVE-2026-48504`).
- [x] 5.3 Fixed both: `cmov` via scoped `cargo update -p cmov --precise
      0.5.4`; `opentelemetry_sdk` via bumping the whole `opentelemetry`
      family in `Cargo.toml` (`opentelemetry` 0.31→0.32,
      `opentelemetry-otlp` 0.31.1→0.32.0, `opentelemetry_sdk`
      0.31→0.32.1, plus `tracing-opentelemetry` 0.32.0→0.33.0 — its
      version doesn't track `opentelemetry`'s 1:1, needed the bump to
      compile against the new API).
- [x] 5.4 Re-ran the full verify suite (task 3) after these fixes — all
      green, no regressions.
- [x] 5.5 Updated `docs/DEPENDENCY_MANAGEMENT.md` with both fixes and the
      lesson that `cargo audit` alone under-covers vs. GitHub's own GHSA
      database.

## 6. Update KBD state

- [x] 6.1 Update
      `.kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/progress.json`
      (`change_status.push-and-verify-security-audit-workflow` → DONE,
      `changes_completed` → 4/4, phase ready for `/kbd-reflect`).
