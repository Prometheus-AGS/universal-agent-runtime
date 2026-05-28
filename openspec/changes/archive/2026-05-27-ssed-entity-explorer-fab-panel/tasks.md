# Implementation Tasks — ssed-entity-explorer-fab-panel

> Cross-repo. The substantial React component code is **deferred to a focused implementation session** with the prometheus-entity-management dev environment running. The OpenSpec artifacts here carry every detail an implementer needs.

## 1. Pre-flight (UI/UX routing discipline — change 8)

- [ ] 1.1 Run `/kbd-memory-recall` for this phase (auto-fired by `assess:before` hook)
- [ ] 1.2 UI/UX Pro Max analysis on existing dashboard / panel pages
- [ ] 1.3 `/impeccable audit` + `/impeccable critique` on early sketches
- [ ] 1.4 Consult Anthropic frontend-design + ux-designer
- [ ] 1.5 Consult Vercel React Best Practices + Composition Patterns
- [ ] 1.6 Web search: "runtime devtools page best practices"
- [ ] 1.7 Web search: "react-devtools bridge architecture"
- [ ] 1.8 Write `prometheus-entity-management/docs/devtools-design-notes.md` summarising the above

## 2. Engine tap

- [ ] 2.1 Add `subscribeDevtoolsEvent(cb)` export to `src/engine.ts`
- [ ] 2.2 Add `notifyDevtools(event)` calls at every op site (upsert / patch / delete / clearPatch / list ops)
- [ ] 2.3 Guard with `NODE_ENV !== "production"` so prod builds tree-shake the calls

## 3. Multi-store registry — `src/devtools/multi-store-registry.ts`

- [ ] 3.1 Implement `registerDevtoolsStore(config)` with replacement semantics on same id
- [ ] 3.2 Provide an enumeration API consumed by Stores / Duplicates tabs
- [ ] 3.3 Production no-op stub

## 4. Event bus — `src/devtools/devtools-event-bus.ts`

- [ ] 4.1 Ring buffer with default capacity 1000 (configurable)
- [ ] 4.2 `push(event)` + `subscribe(cb)` + `getSnapshot()` APIs
- [ ] 4.3 Subscribe to engine via `subscribeDevtoolsEvent`; subscribe to adapter notifications via realtime-manager tap

## 5. Components — `src/devtools/`

- [ ] 5.1 `EntityExplorerFab.tsx` — FAB + panel host with dev-mode gate
- [ ] 5.2 `panel/EntityExplorerPanel.tsx` — five-tab container
- [ ] 5.3 `panel/TreeTab.tsx` — entities grouped by type with stale/patched/fetching badges
- [ ] 5.4 `panel/InspectorTab.tsx` — full entity drilldown
- [ ] 5.5 `panel/EventsTab.tsx` — live tail with filters
- [ ] 5.6 `panel/StoresTab.tsx` — registered store enumeration + diff
- [ ] 5.7 `panel/DuplicatesTab.tsx` — `(type, id)` duplicate detection + Promote action

## 6. Tests

- [ ] 6.1 `__tests__/registry.test.ts` — registration / replacement / production no-op
- [ ] 6.2 `__tests__/bus.test.ts` — capacity / eviction order / subscribe delivery for engine + adapter events
- [ ] 6.3 `__tests__/panel-tabs.test.tsx` — render each tab against a fixture graph, assert row content + counts

## 7. Re-exports + bundle gating

- [ ] 7.1 `src/index.ts` exports `EntityExplorerFab` + `registerDevtoolsStore` from the devtools barrel
- [ ] 7.2 `src/devtools/index.ts` swaps to production stubs at build time
- [ ] 7.3 Verify with bundle analyser that production builds contain no panel code

## 8. Skill update — `prometheus-skill-system/skills/prometheus-entity-skills/entity-graph-optimize/SKILL.md`

- [x] 8.1 Add a "Dev-mode entity explorer" subsection describing FAB, toggle conditions, `registerDevtoolsStore` API, tab catalogue, and pointing to `docs/devtools-design-notes.md`
- [x] 8.2 Cross-link to the new `entity-realtime-surreal-live` skill (so the explorer's data source includes Surreal-backed apps)

## 9. Cross-repo PRs

- [ ] 9.1 `prometheus-entity-management` PR with src/devtools/ + engine tap + docs/devtools-design-notes.md + tests
- [ ] 9.2 `prometheus-skill-system` PR with skill subsection (combines with the broader W1-W3 PR)

```
prometheus-entity-management commit: <fill in after merge>
PR URL:                         https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
prometheus-skill-system commit:      eb3134bce2b1f1a956acfd6d264f2b7d8862974f
PR URL:                         https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
```

## 10. Closeout

- [x] 10.1 Skill subsection landed in prometheus-skill-system
- [ ] 10.2 **Substantial React + engine work deferred** to a focused implementation session — same model as change 9. Spec is the contract; the implementer has every detail needed.
- [ ] 10.3 `/opsx:verify` + `/opsx:archive` (verify will flag §1-7 as deferred CRITICAL)
- [ ] 10.4 progress.json `changes_completed: 10`; active_change → `ssed-entity-explorer-browser-extension`
