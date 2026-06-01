## Why

W6 implements the visible Entity Explorer UI: the FAB button and the 4-tab devtools panel
that it reveals. The W4 UI/UX preflight (`docs/devtools-design-notes.md`) and W5 event bus
(`createDevtoolsEventBus`) are prerequisites — both are now complete.

Per the W4 distillation (§8 + §10), the panel launches with **4 tabs** (not 5): Entities,
Patches, Events, Performance. The Subscriptions concept is folded into Events as a filter
(`kind:"subscribe"/"unsubscribe"` events), reducing initial complexity without data loss.

The implementation follows the component tree mandated by design notes §5 + §10:
- `EntityExplorerProvider` — React context holding `{ activeTab, selectedEntityId, dispatch }`
- `EntityExplorerFAB` — portal-mounted floating action button, `z-index: 2147483600`
- `EntityExplorerPanel` — portal-mounted container with `TabBar` + `TabContent` + `DetailPane`
- All 4 tab panels mounted on first FAB click; inactive tabs hidden via `display: none`
  (preserves event buffer and scroll position)
- Entity list virtualized via `@tanstack/react-virtual`
- FAB → panel reveal: `200ms ease-out` slide-up, no bounce

## What Changes

**New directory:** `src/ui/entity-explorer/`

```
src/ui/entity-explorer/
  index.ts                   ← barrel (re-exports EntityExplorerFAB, EntityExplorerPanel)
  context.tsx                ← EntityExplorerProvider + useEntityExplorer hook
  fab.tsx                    ← FAB button (portal, keyboard shortcut Alt+Shift+E)
  panel.tsx                  ← Panel shell (portal, TabBar, TabContent, DetailPane)
  tabs/
    entities-tab.tsx         ← virtualized entity list + detail pane
    patches-tab.tsx          ← pending patch queue with diff view
    events-tab.tsx           ← live DevtoolsEvent stream (ring buffer replay + auto-scroll)
    performance-tab.tsx      ← ops/sec counter + last-op latency
  entity-explorer.css        ← scoped CSS variables (palette, typography, spacing)
```

**Modified:** `src/index.ts` — re-export `EntityExplorerFAB` and `EntityExplorerPanel`.

No new test infrastructure beyond what vitest already provides. Component tests use
`@testing-library/react` (already a dev dep).

## Capabilities

### New Capabilities

- **`entity-explorer-panel`**: A React portal-mounted devtools panel with FAB toggle,
  4-tab layout (Entities / Patches / Events / Performance), virtualized entity list,
  live event stream with ring-buffer replay, and keyboard shortcut (`Alt+Shift+E`).
  Designed per the W4 preflight: amber accent on `#0D1117` stack, Geist Mono data text,
  IBM Plex Sans chrome. Consumes the W5 `DevtoolsEventBus` for the Events tab.

### Modified Capabilities

- **`src/index.ts`**: Re-exports only.
