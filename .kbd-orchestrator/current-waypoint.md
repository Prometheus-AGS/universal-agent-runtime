# Current Waypoint — universal-agent-runtime

- **Phase:** uar-production-ready-uiux-2026-07
- **Status:** executing
- **Progress:** 6 of 9 changes
- **Next pending change:** `bdd-chat-scenario-suite` (Round 3, no blockers)
- **Exact next command:** `/opsx:new bdd-chat-scenario-suite` + `/kbd-apply bdd-chat-scenario-suite`
- **Recommendation source:** created manually via `/kbd-new-phase` at the user's direct request ("production-ready with UI/UX full capable and tested") — not seeded from a reflection recommendation, since `uar-security-audit-alerts-gate-2026-07`'s reflection had no single dominant next-phase recommendation.

## Execute-phase dispatch (2026-07-08, see `phases/uar-production-ready-uiux-2026-07/execution.md` for full contract)

Backend: `openspec`, self-executing via Claude Code CLI, driven per-change through `/kbd-apply` (never bare `/opsx:apply`). **User confirmed via `AskUserQuestion`: keep as one flat phase** — no child-phase split for the BDD or Docusaurus tracks.

**Round 1: 3/3 DONE.**

- **`fix-comprehensive-tests-ci-gate`: DONE** (archived `openspec/changes/archive/2026-07-08-fix-comprehensive-tests-ci-gate/`, 9/9 tasks). Root cause was deeper than the assessment found: `test-config.yaml` was both never created **and** listed in `.gitignore` since the initial commit — fixed both, pushed (`9d4da6a`). Verified live on GitHub Actions for the first time in this project's history: `comprehensive-tests.yml`'s Pre-flight Checks **passed** (run [28966990812](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/28966990812)), but Security Audit and Code Quality then failed for real, newly-surfaced reasons (unfiltered inline `cargo audit`; frozen root `bun.lockb`). `tests-full.yml` (run [28967666152](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/28967666152)) got 8 real minutes into work (previously failed in <1 minute) before failing on a Docker Compose `surreal`/`unstructured` health-check timeout. Per this change's own design non-goal, none of these 3 newly-surfaced issues were fixed here — documented in `findings.md` as follow-up work, not yet triaged into this phase's plan.
- **`fix-auth-revoke-key-error-surfacing`: DONE** (archived `openspec/changes/archive/2026-07-08-fix-auth-revoke-key-error-surfacing/`, 3/3 tasks). `auth-keys-store.ts`'s `revokeKey` now sets an error message on failure instead of silently swallowing it, matching `load`/`createKey`'s existing pattern. `pnpm run build` clean.
- **`upgrade-a2ui-testing-live-round-trip`: DONE** (archived `openspec/changes/archive/2026-07-09-upgrade-a2ui-testing-live-round-trip/`, 13/13 tasks). Rescoped mid-flight from the plan's default `retire-a2ui-testing-page-from-prod` after the user rejected removal ("It seems we need something like that"). Added `POST /api/uar/runs/{run_id}/a2ui/test-trigger` (`src/uar/a2ui/routes.rs`) emitting a real `ArtifactInputRequest` via the same `RunManager::emit_to_run` path a live agent tool call uses; reworked `A2uiTestingPage.tsx` to target a real active run and hand off to the real `/threads` chat UI on success instead of a parallel mock render; also fixed the pre-existing schema browser, which silently read stale field names never matching the real `ArtifactSchema` shape (always fell through to "Unknown schema type"). Live-verified via new `tests/integration/live/a2ui_test_trigger_cases.rs` (2/2 passing against the real booted server + stub LLM): full trigger→submit round trip, and 404-on-nonexistent-run. Empirically confirmed (not just by code read) that `RunManager::emit_to_run` does not lose the event when no SSE client is subscribed at trigger time, resolving `design.md`'s stated buffering risk. Caught and removed one stray untracked `frontend/vite.config.js` (non-gitignored compiled duplicate of the tracked `vite.config.ts`) during the git-scope check.

**Round 2: 3/3 DONE.** Fix-vs-remove decision resolved via `AskUserQuestion` (2026-07-09): cheap "not yet wired" gating banner for all three. New `NotWiredRuntimeState` component (built on the project's existing, previously-unused `Alert`/`AlertTitle`/`AlertDescription` shadcn primitive) applied to `RuntimeProtocolsPage`'s 3 dead panels, `RuntimeCockpitPage`'s Provider Health + Memory Activity, and `RuntimeRunsPage`'s Artifacts panel. `resolve-runs-artifacts-and-inspect-button` also did a real fix (not a facade decision): the previously-dead "Inspect" button now wires through a new `onInspect(runId)` prop on `RunRow` — `RuntimeRunsPage` tracks `selectedRunId` (seeded from a new `?run=` search param) and shows that run's detail instead of always the first run; `RuntimeCockpitPage`'s Inspect navigates to `/admin/runs?run=<id>` since Cockpit has no detail column. All 3 changes: `pnpm run build` + `pnpm run typecheck` clean, archived.

**1 open blocker to surface to the user before its round:**
- Round 4 (`bootstrap-docusaurus-site`): hosting/deployment target, not yet resolved.

## Plan (2026-07-08, see `phases/uar-production-ready-uiux-2026-07/plan.md` for full detail)

**9 changes, 4 rounds.** User expanded scope when invoking `/kbd-plan`:
- **Round 1** (parallel): `fix-comprehensive-tests-ci-gate` (M/frontier, highest-value — the headline assessment finding), `fix-auth-revoke-key-error-surfacing` (S/small), `retire-a2ui-testing-page-from-prod` (S/small)
- **Round 2** (same file, sequence to avoid conflicts): `resolve-runtime-protocols-page-facade`, `resolve-runtime-cockpit-dead-panels`, `resolve-runs-artifacts-and-inspect-button` — **2 of these need a fix-vs-remove product decision at execute time**, not decided here
- **Round 3** (independent, new scope): `bdd-chat-scenario-suite` (L/frontier) — Cucumber/Gherkin + Playwright BDD suite via the `bdd-testing` skill, with `bdd-video-proof` capturing IPFS-pinned video evidence per scenario, covering chat with/without knowledge base, skills activated, tool calls, agent switching, and provider/model config — user's explicit, detailed ask
- **Round 4** (independent, docs/branding): `bootstrap-docusaurus-site` (L/frontier — **no Docusaurus site existed anywhere in the repo**; user confirmed via `AskUserQuestion` to bootstrap a new one from `docs/`), `refresh-readme-diagrams-and-branding` (M/medium — **confirmed a real factual error**: README.md claims "HTMX, Web Components, Alpine.js; no React" but the actual frontend is 100% React/TypeScript; `CLAUDE.md` has the identical stale claim, flagged but out of explicit scope)

**Scope note**: given the plan's own breadth (bug fixes + a product-completeness decision + a new test framework + a new docs platform), `plan.md` recommends considering `/kbd-new-child` to split Rounds 3–4 into their own child phases before executing, so each track gets a focused reflection. Not decided — flagged for the user.

Sycophancy self-check on the plan: 0.018.

## Assessment findings (2026-07-08, see `phases/uar-production-ready-uiux-2026-07/assessment.md` for full detail)

**Headline finding:** `.github/workflows/comprehensive-tests.yml` and `.github/workflows/tests-full.yml` have **never passed their Pre-flight/Prerequisite check since the initial commit** (2026-01-19) — both require `test-config.yaml`, which has never existed in this project's history. Confirmed 0/30 recent successes via `gh run list`. Traces to an abandoned Spec Kit feature (`specs/001-testing-infrastructure/`, 0/74 tasks complete) whose partial dead-code (`src/testing/`) was already deleted in a prior phase (`eval-harness-hardening`'s HK1) without cleaning up the spec directory or the 2 broken workflows referencing it. A prior tool assessment (`docs/CODEX_ASSESSMENT.md`, 2025-12-31) already found this exact bug and proposed a fix that was never applied.

**Both user-mandated load-bearing capabilities confirmed genuinely solid** (deep code tracing to real backend routes, no facades): chat + agent configuration, and provider/model configuration.

**4 concrete dead facades found** in Runtime Console: the Protocols page (entirely dead for dynamic content, no gating message — the exact "Protocols page gating" item carried open since `uar-production-readiness-gaps`, confirmed still unresolved); Cockpit's Provider Health + Memory Activity panels (permanently empty, backend never emits these events); Runs page's Artifacts panel (same); a Run row's "Inspect" button with no `onClick` at all.

**1 non-essential page flagged**: `A2uiTestingPage` is real but dev-only testing tooling shipping to the production admin nav. **1 minor bug**: `auth-keys-store.ts`'s `revokeKey` silently swallows errors.

**5 old carryover items re-verified**: 3 confirmed resolved (credentials admin UI, agent-config error surfacing, skill-activation-outcome now recorded backend-side), 2 confirmed still open (CH-06 per-agent budget config UI, Protocols page gating).

Build health: `cargo test --lib` 387/1/0 clean; frontend `vitest` 46/46 but thin unit coverage (12/206 files, ~5.8%); e2e Playwright (40 tests/12 files) parses cleanly but wasn't executed live this session. Sycophancy self-check: 0.018.

## Why this phase

See `phases/uar-production-ready-uiux-2026-07/goals.md` for the full narrative. In short: this project has been through several rounds of hardening (`uar-production-readiness-gaps`, `uar-harness-parity`, `uar-next-harness` for backend/feature parity; several frontend migration phases for the UI surface; three security/dependency phases ending in `uar-security-audit-alerts-gate-2026-07`), but no single phase has asked the holistic question this one does: is this genuinely production-ready, is the UI/UX complete and polished (not just functional), and is it tested to a level that justifies calling it done?

## Goals (see `phases/uar-production-ready-uiux-2026-07/goals.md` for full detail)

1. Survey and close remaining production-readiness gaps beyond what prior phases already closed — re-verify against the live codebase, don't assume.
2. Audit UI/UX completeness across all frontend surfaces, following CLAUDE.md's mandatory "UI/UX work routing" (memory consult, UI/UX Pro Max analysis, `/impeccable audit`+`critique`, `frontend-design`/`ux-designer` skills, Vercel React best-practices) before any UI code is written.
3. Identify and close test coverage gaps (unit, integration, browser/e2e) for genuine production-ready confidence.

**Deliberately broad** — expect `/kbd-assess` to surface a large gap list and `/kbd-plan` to produce multiple rounds, possibly split into nested child phases if assessment reveals separable sub-initiatives.

## Prior phase archive

- **`uar-security-audit-alerts-gate-2026-07`** (2026-07-08): 3/3 changes, 4/4 goals MET. Dependabot-alerts CI gate built and confirmed live on GitHub Actions; Vite `manualChunks`→`codeSplitting` migration; both pushed and verified. See its own `reflection.md` for full detail, including the OpenSpec ≥1-delta-per-change finding and the now-fixed `AGENTS.md` doc-staleness items.

**No single dominant next-phase recommendation.** Only optional doc-fix pickups (documenting the validate requirement, fixing the CLI version reference) and standing process questions needing human review (whether OpenSpec needs a lighter-weight schema for hygiene-only changes; the 129-directory unarchived `openspec/changes/` backlog; the pre-existing uncommitted `ci.yml` diff). Recommend asking the user what to focus on next rather than auto-seeding from any of these.

## Execute-phase dispatch (2026-07-08, see `phases/uar-security-audit-alerts-gate-2026-07/execution.md` for full contract)

Backend: `openspec`, self-executing via Claude Code CLI, driven per-change through `/kbd-apply` (never bare `/opsx:apply`) — matches every prior phase in this project.

**ALL 3 CHANGES DONE:**

- **`add-dependabot-alerts-ci-gate`: DONE** (archived `openspec/changes/archive/2026-07-08-add-dependabot-alerts-ci-gate/`, 8/8 tasks). Token-source decision resolved via `AskUserQuestion`: user chose to reuse `secrets.SUBMODULES_TOKEN` over provisioning a new secret. New `dependabot-alerts-gate` job added to `security-audit.yml` with a fail-loud preflight check + inline `DISCLOSED_GHSA_IDS` allowlist; `docs/DEPENDENCY_MANAGEMENT.md` updated.
- **`migrate-vite-rolldown-codesplitting`: DONE** (archived `openspec/changes/archive/2026-07-08-migrate-vite-rolldown-codesplitting/`, 5/5 tasks). `frontend/vite.config.ts` migrated to `build.rolldownOptions` + `codeSplitting.groups`; `pnpm run build` confirmed the same 4 vendor chunks, `chunkSizeWarningLimit` still honored, no new warnings.
- **`verify-dependabot-alerts-gate-live`: DONE** (archived `openspec/changes/archive/2026-07-08-verify-dependabot-alerts-gate-live/`, 5/5 tasks). User approved commit+push; 5 commits pushed to `origin/main` (`b0a9eca..cbedb82`, no drift). Dispatched `security-audit.yml` for real — [run 28950786923](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/28950786923) — **all 5 jobs succeeded**, including `dependabot-alerts-gate`, whose log confirms `All 2 open Dependabot alert(s) are already disclosed.` using the real `SUBMODULES_TOKEN` in Actions. The token-scope risk flagged throughout planning and execution resolved cleanly on the **first** live attempt — the fail-loud preflight check never fired. Extended the existing `CI Trigger Actually Fires` requirement with a new scenario about credential-runtime-scope verification.

**Execution phase complete.** All goals from `goals.md` addressed: Goal 1+2 (CI gate built + confirmed green on real Actions) MET, Goal 3 (vite migration) MET, Goal 4 (Tailwind grep) was already MET at assessment time.

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
