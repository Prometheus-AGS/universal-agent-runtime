# PLAN: uar-uiux-full-migration-2026-08

**Project:** universal-agent-runtime
**Date:** 2026-08-07
**Backend:** OpenSpec (`openspec/` present, `project.json.specSystem = "openspec"`)
**Evolver cycle:** none
**Inputs:** `assessment.md` rev 3, `analysis.md` (post-review), `library-candidates.json`
(12 candidates, schema-valid), `decision-log.md` D1–D3, `handoffs/analyze.handoff.json`

---

## 0. Two open questions closed at plan entry

Analyze deferred both to this stage. Both are now answered, and both **reduce** scope.

### OQ9 — Is `platform/` already present under other names? **PARTIALLY — and the first answer was wrong.**

> **Corrected after adversarial review.** The first draft concluded "`services/` and
> `protocols/` already satisfy the platform contract exactly… a rename plus three file
> moves," and sized C-04 as **S**. That inference was invalid and is retracted.

Measured facts (all confirmed):

| Directory | Files | React imports | JSX files |
|---|---:|---:|---:|
| `services/` | 23 | **0** | **0** |
| `protocols/` | 4 | **0** | **0** |
| `lib/` | 12 | 1 (`db-context.tsx`) | 3 `.tsx` (2 are tests) |
| `stores/` | 45 | 2 | 0 |

**Why "React-free" was not sufficient.** The target (`docs/ui/uar-frontend-migration-plan.md`
§1.2) defines `platform/` as exactly four adapter subdirectories — `pglite/`, `entities/`,
`agui/`, `telemetry/` — and names `protocols/agui-adapter.ts` → `platform/agui/` explicitly.
It **never mentions `services/`**, because `services/` holds 23 per-domain **REST clients**
(`fetchAgentsList` → `fetch("/api/agents")`). Under §1.2 those belong in `features/*/api/`,
not `platform/`. A React-free HTTP client is not an infrastructure adapter. "No React" is a
*necessary* condition I treated as *sufficient*.

**The self-contradiction that settles it.** C-04 as drafted also installed the §6.3
boundary zones, which forbid `features/` importing `platform/` internals. Measured fan-in
to `services/`: **46 importers — 36 from `stores/`, 10 from `entities/`**. Neither exists as
a top-level directory in the target (both migrate into `features/*/model`). So the change
renamed a directory and simultaneously installed the rule making its own call sites illegal.

**Corrected scope:** C-04 now moves only what the target actually specifies
(`protocols/agui-*` → `platform/agui/`, `lib/db.ts` → `platform/pglite/`, new
`platform/entities/`), and **defers the boundary zones** until after the store/entity
migration. `services/` → `features/*/api/` becomes part of per-feature work (C-14).
**Re-sized S → M.**

### OQ-PEM-API — Do the plan §5.2 PEM APIs exist in the vendored build? **YES, all five.**

`createPGlitePersistenceAdapter`, `startLocalFirstGraph`, `registerEntityFromSql`,
`useEntityList`, and `createGraphAction` are all present in
`node_modules/@prometheus-ags/prometheus-entity-management/dist`. The plan's proposal to
delete the hand-rolled outbox in favour of `startLocalFirstGraph` is **viable**. Folded
into **C-07**.

---

## 1. OpenSpec reconciliation (do this before authoring anything new)

187 unarchived changes exist. The analyze handoff flagged writing duplicates as the most
likely failure mode. Disposition of the UI-owning set:

### 1.1 Complete — ARCHIVE, do not re-propose (C-00)

| Change | Tasks |
|---|---|
| `a2ui-uar-renderer-on-webcore` | 49/49 |
| `base-ui-foundation` | 24/24 |
| `a2ui-inspector-lit-svelte-renderers` | 21/21 |
| `a2ui-world-class-theming-a11y-i18n` | 20/20 |

These are the "PRESERVE" surfaces in the analysis matrix. Archiving them makes that
preservation explicit and stops a later agent from re-opening them.

### 1.2 Unstarted and in-scope — ABSORB, do not duplicate

| Existing change | Tasks | Continued as |
|---|---|---|
| `base-ui-composition-patterns` | 0/40 | **C-03b** (own change — 40 tasks is not an absorption) |
| `base-ui-icon-migration` | 0/28 | **C-03c** (own change) |
| `base-ui-verification` | 0/33 | **C-14d** (own change) |
| `migrate-cross-cutting-pages` | 0/31 | **C-10** |
| `retire-a2ui-testing-page-from-prod` | 0/5 | **C-12** |
| `complete-agui-event-parity` | 0/3 | **C-06** |

These already own their scope, and their `tasks.md` is the work list. This phase
**continues** them rather than writing parallel proposals.

> **Corrected after adversarial review.** The first draft "absorbed" 40+28 tasks into C-03
> (rated M) and 33 more into C-14 (rated L). Authored tasks are not free merely because a
> gate ships with an allowlist — the allowlist defers the *violations*, not the tasks. The
> three large ones are now their own changes (C-03b, C-03c, C-14d).

### 1.3 Nearly complete — but its remainder is NOT implementable

`docs-storybook-visual-regression-perf-budget` is at **26/30**. The first draft proposed
"finishing 4 tasks." **All four sit under `## 6. Deferred (see proposal.md "Out of scope")`:**

| Task | Why it cannot be done here |
|---|---|
| 6.1 A2UI Inspector Storybook addon | blocked on Change 22 |
| 6.2 `--primary`/`--muted-foreground` contrast gaps | "design-system decision, not this change's scope" |
| 6.3 `CHROMATIC_PROJECT_TOKEN` provisioning | **operator credential decision** — an account must exist first |
| 7.9 full-workspace `cargo check`/`clippy` | deferred to the phase's consolidated validation pass |

Storybook visual regression itself (tasks 5.1–5.3) is **already `[x]`**. So the CI bundle
budget goal 12 needs is *not* in this change at all — C-13 authors it as new work, and 6.3
is surfaced to the operator as a prerequisite rather than assigned to an agent.

### 1.4 Empty scaffolds — no `tasks.md`

`impeccable-uiux-audit` and `uiux-remediation-wave-1` have no task lists. Treat as
placeholders; the audit intent is discharged by C-13/C-14. No action beyond noting them.

---

## 2. Change list (21 changes, dependency-ordered)

Complexity: **S** < 1h, **M** 1–4h, **L** 4–8h (skilled AI agent).
Model class per `references/model-routing.md`. `library:` refers to
`library-candidates.json` ids.

### Wave 0 — Reconciliation and authority (no code)

| # | Change | What | Complexity | Model | Agent |
|---|---|---|---|---|---|
| **C-00** | `archive-completed-ui-changes` | Archive the 4 complete UI changes (§1.1) via `openspec archive`. **Blocker to clear first: `base-ui-foundation` has ZERO spec deltas** (only `proposal.md` + `tasks.md`), so `openspec validate` fails it — write its delta before archiving. The other three carry 1, 4, and 3 deltas respectively and archive cleanly. Capability: `frontend-component-primitives`. | S | small | OpenCode |
| **C-01** | `amend-goal4-base-ui-divergence` | Amend Goal 4 to name Base UI; add the §6.1/§6.3 divergence to `docs/ui-design-authority.md` citing D1. **The vendored standard header is already amended** — this change makes the goal text match. Spec delta: `frontend-design-authority`. | S | small | Manual + OpenCode |

> **Why first:** every downstream change is judged against Goal 4. Leaving "shadcn" in the
> goals makes 17 downstream changes read as off-spec.

### Wave 1 — Foundation (blocks everything visual)

| # | Change | What | Complexity | Model | Agent |
|---|---|---|---|---|---|
| **C-02** | `tailwind4-css-first-tokens` | Upgrade to `tailwindcss@4.3.3` + `@tailwindcss/vite@4.3.3`; delete `tailwind.config.ts` and `postcss.config.js`; port the KnowMe token ladder into `@theme`. **Explicitly reverses the 2026-07-21 operator decision recorded at `index.css:9`.** Does **not** yet touch the 337 `hsl(var())` call sites. **Must also update 3 dangling references to the deleted config** — `storybook-visual-regression.yml:23,31` path filters and `frontend/components.json:7`; see `execution.md` §4a. | M | frontier | Claude Code |
| **C-03** | `flat2-style-gate` | Add `no-restricted-syntax` Flat 2.0 rule + `unicorn/filename-case` to `frontend/eslint.config.js`, following the existing `check-frontend-boundaries.mjs` harness pattern. Land the gate **with an allowlist for current violations**, so it blocks *new* ones immediately. **Gate only — no component work.** | M | frontier | Claude Code |
| **C-03b** | `base-ui-composition-patterns` | Continue the existing change (**0/40**). Its `tasks.md` is the work list. | L | frontier | Claude Code |
| **C-03c** | `base-ui-icon-migration` | Continue the existing change (**0/28**). | M | medium | Codex |
| **C-04** | `platform-adapter-layer` | Create `platform/` per target §1.2: move `protocols/agui-*` → `platform/agui/`, `lib/db.ts` → `platform/pglite/`, and establish `platform/entities/` as the sole PEM import site. Leave `lib/db-context.tsx` (React) for `shared/`. **`services/` is NOT renamed** — its 23 REST clients belong in `features/*/api/` per §1.2 and move in C-14. **Boundary zones deferred to C-14** (36 stores + 10 entities import `services/`; installing zones now outlaws 46 live call sites). | M | frontier | Claude Code |

> **C-03 before the codemod, not after.** The gate must exist to prevent regression while
> C-05 runs. Landing it with an allowlist is what makes that possible without a flag day.

### Wave 2 — The mechanical sweep

| # | Change | What | Complexity | Model | Agent |
|---|---|---|---|---|---|
| **C-05** | `hsl-var-token-codemod` | Codemod the **30 non-admin** `hsl(var(--x))` → `var(--color-x)` occurrences: `index.css` (15), `enhanced-thread.tsx` (3), `loading-cursor`/`error-bar`/`empty-frame`/`KnowMeLogo` (2 each). **The 307 occurrences inside `admin/pages/*` are deliberately NOT in scope** — C-14 rewrites those files, so codemodding them first is wasted work (see §3 note). Shrink the C-03 allowlist as violations clear. | S | small | OpenCode |
| **C-06** | `agui-event-parity-and-normalizer` | Widen the AG-UI adapter to emit the three target consumers (message chunks, phase timings, event rows) and relocate under `platform/agui/`. Absorbs `complete-agui-event-parity` (0/3). | M | frontier | Claude Code |

### Wave 3 — Data foundation (blocks the headline feature)

| # | Change | What | Complexity | Model | Agent |
|---|---|---|---|---|---|
| **C-07** | `pglite-run-event-persistence` | Add `run` + `run_event` migrations with phase-timing storage. **Coalesce `TEXT_MESSAGE_CONTENT` / `REASONING_MESSAGE_CONTENT` deltas in memory, persist once at `*_END`** (plan §5.1) or a long run writes tens of thousands of rows. Adopt `startLocalFirstGraph` and delete the hand-rolled outbox — **verified available** (§0). | L | frontier | Claude Code |
| **C-08** | `markdown-pipeline-single-renderer` | One renderer at `shared/markdown/markdown-bubble.tsx`. Add `remark-math`, `remark-breaks`, `rehype-raw`, `rehype-sanitize`, `rehype-katex`, `katex`, `dompurify`. **`rehype-raw` and `rehype-sanitize` MUST land in this single change** — A-3 trust boundary. Collapse the second renderer at `admin/pages/skills-page.tsx:84`. Sanitize schema gets unit tests with XSS fixtures. | L | frontier | Claude Code |
| **C-09** | `markdown-lazy-blocks` | Add `mermaid` + `shiki` as **lazy** `import()` chunks (`vendor-mermaid`, `vendor-shiki`), never in the initial graph. Mermaid `securityLevel: "strict"`, finalize-only (never mid-stream). Every block gets an error boundary + source fallback. Specify the KaTeX CSS/font strategy. | M | frontier | Claude Code |

> **C-08 blocks C-09 and C-11.** Every chunk renderer depends on the markdown pipeline
> (plan §13). C-07 blocks C-11 — there is no trace to render without an event record.

### Wave 4 — Surfaces

| # | Change | What | Complexity | Model | Agent |
|---|---|---|---|---|---|
| **C-10** | `app-shell-and-navigation` | Nav rail (240/60px) + mobile tab bar + `nav-destinations.ts` + sheet host + breadcrumb header, one responsive tree. **Resolves cand-011 first:** decide `cmdk` vs a Base UI palette equivalent — keeping `cmdk` means a mixed-primitive shell after D1. Copy `docs/ui/logo/` → `frontend/public/brand/`. Absorbs `migrate-cross-cutting-pages` (0/31). | L | frontier | Claude Code |
| **C-11** | `run-trace-and-inspector` | Trace bar, hierarchical timeline, inspector, replay from the C-07 event record. **Resolves cand-010 first:** pick a virtualizer for a *tree* past ~200 rows against the ≤100ms/500-event budget. Wire the two unconsumed endpoints — `/runs/{id}/checkpoints`, `/runs/{id}/resume` — plus A2UI `surface-replay`. | L | frontier | Claude Code |
| **C-12** | `chunk-catalog-renderers` | The complete plan §8 chunk catalog on the shared `ContentBlock` protocol, with compile-time exhaustiveness. **Divider renders as a spacer `<div role="separator">`, never `<hr>`** (standard §3.2). Absorbs `retire-a2ui-testing-page-from-prod` (0/5). | L | frontier | Claude Code |

### Wave 5 — Migration and certification

| # | Change | What | Complexity | Model | Agent |
|---|---|---|---|---|---|
| **C-13** | `ci-bundle-and-perf-budget` | **Author the CI bundle budget as new work** (initial JS ≤250 KB gzip excluding PGlite/mermaid/shiki chunks) plus the latency budgets from goal 12. **NOT "finish 4 tasks"** — see §1.3 correction: all 4 open items on `docs-storybook-visual-regression-perf-budget` are in its `## 6. Deferred (out of scope)` section (blocked on another change, a design decision, an operator credential, and a deferred validation run). Storybook VR tasks 5.1–5.3 are already `[x]`. **Task 6.3 (`CHROMATIC_PROJECT_TOKEN`) is an operator prerequisite** — surfaced, not assigned to an agent. | M | medium | Codex |
| **C-14a** | `admin-pages-to-features` | Re-home the 13 admin pages into `features/*`, **one commit per page**, behaviour-preserving. Moves each page's `services/*-api.ts` into `features/*/api/` per target §1.2. Absorbs the **307** `hsl(var())` occurrences in these files (models-page 105, memory-page 103, cost-dashboard 39, skills-page 33, compiler-page 27) as part of each page's rewrite — not as a separate codemod pass. | L | frontier | Claude Code |
| **C-14b** | `settings-page-decomposition` | Split `settings-page.tsx` (**3,336 lines**) by domain; no page > ~600 lines. | L | frontier | Claude Code |
| **C-14c** | `retire-admin-and-legacy-deps` | Delete `src/admin/`, the `[data-admin-theme="terminal"]` CRT block (`index.css:222-228`, `admin-page.tsx:54-57`), the TanStack Query provider (`App.tsx:2`) + dependency, and **`highlight.js@^11.10.0`** (live at `enhanced-markdown-text.tsx:9`; superseded by shiki in C-09). Prune the 27 `@radix-ui/*` declarations **only after a transitive check** — `cmdk` pulls 4 Radix deps, `@assistant-ui/react` pulls 9. Name and delete the retired stores (goal 11). **Install the deferred §6.3 boundary zones here**, once `services/` has moved. | M | frontier | Claude Code |
| **C-14d** | `base-ui-verification` | Continue the existing change (**0/33** — verified; an earlier review claim of 0/37 was wrong). | L | frontier | Claude Code |
| **C-15** | `a11y-and-responsive-certification` | WCAG 2.2 AA: keyboard-only flows, landmarks, live regions, reduced motion, axe. Responsive sweep at 320/768/1024/1440 in both themes. **Status never by colour alone** (standard §3.3). 3px ember focus ring everywhere. Run the §12 acceptance checklist. | L | frontier | Claude Code + a11y-gate skill |

---

## 3. Dependency graph

```
C-00 ─┐
C-01 ─┴─> C-02 ─> C-03 ─> C-05 (30 non-admin only)
                    │
                    ├─> C-03b, C-03c ────────────┐
              C-04 ─┤                            │
                    │                            ├─> C-14a ─> C-14b ─> C-14c ─> C-14d ─> C-15
       C-06 ─> C-07 ┼─> C-11 ─┐                  │
                    │         ├─> C-12 ──────────┤
       C-08 ─> C-09 ┴─> C-10 ─┘                  │
                                                 │
       C-13 (independent) ───────────────────────┘
```

**Hard orderings**
- C-03 **before** C-05 — gate before codemod, or violations re-accumulate.
- C-08 **before** C-09 and C-12 — every chunk renderer needs the markdown pipeline.
- C-07 **before** C-11 — no event record, no trace to render.
- C-04 **before** C-14a — pages move their API clients into `features/*/api/`, which needs
  the platform split settled first.
- C-14a → C-14b → C-14c → C-14d — strictly sequential; all touch the same tree, and C-14c's
  deletions and boundary zones are only safe once C-14a/b have moved everything out.
- C-09 **before** C-14c — `highlight.js` cannot be removed until shiki replaces it.
- C-15 **last** — certification gates the phase.

**C-05 / C-14a overlap — resolved.** 307 of the 337 `hsl(var())` occurrences live in five
`admin/pages/*.tsx` files that C-14a rewrites and C-14c deletes. Codemodding them in Wave 2
would be work thrown away in Wave 5. **C-05 is therefore scoped to the 30 non-admin
occurrences only**; the other 307 are handled inside each page's rewrite. The first draft
had no edge between these changes and would have paid for the same work twice.

**Parallelisable:** C-04 ∥ C-02/C-03; C-06/C-07/C-08 ∥ C-05; C-13 is fully independent
(it touches Storybook/CI config, not lint-gated source — the first draft's "after C-03"
dependency was fabricated). Concurrent agents use separate worktrees under
`~/.claude/worktrees/` per CLAUDE.md.

---

## 4. Library annotations

| Change | Candidate | Verdict |
|---|---|---|
| C-02 | `cand-002` tailwindcss + @tailwindcss/vite | adopt |
| C-03 | *build_required* — Flat 2.0 ESLint gate | build |
| C-03b, C-03c, C-14d | `cand-001` @base-ui/react | adopt (**divergence from standard §6.1/§6.3 per D1**) |
| C-07 | `cand-006` @electric-sql/pglite, `cand-007` PEM | adopt (schema-only; no dep change) |
| C-08 | `cand-003` rehype-sanitize, `cand-004` remark-math/rehype-katex/katex/remark-breaks/rehype-raw | adopt |
| C-09 | `cand-005` mermaid + shiki | adopt |
| C-10 | `cand-011` cmdk | **reference — resolve in-change** |
| C-11 | `cand-010` virtualization | **reference — resolve in-change** |
| C-12 | `cand-012` recharts | **reference — resolve in-change** |
| C-14c | `cand-008` @tanstack/react-query | reject (remove) |
| C-06 | `cand-009` @assistant-ui/react | adopt (held at 0.14.26) |

**Three `reference` candidates are not decisions.** C-10, C-11, and C-12 each open with a
library selection task. Do not treat them as settled adoptions.

---

## 5. Scope honesty

**What this phase is not.** Despite the goal wording "discard and rebuild completely,"
per-surface scoping (D2) plus the OQ9 finding means the real shape is:

- **Partial extraction, not a rename:** `platform/` takes `agui` + `pglite` + `entities`
  (C-04, M). `services/` is **not** platform code and migrates into `features/*/api/`
  with its pages.
- **Preserve:** the A2UI renderer, theming, and inspector — complete work, archived not redone.
- **Rebuild:** shell, chat surface, run trace, chunk catalog — no foundation to extend.
- **Rewrite in place:** the 13 admin pages, page by page, absorbing 307 token rewrites.

**Known risks carried forward**

| Risk | Mitigation |
|---|---|
| Coverage is already red (19.45% vs 60%) and the C-14 family touches 10,920 lines | One commit per page; C-13 lands the CI budget independently and early |
| Bundle already warns >1100 kB before adding mermaid/shiki | C-09 enforces lazy chunks; C-13 authors the CI budget |
| `admin/` rewrite loses undocumented behaviour | Behaviour-preserving per-page commits, diffable individually |
| Radix prune breaks `cmdk` (4 Radix deps) / assistant-ui (9) / storybook | Transitive check gated inside C-14c |
| **C-14 family is 4 sequential changes on one tree** — a stall in C-14a blocks three others | Strict ordering declared in §3; do not parallelise the C-14 chain |
| **C-13 depends on an operator credential** (`CHROMATIC_PROJECT_TOKEN`) | Surfaced as a prerequisite in §6, not assigned to an agent |
| 630 border idioms | C-03 allowlist shrinks as C-05 and C-14a clear them; matcher published below |

> **Reproducibility note:** the "630 border idioms" figure comes from
> `grep -rEo 'border(-[a-z0-9/-]+)?' frontend/src --include='*.tsx' | wc -l`. An
> independent reviewer's variant returned 546. The exact matcher is published here so
> C-03's allowlist and C-05's completion criterion use **one** definition. Progress is
> measured by that command, not by the number.

**No cost or duration estimate exists.** Complexity letters are relative sizing for agent
sessions, not schedule. Flagged in analyze and still true.

### 5.1 Vertical-slice exception (protocol deviation, declared)

The KBD plan protocol says *"one change = one vertical slice… never create purely
horizontal changes."* **Five of twenty-one changes are horizontal and this is deliberate:**

| Change | Why horizontal is correct here |
|---|---|
| C-00, C-01 | Bookkeeping and authority — no code, no slice to cut |
| C-03 | A lint gate is inherently cross-cutting; slicing it per feature would mean N partial gates |
| C-04 | A layer boundary cannot be created one feature at a time without an intermediate state that violates it |
| C-05 | A 30-occurrence token codemod; slicing costs more than it yields |

The one horizontal change that *could* have been vertical — the original C-05 covering all
337 occurrences — **has been restructured**: its 307 admin-page occurrences now ride inside
C-14a's per-page slices, which is both vertical and avoids duplicated work.

### 5.2 Spec-delta capabilities (named up front, per CLAUDE.md)

CLAUDE.md warns that `openspec validate` fails a change with zero deltas, and that
CI-only/tooling changes are the hard cases. Named now rather than discovered later:

| Change | Capability |
|---|---|
| C-00, C-03b, C-03c, C-14d | `frontend-component-primitives` |
| C-01 | `frontend-design-authority` (**new capability — does not yet exist in `openspec/specs/`**; C-01 creates it, along with `docs/ui-design-authority.md`) |
| C-02, C-03, C-05 | `frontend-design-system` |
| C-04, C-14c | `frontend-architecture-boundaries` |
| C-06 | `ag-ui-chat-conformance` (existing) |
| C-07 | `frontend-local-first-persistence` |
| C-08, C-09, C-12 | `frontend-content-rendering` |
| C-10 | `frontend-app-shell` |
| C-11 | `runtime-console` (existing) |
| C-13 | `frontend-build-tooling` (existing — CLAUDE.md names it for exactly this case) |
| C-14a, C-14b | `frontend-configuration-surfaces` |
| C-15 | `frontend-accessibility-certification` |

---

## 6. Next action

```
openspec archive a2ui-uar-renderer-on-webcore
```

Then the remaining C-00 archives — **but write `base-ui-foundation`'s missing spec delta
first**, or its archive fails validation. Then `/opsx:new amend-goal4-base-ui-divergence`.

**Operator prerequisite (not agent-assignable):** `CHROMATIC_PROJECT_TOKEN` provisioning
requires a Chromatic account/project to exist. C-13's visual-regression half is blocked
until that is done; its bundle-budget half is not.

---

## 7. Adversarial review record

Reviewed 2026-08-07 by an isolated fresh-context critic receiving only the artifact and its
declared inputs (E-2 artifact-only isolation). liter-llm judge unavailable (HTTP 401), so
the review ran harness-native. Producer: `claude-opus-5`.

**Verdict on the first draft: NOT fit to emit. Four CRITICAL findings; three upheld in
full, one upheld in part.** Because the next step mints durable OpenSpec artifacts, each
would have become a permanent error.

| # | Finding | Verified | Fix |
|---|---|---|---|
| C1 | **OQ9's conclusion was invalid.** "No React imports" is necessary, not sufficient. Target §1.2 defines `platform/` as `pglite/entities/agui/telemetry/` and never mentions `services/` — 23 REST clients belong in `features/*/api/`. C-04 also installed boundary zones that would outlaw **46** of its own call sites (36 stores + 10 entities). | confirmed | §0 retracted and rewritten; C-04 re-scoped, **S → M**; zones deferred to C-14c |
| C2 | **337 `hsl(var())`, not 237** — I reported `grep -c` *line* counts as occurrence counts. And **307 of 337 sit in admin pages that C-14 rewrites**, with no dependency edge — Wave 2 work destroyed in Wave 5. | confirmed: 337 | C-05 scoped to the 30 non-admin occurrences (**M → S**); 307 folded into C-14a; edge documented |
| C3 | **C-14 and C-03 were multi-session changes.** C-14 = 33 absorbed tasks + 13 page commits + 5 deletion workstreams + 307 rewrites. C-03 = 68 absorbed tasks rated M. | confirmed | C-14 split into **C-14a/b/c/d**; C-03 split into **C-03/C-03b/C-03c** |
| C4 | **C-13's premise was false.** All 4 remaining tasks are under `## 6. Deferred (out of scope)` — blocked on another change, a design decision, an operator credential, a deferred validation run. Storybook VR (5.1–5.3) is already `[x]`. | confirmed | C-13 re-scoped to **author** the bundle budget; 6.3 surfaced as an operator prerequisite |
| W5 | `highlight.js` (live at `enhanced-markdown-text.tsx:9`) and "retired stores" — both named in goal 11, both absent from the plan | confirmed | added to C-14c, gated behind C-09 |
| W6 | "Every change carries a delta" was asserted, unverified — and **`base-ui-foundation` has zero deltas**, so C-00's first archive would fail validation | confirmed | §5.2 names a capability per change; C-00 carries the blocker |
| W7 | Vertical-slice protocol violated without declaring an exception | confirmed | §5.1 declares it explicitly |
| N8 | 187 vs 188 changes; `lib/` has 3 `.tsx` not 1; C-13's "after C-03" dependency fabricated | 187 correct (critic counted `archive/`); other two confirmed | corrected |

**One finding rejected.** The critic reported `base-ui-verification` as 0/37 and called the
plan's 0/33 wrong. Re-measured: `grep -c '^- \[' openspec/changes/base-ui-verification/tasks.md`
→ **33**. The plan was correct. (The same critic made this identical false claim during the
analysis review; it was rejected then too.)

**Delta, stated plainly.** Two of these are the same failure in different clothes: **I drew
a strong conclusion from a partial measurement, then built sizing on it.** OQ9 measured
"React imports" and concluded "satisfies the platform contract." C-05 measured lines and
reported occurrences. Both shrank scope, both were wrong in the direction that made the
plan look easier, and neither was re-verified — even though §5 was titled "Scope honesty"
and sat two paragraphs above a note correctly insisting that a disputed grep be published
rather than asserted. The discipline was available and applied unevenly.

The corrected plan is **21 changes, not 15**, and its two "cheap" foundation changes
(C-04, C-05) moved in opposite directions once measured properly.
