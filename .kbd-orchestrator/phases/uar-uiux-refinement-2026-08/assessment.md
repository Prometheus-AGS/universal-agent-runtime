# ASSESSMENT: uar-uiux-refinement-2026-08

Project: universal-agent-runtime (Universal Agent Runtime)
Date: 2026-08-01
Codebase baseline: React 19 + TypeScript frontend (288 source files, 34,778 LOC) at merge commit `704c625`, with a Flat 2.0 design idiom already partially CSS-enforced, a 5-group admin console, and a 40-store Zustand state layer — but a broken workspace install and a design authority document that does not exist.
Cross-tool progress: NONE for this phase (0/0 changes; phase created 2026-08-01 by `kbd-new-phase`, no execution yet).

---

## HEADLINE FINDINGS

Four findings materially change what Plan should produce. They are listed first because each one invalidates or reshapes a stated phase goal. **H5 (187 existing OpenSpec changes already own much of this phase) is detailed in the Spec Gap Summary and is, with H1, one of the two things to act on before anything else.**

### H1 — ~~BLOCKER~~ **RESOLVED 2026-08-01** (operator-directed, after this assessment was first written)

> **Status update.** The operator directed the submodule fix immediately after the assessment. It is done and **`pnpm typecheck` now passes clean (exit 0, zero errors)**. Resolution took four steps, because initializing the submodule alone was *not* sufficient:
>
> 1. `git submodule update --init --recursive frontend/packages/prometheus-entity-management` — clears the `ERR_PNPM_WORKSPACE_PKG_NOT_FOUND` install failure.
> 2. Install still resolved but `tsc` then reported **TS2307 across 20+ files**: the package resolves to `dist/index.d.ts`, which did not exist. The submodule is its own pnpm workspace and its devDeps (incl. `tsup`) were not installed.
> 3. `pnpm install --no-frozen-lockfile --filter …core… --filter …entity-management…` inside the submodule. `--frozen-lockfile` fails on a **pre-existing** lockfile drift (`recharts` 3.8.0 → 3.9.2) in `examples/nextjs-app`, an example app unrelated to the two consumed packages.
> 4. `tsup` build of `entity-graph-core`, then `entity-graph-react` (core first — react depends on it). Both emit `.d.ts`.
>
> **Implication for Plan: this is not a one-liner and it is not self-healing.** Any fresh worktree, CI runner, or contributor clone hits the same four-step sequence, and step 3 fails outright under the `--frozen-lockfile` that CI uses by default. Two follow-ups are warranted: (a) fix the `recharts` drift in the submodule's lockfile so `--frozen-lockfile` works, and (b) ensure PEM's `dist/` is built as a prerequisite step rather than assumed present.
>
> Residual side effects, both benign: `frontend/vite.config.js` and `vite.config.d.ts` are untracked `tsc -b` outputs of the tracked `vite.config.ts`; the submodule's `pnpm-lock.yaml` now carries the reconciled `recharts` bump.
>
> **Full gate sweep after the fix:** typecheck **PASS**, lint **PASS**, unit tests **PASS** (153/153), coverage **FAIL** (19.45% vs 60%). Three of four gates are green; coverage is the single remaining red one and is a breadth gap rather than a correctness failure — every test that exists passes.
>
> The original finding is preserved below as the record of what was assessed.

#### Original finding (as assessed, before the fix)

`pnpm typecheck` fails before typechecking begins:

```
[ERR_PNPM_WORKSPACE_PKG_NOT_FOUND] "@prometheus-ags/prometheus-entity-management@workspace:*"
is in the dependencies but no package named "@prometheus-ags/prometheus-entity-management"
is present in the workspace
```

Root cause is confirmed, not inferred. `frontend/packages/prometheus-entity-management` is a **git submodule that is not initialized in this worktree**:

```
git submodule status
-7f982fcc5b981314053323fe6ef3016ea1eb4988 packages/prometheus-entity-management
^ leading "-" = not initialized
```

Meanwhile `frontend/pnpm-workspace.yaml` declares two packages *inside* that uninitialized submodule:

```yaml
packages:
  - "packages/prometheus-entity-management/packages/entity-graph-core"
  - "packages/prometheus-entity-management/packages/entity-graph-react"
```

56 source files import from PEM. So this is not a peripheral break — the entity layer that Wave 2 is supposed to complete is the layer that will not install.

Consequence: **every build-dependent claim in this assessment is UNKNOWN, not PASS.** Typecheck, lint, build, and test results could not be obtained. No wave gate in this phase can be evidenced until `git submodule update --init --recursive` (or equivalent) is run in this worktree. This should be change #1 and it blocks essentially everything else.

### H2 — The binding design authority named in the goals does not exist

Goal Wave 0 R1 says to "adopt `docs/knowme-ui-ux-standard.md` as the single binding design authority." That file is **absent from the working tree and absent from all of git history**:

```
find . -iname "*knowme*"        → only frontend/src/components/KnowMeLogo.tsx
git log --all -- 'docs/knowme*' → (no commits)
```

The two documents the goals say to *archive* both exist (`docs/UI_DESIGN.md`, `docs/admin-aesthetic-spec.md`), and `docs/ui-design-authority.md` does not yet exist. So the phase is currently structured to retire two real documents in favor of one that has never been written.

"KnowMe" is nonetheless real as a **code-level idiom** — 19 references across `AppShell.tsx`, `Titlebar.tsx`, `enhanced-thread.tsx`, `about-page.tsx`, with explicit design commentary ("KnowMe idiom: a filled surface, never an outlined input"). The vocabulary exists in the code; the specification does not exist as a document.

This is an open question for the operator, not something Plan can resolve by itself: either the standard is an external artifact that must be imported into the repo, or Wave 0's first change is to *author* it by extracting the idiom already encoded in the components and CSS.

### H3 — Two Wave premises are already satisfied or do not apply

- **"Console IA regroup into 5 nav groups"** (Wave 1): `admin-shell.tsx` already defines exactly 5 nav groups — Runtime, AI & Models, Agents & Tools, Infrastructure, Development. The regroup is DONE. What remains is **dev-flag gating, which is entirely absent** — there is no environment or feature flag anywhere in `admin-shell.tsx`, so the Development group (Compiler, A2UI Testing) ships to production users today. That is the real Wave 1 nav work, plus the A2UI Testing retirement (Wave 4).
- **"finish the PEM entity migration for the remaining `migrate-*` pages"** (Wave 2): there are **no `migrate-*` files** in `frontend/src` — but the goal was never pointing at filenames. `migrate-*` is a family of **OpenSpec change IDs** (`migrate-compiler-page-direct-and-redesign`, `migrate-knowledge-page-…`, `migrate-memory-page-…`, `migrate-models-page-…`, `migrate-skills-page-…`, `migrate-settings-mutations-and-retire-stores` at 0/15, and others). See H5. Plan should resolve these against the 56 PEM-importing source files rather than treat the goal as vacuous.

Goals citing counts that *are* accurate (see below) sit alongside these two that are not, so the goal list should be treated as a strong but unverified draft, re-grounded per item.

---

## IMPLEMENTATION STATUS

| Area | Status | Evidence |
|---|---|---|
| Workspace install / build health | **BLOCKED** | Uninitialized PEM submodule; `pnpm typecheck` fails at install (H1) |
| Design authority docs | **MISSING** | `knowme-ui-ux-standard.md` and `ui-design-authority.md` absent (H2). ADR infrastructure, by contrast, **exists** — see below |
| ADR infrastructure | **DONE** | `docs/adr/` holds **17 ADRs** with an established convention (`0001-record-architecture-decisions.md` … `0016-…`, plus `ADR-007-react-first-frontend.md`). Wave 0's R1/R2/R3 ADRs are authored *into an existing system*, not bootstrapped |
| Flat 2.0 enforcement | **PARTIAL** | Already CSS-enforced in `index.css` (`* { @apply border-transparent }`, 6 enforcement sites) — but 629 border idioms, 50 `variant="outline"`, 9 shadow files, 7 blur/gradient files still in markup |
| Console IA (5 nav groups) | **DONE** | 5 groups already present in `admin-shell.tsx:72-111` (H3) |
| Terminal CRT theme | **PARTIAL** | Only 6 `data-admin-theme` references — retirement is small, well-scoped |
| Style lint gate | **MISSING** | No `no-restricted-syntax` rule in `eslint.config.js` |
| Storybook visual-regression gate | **NEARLY DONE (spec) / STUB (stories)** | `.storybook/` is configured and the owning change `docs-storybook-visual-regression-perf-budget` is at **26/30 tasks**. But only **1 story file** exists across 288 sources, so VR coverage is minimal regardless of task count. Plan should confirm what those 26 tasks delivered before assuming either "done" or "missing" (H5) |
| TanStack Query removal | **CONFIRMED DEAD** | Zero `useQuery`/`useMutation`/`useQueryClient` call sites; provider-only in `App.tsx:2,12,62` — safe deletion |
| PEM entity migration | **PARTIAL/UNVERIFIABLE** | 56 files import PEM; migration state cannot be verified while the submodule is uninitialized (H1) |
| Store inventory/charter | **MISSING** | 40 store modules in `src/stores/`, no charter table; retirement candidates unidentified |
| settings-page.tsx split | **NOT STARTED** | Exactly **3,336 lines** — goal metric confirmed precisely; next-largest file is 1,200 |
| Chat run-scoped store (runId) | **PARTIAL** | `chat-stream-store.ts` already threads `runId` through streaming + SSE resume, but `runId` is client-generated (`run-${Date.now()}`) and not a store-keying dimension for background-run continuity |
| Per-thread drafts | **MISSING** | No draft state in any store (only an unrelated Immer-draft comment) |
| Library FTS / pin / archive / export | **MISSING** | No FTS in `src/lib/db.ts` |
| Library recency grouping | **MISSING** | No date-bucketing ("Today"/"Yesterday"/"Previous 7 days") in the thread list; `thread-registry-store.ts` carries no grouping selector |
| Running/failed/unread badges | **MISSING** | Run status is tracked in `chat-stream-store`, but no per-thread badge state is projected onto the library list; no unread concept exists in the thread model |
| Undo-delete | **MISSING** | Thread deletion has no soft-delete, tombstone, or undo affordance |
| Console nav dev-flag gating | **MISSING** | `admin-shell.tsx` contains **no** environment or feature flag — the "Development" group (`compiler`, `a2ui-testing`, lines 108-111) renders unconditionally in production builds. The only `import.meta.env.DEV` use in the entire frontend is `main.tsx:22`, unrelated to nav. Every nav item is currently ungated |
| ContentBlock exhaustiveness | **MISSING + CONTRACT SPLIT** | See H4 below |
| Regenerate-as-variant | **MISSING** | No regenerate/variant action in any chat store or feature module; the only `variant` hits are A2UI component props and Button styling, unrelated to message regeneration |
| Branch-from-message | **MISSING** | No `branchFrom`/`branch-from` symbol anywhere in `frontend/src`; the thread model has no fork/parent-message concept |
| Lane labeling | **STUB** | Exactly one lane reference exists — `AppShell.tsx:52`, a *readiness* lane (status dot + deployment mode), correctly avoiding color-alone. This is not the cloud/local/uar execution-lane labeling the goal intends; no per-message or per-run lane surface exists |
| First-run recovery (No-Model gate) | **PARTIAL, and the goal's premise needs checking** | Onboarding machinery already exists (`use-onboarding`, `use-onboarding-welcome`, `admin-welcome.tsx`, `uar-onboarding-settings-visited`). The "No model" strings found are **non-blocking advisories** in the admin surface (`agent-editor.tsx:371`, `agents-page.tsx:334` `aria-label="No model configured"`) — consistent with the prior phase's completed `admin-agent-model-warning-clarity` change. A *blocking* chat-side No-Model gate was not located. Plan should confirm the blocking gate still exists before scheduling its replacement |
| Dedicated chat error surfaces | **PARTIAL** | Error handling is dispersed across `use-chat-runtime.ts`, `session-config-panel.tsx`, `a2ui-artifact-block.tsx`, `attachment-preview.tsx`; no dedicated error surface component and no chat-level ErrorBoundary |
| Runs trace tree / replay | **STUB** | `runs` nav item exists; only `runtime-console-page.tsx` (502 lines) backs it; replay fixtures present in `src/entities/` |
| Approvals queue UX | **STUB** | Nav item only; no dedicated page |
| Home instrument panel | **MISSING** | No Home destination in nav or pages |
| A2UI KnowMe retheming | **PARTIAL — narrower than expected** | `a2ui-surface-renderer.tsx` already renders through **semantic design tokens** (`text-foreground`, `text-muted-foreground`, `text-destructive`, shared `Label`), and already meets 44px touch targets (`min-h-11`). Retheming is therefore a token/idiom alignment pass over one renderer plus the `a2ui-*` packages, not a rewrite. Distinct from the A2UI Testing nav retirement, which is a separate one-line nav change |
| React boundary gate | **PARTIAL — 1 violation from green** | `scripts/check-frontend-boundaries.mjs` (99 lines) is fully implemented and **already enforces an empty violation allowlist** (exits 1 on any). Live run: **1 violation**, `frontend/src/main.tsx\|component-store-import`. Two narrow infrastructure fetch exceptions are allowlisted (`entities/sync.ts`, `lib/pglite-assets.ts`). A negative-fixture test exists (`scripts/test-frontend-boundaries-negative.mjs`) |
| Responsive sweep (320/768/1024/1440) | **MINIMAL** | Only **1440** has any automated coverage — `tests/bdd/features/ui-shell.feature:8-12` opens at 1440px and captures `shell-rail-1440`. **320, 768, and 1024 have no frontend coverage** (the other numeric hits are unrelated byte sizes in Rust integration tests). No dual-theme responsive assertions found |
| 20-screen inventory | **EXISTS — reuse, do not rebuild** | `docs/product-surface-inventory.md` already maps every `App.tsx` route and `PAGE_MAP` admin section (23 table rows) to owner, API contract, evidence, maturity, and required certification. It already records per-screen maturity (Certified stable / Stable candidate / Preview) and names the missing certifications. Wave 5 should execute against this table rather than author a new inventory |
| Screen-by-screen validation | **PLANNED, 0% EXECUTED** | `openspec/changes/screen-by-screen-validation/` exists with a proposal and **0 of 5 tasks complete**. Its own "Why" states none of the 18 admin screens had BDD coverage at authoring time. This change is the Wave 5 vehicle and is already specified — Plan should absorb or supersede it rather than duplicate it |
| Video-proof evidence bundles | **TOOLING PRESENT, EVIDENCE ABSENT** | `tests/bdd/playwright.config.ts:56` sets `video: 'on'`, and line 47 documents a Cucumber-format JSON report "consumed by the bdd-video-proof skill". Capture infrastructure therefore exists; what is missing is executed runs and minted bundles |
| Accessibility certification | **NOT STARTED** | `aria-label`s present in shell; no axe run, no keyboard/landmark/reduced-motion evidence |
| Test coverage | **MEASURED — FAILS the 60% gate** | **19.45% lines** (1179/6059), 19.09% statements, 14.59% functions, 11.07% branches. `vitest run --coverage` exits 1 on all four thresholds. Binding floor is 60% (ADR 0003 + `docs/coverage-baseline.md`, CI-enforced). The 153 tests that exist all pass — this is a breadth gap, not a correctness problem. Uncovered mass is concentrated in Wave 2/3 target stores (`chat-stream-store.ts` at 6.95%; nine stores at 0%) |

### H4 — ContentBlock: two disconnected type systems, not one union needing exhaustiveness

The goal says "ContentBlock exhaustiveness with compile-time enforcement" over 11 variants. What the code actually shows:

- `src/types/chat-content.ts` defines a **9-member** union (text, reasoning, tool-call, citation, rag-citations, skill-activation, context-update, image, error) — not 11.
- `enhanced-thread.tsx:406-450` switches on a **completely different** tag set from assistant-ui's own part model (`group-chainOfThought`, `group-tool`, `group-reasoning`, `text`, `reasoning`, `tool-call`, `data`, `indicator`) and ends in `default: return <>{children}</>`.
- There is **no `assertNever` / `never` guard** anywhere in the chat type or render path.

So the app's `ContentBlock` union and the renderer's part union are separate contracts that merely overlap in naming. Adding exhaustiveness to the 9-member union would not make the renderer exhaustive, because the renderer never switches on that union. Plan must first decide whether to unify the two contracts or enforce them independently — the goal as written assumes a single union that does not govern rendering.

---

## CROSS-TOOL PROGRESS

NONE — no cross-tool activity recorded for this phase. `progress.json` shows 0/0 implementation, all stage flags false, `createdBy: kbd-new-phase`.

Prior-phase context: `uar-hybrid-app-architecture` sits at 4/12 changes complete (`desktop-stable-port`, `pglite-oxide-intel-mac-spike`, `admin-sw-scheme-safe-caching`, `admin-agent-model-warning-clarity`) and is **not reflected**. Its waypoint still lists three unstarted round-1 changes (`admin-agent-provider-first-model-picker`, `governance-tool-approval-reconciliation`, `admin-ui-freeze-diagnostics`). A second phase, `perform-the-soak-run-...`, remains paused at 0/14.

**Open process question:** this phase starts while the previous phase is 4/12 and unreflected, and its three pending admin-UI changes overlap this phase's Console/Admin surface. Plan should state explicitly whether those three are absorbed here or the prior phase is resumed — leaving both live risks duplicate work on the same files.

---

## SPEC GAP SUMMARY

- **H5 — CORRECTION, and the largest planning risk in this phase: 187 unarchived OpenSpec changes already exist, many owning this phase's goals outright.** An earlier draft of this assessment stated `openspec/changes/` was empty. That was wrong — it came from a `ls | head` whose output I misread as empty. The directory holds **187 change dirs** (plus 90 archived). Directly overlapping this phase:

  | Existing change | Tasks done | Phase goal it owns |
  |---|---|---|
  | `impeccable-uiux-audit` | no `tasks.md` | the `/impeccable` UI/UX overhaul |
  | `uiux-remediation-wave-1` | no `tasks.md` | Wave 1 design-system convergence |
  | `docs-storybook-visual-regression-perf-budget` | **26/30** | Wave 1 Storybook VR gate — nearly complete |
  | `retire-a2ui-testing-page-from-prod` | 0/5 | Wave 4 A2UI Testing nav retirement |
  | `screen-by-screen-validation` | 0/5 | Wave 5 certification + video proof |
  | `migrate-settings-mutations-and-retire-stores` | 0/15 | Wave 2 store retirement |
  | `migrate-*-page-direct-and-redesign` (compiler, knowledge, memory, models, skills) | not surveyed | Wave 2 — **this is the `migrate-*` work H3 could not find in `frontend/src`; it lives in `openspec/changes/`, not as page filenames** |
  | `typescript-7-hybrid-migration` | not surveyed | the TS 7.0 migration goal |

  Two consequences. First, H3's "migrate-* targets nothing" was half right: no such *source files* exist, but a whole family of *specified changes* does — the goal was pointing at spec IDs, not filenames. Second, `docs-storybook-visual-regression-perf-budget` at 26/30 means the Wave 1 VR gate is 4 tasks from done, not missing.

  **Plan must inventory `openspec/changes/` in full and reconcile before authoring anything new.** Writing fresh proposals over 187 existing ones is the most likely way this phase does duplicated or conflicting work. The phase's "every change carries a spec delta" constraint is about *deltas*, not about new proposals — several already exist.
- **ADR infrastructure is present** — `docs/adr/` contains 17 accepted ADRs with a numbering convention already in force. Wave 0's R1/R2/R3 ADRs should follow it as `0017-`/`0018-`/`0019-`; nothing needs bootstrapping. (An earlier draft of this assessment wrongly reported the directory as absent — the check had been run from inside `frontend/`. Corrected after adversarial review.)
- **No spec covers the design system** — the Flat 2.0 rules live only in CSS comments and component comments, never in a spec. This is precisely the gap H2 describes.
- **Brand identity (R2) is unresolved and load-bearing** — "KnowMe" is hardcoded across shell, titlebar, thread, and about page. A config-driven brand slot is not a cosmetic change; it touches every surface that currently imports `KnowMeLogo`.

## BUILD HEALTH

- build check (as originally assessed): **FAIL** — `pnpm typecheck` failed at the `pnpm install` prerequisite, `ERR_PNPM_WORKSPACE_PKG_NOT_FOUND`
- **build check (after the H1 fix, 2026-08-01): PASS** — `pnpm typecheck` → `tsc -b`, **exit 0, zero errors**
- **lint (2026-08-01): PASS** — `pnpm lint` → `eslint .`, exit 0, zero errors and zero warnings
- **unit tests (2026-08-01): PASS** — `pnpm test` → `vitest run`, **32 files / 153 tests, all passing**, 8.98s
- **coverage (2026-08-01): FAIL — the only red gate remaining.** `vitest run --coverage`, exit 1, failing all four thresholds against the 60% floor:

  | Metric | Actual | Threshold | Gap |
  |---|---|---|---|
  | Lines | **19.45%** (1179/6059) | 60% | −40.55 |
  | Statements | **19.09%** (1332/6976) | 60% | −40.91 |
  | Functions | **14.59%** (360/2467) | 60% | −45.41 |
  | Branches | **11.07%** (550/4966) | 60% | −48.93 |

  This settles the open question flagged earlier: the 10% test-*file* ratio was a proxy, and the true line figure is **19.45%** — roughly double the proxy, but still less than a third of the required floor. The gap is ~40 points, not a rounding error.

  Concentration matters more than the aggregate for planning. The uncovered mass is in exactly the stores this phase intends to rework: `chat-stream-store.ts` **6.95%** lines (the 1,200-line file, second-largest in the tree), `chat-message-store.ts` 25.78%, `runtime-console-store.ts` 29.54%, `settings-store.ts`, `theme-store.ts`, `ui-store.ts`, `provider-models-store.ts`, `attachment-upload-store.ts`, and `user-jwt-settings-store.ts` all at **0%**. Several 0% stores are Wave 2 retirement candidates — coverage on a store that is about to be deleted is wasted effort, so Plan should sequence the store charter (Wave 2) *before* any coverage remediation, or the work is done twice
- known violations: 629 border idioms, 50 `variant="outline"`, 9 shadow files, 7 backdrop-blur/gradient files (all contradicting the Flat 2.0 rule already enforced in `index.css`)
- test coverage: **UNKNOWN (line %)**; file ratio is 29 test files / 288 sources ≈ 10%. The project floor is **60% lines**, not 80% — ADR 0003 explicitly rejected 80% on day one in favor of a calibrated 60%, and `docs/coverage-baseline.md` records it as CI-enforced. A low test-*file* ratio does not by itself establish a coverage violation; the line number cannot be obtained until H1 is fixed
- React boundary gate: **FAIL — 1 violation**, `frontend/src/main.tsx|component-store-import` (`node scripts/check-frontend-boundaries.mjs`, exit 1). Runs independently of the broken install
- Rust side: not exercised this stage; this phase is frontend-scoped

## CONSTRAINT CHECK

- **CLAUDE.md violations**: NONE detected in the layering contract sampled. Stores call services; the components sampled (`admin-shell.tsx`, `enhanced-thread.tsx`) render and delegate. Layering was spot-checked, not exhaustively audited — an audit is warranted before Wave 2 restructures the state layer.
- **Coverage floor (60% lines, ADR 0003)**: **VIOLATED — now measured, not inferred.** Actual is **19.45% lines**; the gate fails on all four metrics (see Build Health). An earlier draft recorded this as UNVERIFIED because the install was broken; it has since been measured directly. Note the governing threshold is 60%, not the 80% my global agent rules assert — ADR 0003 explicitly rejected 80% on day one, and per the repo's own precedence rule project-specific rules override base rules.
- **Worktree convention**: COMPLIANT — running under `~/.claude/worktrees/uar-uiux-refinement-2026-08`.
- **Stale CLAUDE.md execution lock**: `CLAUDE.md` still declares an "Active production-completion execution lock" for `uar-final-production-hardening-2026-07` with a 24/24 objective. The canonical waypoint is now this phase. Per the repo's own rule that `current-waypoint.json` is authoritative, this is stale text that should be updated so it stops contradicting the active phase.

## GOAL PROGRESS

| Goal | Status | Reason |
|---|---|---|
| Wave 0 — Decisions & authority | **NOT MET** | Binding authority doc does not exist (H2); brand slot unresolved with KnowMe hardcoded across 4+ surfaces. ADR infrastructure is present (17 ADRs), so the ADR sub-goal is cheap |
| Wave 1 — Design-system convergence | **PARTIAL** | Counts confirmed (629/50/9/7); Flat 2.0 already CSS-enforced; 5-group IA already done but **nav dev-flag gating entirely absent** (Development group ships to prod); lint gate missing; VR gate spec at 26/30 but only 1 story exists (H3, H5) |
| Wave 2 — State-layer completion | **PARTIAL** | TanStack deletion confirmed safe (zero call sites); settings-page 3,336 lines confirmed; `migrate-*` premise targets nothing (H3); PEM state unverifiable (H1) |
| Wave 3 — Chat elevation | **NOT MET** | `runId` threaded but not a store-keying dimension; no drafts, no FTS, no pin/archive/export; **no regenerate-as-variant, no branch-from-message**; lane labeling is a readiness lane only, not execution lanes; error surfaces dispersed with no chat ErrorBoundary; first-run gate premise needs re-confirmation; ContentBlock contract is split, not merely non-exhaustive (H4) |
| Wave 4 — Console elevation & Home | **NOT MET** | Runs is a nav entry over a single 502-line console page; no Approvals page; no Home destination. A2UI retheming is narrower than framed — the renderer already uses semantic tokens |
| Wave 5 — Certification | **NOT MET, but no longer blocked and already scaffolded** | H1 is fixed, so evidence is now obtainable: typecheck/lint/tests are green and coverage is measured at 19.45% (fails the 60% gate — the one red gate left). Additionally: the **boundary gate runs today** and sits **1 violation** from green (`main.tsx`); responsive coverage exists only at **1440** (320/768/1024 absent); the **20-screen inventory already exists** (`docs/product-surface-inventory.md`, 23 rows with per-screen maturity); **video capture is already configured** (`video: 'on'` + bdd-video-proof wiring); and `screen-by-screen-validation` is specified at 0/5. Wave 5 is execution over existing scaffolding, not greenfield |
| Process constraints | **AT RISK** | Zero OpenSpec changes authored; prior phase 4/12 and unreflected with overlapping admin-UI scope |

## RECOMMENDED SEQUENCING INPUT FOR PLAN

Not a plan — the ordering constraints the evidence imposes:

1. ~~**Restore the workspace first.**~~ **DONE 2026-08-01** — submodule initialized, PEM deps installed, `entity-graph-core` + `entity-graph-react` built, `pnpm typecheck` green (see H1). Two follow-ups remain and should be scheduled as real changes: fix the `recharts` lockfile drift so the submodule installs under `--frozen-lockfile` (CI's default), and make PEM's `dist/` build an explicit prerequisite step so a fresh clone or CI runner does not hit the same four-step manual sequence. Lint, test, and coverage are now obtainable and should be measured before Plan commits to remediation scope.
2. **Inventory the 187 existing OpenSpec changes before authoring any new one (H5).** This is now the highest-leverage step after the build fix. At least eight existing changes own this phase's goals, one is 26/30 done, and the `migrate-*` family resolves a goal this assessment first read as vacuous. Reconciling first prevents duplicated and conflicting work; skipping it near-guarantees both.
3. **Resolve H2 with the operator before Wave 0 changes.** Import the KnowMe standard, or author it from the idiom already in `index.css` and the components. Wave 0 cannot execute against a missing authority.
4. **Re-derive Wave 1 and Wave 2 scope** from actual code plus the existing changes: drop the completed 5-group regroup, add the entirely-absent nav dev-flag gating, resolve `migrate-*` against the OpenSpec family (H5), keep the confirmed Flat 2.0 counts.
5. **Decide the ContentBlock contract question (H4)** before scheduling 3.1, since the goals state 3.1 blocks 3.2–3.5 and the blocking change is currently mis-specified.
6. **Settle the prior-phase overlap** — absorb its three pending admin-UI changes or resume that phase; do not run both against the same files.
7. ~~**Measure coverage before targeting it.**~~ **DONE 2026-08-01 — and it is a real gap.** Lint and the 153 unit tests pass clean; **coverage is 19.45% lines against a 60% floor**, failing all four thresholds. Remediation is genuinely needed. But **sequence it after the Wave 2 store charter, not before**: nine stores sit at 0% and several are retirement candidates, so covering them first means writing tests for code about to be deleted. Decide what survives, then cover what survives.
8. **Harvest the Wave 5 work that is not blocked.** Two items need no install: fix the single `main.tsx` boundary violation to close the gate, and add 320/768/1024 responsive coverage to sit alongside the existing 1440 BDD scenario. These are small and de-risk the certification wave early. Wave 5 also does not need a new inventory — `docs/product-surface-inventory.md` and the `screen-by-screen-validation` change already exist.

## SYCOPHANCY SELF-CHECK

The `sycophancy-correction` MCP was not reachable this session (the `assess:before` memory-recall hook also returned a stub, `memory unreachable`), so `detect_sycophancy` could not be invoked and no JSON artifact was written to `sycophancy/`. Manual check applied instead:

- **S-02 (no unearned agreement with existing patterns):** The Flat 2.0 CSS enforcement is reported as *partial and contradicted* by 629 live border idioms, not endorsed. The 5-group IA is credited as done based on cited line numbers, not on the goal's assertion.
- **S-03 (surface real friction):** Four load-bearing negatives are recorded — a blocking build failure, a missing binding document, two goal premises that target nothing, and a split type contract. Two goals are downgraded relative to their own framing.
- **S-06 (no "clearly"/"obviously"):** Conclusions carry file and line evidence or are marked UNKNOWN. Every unverifiable claim is labeled unverifiable rather than assumed passing.

Residual risk: build-dependent status values are UNKNOWN rather than PASS, and the CLAUDE.md layering contract was spot-checked rather than exhaustively audited. Both are stated above rather than smoothed over.

## ADVERSARIAL REVIEW (round 1)

Cross-model isolated review ran against this artifact and returned **BLOCK** — 2 CRITICAL, 3 WARNING.

- judge: `kbd-judge` via `rest-gateway:http://localhost:8181/v1`
- producer: `claude-opus-5`; `cross_model_check: verified-distinct`
- anti-theater screen: **PASS** (score 0.0, strict) — the report was not evasive
- artifacts: `review/assess/packet.json`, `review/assess/findings.json`

All five findings were verified against the filesystem and **all five were upheld**. The judge caught two genuine factual errors of mine:

1. **CRITICAL — ADR directory wrongly reported absent.** `docs/adr/` holds 17 ADRs. My check had run with the shell inside `frontend/`; I read the resulting error as absence instead of re-running from the repo root. Corrected in Implementation Status, Spec Gap Summary, and Wave 0.
2. **CRITICAL — Wave 3 under-assessed.** Five goal items (regenerate-as-variant, branch-from-message, lane labeling, first-run recovery, dedicated error surfaces) had no gap analysis. All five now carry file-level evidence rows. This also surfaced that the goal's "lane labeling" and "No-Model gate" premises may not match what the code does.
3. **WARNING — 80% coverage requirement was unsourced.** The binding floor is 60% (ADR 0003, which explicitly rejected 80%). The 80% figure came from my global agent rules, which the repo's own precedence rule overrides. Downgraded from "VIOLATED" to "UNVERIFIED" and re-sourced.
4. **WARNING — Wave 5 not actually assessed.** Locating the gate produced the most actionable finding in this document: the boundary gate already enforces an empty allowlist and is **1 violation** from green. Responsive coverage exists at 1440 only.
5. **WARNING — A2UI retheming unassessed.** Added; the renderer already uses semantic tokens, so the work is narrower than the goal implies.

Net effect: two claims were false, three goals were under-assessed, and the corrections made Wave 5 *more* actionable than originally reported. No finding was dismissed.

## ADVERSARIAL REVIEW (round 2)

The revised artifact was re-vetted. Verdict **BLOCK** again — 2 CRITICAL, 2 WARNING — but with **four entirely new findings and zero repeats**, which is convergence rather than stalling: round 1's issues stayed fixed. Same judge/isolation/cross-model posture; artifacts in `review/assess/round2/`.

**Three of four upheld, one rejected on evidence.**

1. **CRITICAL (upheld) — Wave 3 library sub-goals still incomplete.** Recency grouping, running/failed/unread badges, and undo-delete had no rows. All three added as MISSING with specific evidence.
2. **CRITICAL (upheld) — Wave 5 inventory and video-proof unassessed.** Investigating this produced the round's best result: `docs/product-surface-inventory.md` (23 rows, per-screen maturity and required certification) already **is** the 20-screen inventory, video capture is already configured (`tests/bdd/playwright.config.ts:56`), and `screen-by-screen-validation` is specified at 0/5. Wave 5 needs execution, not authoring.
3. **WARNING (upheld) — dev-flag gating asserted as remaining but never assessed.** Checking it found **zero** flags in `admin-shell.tsx`: the Development group (Compiler, A2UI Testing) currently ships to production users. Now a concrete Wave 1 finding.
4. **WARNING (REJECTED) — "coverage.yml is contradicted by the file tree."** Not upheld. `.github/workflows/coverage.yml` exists and line 60 reads `--fail-under-lines 60`, exactly as cited. The judge inferred absence from the packet's `file_tree`, which **excludes dotfile directories** — `.github` appears zero times in it. This is a packet-construction limitation, not an assessment defect, and is recorded here so the next stage does not re-litigate it. It also means the judge cannot see any dotfile-based CI evidence; treat its absence-claims about `.github`, `.storybook`, or `.kbd-orchestrator` as unreliable.

**Chasing finding #1 also surfaced H5 independently** — the largest correction in this document. While verifying the library sub-goals I inventoried `openspec/changes/` properly and found **187 changes**, not the empty directory an earlier draft claimed. That reframed the `migrate-*` question, moved the Storybook VR gate from "missing" to 26/30, and made "reconcile before authoring" the second-highest sequencing priority.

Per the artifact-mode contract, two review rounds have run. Remaining CRITICALs from round 2 were resolved by revision rather than deferred, so no unresolved-findings section is required. The rejected WARNING is documented above with its evidence rather than silently dropped.

ASSESSMENT COMPLETE
