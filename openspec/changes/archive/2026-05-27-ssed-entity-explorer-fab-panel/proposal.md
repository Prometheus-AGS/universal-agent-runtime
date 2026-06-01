## Why

The user's phase request: "add support in the prometheus entity management submodule for an entity explorer in dev mode, driven by a floating action button that opens up a view of the entire entity tree currently being managed as well as any other debugging tools that would work with the browser." Additional goals: visualise multi-store entity coordination, discover duplicates, see realtime events, ensure a "single representation of an entity at any time".

The foundation already exists: `prometheus-entity-management/src/devtools.ts` (95 LOC) implements `collectGraphDevStats` — entity counts, patches, stale/fetching sets, list keys, subscriber stats. What's missing is a UI surface that consumes these stats and the realtime events stream, plus the multi-store enumeration the user named.

**Per change 8's routing discipline, this UI is built only AFTER**: a memory consult, UI/UX Pro Max analysis, Impeccable audit + critique, Anthropic frontend-design + ux-designer review, Vercel React Best Practices + Composition Patterns review, AND web search for "runtime devtools page best practices" + "react-devtools bridge architecture". The implementation pre-flight is part of this change's tasks.

## What Changes

### `prometheus-entity-management` — new module `src/devtools/`

```
src/devtools/
├── EntityExplorerFab.tsx       — floating action button + panel host
├── panel/
│   ├── EntityExplorerPanel.tsx — top-level panel with tabs
│   ├── TreeTab.tsx             — entity tree by type
│   ├── InspectorTab.tsx        — per-entity drilldown
│   ├── EventsTab.tsx           — realtime event log
│   ├── StoresTab.tsx           — multi-store enumeration + duplicates
│   └── DuplicatesTab.tsx       — same-entity-id-in-multiple-stores detector
├── devtools-event-bus.ts       — append-only event stream (engine ops + adapter notifications)
├── multi-store-registry.ts     — registerDevtoolsStore() API
└── index.ts
```

### Toggle behavior

- The FAB renders only when `process.env.NODE_ENV !== "production"` OR `window.location.search.includes("prometheus-devtools=1")`. Production builds tree-shake the module.
- A small toggle button in the FAB area dismisses the panel.

### Multi-store registration

A new public API:

```ts
import { registerDevtoolsStore } from "@prometheus-ags/prometheus-entity-management";

// In each Zustand store's bootstrap:
registerDevtoolsStore({
  id: "client-store",
  label: "Clients module",
  store: clientStore,       // the actual Zustand store instance
  describesEntityTypes: ["client", "contact"],
});
```

The registry tracks every store that's holding entity data, so the explorer can enumerate "which stores contain which entity ids" and surface duplicates.

### Tree tab

Hierarchical view by `EntityType`, expandable. Each row shows:
- type
- entity id (clickable → opens Inspector for that id)
- which registered stores hold this id (badges)
- last-updated timestamp
- "stale" / "fetching" / "patched" status badges

Counts at the top: total entities, by type, total patches, stale count.

Visual style follows the UI/UX Pro Max + Impeccable analysis output (run during pre-flight per change 8's discipline). Reasonable defaults: monospace for ids, body font for labels, focus-visible outlines.

### Inspector tab

When a user clicks an entity id (from Tree, Stores, or Events), the Inspector opens with:
- Full normalised entity data (`useGraphStore.getState().entities[type][id]`)
- Patches (from `useGraphStore.getState().patches[type][id]`)
- Subscriber count for this entity
- Origin store(s)
- Timeline of events touching this entity (filtered from the event bus)

### Events tab

Live tail of `devtools-event-bus` entries:
- Engine ops (`upsert`, `patch`, `delete`, `clearPatch`, etc.)
- Adapter notifications (`ChangeSet` arrivals)
- List operations
- Render-trigger events (when a `useEntity` subscriber fires)

Each row: timestamp, event type, payload preview (with "expand" affordance), source (engine | adapter | hook).

Filter controls: by event type, by entity type, by source, by entity id.

### Stores tab

Lists every store registered via `registerDevtoolsStore`:
- store id + label
- entity types it claims
- count of entity ids it currently holds
- "diff against canonical" — entities this store holds vs. the engine's canonical graph

### Duplicates tab

Surfaces "same `(type, id)` held in multiple stores" cases:
- table of duplicates with side-by-side data preview
- "Promote to canonical" action (writes the chosen version through the engine; other stores' copies become observers)
- visualisation of which stores hold the duplicate

### `devtools-event-bus.ts`

A small append-only ring buffer (default 1000 events) subscribed to by every panel tab. Hooks into engine ops via a new `subscribeDevtoolsEvent(cb)` API (added to `engine.ts`) and into adapter notifications (the panel registers itself as a tap on every `ChangeSet` emit).

### Pre-flight artifact: `docs/devtools-design-notes.md`

Per the routing discipline (change 8), the implementation begins by:
1. Running `/kbd-memory-recall` (auto-fired on `assess:before`).
2. Running UI/UX Pro Max audit on existing dashboard pages for stylistic baseline.
3. Running `/impeccable audit` + `/impeccable critique` on the early panel sketches.
4. Web search: "runtime devtools page best practices" + "react-devtools bridge".
5. Writing a `docs/devtools-design-notes.md` summarising the synthesised best practices BEFORE any production code lands. The file is committed.

This is the "summarise best practices in one paragraph" step from the UI/UX routing region; for this change it expands to a doc because the design surface is non-trivial.

### Skill update in `prometheus-entity-skills/entity-graph-optimize/SKILL.md`

Add a section "Dev-mode entity explorer" describing the FAB + panel, the toggle conditions, the multi-store registration API, and the canonical workflow for using it during development.

### Non-changes

- **No telemetry / remote logging.** Everything stays in the browser.
- **No prod build.** The module is gated; prod builds drop it.
- **No Chrome extension.** That's change 11. This change establishes the in-app shell the extension will reuse.
- **No replacement of `src/devtools.ts`.** It's the data source; the new UI consumes it.

## Capabilities

### New Capabilities

- `entity-explorer-fab-panel`: A dev-mode floating-action-button-launched entity explorer for `prometheus-entity-management` consumers, with five tabs (Tree, Inspector, Events, Stores, Duplicates), a multi-store registration API, an append-only event bus, and a documented pre-flight UI/UX research pass.

### Modified Capabilities

- `entity-graph-optimize` (skill): gains a "Dev-mode entity explorer" subsection pointing to the new module and the registration API.

## Impact

- **Risk**: Medium. New UI surface; touches engine to add the `subscribeDevtoolsEvent` tap. The dev-mode gate prevents production impact; tests can mock the bus.
- **Affected files**:
  - `prometheus-entity-management`: `src/devtools/` (new directory), `src/engine.ts` (add `subscribeDevtoolsEvent`), `src/index.ts` (re-export gated APIs), `docs/devtools-design-notes.md` (new).
  - `prometheus-skill-system`: `skills/prometheus-entity-skills/entity-graph-optimize/SKILL.md` (subsection added).
- **Cross-repo**: Yes — two repos.
- **Reversibility**: Trivial — drop the `src/devtools/` directory and the engine tap.
- **Unblocks**: Change 11 (browser extension) reuses panel UI + event bus.
