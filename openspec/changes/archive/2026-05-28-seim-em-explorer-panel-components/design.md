## Context

W6 is the first change that produces production React/TSX. It is bound by the W4 preflight
(`docs/devtools-design-notes.md`) as its design source of truth. All palette, typography,
spacing, animation, and composition decisions are derived from that document — this design.md
records implementation decisions only.

## Goals / Non-Goals

**Goals:**
- Ship a working 4-tab FAB panel in `src/ui/entity-explorer/`
- Virtualized entity list (TanStack Virtual — already a dep)
- Live Events tab consuming W5 bus
- All 4 tabs mounted/hidden (not unmounted) for state preservation
- CSS variables scoped to `entity-explorer.css`
- Keyboard shortcut `Alt+Shift+E`
- `@testing-library/react` component tests (≥10 it blocks)

**Non-Goals:**
- Performance tab flame graph (ops/sec counter + latency only at launch — §3 distill)
- Subscriptions standalone tab (folded into Events filter — §10 distill)
- Chrome extension bridge (W8)
- SSR / RSC (all client components)

## Decisions

**D1. File layout**
```
src/ui/entity-explorer/
  index.ts          ← barrel
  context.tsx       ← EntityExplorerProvider + useEntityExplorer
  fab.tsx           ← FAB (portal, keyboard shortcut)
  panel.tsx         ← Panel shell (portal, TabBar, TabContent, DetailPane)
  tabs/
    entities-tab.tsx
    patches-tab.tsx
    events-tab.tsx
    performance-tab.tsx
  entity-explorer.css
  entity-explorer.test.tsx
```

**D2. Portal strategy**
Both FAB and Panel are rendered via `ReactDOM.createPortal(element, document.body)`.
The FAB has `position: fixed; bottom: 24px; right: 24px; z-index: var(--entity-explorer-z, 2147483600)`.
Test environments that don't support `position:fixed` (jsdom) are unaffected — the z-index
value is irrelevant to unit tests; the portal placement is what's tested.

**D3. Context shape**
```ts
interface EntityExplorerState {
  open: boolean;
  activeTab: "entities" | "patches" | "events" | "performance";
  selectedEntityId: string | null;
  selectedEntityType: string | null;
}
type Action =
  | { type: "TOGGLE" }
  | { type: "SET_TAB"; tab: EntityExplorerState["activeTab"] }
  | { type: "SELECT_ENTITY"; entityType: string; id: string }
  | { type: "CLEAR_SELECTION" };
```
`useReducer` inside `EntityExplorerProvider`; exposed via `useEntityExplorer()`.

**D4. Tab hidden pattern**
Each tab panel is always mounted after first open. The wrapping `<div>` for each tab panel
gets `style={{ display: activeTab === tabName ? undefined : "none" }}`. No `visibility:hidden`
(it still participates in layout); no conditional render (loses event buffer).

**D5. Events tab bus lifecycle**
`EntityExplorerProvider` calls `createDevtoolsEventBus({ bufferSize: 500, coalesceBurstThreshold: 10 })`
once on mount and passes the bus via context. The bus is destroyed in the provider's cleanup
(`useEffect` return). The EventsTab subscribes to the bus and maintains its own
`useRef<DevtoolsEvent[]>` buffer (max 500) outside React state; only the visible window
slice is in React state to avoid rerendering the full buffer on every event.

**D6. Entity list virtualization**
`useVirtualizer` from `@tanstack/react-virtual` with `estimateSize: () => 32` and
`overscan: 10`. The scroll container is a fixed-height `div` inside the entities tab.
Row data comes from `useStore(useGraphStore, s => s.entities)`.

**D7. Auto-scroll in Events tab**
Track `autoScroll: boolean` in local state (default `true`). On new event: if `autoScroll`,
scroll the list container to top (newest-at-top rendering means index 0 = newest).
On scroll event: if `scrollTop > 8`, set `autoScroll = false` and show resume banner.
On banner click: `setAutoScroll(true)`, scroll to top, clear new-count badge.

**D8. Keyboard shortcut**
`useEffect` in `EntityExplorerProvider` attaches a `keydown` listener to `window`:
```ts
if (e.altKey && e.shiftKey && e.key === "E") dispatch({ type: "TOGGLE" });
```
Cleanup removes the listener. No global singleton — the shortcut lives with the provider.

**D9. CSS scoping**
All CSS variables are prefixed `--ee-` to avoid collision with the host app. The panel root
element gets `class="ee-root"` which scopes all descendant selectors. No CSS Modules (adds
build complexity); plain CSS file imported in the component barrel `index.ts`.

**D10. Commit scope**
```
feat(ui): Entity Explorer FAB + 4-tab panel (W6)
```
Files: `src/ui/entity-explorer/**` (new) + `src/index.ts` (re-exports).

## Risks

- **jsdom limitations in tests** — `ReactDOM.createPortal` works in jsdom; `document.body`
  is available. However `ResizeObserver` (used by TanStack Virtual) is not. Mock it:
  ```ts
  global.ResizeObserver = class { observe() {} unobserve() {} disconnect() {} };
  ```
- **`@testing-library/react` version** — confirm it's a dev dep in `package.json`. If absent,
  add it rather than writing raw vitest DOM tests.
- **Events tab newest-at-top rendering** — list is rendered in reverse (slice.reverse()) before
  passing to the virtualizer. Alternatively, render normally but scroll to bottom; the
  reverse approach is simpler with TanStack Virtual's `scrollToIndex`.
