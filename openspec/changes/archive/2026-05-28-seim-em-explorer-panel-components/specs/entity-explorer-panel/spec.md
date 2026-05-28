# entity-explorer-panel Specification

## Purpose

A React component kit that renders the Entity Explorer devtools surface: a floating action
button (FAB) that reveals a 4-tab panel (Entities, Patches, Events, Performance) mounted as
a React portal into `document.body`. The panel consumes `useGraphStore` for entity/patch
data and `createDevtoolsEventBus` (W5) for the live event stream. All design decisions derive
from `docs/devtools-design-notes.md` (W4).

## Requirements

### Requirement: FAB Component

`EntityExplorerFAB` SHALL be a React component that renders a circular floating action button
portal-mounted into `document.body` with `z-index: var(--entity-explorer-z, 2147483600)`.

#### Scenario: FAB renders in document.body
- **WHEN** `<EntityExplorerFAB />` is mounted anywhere in the React tree
- **THEN** the rendered button MUST appear as a direct child of `document.body`, not of the
  mount point's DOM parent.

#### Scenario: FAB toggles panel open/closed
- **WHEN** the FAB button is clicked
- **THEN** `EntityExplorerPanel` MUST become visible; clicking again MUST hide it.

#### Scenario: FAB has accessible label and expanded state
- **WHEN** the FAB is inspected
- **THEN** it MUST have `aria-label="Open Entity Explorer"` (or "Close Entity Explorer" when
  open) and `aria-expanded={boolean}` reflecting panel state.

#### Scenario: Keyboard shortcut Alt+Shift+E
- **WHEN** `keydown` fires with `altKey && shiftKey && key === "E"` on `window`
- **THEN** the panel MUST toggle (open if closed, close if open), matching a FAB click.

---

### Requirement: Panel Shell

`EntityExplorerPanel` SHALL be a React portal mounted into `document.body`, visible only
when the FAB's state is open.

#### Scenario: Panel mounts in document.body
- **WHEN** the panel is open
- **THEN** the panel root element MUST be a direct child of `document.body`.

#### Scenario: Panel slide-in animation
- **WHEN** the panel transitions from closed to open
- **THEN** the CSS animation MUST be `200ms ease-out` slide-up; no bounce or spring physics.

#### Scenario: Panel contains TabBar and 4 tab panels
- **WHEN** the panel is open
- **THEN** it MUST contain a `role="tablist"` element with exactly 4 `role="tab"` children
  (Entities, Patches, Events, Performance).

---

### Requirement: Tab Architecture

All 4 tab panels SHALL be mounted on first FAB open and SHALL remain mounted for the
lifetime of the panel. Inactive tabs MUST be hidden via CSS (`display: none`), not unmounted.

#### Scenario: Inactive tabs are hidden, not removed
- **WHEN** the Patches tab is active and the DOM is inspected
- **THEN** the Entities, Events, and Performance tab panels MUST exist in the DOM with
  `display: none`; they MUST NOT be absent from the DOM.

#### Scenario: Tab keyboard navigation
- **WHEN** focus is on a `role="tab"` and `ArrowRight` / `ArrowLeft` is pressed
- **THEN** focus MUST move to the next/previous tab per the ARIA Tabs pattern (roving tabindex).

#### Scenario: Tab panel ARIA wiring
- **WHEN** a tab is selected
- **THEN** the corresponding `role="tabpanel"` MUST be visible and MUST have
  `aria-labelledby` pointing to the active tab's `id`.

---

### Requirement: Entities Tab

The Entities tab SHALL render a virtualized list of all entity types and their records,
using `@tanstack/react-virtual`.

#### Scenario: List is virtualized
- **WHEN** the Entities tab renders with >50 entities
- **THEN** only the visible rows MUST be rendered in the DOM; all others MUST be in
  virtual space (standard TanStack Virtual behaviour).

#### Scenario: Row click opens DetailPane
- **WHEN** a row in the entity list is clicked
- **THEN** a detail pane MUST slide in showing the entity's full data object.

#### Scenario: DetailPane is hidden when no entity selected
- **WHEN** no entity is selected
- **THEN** the detail pane MUST NOT be present in the DOM (or MUST have `display: none`).

---

### Requirement: Events Tab

The Events tab SHALL render the `DevtoolsEventBus` ring buffer as an event stream, newest
at top, with auto-scroll and a "N new events" resume banner.

#### Scenario: Initial render shows buffered events
- **WHEN** the Events tab mounts
- **THEN** it MUST display the events already in `bus.getBuffer()` from oldest to newest
  (scroll position at bottom = newest).

#### Scenario: New events appear at top of list
- **WHEN** a new event fires while the Events tab is active
- **THEN** the event MUST appear at the top of the list.

#### Scenario: Auto-scroll pauses when user scrolls up
- **WHEN** the user scrolls up in the Events list
- **THEN** auto-scroll MUST pause and an amber "N new events — click to resume" banner
  MUST appear.

#### Scenario: Resume banner re-enables auto-scroll
- **WHEN** the resume banner is clicked
- **THEN** auto-scroll MUST re-enable and the list MUST scroll to the newest event.

---

### Requirement: Context and Provider

`EntityExplorerProvider` SHALL wrap both `EntityExplorerFAB` and `EntityExplorerPanel`
and provide shared state via React context.

#### Scenario: Context provides activeTab and selectedEntityId
- **WHEN** `useEntityExplorer()` is called inside the provider tree
- **THEN** it MUST return `{ activeTab, selectedEntityId, dispatch }` without throwing.

#### Scenario: Context throws outside provider
- **WHEN** `useEntityExplorer()` is called outside an `EntityExplorerProvider`
- **THEN** it MUST throw an error with a descriptive message.

---

### Requirement: CSS Design System

`entity-explorer.css` SHALL define the CSS custom properties matching the W4 palette.

#### Scenario: Required CSS variables present
- **WHEN** `entity-explorer.css` is parsed
- **THEN** it MUST define at minimum: `--ee-bg-shell`, `--ee-bg-surface`, `--ee-bg-elevated`,
  `--ee-text-primary`, `--ee-text-muted`, `--ee-text-code`, `--ee-accent`, `--ee-border`,
  `--ee-ring`, `--ee-semantic-add`, `--ee-semantic-mod`, `--ee-semantic-del`.

---

### Requirement: Public Exports

`EntityExplorerFAB` and `EntityExplorerPanel` SHALL be exported from `src/index.ts`.

#### Scenario: Exports resolve
- **WHEN** a consumer imports `{ EntityExplorerFAB, EntityExplorerPanel }` from the package
- **THEN** both MUST resolve to React component functions.

---

### Requirement: Test Coverage

#### Scenario: Component tests exist
- **WHEN** the test surface is inspected
- **THEN** `src/ui/entity-explorer/entity-explorer.test.tsx` MUST exist covering: FAB portal
  rendering, FAB toggle, keyboard shortcut, tab switching, inactive tab `display:none`,
  context throw outside provider — minimum 10 `it` blocks.
