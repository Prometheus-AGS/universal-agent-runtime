# PLAN: provider-settings-search-then-accessibility

Project: universal-agent-runtime
Date: 2026-08-26
OpenSpec available: YES
Changes to implement: 1
Model routing: `.kbd-orchestrator/project.json` has no `model_policy`; the required fallback is frontier.

## Historical completion boundary

The canonical implementation ledger remains 2/2 complete for
`provider-model-search` and `provider-settings-accessibility-dirty-state`.
This plan does not reopen, rename, or relabel either archived change. It adds
one follow-up slice for the responsive-width non-conformance observed during
reassessment.

The phase's reflection supplies four prose goals but no stable goal IDs:

- Deliver searchable large provider model inventories first — remains met and is outside this change.
- Deliver provider accessibility and dirty-state protection second — the responsive-width portion has the observed follow-up gap.
- Preserve the existing provider settings architecture — remains a hard constraint.
- Apply the requested design-review standard — remains a required execution and certification gate.

## Change list (ordered)

1. `fix-provider-settings-panel-width-responsiveness`: Make Provider Overrides respond to its panel width and certify the behavior
   - Scope: UI classes | focused component contract | focused browser contract | OpenSpec/KBD evidence
   - Depends on: NONE; the two predecessor changes are already complete
   - Recommended agent: Codex
   - Est. implementation time: M (1–4 agent hours)
   - Complexity score: High
   - Model class: frontier
   - Routing rationale: the time estimate and routing classifier use different scales. The OpenSpec change has nine tasks, so KBD classifies it High and routes it to frontier even though the production edit is class-only.
   - Libraries: `cand-001` — adopt installed Tailwind CSS 4.3.3 native container queries; `cand-002` — adopt installed Playwright Test 1.62.1. Add no dependency.
   - Files: `frontend/src/features/settings/ui/panels/ai-settings-panels.tsx`; `frontend/src/features/settings/ui/panels/ai-settings-panels.test.tsx`; new `frontend/e2e/provider-settings-responsive.spec.ts`; the owning OpenSpec/KBD evidence only.
   - Details: Add `@container/provider-panel` to the existing provider-list body, retain `grid-cols-1` as the base field layout, and replace viewport `lg:grid-cols-2` with `@xl/provider-panel:grid-cols-2`. Preserve `useSettings("provider")`, the draft cache, save/reload behavior, provider compatibility, and realtime reconciliation; introduce no width state, ResizeObserver, transport, entity, or dependency change.
   - Browser fixture: use a 1440×900 viewport for both states. Apply an explicit below-576px inline-size constraint to the actual named provider container for the one-column state. Remove that constraint for the desktop state, use the natural panel width, and assert its measured content box is at least 36rem before checking layout.
   - Acceptance: assert the computed provider grid has exactly one track below 36rem and exactly two tracks at or above 36rem; comparing only two adjacent controls is insufficient. Assert document, provider-card, and visible-control overflow bounds; keyboard reachability and visible focus; and preservation of the edited value and modified indicator while crossing the boundary.
   - Negative control: the focused component contract must fail against `lg:grid-cols-2` before the production class edit, then pass only for the named container and `@xl/provider-panel:grid-cols-2` contract.
   - Archive gate: Playwright discovery may run during ordinary implementation, but behavior execution is Tier 3. Before any archive request, obtain explicit operator authorization for a milestone or release, run the focused Playwright spec, and require it to pass. Without authorization or a passing receipt, keep the OpenSpec change open and report the responsive behavior as uncertified.
   - Trade-off / scope cut: do not add native unload-dialog copy checks, a discard workflow, state restructuring, provider compatibility work, realtime changes, dependency work, baseline repairs, or unrelated visual redesign.

## Execution round order

Round 1 (sequential): `fix-provider-settings-panel-width-responsiveness`

Within the change, execute one task and its prescribed cheap check before the
next task. Add the focused contracts first, preserve the negative control, then
make the two class-string edits. Do not start an extraction merely because the
panel is near its structure limit; stop and re-plan if class-only implementation
is insufficient.

## Verification order

1. Workflow gate: record the task-specific UI/React/entity guidance summary before React edits.
2. Browser-contract discovery: `pnpm -C frontend exec playwright test --list e2e/provider-settings-responsive.spec.ts`; this proves discovery only, not behavior.
3. Tier 0 after each edit: `pnpm typecheck` and `pnpm lint`.
4. Structure: `pnpm -C frontend settings:structure`; the existing 600-line panel limit remains binding.
5. Tier 1 after the unit is complete: `pnpm -C frontend test src/features/settings/ui/panels/ai-settings-panels.test.tsx`.
6. OpenSpec: `openspec validate fix-provider-settings-panel-width-responsiveness --strict --no-interactive`.
7. Tier 2 at phase completion: `pnpm build` and `pnpm test`; record the observed full-suite result and distinguish unchanged baseline failures from regressions.
8. Tier 3 only after explicit milestone/release authorization: `pnpm -C frontend exec playwright test e2e/provider-settings-responsive.spec.ts`; this passing receipt is mandatory before archival.
9. Certification: deterministic artifact-refiner validation, then isolated diff-mode adversarial review with a judge distinct from the producer, then OpenSpec verification and archival if every required tier is satisfied.

## Evidence carried into Execute

Canonical progress is `{ "completed": 2, "total": 2, "status": "COMPLETE" }`
for the historical changes. The follow-up is additional work and must be
registered without rewriting those completed entries.

Quoted local Tailwind container precedents:

- `frontend/src/components/assistant-ui/enhanced-thread.tsx:110`: `className="aui-root aui-thread-root @container flex min-h-0 flex-1 flex-col bg-background"`
- `frontend/src/components/ui/card.tsx:28`: `group/card-header @container/card-header grid`
- `frontend/src/components/ui/field.tsx:44`: `group/field-group @container/field-group flex`

Quoted local Playwright precedents:

- `frontend/e2e/a11y-responsive-certification.spec.ts:65`: `await page.setViewportSize({ width, height: 900 });`
- `frontend/e2e/a11y-responsive-certification.spec.ts:96-99`: reads `document.documentElement.scrollWidth` and `window.innerWidth`, then requires document width to be no greater than viewport width.

Research-process disclosure: Analyze Tier 1 used 9 requests against the cap of
8 because four repository metadata calls were bundled before the count was
checked. It stopped immediately afterward. This lowers process confidence but
does not change the adopt decisions, which also rely on official Tier 2 docs
and installed-stack inspection.

## OpenSpec dispatch

The OpenSpec change already exists and all four planning artifacts are complete;
do not run `/opsx:new` again. Execute through the KBD-owned apply driver so task
hooks, progress, and waypoint state remain coherent:

`/kbd-apply fix-provider-settings-panel-width-responsiveness`

## Uncomfortable constraint

The source change can be implemented and pass Tiers 0–2 while the requirement
still lacks real-browser certification. That is not completion. If Tier 3 is not
authorized, the correct result is implemented but uncertified and unarchived,
not a fabricated PASS or a weaker component-only substitute.

## Plan complete
