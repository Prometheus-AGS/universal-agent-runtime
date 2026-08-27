# ANALYSIS: provider-settings-search-then-accessibility

Project: Universal Agent Runtime
Date: 2026-08-26
Mode: Stack specified — React 19.2.8, Tailwind CSS 4.3.3, and Playwright 1.62.1 are already present in the workspace.

## Scope and inherited evidence

Analyze covers the residual gaps identified by `assessment.md`; it does not reopen the two archived implementation changes.

- `gap-responsive-provider-panel-width` — the provider field grid at `frontend/src/features/settings/ui/panels/ai-settings-panels.tsx:184` uses viewport variant `lg:grid-cols-2`, while `openspec/specs/frontend-configuration-surfaces/spec.md:239-249` requires the composition to respond to available provider-panel width.
- `gap-browser-responsive-certification` — the spec requires narrow-width one-column layout without horizontal page scrolling, retained keyboard accessibility, and desktop two-column layout. The focused component test only checks CSS class strings.
- `gap-review-packet-evidence-integrity` — the assessment's second adversarial packet omitted canonical progress, prose goal provenance, and cited source excerpts. This is a KBD artifact-construction gap, not a product dependency gap.

The canonical progress file still records both phase changes complete. The reflection supplies four prose goals but no stable goal identifiers. Analyze therefore does not invent goal IDs; the next review packet must include the reflection goal table and progress entries directly.

## Engineering landscape

The existing stack already contains the two primitives required by the product gaps:

1. Tailwind CSS v4 has built-in container queries. Official documentation shows `@container` on the containing element, container breakpoint variants such as `@lg:grid-cols-4`, named containers such as `@container/main` with `@lg/main:*`, and `@max-*` variants. The repository already uses `@container` in three components, so this is an established local convention.
2. Playwright can set viewport dimensions, evaluate DOM dimensions, use web-first assertions, and capture locator screenshots. The repository already uses `page.setViewportSize` and asserts `scrollWidth <= innerWidth` in responsive certification tests.

No external framework or runtime measurement library is needed. A CSS container boundary solves the layout requirement at the presentation layer without React state, ResizeObserver callbacks, or rerender coupling.

## Candidate evaluation

| ID | Candidate | Verdict | Fit | Main risk |
| --- | --- | --- | --- | --- |
| `cand-001` | Tailwind CSS v4 built-in container queries | **ADOPT** | Make the provider card or panel a query container and replace the viewport breakpoint with an explicit container breakpoint. | The breakpoint must be chosen from the actual minimum viable two-column width, not copied mechanically from `lg`. |
| `cand-002` | Playwright Test | **ADOPT** | Add browser proof for constrained panel width, desktop width, horizontal overflow, and keyboard reachability using the installed test stack. | A viewport-only fixture would reproduce the original mistake; the test must constrain the provider panel inside a wide viewport. |
| `cand-003` | `@tailwindcss/container-queries` | **REJECT** | It offers container-query utilities for Tailwind v3.2+, but Tailwind v4 already provides the feature natively. | Redundant dependency and two possible syntaxes for the same behavior. |
| `cand-004` | `use-resize-observer` | **REJECT** | It can measure an element and drive a JavaScript layout branch. | Adds runtime state and resize-driven rerenders for a presentational behavior CSS already expresses directly. |
| `cand-005` | Intrinsic CSS Grid `auto-fit/minmax` | **REFERENCE** | Can wrap fields from two columns to one according to available width without an explicit container breakpoint. | An unconstrained `auto-fit` template can create three or more columns on a wide panel, conflicting with the explicit two-column desktop contract; preventing that adds a separate width constraint. |

## Build-vs-adopt decision

Adopt `cand-001` and `cand-002` as already-installed capabilities. Keep `cand-005` as a reference alternative, but do not select it: the requirement explicitly fixes the desktop composition at two columns, while a general `auto-fit/minmax` grid can exceed two columns unless another cap is introduced. Add no dependency.

The follow-up change must build only the application-specific integration:

- establish an explicit provider layout query container;
- apply one-column by default and two columns only above a documented container width;
- preserve `min-w-0` and verify no horizontal page scrolling;
- add browser assertions that constrain the provider panel independently of viewport width and separately prove the desktop composition;
- exercise keyboard traversal for the provider controls required by the narrow-width scenario.

Native unload-dialog copy is not part of this responsive follow-up. `spec.md:227-229` requires cancellation so the browser can request confirmation; the existing unit test covers cancellation. A browser-owned prompt check may be supporting evidence, but it is not a newly discovered implementation requirement.

## Open questions for Spec and Plan

1. What exact container width is the smallest supported two-column composition? The current spec intentionally describes capability rather than a number. Spec or Plan must choose a testable threshold based on field minimums and padding.
2. Which existing Playwright surface should host the proof: a focused provider-settings spec or the broader responsive-certification suite? Prefer the smallest fixture that can constrain the provider panel within a wide viewport and supply deterministic provider data.
3. The review-packet builder did not include phase progress, goals, or source excerpts in the previous assessment. Before the next adversarial dispatch, verify the generated packet itself contains those inputs; do not rely on filesystem paths that are invisible to the judge.

## Evidence excerpts supplied to review

These excerpts make the decisive local evidence visible inside the artifact packet.

- Canonical progress (`progress.json`): implementation is `{ "completed": 2, "total": 2, "status": "COMPLETE" }`; changes `provider-model-search` and `provider-settings-accessibility-dirty-state` both carry `status: "DONE"` and `implementation_status: "COMPLETE"`.
- Goal provenance (`reflection.md`, prose labels only): “Deliver searchable large provider model inventories first”; “Deliver provider accessibility and dirty-state protection second”; “Preserve existing provider settings architecture”; “Apply the requested design-review standard.” The source defines no stable goal IDs.
- Governing responsive contract (`spec.md:239-249`): the editor must stack at narrow widths, retain two columns at desktop widths, stay inside the available viewport, avoid clipped keyboard focus and horizontal page scrolling, and use the available provider-panel width as the scenario trigger.
- Current layout source (`ai-settings-panels.tsx:184`): `<div className="grid min-w-0 grid-cols-1 gap-3 lg:grid-cols-2">`.
- Existing product-frontend convention: `frontend/src/components/assistant-ui/enhanced-thread.tsx:110`, `frontend/src/components/ui/card.tsx:28`, and `frontend/src/components/ui/field.tsx:44` already use Tailwind `@container` classes.
- Existing browser convention: `a11y-responsive-certification.spec.ts:65,96-99` sets a viewport and compares document `scrollWidth` with `window.innerWidth`; the new proof must additionally constrain the provider panel inside a wide viewport.

## Research evidence and limits

- Tier 1 found the official Tailwind repository and its container-query plugin plus the maintained Tailwind, Playwright, and ResizeObserver-hook repositories. GitHub code search also found the former Tailwind plugin implementation. Two malformed CLI searches returned 422 and one repository search returned no candidates.
- Tier 2 Context7 confirmed Tailwind v4 container-query and arbitrary grid-template syntax plus Playwright viewport, screenshot, web-first assertion, and DOM measurement capabilities from official, high-reputation documentation.
- Tier 3 npm metadata confirmed Tailwind CSS 4.3.3 (MIT), Playwright 1.62.1 (Apache-2.0), the v3-oriented container-query plugin 0.1.1 (MIT), and `use-resize-observer` 10.0.0 (MIT).
- Tier 4 was not used because Tiers 1–3 answered the build-vs-adopt question.
- Process defect: Tier 1 used 9 requests against the configured cap of 8 because four repository metadata calls were bundled before the count was checked. Tier 1 stopped immediately afterward. This overrun lowers process confidence but does not change the candidate ranking; the decisive API-fit evidence comes from official Tier 2 documentation and installed-stack inspection.

## Recommendation

Retain the existing stack. Use Tailwind v4 native container queries for the layout and the installed Playwright runner for requirement-level browser proof. Plan a narrow local integration and test change; do not introduce a container-query plugin or ResizeObserver hook.
