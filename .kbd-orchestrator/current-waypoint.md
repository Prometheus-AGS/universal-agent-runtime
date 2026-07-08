# Current Waypoint — universal-agent-runtime

- **Phase:** uar-security-audit-alerts-gate-2026-07
- **Status:** executing
- **Progress:** 2 of 3 changes (Round 1 COMPLETE)
- **Next pending change:** `verify-dependabot-alerts-gate-live` (Round 2)
- **Exact next command:** ask user to approve commit+push, then `/opsx:new verify-dependabot-alerts-gate-live` + `/kbd-apply verify-dependabot-alerts-gate-live`
- **Recommendation source:** seeded from `uar-post-dependabot-followup-2026-07`'s reflection.md §7 "Next Phase Recommendations", high-priority item

## Execute-phase dispatch (2026-07-08, see `phases/uar-security-audit-alerts-gate-2026-07/execution.md` for full contract)

Backend: `openspec`, self-executing via Claude Code CLI, driven per-change through `/kbd-apply` (never bare `/opsx:apply`) — matches every prior phase in this project.

**Round 1 COMPLETE (2/2):**

- **`add-dependabot-alerts-ci-gate`: DONE** (archived `openspec/changes/archive/2026-07-08-add-dependabot-alerts-ci-gate/`, 8/8 tasks, verify PASS, QA gate skipped — 2 files changed, under the 3-file threshold, and matches this project's standing `D-E` artifact-refiner-gate-decision). Token-source decision resolved via `AskUserQuestion`: user chose to reuse `secrets.SUBMODULES_TOKEN` over provisioning a new secret. New `dependabot-alerts-gate` job added to `security-audit.yml` with a fail-loud preflight check + inline `DISCLOSED_GHSA_IDS` allowlist; `docs/DEPENDENCY_MANAGEMENT.md` updated. Dry-run verified locally across 3 scenarios (pass / undisclosed-alert fail / API-error fail), all correct.
- **`migrate-vite-rolldown-codesplitting`: DONE** (archived `openspec/changes/archive/2026-07-08-migrate-vite-rolldown-codesplitting/`, 5/5 tasks, verify PASS, QA gate skipped — 1 file changed). `frontend/vite.config.ts` migrated to `build.rolldownOptions` + `codeSplitting.groups`; `pnpm run build` confirmed the same 4 vendor chunks, `chunkSizeWarningLimit` still honored, no new warnings. Added a new `frontend-build-tooling` capability spec (OpenSpec's validator requires ≥1 delta per change — the proposal's initial "no capability" framing needed correcting before `validate` passed).

**Neither is committed or pushed yet.** Files changed: `.github/workflows/security-audit.yml`, `docs/DEPENDENCY_MANAGEMENT.md`, `frontend/vite.config.ts` (+ OpenSpec/KBD state). Per this phase's standing approval gate (the one irreversible step), commit+push needs explicit user sign-off before `verify-dependabot-alerts-gate-live` (Round 2) can dispatch `security-audit.yml` on real GitHub Actions and confirm `SUBMODULES_TOKEN` actually has sufficient scope for the new job.

## Plan (2026-07-08, see `phases/uar-security-audit-alerts-gate-2026-07/plan.md` for full detail)

3 changes, 2 rounds:
- **Round 1 (parallel, no shared files):** `add-dependabot-alerts-ci-gate` (M/frontier — new job in `security-audit.yml` + `docs/DEPENDENCY_MANAGEMENT.md` update), `migrate-vite-rolldown-codesplitting` (S/small — `frontend/vite.config.ts`)
- **Round 2:** `verify-dependabot-alerts-gate-live` (S/small — depends on Round 1's CI change; likely blocked on an operator-supplied token secret)

Key finding from planning (web-verified per Rule 22/23): the default `GITHUB_TOKEN` can **never** read the Dependabot alerts REST API from inside Actions, regardless of any `permissions:` block — a hard platform limitation, not a config gap (GitHub community discussion #60612). `add-dependabot-alerts-ci-gate` must read a dedicated secret instead; **which token source to use is an open product decision to surface to the user at execute time**, not assumed. Also web-verified Rolldown's real `manualChunks` → `codeSplitting.groups` migration path for the vite change.

Goal 4 (Tailwind v4 syntax grep) needed no change — already confirmed MET during assessment.

Sycophancy self-check on the plan: score 0.0.

## Assessment findings (2026-07-08, see `phases/uar-security-audit-alerts-gate-2026-07/assessment.md`)

- **Goal 1** (Dependabot-alerts CI job): MISSING — zero implementation exists.
- **Goal 2** (verify a real green run): blocked on Goal 1. Current 4-job baseline reconfirmed green; Dependabot alert state independently reconfirmed clean (2 open, both disclosed `hickory-proto`); `cargo audit` unchanged (11 vuln/7 warn, no drift).
- **Goal 3** (vite `manualChunks` migration): PARTIAL — object-form removal already done last phase.
- **Goal 4** (Tailwind v4 syntax grep): MET — confirmed clean via this assessment's own grep across `frontend/src`.
- Sycophancy self-check on the assessment: score 0.018.

## Why this phase

`uar-post-dependabot-followup-2026-07` closed 4/4 changes and 4/4 goals
(2026-07-08), but its own execution proved the case for this phase: while
chasing down a confusing push-time vulnerability count, a manual
`gh api dependabot/alerts` check turned up 2 real, reachable CVEs
(`cmov` / CVE-2026-50185, `opentelemetry_sdk` / CVE-2026-48504) that
`cargo audit` never flagged. `security-audit.yml` — the scheduled CI
workflow this project ships specifically to catch this class of issue —
has no equivalent check, so it can go green while real, disclosed,
upstream-fixed CVEs sit unflagged. That's the structural gap this phase
closes.

## Goals (see `phases/uar-security-audit-alerts-gate-2026-07/goals.md` for full detail)

1. Add a `gh api dependabot/alerts` check (or equivalent) to
   `security-audit.yml` as a required complement to `cargo audit`.
2. Confirm the next real scheduled or dispatched `security-audit.yml` run
   stays green with the new job included, and that the currently-clean
   alert state (2 open, both already-disclosed/not-reachable
   `hickory-proto` findings) still holds.
3. (Medium priority) Migrate `vite.config.ts`'s deprecated `manualChunks`
   function form to Rolldown's `codeSplitting` API.
4. (Medium priority) Broader grep for Tailwind v4-only CSS syntax beyond
   the 6 sites already fixed last phase.

## Decisions carried forward (still load-bearing)
- D-A: RAG hardened in-process; Knowledge Service extraction deferred
- D-B: MemPalace stays off
- D-C: LibreFang integration scoped to UAR side
- D-D: dependency pins deliberate — `rmcp`/`surreal-memory`/
  `prometheus_parking_lot` are SHA-pinned, `kreuzberg` is tag-pinned, none
  float (corrected + reconfirmed in the prior phase, 2026-07-08)

## Carried-over debt (unrelated to this phase's scope, tracked for later)
- CH-06 per-agent/per-task budget configuration surface (global-only today)
- CH-08 activation-outcome correlation (recall wired; outcome half unsolved)
- Durable cost/spend history for CH-07 dashboard
- `main()` always loads full `AppConfig` before dispatching any subcommand
- Standing process question (carried 3+ phases): whether OpenSpec needs a
  lighter-weight schema variant for hygiene-only changes — human call,
  not yet resolved
- 129 unarchived `openspec/changes/` directories (long-standing backlog
  predating this phase, flagged during `/kbd-status`, not itemized)

## Prior phase archive

- **`uar-post-dependabot-followup-2026-07`** (2026-07-08): 4/4 changes,
  4/4 goals MET. Fixed ARCHITECTURE.md's D-D pin-type swap, SHA-pinned
  `surreal-memory`, verified `security-audit.yml` fires for real on
  GitHub (after resolving a genuine `origin/main` push conflict + 2 Vite
  7→8 regressions), triaged all 9 remaining unmaintained/unsound
  `cargo audit` warnings. Bonus finding (this phase's root cause): 2 real
  CVEs caught only via manual `gh api dependabot/alerts`, never by
  `cargo audit`. See its own `reflection.md` for full detail.
- **`uar-dependabot-remediation-2026-07`** (2026-07-08): 8/8 changes,
  3/4 goals MET (D-D re-affirmation goal carried forward, closed by the
  phase above).
- **`uar-security-deps-and-hygiene`** (2026-07-04): 10/10 changes across
  4 risk-ordered rounds. Upgraded `surrealdb`, `rmcp`, `wasmtime`; added
  `.github/dependabot.yml`.
- **`uar-spec-v2-and-polish`** (2026-07-04): 7/7 changes, G4+G5 MET.
- **`uar-next-harness`**: 16/24 changes, G1–G3 MET, G4–G5 deferred.
