# ANALYSIS: uar-uiux-full-migration-2026-08

**Date:** 2026-08-07
**Mode:** stack-specified (the stack is fixed by `docs/ui/uar-frontend-migration-plan.md` §0, amended by operator decision D1)
**Inputs:** `assessment.md` rev 3, `handoffs/assess.handoff.json`, operator decisions D1–D3
**Research budget:** tiers 1–3 run, tier 4 not needed, within cap.

---

## 1. What analyze was actually for

The assessment ended with two questions that made the change list unwritable, and a third
that made the phase unexecutable by any agent working only in this repo. All three are now
settled by operator decision, so this stage's job narrowed to: **verify the target stack is
real, current, and compatible**, and **turn "discard and rebuild" into a per-surface
disposition** (D2).

The three decisions, recorded in `decision-log.md`:

| ID | Decision | Effect on this stage |
|---|---|---|
| **D1** | Keep Base UI; amend Goal 4's "shadcn" | Removed the only contested library choice. No stack scoring needed. |
| **D2** | Per-surface scoping, not literal greenfield | Made §4's matrix the primary deliverable. |
| **D3** | Copy the KnowMe standard into UAR `docs/` | **Executed** — see §2. |

---

## 2. Design authority is now resolvable in-repo (D3 — done)

`docs/knowme-ui-ux-standard.md` now exists in this repository: a vendored copy of
`hybrid-mobile-architecture-src/docs/knowme-ui-ux-standard.md` (510 source lines) behind a
provenance header. The **source** SHA-256 is `b24cff202750…`, recorded in the header so
upstream drift is detectable; the vendored file itself hashes differently because of the
header.

The header records source, source SHA, vendoring date, the governing KBD decision, a
refresh instruction, and **two recorded divergences**:

1. **Brand.** Where the standard specifies product identity (§4.3's K monogram), UAR's own
   assets in `docs/ui/logo/` (Slash Gate) take precedence.
2. **Component primitives.** §6.1 and §6.3 name Shadcn UI as the required owner of general
   controls, navigation, overlays and sidebars. UAR uses **Base UI** instead, per operator
   decision D1. **This is an override of the standard, not compliance with it.** §6.3's
   other ownership rows (assistant-ui, PEM, Zustand, PGlite) apply unchanged.

Everything else applies verbatim: Flat 2.0 (§3), token ladder (§4.1), typography (§4.2),
the remaining §6.3 boundaries, and acceptance criteria (§12).

This resolves the assessment's §0 gap and open question 5 (brand). Divergence 2 was added
after adversarial review caught that the original header falsely claimed §6.3 applied
verbatim while the repo had already left shadcn — see §3.1.

---

## 3. Library landscape — findings

Full machine contract in `library-candidates.json`. The narrative:

### 3.1 Base UI is technically adequate for Flat 2.0 — and is a recorded DIVERGENCE from the standard

> **This section was rewritten after adversarial review refuted its original claim.** The
> first version argued Base UI was "materially better than shadcn" for Flat 2.0. That
> argument was unsound and is retracted. What follows is the defensible version.

**The standard requires shadcn.** Not ambiguously — normatively, in the two sections that
actually govern library choice:

> §6.1: "**Shadcn UI** is the default component vocabulary for buttons, inputs, menus,
> dialogs, sheets, sidebars, tabs, tooltips, scroll areas, command palettes, and accessible
> primitives."

> §6.3 (line 227): "General React controls, navigation, overlays, and sidebars →
> **Shadcn UI, restyled with KnowMe tokens**" — under the heading "These libraries are
> complementary and **must not be treated as interchangeable choices**."

The original draft quoted §3.3 ("No Shadcn defaults may be accepted without removing their
border-based styling") as if it were a warning against shadcn. It is not: it sits under a
heading titled **"Prohibited treatment"** and is a *styling* rule. §6.1 disambiguates it
directly — "Shadcn and Assistant UI are **starting points, not permission to ship their
default appearance**." That is an instruction to restyle shadcn, not to replace it.

**The "nothing to strip" argument was a non-sequitur.** shadcn does not ship CSS as a
dependency; it copies component source *into the repo*, which you then own and edit. Its
stripping cost is a one-time deletion of `border` classes from files you control. Base UI
ships no styles at all — meaning every Flat 2.0 surface must be authored from scratch,
which is plausibly *more* work, not less. **No effort comparison was performed, so no
superiority claim is supported in either direction.**

**The honest position:** D1 is an **operator override** of the standard's §6.1/§6.3
requirement. It is a legitimate decision — Base UI is unstyled, actively maintained, built
by the teams behind Radix/Floating UI/MUI, its data-attribute state model (`data-checked`,
`data-highlighted`, `data-popup-open`) maps cleanly onto the standard's §3.2 "state by fill"
rule, and `base-ui-foundation` is already 24/24 complete. It is technically adequate. But
it is a **divergence to be recorded, not compliance to be claimed.**

**Consequent action:** the vendored standard's provenance header currently asserts that
"§6.3 ownership boundaries … apply verbatim." That is now false. The header must be amended
to except §6.1/§6.3's shadcn requirement, citing D1. *(Done — see §2.)*

**Radix residue is not a clean prune.** 27 `@radix-ui/*` packages are declared with zero
*direct* `src` imports, but `cmdk`, `@assistant-ui/react`, `@assistant-ui/react-markdown`,
and `storybook` all depend on Radix **transitively**. Removing the declarations does not
remove Radix from the bundle. This needs a dependency-graph check, not a declaration sweep —
and it raises a live design question: keeping `cmdk` for the command palette (goal 9) means
shipping a mixed-primitive shell after D1 moved primitives to Base UI. See cand-011.

### 3.2 Tailwind 4 is mechanical but NOT small

> **Corrected after adversarial review.** The first version claimed "15 `hsl(var())`
> occurrences" and concluded this was "a small, mechanical migration." The real count is
> **237 across 12 files** — a 16× undercount. The original grep covered `index.css` only.

Measured surface: **3** `@apply` directives (index.css:290, 300, 326), a **121**-line
`tailwind.config.ts`, a **429**-line `index.css`, and **237** `hsl(var())` occurrences
distributed as:

| File | Count |
|---|---|
| `admin/pages/models-page.tsx` | 60 |
| `admin/pages/memory-page.tsx` | 56 |
| `tailwind.config.ts` (being deleted) | 36 |
| `admin/pages/cost-dashboard-page.tsx` | 31 |
| `index.css` | 15 |
| `admin/pages/skills-page.tsx` | 15 |
| `admin/pages/compiler-page.tsx` | 13 |
| 5 further files | 11 |

**Why this matters beyond bookkeeping:** roughly **175** of those occurrences sit inside
the 13 admin pages that §4 dispositions as *"PORT, NOT REWRITE."* A port that must rewrite
175 inline colour expressions is substantially more than a relocation. The two sections
contradicted each other in the first draft; §4 has been corrected accordingly.

The migration remains *mechanical* — every occurrence is a find-and-replace from
`hsl(var(--x))` to `var(--color-x)` — but it is a large mechanical job that should be
codemodded, not hand-edited, and it is the dominant cost in the token workstream.

`@tailwindcss/vite@4.3.3` is published and version-matched to `tailwindcss@4.3.3`.

Tailwind 4 compiles `@theme` variables into real CSS custom properties on `:root` — which
is precisely the token model the standard §4.1 specifies (complete colors, not channel
triplets). The v3 `hsl(var(--token))` indirection disappears as a side effect of the
migration rather than as separate work.

One honest note: `index.css:9` records a **deliberate prior operator decision** to keep the
Tailwind 3 channel mechanism. This phase reverses that decision. It should be reversed
explicitly in the change record, not silently overwritten.

### 3.3 The markdown pipeline is the largest net-new block — and it opens a trust boundary

Eight of nine required packages are **absent**: `remark-math`, `rehype-katex`,
`rehype-sanitize`, `rehype-raw`, `mermaid`, `shiki`, `katex`, `dompurify`. Only
`remark-gfm` is installed. The current renderer passes **no rehype chain at all**
(`enhanced-markdown-text.tsx:72`).

The security shape needs stating precisely, because it is easy to get backwards:

- **There is no current vulnerability.** Without `rehype-raw`, react-markdown escapes raw
  HTML by default. Today's pipeline is safe by omission.
- **The target deliberately removes that protection.** Plan §7.2 enables `rehype-raw` so
  agents can emit inline SVG.
- Therefore `rehype-sanitize` is a **hard prerequisite**, not a companion nicety. Under
  A-3 this is a real trust boundary — model output becomes executable markup — and the two
  packages **must land in the same change**. Splitting them across changes would ship a
  window in which agent output can inject script.

**Both** renderer sites were verified, not just one: `enhanced-markdown-text.tsx:71` and
`admin/pages/skills-page.tsx:84` each pass `remarkPlugins={[remarkGfm]}` with no rehype
chain, so both escape raw HTML today. The second one sits inside the admin surface and must
be named explicitly as a migration target, or the one-renderer contract silently fails.

The one `dangerouslySetInnerHTML` in the tree (`components/ui/chart.tsx:96`) injects CSS
custom properties (`--color-${key}: ${color}`) from a developer-authored `ChartConfig`. It
is benign **as configured**. The invariant the plan stage must enforce is precise:
`ChartConfig.color` and `.theme` must never accept model or user input.

### 3.4 Settled incumbents

PGlite is already at latest (0.5.4) with the `live` extension present — the run/run_event
gap is **schema, not tooling** (cand-006). PEM stays on `workspace:*`: it is a vendored
submodule, so the plan's "pin an exact `next` version" risk row does not apply to this
repo's resolution model — pinning to npm would be a *regression* (cand-007). assistant-ui
stays at 0.14.26; the 0.15 bump is low-risk but not a prerequisite, so it sequences
separately (cand-009).

The Flat 2.0 gate is **BUILD, not adopt**: the plan hands over the ESLint rule verbatim,
and `scripts/check-frontend-boundaries.mjs` (wired at **repo-root** `package.json:22`)
already proves the gate-harness pattern in-repo.

### 3.5 Three gaps this stage did NOT research — carried as `reference`, not decisions

> **Added after adversarial review.** The first draft asserted "nothing else needs
> adopting." That was wrong: three target requirements have no researched candidate.

| Requirement | Source | Status |
|---|---|---|
| **Timeline virtualization** | Plan §8.5 requires the trace timeline virtualized past ~200 rows; goal 12 budgets a 500-event render at ≤100ms | **Nothing installed.** `@tanstack/react-table` is present but is not a virtualizer. `@tanstack/react-virtual` is the obvious candidate (cand-010) but was not researched to tier-2/3 depth. The timeline is a *tree*, not a flat list, which constrains the choice. |
| **Command palette** | Goal 9 + plan §6.1 | `cmdk` installed and Radix-dependent (cand-011). Never compared against a Base UI equivalent — and keeping it means a mixed-primitive shell after D1. |
| **Chart block** | Plan §8.3 `chart-block.tsx` | `recharts@^3.10.1` installed, used in 2 files, **no keep/replace verdict reached** (cand-012). |

All three are marked `verdict: reference` in `library-candidates.json` — explicitly *not*
adoption decisions. **The plan stage must resolve them before building the trace timeline
or the shell.** This is a real gap in this analyze stage, not a deferral by design.

---

## 4. Per-surface disposition (D2)

The operator rejected literal greenfield. This matrix replaces the single global verdict.
Dispositions are driven by one question: **is there a foundation to extend, and would
discarding it destroy completed work?**

| Surface | Disposition | Basis |
|---|---|---|
| **Design tokens / theme** | **REPLACE** | Tailwind 3→4 is a mechanical rewrite of a 121-line config into `@theme`. Small, well-understood, and a prerequisite for everything visual. |
| **Component primitives** | **CONTINUE** | Migration already underway and correct (§3.1). Finish `base-ui-{composition-patterns,icon-migration,verification}` (0/40, 0/28, 0/33); prune 27 Radix packages. |
| **A2UI renderer / theming / inspector** | **PRESERVE** | `a2ui-uar-renderer-on-webcore` **49/49**, `a2ui-world-class-theming-a11y-i18n` **20/20**, `a2ui-inspector-lit-svelte-renderers` **21/21**. Complete. Greenfield would destroy this for no gain. |
| **Markdown pipeline** | **BUILD NEW** | 8 of 9 deps absent; two renderers must collapse to one. Effectively net-new (§3.3). |
| **Run persistence (run/run_event)** | **BUILD NEW** | No table exists. Critical path for the trace bar, timeline, inspector, and replay. |
| **Run transport (AG-UI)** | **EXTEND** | Adapter + schema exist with tests; `/stream`, `/cancel`, `/tool-approval` already wired. Widen to feed phase timings and persisted rows; relocate to `platform/agui/`. |
| **Shell / navigation** | **REPLACE** | 4 routes today vs a rail + tab bar + palette + sheet host. No foundation to extend. |
| **Chat surface** | **REBUILD ON EXISTING SPINE** | `chat-stream-store.ts` (1,200 lines) assembles in memory with no event record. Rebuild the store against run_event; keep the working transport beneath it. |
| **Configuration surfaces (13 admin pages)** | **REWRITE PER-PAGE, BEHAVIOUR-PRESERVING** | 10,920 lines encoding real behaviour against ~40 endpoints. Goal 11 names `src/admin/` as an explicit **removal target**, so these do not survive as-is. But ~175 of the 237 `hsl(var())` occurrences live here and each page must be re-homed into `features/*` — this is a rewrite, not a relocation. Sequence **one change per page** so behaviour can be diffed page by page rather than lost in a bulk sweep. |
| **Flat 2.0 purge + gate** | **BUILD GATE, THEN CODEMOD** | ~630 border idioms (see §5 note on reproducibility), stable — not regressing. Gate must land with or before the purge. |
| **Removal targets (goal 11)** | **DELETE** | TanStack Query (1 dead call site, cand-008), the `[data-admin-theme="terminal"]` CRT block (`index.css:222-228` + `admin-page.tsx:54-57`), `tailwind.config.ts`, `postcss.config.js`, and the 27 `@radix-ui/*` declarations — the last **only after** a transitive-dependency check (§3.1). `highlight.js` is not installed, so nothing to remove. |
| **Chunk catalog (goal 10, plan §8)** | **BUILD NEW** | The largest single section of the plan and it had no row in the first draft. Depends on both the markdown pipeline and run-event persistence, so it sequences after both. |
| **Exit gates (goal 12)** | **BUILD NEW, ASSIGN OWNER** | WCAG 2.2 AA certification, responsive sweep at 320/768/1024/1440 in both themes, CI bundle/latency budgets. None exist. `docs-storybook-visual-regression-perf-budget` (26/30) is the nearest in-flight vehicle and should be finished rather than duplicated. |
| **Layering (`platform/`, `shared/`)** | **UNRESOLVED — but see §5** | Neither `platform/` nor `shared/` exists at all (0 files each). Two of the four target layers are absent outright; the open question is whether `services/`+`stores/`+`protocols/`+`lib/` already discharge their *function*. |

**Consequence for the change list:** this phase is *not* one big rebuild. It is
roughly four workstreams — foundation (tokens, gate, primitives), data (run persistence,
markdown), surfaces (shell, chat), and migration (admin pages) — with hard ordering
between the first two and the rest.

---

## 5. What analyze did NOT resolve

Stated plainly, because the plan stage must not inherit these as settled:

1. **OQ9 — is `platform/` already present under other names?** Partially answered here,
   and the answer is sharper than the assessment's: **`platform/` and `shared/` contain
   zero files each — two of the four target layers do not exist at all.** What exists is
   `services/` (23), `stores/` (45), `protocols/` (4), `lib/` (12), `features/` (28),
   `app/` (2). So the question is not "does `platform/` exist under another name" but
   "do `services/`+`protocols/`+`lib/` already discharge `platform/`'s *function* — being
   the React-free infrastructure adapter layer?" The boundary gate passing is evidence they
   are at least well-separated. Tracing that equivalence is code work, not library
   research, so it remains out of scope for this stage — but it is now a narrower question
   and should be the plan stage's first task.

2. **OQ-PEM-API — do the PEM APIs the plan names actually exist?** Plan §5.2 proposes
   deleting the hand-rolled outbox in favor of `createPGlitePersistenceAdapter`,
   `startLocalFirstGraph`, and `registerEntityFromSql`. These must be verified present in
   the *vendored* PEM before any change depends on them. If absent, the outbox stays.

3. **Cost and duration.** No estimate exists for any workstream, in any unit. Round 2 of
   the assessment review flagged this and it remains true.

4. **Design-comp decomposition.** 3,373 lines of binding visual authority across the three
   goal-named comps have not been decomposed into a surface/component inventory. The plan
   stage cannot size the shell or chat work without it.

---

## 6. Recommendations into plan

1. **Amend Goal 4** to name Base UI, citing D1. Leaving "shadcn" in the goals makes every
   downstream change read as off-spec.
2. **Sequence foundation first**: Tailwind 4 + tokens, then the Flat 2.0 gate, then the
   codemod. The gate before the purge, or the purge re-accumulates.
3. **Keep `rehype-raw` and `rehype-sanitize` in one change.** Non-negotiable under A-3.
4. **Answer OQ9 before writing layering changes.** It is the difference between a rename
   and a restructure, and it is cheap to answer.
5. **Reconcile the ~49 UI-owning OpenSpec changes before authoring new ones.** Four are at
   100%; several more are >80%. The prior phase named duplicate-proposal-writing as its
   single most likely failure mode.
6. **One change per admin page**, behaviour-preserving, so each can be diffed and verified
   individually — and budget for the ~175 `hsl(var())` rewrites concentrated there.
7. **Resolve cand-010/011/012 (virtualization, palette, charts) before building the trace
   timeline or shell.** They are `reference`, not decisions.
8. **Run the schema validator on every machine artifact**, in the stage that writes it.

---

## 7. Adversarial review record

Reviewed 2026-08-07 by an isolated fresh-context critic receiving only the artifacts plus
their declared inputs (E-2 artifact-only isolation). The liter-llm judge was unavailable
(HTTP 401 at the gateway), so the review ran harness-native: `isolation_mode =
harness-native`. Producer: `claude-opus-5`.

**Verdict on the first draft: NOT fit for the plan stage. Three CRITICAL findings, all
independently re-verified by the author and all upheld.**

| # | Finding | Verified | Fix |
|---|---|---|---|
| C1 | §3.1's "Base UI is materially better than shadcn" was refuted by the standard itself — §6.1 and §6.3 (line 227) name **Shadcn UI as the required owner**. The draft quoted §3.3, a *styling* prohibition, as if it governed library choice, and cited §6.3 approvingly three times elsewhere while passing over it here. Shape of S-04 self-rationalization: the migration had already landed, and the draft argued the existing outcome was optimal. | confirmed at `docs/knowme-ui-ux-standard.md:227` | §3.1 rewritten: D1 recorded as an **operator override / divergence**, superiority claim retracted, vendored header amended |
| C2 | `library-candidates.json` failed its schema **102 ways** — wrong root keys, no `id` on any candidate, invalid `verdict` values. The schema states plan.md references candidates as `library: cand-###`; **no candidate had an id**, so the plan stage could not reference any of them. | confirmed against `references/schemas/library-candidates.schema.json` | Rewritten to schema; **validator run: 0 errors**, 12 addressable ids |
| C3 | "15 `hsl(var())` occurrences" undercounted by **16×** — actual **237 across 12 files**. Falsified §3.2's "small, mechanical migration" headline, and ~175 of them sit in the admin pages the matrix called "PORT, NOT REWRITE" — a direct self-contradiction. | confirmed: 237 | §3.2 rewritten with per-file table; admin disposition corrected to REWRITE PER-PAGE |
| W4 | Matrix omitted removal targets, exit gates, and the chunk catalog; "PORT" contradicted goal 11, which names `src/admin/` a removal target | confirmed | 3 rows added, admin row corrected |
| W5 | Radix prune unsafe as stated — `cmdk`, `@assistant-ui/react`, `storybook` depend on Radix **transitively** | confirmed (`cmdk@^1.1.1`) | §3.1 + cand-001 corrected |
| W6 | Virtualization, command palette, and charts were never researched, yet §3.4 asserted "nothing else needs adopting" | confirmed | new §3.5; cand-010/011/012 added as `reference` |
| W7 | Security section verified only one of the two renderers | confirmed both safe | §3.3 now covers both |
| N8 | OQ9 deferral legitimate, but under-reported available evidence | confirmed: `platform/` and `shared/` are **0 files each** | §5 sharpened |
| N9 | SHA misattribution, `enhanced-markdown-text.tsx:72`→`:71`, `package.json:22` is repo-root not frontend | confirmed | corrected throughout |

**One finding rejected.** The critic reported `base-ui-verification` as 0/37, calling the
document's 0/33 wrong. Re-measured: `grep -c '^- \[' openspec/changes/base-ui-verification/tasks.md`
returns **33**. The document was correct; no change made.

**Delta, stated plainly.** Two failures here are worth naming because they are process
failures, not just factual ones:

1. **I wrote a machine contract without reading its schema.** The plan stage consumes
   `library-candidates.json` by candidate id, and I invented a different JSON shape
   entirely. This was mechanically checkable in one command and no earlier round caught it,
   because no round ran the validator. Any stage that emits a schema-governed artifact must
   validate it before writing the handoff.
2. **I argued from the authority selectively.** §6.3 was cited three times to justify
   keeping assistant-ui and PEM, then passed over where it contradicted D1. The correct
   move was always available and is what the document now does: record the divergence
   honestly. An override does not need to be dressed as compliance.

The corrected position is narrower and more useful: **Base UI is adequate and D1 stands,
but it is a divergence from the binding standard; the Tailwind migration is large, not
small; and three target requirements still have no researched candidate.**
