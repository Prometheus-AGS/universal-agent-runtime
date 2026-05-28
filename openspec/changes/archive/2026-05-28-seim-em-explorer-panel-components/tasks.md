## 1. Setup + dependencies check

- [x] 1.1 Confirm `@testing-library/react` is in `devDependencies`; add if absent: `pnpm add -D @testing-library/react @testing-library/user-event`
- [x] 1.2 Confirm `@tanstack/react-virtual` is a dependency; add if absent: `pnpm add @tanstack/react-virtual`
- [x] 1.3 `mkdir -p ~/.claude/worktrees/seim-entity-management/src/ui/entity-explorer/tabs`

## 2. CSS design system

- [x] 2.1 Create `src/ui/entity-explorer/entity-explorer.css` with all `--ee-*` CSS variables per design D9 + W4 palette (§2): `--ee-bg-shell:#0D1117`, `--ee-bg-surface:#161B22`, `--ee-bg-elevated:#1C2333`, `--ee-text-primary:#E6EDF3`, `--ee-text-muted:#8B949E`, `--ee-text-code:#CDD9E5`, `--ee-accent:#F0A500`, `--ee-border:#30363D`, `--ee-ring:#F0A500`, `--ee-semantic-add:#3FB950`, `--ee-semantic-mod:#D29922`, `--ee-semantic-del:#F85149`
- [x] 2.2 Add base `.ee-root` scope, FAB styles, panel slide-in animation (`@keyframes ee-slide-up`), tab bar styles, tab panel hidden rule (`.ee-tab-panel[data-hidden] { display: none }`), scrollbar styles

## 3. Context

- [x] 3.1 Create `src/ui/entity-explorer/context.tsx`: define `EntityExplorerState`, `Action`, reducer, `EntityExplorerContext`, `EntityExplorerProvider` (includes bus creation + keyboard shortcut), `useEntityExplorer` hook (throws outside provider)

## 4. FAB component

- [x] 4.1 Create `src/ui/entity-explorer/fab.tsx`: `EntityExplorerFAB` renders via `ReactDOM.createPortal` into `document.body`; circular button with amber background; `aria-label`, `aria-expanded`; calls `dispatch({ type: "TOGGLE" })` on click

## 5. Panel shell

- [x] 5.1 Create `src/ui/entity-explorer/panel.tsx`: `EntityExplorerPanel` portal-mounted; only renders content when `open === true`; contains `TabBar` (4 tabs with `role="tablist/tab"`, arrow-key roving tabindex) + `TabContent` (4 always-mounted panels hidden via `display:none`) + `DetailPane` (slide-in on entity selection)

## 6. Tab implementations

- [x] 6.1 Create `src/ui/entity-explorer/tabs/entities-tab.tsx`: `useVirtualizer` list of entities from `useStore(useGraphStore, s => s.entities)`; row click → `dispatch({ type: "SELECT_ENTITY" })`; `ResizeObserver` mock note in comment
- [x] 6.2 Create `src/ui/entity-explorer/tabs/patches-tab.tsx`: list of pending patches from `useStore(useGraphStore, s => s.patches)`; diff display (green +, red -)
- [x] 6.3 Create `src/ui/entity-explorer/tabs/events-tab.tsx`: subscribes to bus from context; maintains `useRef<DevtoolsEvent[]>` ring buffer (max 500); newest-at-top list; auto-scroll logic (D7); amber "N new events" resume banner
- [x] 6.4 Create `src/ui/entity-explorer/tabs/performance-tab.tsx`: ops/sec counter (count events in last 1s rolling window) + last-op latency (ms since last event `at` field)

## 7. Barrel

- [x] 7.1 Create `src/ui/entity-explorer/index.ts`: export `EntityExplorerFAB`, `EntityExplorerPanel`, `EntityExplorerProvider`, `useEntityExplorer`

## 8. Update `src/index.ts`

- [x] 8.1 Add: `export { EntityExplorerFAB, EntityExplorerPanel, EntityExplorerProvider } from "./ui/entity-explorer"`

## 9. Tests

- [x] 9.1 Create `src/ui/entity-explorer/entity-explorer.test.tsx`
- [x] 9.2 Add `beforeAll(() => { global.ResizeObserver = class { observe(){} unobserve(){} disconnect(){} }; })`
- [x] 9.3 Test: FAB renders in `document.body` (portal)
- [x] 9.4 Test: FAB click opens/closes panel (toggle)
- [x] 9.5 Test: `Alt+Shift+E` keyboard shortcut toggles panel
- [x] 9.6 Test: `aria-expanded` matches panel open state
- [x] 9.7 Test: Panel contains 4 tabs with correct labels
- [x] 9.8 Test: Inactive tabs have `display: none` (not removed from DOM)
- [x] 9.9 Test: Tab keyboard navigation (ArrowRight moves focus)
- [x] 9.10 Test: `useEntityExplorer()` throws outside provider with descriptive message
- [x] 9.11 Test: DetailPane absent when no entity selected; present after row click

## 10. Run tests + typecheck

- [x] 10.1 `cd ~/.claude/worktrees/seim-entity-management && pnpm tsc --noEmit` — zero errors
- [x] 10.2 `pnpm test` — all tests pass; suite ≥158 (147 + ≥11 new)

## 11. Commit

- [x] 11.1 `git add src/ui/entity-explorer/ src/index.ts`
- [x] 11.2 `git diff --cached --name-only` — confirm only `src/ui/entity-explorer/**` + `src/index.ts`
- [x] 11.3 Commit:
  ```
  feat(ui): Entity Explorer FAB + 4-tab panel (W6)
  ```

## 12. Progress update

- [x] 12.1 Update `progress.json`: move W6 to `completed_changes`, `changes_completed: 8`, `active_change: "seim-em-explorer-production-treeshake-check"`, add commit sha, update `updatedAt`
