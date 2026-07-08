# Goals — uar-production-ready-uiux-2026-07

Created manually via `/kbd-new-phase` (2026-07-08), at the user's direct
request: "production-ready with UI/UX full capable and tested." This is
a deliberately broad phase — the goals below state intent; the concrete
gap list is expected to come from `/kbd-assess`, not be pre-guessed here.

## Why this phase exists

This project has already been through several rounds of hardening:
`uar-production-readiness-gaps` (2026-06-02) closed the critical-path
backend gaps (graceful shutdown, config truth, agent persistence, a
real runtime console instead of a dead facade); `uar-harness-parity` and
`uar-next-harness` closed most of the remaining feature parity gaps
(cancellation, OTLP tracing, sycophancy detection, resumable streaming,
guardrails, cost/budget tracking, skill activation metrics); several
frontend migration phases (`ui-base-ui-migration`,
`full-frontend-entity-mgmt-migration`, `runtime-console-ux`,
`runtime-console-validation-hardening`, `thread-topic-chat-sidebar`,
`knowledge-page-aesthetic-pass`, entity-explorer work) have iteratively
built out the UI surface. Security/dependency posture was just closed
out across three phases ending in `uar-security-audit-alerts-gate-2026-07`.

What hasn't happened yet is a single, holistic pass asking: **given all
of that, is this actually production-ready, is the UI/UX genuinely
complete and polished (not just functional), and is it tested to a
level that justifies calling it done?** That's this phase's job.

## Goals

1. **Survey and close remaining production-readiness gaps** beyond what
   `uar-production-readiness-gaps`, `uar-harness-parity`, and
   `uar-next-harness` already closed. Don't assume those phases got
   everything — re-verify against the live codebase, the same way
   `uar-carryover-audit` re-verified old carryover claims and found some
   already resolved and some still open.
2. **Audit UI/UX completeness across all frontend surfaces** (runtime
   console, entity explorer, chat, knowledge, settings, agent
   configuration, and any others discovered during assessment) — not
   just "does it render," but does it meet the bar this project has set
   for itself (command palette, dense registries, detail panes,
   breadcrumbs, sticky context, tokenized light/dark themes, per
   CLAUDE.md's product direction). **Before any UI/UX code is written**,
   the mandatory routing in CLAUDE.md's "UI/UX work routing" section
   applies: memory consult, UI/UX Pro Max analysis, `/impeccable audit`
   + `/impeccable critique` (+ task-specific `/impeccable` commands),
   `frontend-design`/`ux-designer` skill consultation, Vercel React
   best-practices/composition-patterns consultation, and a written
   distillation before implementation.

   **Sharpened at assess time (2026-07-08, user instruction):** every
   function/feature that *appears* in the frontend code — every page,
   button, form, action — must be verified to *actually function to
   task*, not just render. This project has a documented recurring
   pattern of exactly this failure mode (Runtime Console was a dead
   facade until `uar-production-readiness-gaps`; a `useEntityList`/
   `useEntity` hook API drift silently returned empty data across 6
   files until `uar-next-harness` caught it live). Where a feature is
   genuinely non-essential (not load-bearing for the product), remove
   it rather than leave it half-wired. **Two capabilities are explicitly
   load-bearing and must be retained/verified working, not candidates
   for removal:** (a) configuring and chatting with agents directly,
   and (b) configuring agents' providers and models for different
   situations (per-agent/per-task model routing, provider health/
   failover, credential management).
3. **Identify and close test coverage gaps** — unit, integration, and
   browser/e2e — needed for genuine production-ready confidence, not
   just "the existing suite passes." Assess what's actually exercised
   vs. what's just compiled.

## Non-goals

- Re-litigating or reversing any already-completed and archived change
  from prior phases.
- A ground-up UI redesign — this is an audit-and-close-gaps phase, not a
  redesign phase, unless assessment finds the current direction is
  itself the gap.
- Deciding the standing process questions carried from
  `uar-security-audit-alerts-gate-2026-07`'s reflection (OpenSpec
  lighter-weight schema, the 129-directory `openspec/changes/` backlog,
  the pre-existing `ci.yml` diff) — those are separate, human-review
  items, not this phase's scope unless the user redirects.

## Expected shape

Given the breadth here, expect `/kbd-assess` to surface a large gap
list and `/kbd-plan` to produce multiple rounds, likely split across
backend, frontend/UI, and test-coverage tracks. This phase may itself
warrant nested child phases (`/kbd-new-child`) if the assessment reveals
genuinely separable sub-initiatives — that's an assessment-time call,
not decided here.
