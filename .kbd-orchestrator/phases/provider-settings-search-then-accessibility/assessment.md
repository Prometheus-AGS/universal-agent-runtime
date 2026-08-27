# ASSESSMENT: provider-settings-search-then-accessibility

Project: Universal Agent Runtime
Date: 2026-08-26
Codebase baseline: Both phase changes are implemented and archived, with current focused checks passing against the synced frontend configuration specification; this reassessment finds one residual conformance gap that requires a follow-up change.
Cross-tool progress: 2 canonical changes remain complete in `.kbd-orchestrator/phases/provider-settings-search-then-accessibility/progress.json`; no new cross-tool implementation or blockers were recorded after the 2026-08-25 reflection.

## IMPLEMENTATION STATUS

- Adaptive provider model selection: **DONE** — `frontend/src/features/settings/ui/settings-primitives.tsx:217` fixes the threshold at eight; lines 253–268 preserve the simple path below it and lines 270–349 use the bounded Base UI Combobox. Lines 286–293 trim and lowercase the literal query and match both label and raw id.
- Provider model inventory validity: **DONE** — `getProviderModelOptions` excludes malformed, disabled, empty-id, and duplicate-id entries while retaining provider order and display-name fallback. Empty and stale current-value states remain explicit.
- Provider control accessibility: **DONE** — provider cards are named groups; Base URL, Protocol, API Key, Default Model, Enabled, and reveal controls have stable associations or provider-specific names; recovery guidance is connected through `aria-describedby`.
- Async outcomes: **DONE** — loading and successful saves use status semantics; failures use alert semantics; rejected saves do not emit success and the settings cache retains dirty drafts.
- Dirty-draft protection: **DONE at component/unit level; browser confirmation UI unverified** — Save is disabled while clean, Refresh is disabled while dirty or busy, modified providers are identified, failed saves preserve drafts, and `frontend/src/features/settings/ui/panels/ai-settings-panels.tsx:65-74` installs `beforeunload` cancellation only while dirty. The focused test proves event cancellation, not the browser-owned confirmation UI.
- Narrow-panel responsiveness: **PARTIAL** — `frontend/src/features/settings/ui/panels/ai-settings-panels.tsx:184` uses `grid-cols-1 lg:grid-cols-2`, which responds to viewport width rather than the available provider-panel width required by `openspec/specs/frontend-configuration-surfaces/spec.md:239-245`. A constrained panel inside a large viewport can remain two-column. The focused test asserts class presence but does not exercise a real constrained panel, clipping, or keyboard focus in a browser.
- Settings architecture: **DONE** — the provider panel continues through `useSettings("provider")`; dirty drafts remain presentation-cache state and persistence I/O remains owned by the settings store.

## CROSS-TOOL PROGRESS

- `provider-model-search`: COMPLETE (5/5 tasks; originally completed by Codex, reconciled by kbd-runtime).
- `provider-settings-accessibility-dirty-state`: COMPLETE (7/7 tasks; originally completed by Codex, reconciled by kbd-runtime).
- Blockers: NONE recorded in `.kbd-orchestrator/phases/provider-settings-search-then-accessibility/progress.json`.

## SPEC GAP SUMMARY

- Responsive width contract: `openspec/specs/frontend-configuration-surfaces/spec.md:243-245` requires one-column stacking when the available provider-panel width cannot support the desktop composition. The shipped `lg:` class at `frontend/src/features/settings/ui/panels/ai-settings-panels.tsx:184` is viewport-based. A width-aware mechanism such as a container query, or another implementation proven against the panel's actual width, is required; the assessment does not prescribe which mechanism plan must choose.
- Browser evidence: no current real-browser proof covers constrained-panel stacking, horizontal clipping, focus visibility, or browser-provided unload confirmation behavior.
- Other phase requirements: no additional implementation gap was found in the inspected source and focused tests.

## BUILD HEALTH

- Focused provider test: **PASS** — `pnpm exec vitest run src/features/settings/ui/panels/ai-settings-panels.test.tsx`; 1 file and 11 tests passed.
- TypeScript: **PASS** — `pnpm typecheck`; `tsc -b` exited 0.
- Targeted lint: **PASS** — ESLint over the provider panel, its test, model-option helper, settings primitives, and settings hook exited 0.
- Settings structure: **PASS** — 11 modules; largest panel 599/600 lines; 29 navigation and panel keys preserved.
- Production build: **PASS** — `pnpm build`; Vite transformed 8,396 modules and completed in 5.47s. Existing PGlite direct-`eval` dependency warnings were emitted.
- Canonical spec: **PASS** — `openspec validate frontend-configuration-surfaces --type spec --strict --no-interactive`.
- Known violations: NONE in the inspected phase-owned source.
- Test coverage: **PARTIAL** — component behavior is covered, but the responsive panel-width and native browser behaviors above are not browser-certified in this reassessment.

## CONSTRAINT CHECK

- AGENTS.md violations: NONE observed in the phase-owned source. Components use the existing settings hook rather than importing graph stores or transports directly.
- constraints.md violations: N/A — no phase-specific constraints file exists.
- Scope discipline: the implementation adds no catalog fetch, fuzzy ranking, virtualization, free-form model creation, persistence redesign, or discard workflow.

## GOAL PROGRESS

- Deliver searchable large provider model inventories first: **MET** — the exact 7/8 boundary, label/id search, bounded selection, duplicate-label disambiguation, and empty/stale behavior are present and the focused test passes.
- Deliver provider accessibility and dirty-state protection second: **PARTIAL** — accessible context, live outcomes, draft protection, and unload cancellation are present, but the same change's responsive requirement is viewport-based rather than panel-width-aware and lacks browser proof.
- Preserve existing provider settings architecture: **MET** — the authoritative hook, draft cache, store actions, and entity-backed read path remain in place.
- Apply the requested design-review standard: **MET** — the checked-in execution and reflection record Impeccable review, isolated critiques, and distinct-model adversarial review; this reassessment did not find evidence contradicting those receipts.

## ASSESSMENT COMPLETE

Reassessed verdict: the canonical implementation counter remains 2/2 complete, but `provider-settings-accessibility-dirty-state` is non-conformant on its responsive-width requirement. A narrow follow-up OpenSpec change is required for panel-width-aware layout and browser certification. The archived implementation remains complete as recorded; the follow-up should close the newly observed conformance gap rather than relabel completed work as an implementation gap.

## UNRESOLVED ADVERSARIAL REVIEW FINDINGS

The distinct K3 judge returned **BLOCK** on the permitted second review round (2 CRITICAL, 2 WARNING, 0 SUGGESTION). The anti-sycophancy screen passed with score 0.0. These findings remain open for the next planning or assessment pass:

- **CRITICAL — canonical goals were absent from the review packet.** The packet supplied `goals: null`, so the judge could not verify that the four prose goals in this assessment are the authoritative phase goals. A future packet must include stable goal identifiers and map each to evidence, a gap, or an explicit non-applicability decision.
- **CRITICAL — canonical progress evidence was omitted from the review packet.** The assessment cites `.kbd-orchestrator/phases/provider-settings-search-then-accessibility/progress.json`, but the packet did not include that file or its relevant entries. A future packet must carry those entries and explicitly reconcile archived task completion with the newly observed responsive-width non-conformance.
- **WARNING — line-specific source claims lacked source excerpts in the packet.** A future packet should attach the cited ranges or focused diffs for the threshold, filtering, accessibility, dirty-state, and responsive-layout determinations.
- **WARNING — browser evidence gaps were grouped too broadly.** A future assessment should split constrained-panel stacking, horizontal clipping, focus visibility, and native unload confirmation into separate evidence items, each tied to a governing requirement or marked as non-required supporting evidence.
