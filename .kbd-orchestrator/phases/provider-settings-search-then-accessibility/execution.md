# EXECUTION: provider-settings-search-then-accessibility

Project: universal-agent-runtime
Date: 2026-08-26
Selected backend: openspec
Dispatched to: Codex self-execution through the KBD-owned apply driver
Backend rationale: The one follow-up change already has complete, strictly valid OpenSpec artifacts and a task-level verification contract. OpenSpec remains the execution surface while typed KBD commands remain authoritative for phase, change, task, and waypoint state.
Backend entrypoint: `/kbd-apply fix-provider-settings-panel-width-responsiveness`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/provider-settings-search-then-accessibility/plan.md`

## Execution scope

- `fix-provider-settings-panel-width-responsiveness`: replace the viewport-based provider field breakpoint with a named Tailwind container query, add focused unit and browser contracts, and certify the constrained-panel and desktop states.
- Historical boundary: `provider-model-search` and `provider-settings-accessibility-dirty-state` remain complete and are not reopened or edited by this dispatch.

## Dispatch contracts

- `fix-provider-settings-panel-width-responsiveness` → Codex, model class frontier
  - Entry: `/kbd-apply fix-provider-settings-panel-width-responsiveness`
  - OpenSpec source: `openspec/changes/fix-provider-settings-panel-width-responsiveness/`
  - Progress authority: `.kbd-orchestrator/phases/provider-settings-search-then-accessibility/progress.json`
  - Task protocol: execute the nine OpenSpec tasks sequentially through `/kbd-apply`; fire `task:before` and `task:after`; record each typed KBD task transition; do not invoke bare `/opsx:apply`.
  - Implementation boundary: production edits remain the existing provider-list and field-grid class strings. Stop and re-plan if new markup, width state, ResizeObserver behavior, a dependency, or state ownership change becomes necessary.
  - Handoff: after each task, preserve the observed command output in KBD/OpenSpec evidence and leave the next pending task in the waypoint.

## Approval gates

- Tier 3 browser execution requires explicit operator authorization for a milestone or release. Without authorization, implementation may proceed through Tier 2, but the change remains uncertified and unarchived.
- OpenSpec archival requires completed artifact-refiner validation, isolated diff-mode adversarial review, OpenSpec verification, and a passing authorized Tier 3 browser receipt.
- No destructive, external, provider-data, persistence, dependency, or deployment action is authorized by this dispatch.

## Fallback conditions

- If a class-only container-query edit cannot satisfy the measured one-track/two-track contract, stop and return to Plan rather than adding JavaScript width state or compressing the 600-line panel.
- If the deterministic browser fixture cannot constrain the actual named provider container while retaining the real settings shell, stop and repair the fixture contract before changing production behavior.
- If baseline failures obscure whether this change regressed the phase, preserve the full observed output and isolate the focused delta; do not repair unrelated baselines in this change.

## Verification evidence

Task 1.1 guidance evidence (2026-08-26): Impeccable audit/critique, frontend-design, UI/UX Pro Max, Vercel React Best Practices, Vercel Composition Patterns, and Prometheus Entity Graph CRUD were consulted against the existing Operate-mode provider panel and incumbent tokens; `ux-designer` was not installed, so frontend-design supplied the available UX-engineer perspective. The resulting constraint is class-only: use Tailwind's built-in named container query and existing spacing/tokens, preserve `useSettings("provider")` plus its draft cache as the sole provider state authority, add no width state, effects, subscription, provider/context wrapper, store/service/transport access, or entity mutation, and keep the browser fixture read-only with respect to durable settings. UI/UX Pro Max and current Tailwind documentation support container-relative layout with no horizontal page overflow; the React/composition/entity guidance supports CSS classes over layout reads and leaves the existing hook boundary unchanged.

Task 1.2 browser-contract evidence (2026-08-26): Added `frontend/e2e/provider-settings-responsive.spec.ts` with deterministic settings reads, zero accepted durable writes, a measured 36rem boundary, one/two/two exact track assertions, provider-card and page overflow bounds, explicit keyboard focus traversal, and dirty-draft preservation across narrow, wide, and restored desktop states. `pnpm typecheck` and `pnpm lint` exited 0. `pnpm -C frontend exec playwright test --list e2e/provider-settings-responsive.spec.ts` discovered exactly 1 test in 1 file. Behavioral execution remains pending Tier 3 and was not run at this task boundary.

Task 2.1 negative-control evidence (2026-08-27): Updated the focused ProviderPanel test to require `@container/provider-panel`, base `grid-cols-1`, and `@xl/provider-panel:grid-cols-2`, and to reject `lg:grid-cols-2`. Before any production edit, `pnpm -C frontend test src/features/settings/ui/panels/ai-settings-panels.test.tsx` failed exactly 1 of 11 tests at the missing named-container assertion while the other 10 passed. `pnpm typecheck` and `pnpm lint` then exited 0 for the test-only edit.

Task 2.2 implementation evidence (2026-08-27): Added `@container/provider-panel` to the existing provider-list body and replaced only `lg:grid-cols-2` with `@xl/provider-panel:grid-cols-2` on the existing field grid. No markup, state, effect, dependency, provider data, store, service, transport, or inline behavior was added. `pnpm typecheck` and `pnpm lint` exited 0; `pnpm -C frontend settings:structure` passed with 11 modules and `ai-settings-panels.tsx` at 599/600 lines; the one required post-edit Impeccable detector run returned `[]` with exit 0.

Task 2.3 Tier 1 evidence (2026-08-27): `pnpm -C frontend test src/features/settings/ui/panels/ai-settings-panels.test.tsx` exited 0 with 1 test file and all 11 tests passing. The passing focused suite includes the named container/base one-column/`@xl` two-column class contract and the existing provider model, search, keyboard, accessible association, async feedback, save/reload, error, busy, unload, and dirty-draft scenarios.

Task 3.1 structure/spec evidence (2026-08-27): `pnpm -C frontend settings:structure` exited 0 with 11 modules, `ai-settings-panels.tsx` at 599/600 lines, and all 29 navigation and panel keys preserved. `openspec validate fix-provider-settings-panel-width-responsiveness --strict --no-interactive` exited 0 with the change valid, and the required `frontend-configuration-surfaces` delta is present at `openspec/changes/fix-provider-settings-panel-width-responsiveness/specs/frontend-configuration-surfaces/spec.md`.

Task 3.2 Tier 2 evidence (2026-08-27): `pnpm build` exited 0 after Vite transformed 8,396 modules and built in 2.76s; it emitted four non-fatal direct-`eval` warnings from the pinned `@electric-sql/pglite` dependency. `pnpm test` exited 1 with 73/76 files and 350/362 tests passing: two `providers-store.test.ts` failures came from its mock omitting `updateProvider`, two `protocol.stories.tsx` failures came from invalid `ChoicePicker` values, and all eight `entities.stories.tsx` cases failed on invalid Zod schema elements. These 12 failures are unchanged worktree-baseline failures for this change: its source delta is limited to provider-panel responsive classes, its focused contract, and a new Playwright spec; it changed none of the failing tests, stores, API mocks, A2UI stories, schemas, or dependencies. A post-suite rerun of `pnpm -C frontend test src/features/settings/ui/panels/ai-settings-panels.test.tsx` exited 0 with 1/1 file and 11/11 tests passing, so the affected surface has zero observed Tier 2 regressions. The repository-wide test gate remains non-green because of the recorded baseline failures; no unrelated baseline repair was attempted.

Task 3.3 authorized Tier 3 evidence (2026-08-27): The operator explicitly authorized the Tier 3 milestone run. The first `pnpm -C frontend exec playwright test e2e/provider-settings-responsive.spec.ts` attempt could not render the app because Vite's development optimizer rewrote the entity-management package's optional `loro-crdt` peer into an unresolved root import. No dependency or Vite configuration was added for that unrelated environment defect; the already-passing Tier 2 production bundle was served with `vite preview`, and Playwright reused it. That run exposed one focused-contract ambiguity where `getByLabel("API Key")` matched both the password field and its reveal button; the test-only locator was narrowed to the exact API-key textbox, after which `pnpm typecheck` and `pnpm lint` exited 0. The authorized production-bundle rerun exited 0 with 1/1 test passing in 2.9s. Its passing assertions certify one track at the measured 36rem boundary minus one pixel, two tracks at plus one pixel and restored desktop width, page/card/control bounds without horizontal overflow, mechanical Enable → Base URL → Protocol → API Key → reveal → Default Model keyboard focus with visible focus treatment inside the scroll viewport, dirty Base URL preservation across all transitions, and zero durable settings writes. The global Playwright reporter's unrelated generated receipt was removed; this change retains only its own KBD evidence.

Task 3.3 negative-control and strengthened-contract evidence (2026-08-27): After the first isolated review identified missing design evidence, the browser fixture was strengthened to measure the queried content box with `clientWidth`, resolve the field grid through the Base URL control, compare Base URL/Protocol stacked versus same-row geometry, repeat containment and keyboard-focus checks in narrow, wide, and restored desktop states, and prove the dirty value survives a reverse wide-to-narrow crossing. `pnpm typecheck` and `pnpm lint` exited 0. For the required old-layout negative control, exactly the two production classes were temporarily restored to the pre-change `lg:grid-cols-2` implementation, Tier 0 and a production build passed, and the authorized Playwright test failed on its first geometry assertion with `Expected: 1`, `Received: 2` at `boundary - 1`. The final named-container classes were then restored; Tier 0 and `pnpm build` exited 0, and the strengthened authorized production-bundle test passed 1/1 in 3.5s. The temporary build and global reporter receipt were replaced/removed, so only the final production candidate remains.

Task 3.4 final refinement, review, and scope evidence (2026-08-27): Impeccable polish found no justified production edit beyond the requested class-only repair. Artifact-refiner converged and finalized at iteration 5/5; its manifest, constraints, refinement state, referenced files, iteration cap, convergence status, and registry entry validate. The installed artifact-refiner adapter omits its referenced canonical schemas, so schema validation used the explicitly disclosed archived canonical assets. The final isolated receipt is `.kbd-orchestrator/phases/provider-settings-search-then-accessibility/review/fix-provider-settings-panel-width-responsiveness/round5/findings.json`: judge `k3`, producer `gpt-5`, REST-gateway isolation, verified-distinct, PASS with 0 critical, 2 warnings, and 2 suggestions; the anti-theater gate passed at 0.0. The containment warning is closed by the installed `SelectPrimitive.Portal` implementation and a final authorized browser replay that keyboard-opened both listboxes, verified nonzero geometry inside the viewport, and passed 1/1 in 4.3s. The iteration-cap warning is closed by finalizing within iteration 5; no sixth cycle was started. Final checks passed: `pnpm typecheck`, `pnpm lint`, the focused provider-panel suite at 11/11, settings structure at 11 modules with the panel at 599/600 lines and 29 keys preserved, strict OpenSpec validation, and change-owned diff integrity. The review packet builder initially included unrelated dirty-worktree files and missed the nested OpenSpec delta and untracked files; its round-5 packet was mechanically corrected and checked to contain exactly 15 change-owned artifacts plus the full acceptance and constraint texts. The final production diff is exactly two class substitutions: the named provider-panel container and its `@xl/provider-panel:grid-cols-2` variant. No change-owned dependency, state authority, provider compatibility, provider data, store, service, transport, realtime, unload-dialog, or baseline-repair work remains. The worktree's pre-existing `pnpm-lock.yaml` diff was excluded from this change and its review packet.

## Verification requirements

1. Before React edits, record the required Impeccable, frontend-design, UI/UX Pro Max, Vercel React, composition, and entity-state guidance summary.
2. Discover the focused browser spec with `pnpm -C frontend exec playwright test --list e2e/provider-settings-responsive.spec.ts`; discovery is not behavioral proof.
3. Preserve a failing component negative control against `lg:grid-cols-2` before the production class change.
4. Run `pnpm typecheck` and `pnpm lint` after each edit as Tier 0.
5. Run `pnpm -C frontend settings:structure` and keep the panel within its existing limit.
6. Run `pnpm -C frontend test src/features/settings/ui/panels/ai-settings-panels.test.tsx` at Tier 1.
7. Run `openspec validate fix-provider-settings-panel-width-responsiveness --strict --no-interactive`.
8. Run `pnpm build` and `pnpm test` at Tier 2 phase completion and record actual baseline failures separately from regressions.
9. At an explicitly authorized Tier 3 milestone or release, run `pnpm -C frontend exec playwright test e2e/provider-settings-responsive.spec.ts` before archival.
10. In the browser fixture, derive the 36rem boundary from the measured root font size; assert exactly one track at `boundaryPx - 1`, two at `boundaryPx + 1`, and two after restoring the normal desktop width. Resolve the provider-list body through the existing provider group, keep each rendered control box inside its card without rejecting legitimate internal input scrolling, assert zero durable settings writes, and mechanically Tab through Enable, Base URL, Protocol, API Key, reveal, and Default Model with the expected active element inside the scroll viewport and a visible computed focus indicator.
11. After implementation completes, run artifact-refiner validation and isolated diff-mode adversarial review before OpenSpec verification and archival.

## Progress ledger

- [COMPLETE] `provider-model-search` — historical Codex change
- [COMPLETE] `provider-settings-accessibility-dirty-state` — historical Codex change
- [COMPLETE] `fix-provider-settings-panel-width-responsiveness` — Codex through `/kbd-apply`

Runtime lifecycle note: the append-only runtime rejected `complete` →
`in-progress` for the historical phase status. It nevertheless accepted the
new pending change and nine tasks, recalculated implementation to 2/3, and
accepted independent evidence, certification, and publication states of 2/3
in progress. Revision 866 points the active path to `/kbd-apply`; the terminal
phase-status field remains the immutable historical closeout record.

## Expected outputs

- Named provider-panel container and one-column/two-column container-query classes.
- Focused component negative/positive contract.
- Focused deterministic Playwright contract and authorized Tier 3 receipt.
- Updated OpenSpec tasks, KBD evidence, artifact-refiner log, and adversarial-review receipt.

## Blockers

- No dispatch blocker.
- Tier 3 authorization was granted, the strengthened production-bundle browser certification passes, and final corrected-candidate review passes; typed KBD verification and archival are the remaining closeout actions.
- If `/kbd-apply` cannot transition the newly registered change because the historical phase status remains terminal, stop and create a successor phase through typed KBD commands; do not edit projections manually.

## Reflection handoff

Reflect must lead with the delta between this plan and delivery. It must report whether the production edit stayed class-only, whether the fixture proved the actual container boundary and exact track count, whether keyboard focus evidence was mechanical, the observed Tier 0–3 results, every baseline failure, and the development-optimizer limitation that required production-bundle certification.

## Execution ready
