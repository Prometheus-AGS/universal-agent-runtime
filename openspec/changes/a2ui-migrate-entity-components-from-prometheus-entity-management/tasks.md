## 1. Audit the claimed migration source
- [x] 1.1 Searched for the plan's exact claimed path (`prometheus-skill-system/skills/imported/prometheus-entity-management/packages/a2ui-react/src/`) — does not exist.
- [x] 1.2 Searched the real `prometheus-entity-management` submodule (`frontend/packages/prometheus-entity-management`, v1.2.0-rc-26) for the 6 named files and any `a2ui-react` package — none found.
- [x] 1.3 Searched that submodule's full git history (`git log --oneline --all | grep -i a2ui`) — no match.
- [x] 1.4 Searched sibling repos for an `a2ui-react` directory with these files — none found.
- [x] 1.5 Conclusion documented in proposal.md: nothing exists to migrate. Scope reinterpreted as "build new components on Change 17's foundation," not "port existing code."

## 2. EntityDiff (bound component)
- [x] 2.1 `entity-diff-api.ts` — Zod schema: `entityType` (static string), `entityId`/`title` (DynamicString), `fields: {label, before, after}[]` (all DynamicString).
- [x] 2.2 `EntityDiff.tsx` — renders a before/after grid; a row is visually marked changed (`data-a2ui-diff-changed`) when resolved `before !== after`.
- [x] 2.3 Registered in `uar-entity-catalog.ts` via `createUarComponentImplementation`.
- [x] 2.4 2 tests: changed-vs-unchanged row rendering; empty-fields-array renders without crashing.

## 3. EntityStream (binderless component)
- [x] 3.1 `entity-stream-api.ts` — Zod schema: `entityType` (static string), `source: {path: string}` (static, NOT a DynamicString — read directly off `context.componentModel.properties`), `title` (static string, optional).
- [x] 3.2 `EntityStream.tsx` — reads `source.path` from raw component properties, subscribes via `context.dataContext.subscribeDynamicValue`, renders each item, cleans up on unmount.
- [x] 3.3 Registered in `uar-entity-catalog.ts` via `createBinderlessUarComponentImplementation` (infrastructure Change 17 built but never exercised until now).
- [x] 3.4 3 tests: renders items present at mount; reactively appends a new item after an `updateDataModel` message posted after mount; renders an empty state when the source path has no items.

## 4. Documentation
- [x] 4.1 `frontend/packages/a2ui-uar/README.md` — replaced the "Deferred: EntityDiff, EntityStream, ..." section with what was actually built, and why "migrate" became "build fresh."
- [x] 4.2 OpenSpec change scaffold (`proposal.md`, `tasks.md`, `specs/a2ui-entity-components/spec.md`), `openspec validate --strict` passing.

## 5. Deferred (see proposal.md "Out of scope")
- [ ] 5.1 `EntityApproval`, `EntityToolProvider`, `EntityChat`, `EntityCopilot` — mini-application scope, not single components. Follow-up work.
- [ ] 5.2 `use-entity-*` hooks migration — no source hooks were found either (same audit finding as the components); nothing to migrate. Revisit if/when a concrete data-fetching need arises for these components.
- [ ] 5.3 Cross-testing against `@a2ui/react` for the new components — N/A, `@a2ui/react` (Google's reference) has no `Entity*` components to cross-test against.

## 6. Verification
- [x] 6.1 `pnpm --filter @prometheus-ags/a2ui-uar typecheck` — clean.
- [x] 6.2 `pnpm --filter @prometheus-ags/a2ui-uar lint` — clean (fixed one `react-hooks/set-state-in-effect` violation via a lazy `useState` initializer instead of a synchronous `setState` call inside the effect body).
- [x] 6.3 `pnpm --filter @prometheus-ags/a2ui-uar test` — 21/21 passing (16 pre-existing + 5 new).
- [x] 6.4 `openspec validate a2ui-migrate-entity-components-from-prometheus-entity-management --strict` — passing.
- [ ] 6.5 **Deferred to the phase's consolidated validation pass**: full-workspace `pnpm -C frontend lint`/`typecheck` (a pre-existing, unrelated `resizable.tsx` typecheck break is already tracked as a separate follow-up task, not caused by this change).
