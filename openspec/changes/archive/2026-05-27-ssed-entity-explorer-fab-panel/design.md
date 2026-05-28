## Context

Foundation already shipped:
- `prometheus-entity-management/src/devtools.ts` (95 LOC) computes entity counts, patches, stale/fetching sets, list keys, subscriber stats.
- The engine has hooks for ops; we add one new tap (`subscribeDevtoolsEvent`).
- The realtime-manager already invokes registered handlers on every `ChangeSet`.

What's missing is a React UI surface + a registry of Zustand stores that hold entity data + an event bus connecting it all. This design honours the UI/UX routing discipline from change 8: research happens BEFORE code.

## Goals / Non-Goals

**Goals**
- Five-tab panel atop existing `src/devtools.ts` data.
- Multi-store enumeration via an opt-in `registerDevtoolsStore` API.
- Duplicate detection + "Promote to canonical" action.
- Append-only event bus tapping both engine ops and adapter notifications.
- Production tree-shaken; zero runtime cost in prod builds.
- Pre-flight `docs/devtools-design-notes.md` distilling skill consultations.

**Non-Goals**
- No telemetry. Browser-local only.
- No chrome extension here — change 11.
- No replacement of `src/devtools.ts`; the new UI consumes it.
- No write-side mutations beyond "Promote to canonical" (which goes through the existing engine mutation path).

## Decisions

### D1. Routing discipline drives the order of work

Implementation begins with the seven-step UI/UX routing (memory recall → UI/UX Pro Max → Impeccable → Anthropic → Vercel + web search → summarise → code), per change 8's fenced region. The summary is committed as `docs/devtools-design-notes.md` BEFORE any component code lands. This is a spec requirement, not just a nice-to-have.

### D2. Dev gating via build-time + URL-param escape hatch

`NODE_ENV !== "production"` is the primary gate. The URL-param escape (`?prometheus-devtools=1`) lets staging environments observe it without rebuilding. Production bundles tree-shake the entire `src/devtools/` directory by ensuring no production code path imports from it (a single `if (devMode)` guard in `index.ts` re-exports).

### D3. Multi-store registry is opt-in

We don't auto-discover stores — discovery is unreliable. Apps that want explorer coverage call `registerDevtoolsStore`. This puts the responsibility on app authors and keeps the explorer's data model accurate. The registration call is a no-op in production (tree-shaken).

### D4. Event bus is a tiny ring buffer

Default capacity 1000. Each entry: `{ ts, type, source, payload }`. Capacity is per-app (configurable). Memory cost: ~100KB worst case. Subscribers receive events synchronously; expensive consumers should batch via `requestAnimationFrame`.

### D5. Engine tap via `subscribeDevtoolsEvent(cb)`

A new export from `engine.ts`. The engine calls `notifyDevtools(event)` internally at every op site; in production that function is a no-op. The dev-only event bus subscribes once at panel mount.

### D6. Duplicate detection runs lazily

`(type, id)` pair grouping over registered-store snapshots happens on Duplicates tab open, not on every event. The result memoizes against the registry's revision counter.

### D7. "Promote to canonical" writes through the engine

We don't bypass the engine's mutation contract. Promotion calls `engine.upsert(type, id, chosenVariant)`; other registered stores receive the canonical via their existing subscription path (or are stale until they re-fetch). Document the consequence.

### D8. Component library

Use the existing `frontend/` design tokens — assistant-ui + shadcn-style primitives that the rest of the app already loads. No new design system; the explorer fits the host app's look. Impeccable's audit + critique commands run against the panel during development to validate visual quality.

### D9. Accessibility

Tab navigation, focus-visible outlines, `aria-` labels on the FAB and panel tabs, escape-to-close, screen-reader-friendly counts. UI/UX Pro Max's accessibility validators run on each tab before merge.

### D10. The panel package is internal

Exported as `EntityExplorerFab` from the main package, but the internal modules (`panel/*`, `devtools-event-bus`, `multi-store-registry`) are not public API. Consumers compose the FAB at app root and call `registerDevtoolsStore` per store; nothing else.

## Implementation Sketch

### File layout

```
src/devtools/
├── EntityExplorerFab.tsx
├── panel/
│   ├── EntityExplorerPanel.tsx
│   ├── TreeTab.tsx
│   ├── InspectorTab.tsx
│   ├── EventsTab.tsx
│   ├── StoresTab.tsx
│   └── DuplicatesTab.tsx
├── devtools-event-bus.ts
├── multi-store-registry.ts
├── index.ts                       — re-exports gated on dev mode
└── __tests__/
    ├── registry.test.ts
    ├── bus.test.ts
    └── panel-tabs.test.tsx
```

### `src/index.ts` re-export

```ts
export { EntityExplorerFab, registerDevtoolsStore } from "./devtools";
```

The `./devtools` barrel itself re-exports stubs in production builds:

```ts
// src/devtools/index.ts
if (process.env.NODE_ENV === "production") {
  export const EntityExplorerFab = () => null;
  export const registerDevtoolsStore = () => {};
} else {
  export { EntityExplorerFab } from "./EntityExplorerFab";
  export { registerDevtoolsStore } from "./multi-store-registry";
}
```

(Note: that exact form needs a slightly different export shape under TS; see `apps/example/devtools-stub.ts` patterns in the test fixtures.)

### `subscribeDevtoolsEvent` in `engine.ts`

```ts
let listeners: Array<(e: DevtoolsEvent) => void> = [];
export function subscribeDevtoolsEvent(cb: (e: DevtoolsEvent) => void): () => void {
  listeners.push(cb);
  return () => { listeners = listeners.filter((l) => l !== cb); };
}
// At every op site:
//   if (process.env.NODE_ENV !== "production") notifyDevtools({ type: "upsert", ... });
```

### `docs/devtools-design-notes.md` structure (skeleton committed first, filled during implementation)

```
# Entity Explorer — design notes (pre-flight)

## Routing discipline application
… (memory recall summary; cite phase prior-context.md)
## UI/UX Pro Max review
… (chosen palette / typography; accessibility report)
## Impeccable audit + critique
… (commands run, findings, applied fixes)
## Anthropic frontend-design + ux-designer notes
…
## Vercel skills referenced
…
## Web search summaries
- "runtime devtools page best practices" — N sources, distilled patterns: dock side / Cmd+Shift+P palette / event log virtualization / etc.
- "react-devtools bridge architecture" — N sources, distilled patterns: __HOOK__ global, postMessage envelope, content-script bridge, etc.
## Distilled best practices for this change
… (one paragraph)
```

## Risks

1. **Production bundle bloat.** Mitigation: tree-shake gate (D2) + index.ts conditional re-export. Verify via bundle analyser.
2. **Subscriber churn from event bus.** Mitigation: subscribers should batch via rAF; spec calls this out in the Events tab.
3. **Promote-to-canonical confusion.** A user might expect "Promote" to also delete from other stores. Mitigation: Inspector tab shows the post-promote state clearly; tooltip explains.
4. **Skill list freshness.** UI/UX routing discipline drives consultation of external skills whose state changes over time; mitigation is change 8's roster-refresh flow.

## Alternatives Considered

- **Browser extension only (no in-app panel).** Rejected — many devs work without extensions. In-app is the floor; extension (change 11) is the ceiling.
- **Wrap React Devtools.** Rejected — React Devtools is component-tree-centric; we want entity-graph-centric.
- **Embed in the host app's settings page.** Rejected — devs want it accessible from any page, not just settings.
