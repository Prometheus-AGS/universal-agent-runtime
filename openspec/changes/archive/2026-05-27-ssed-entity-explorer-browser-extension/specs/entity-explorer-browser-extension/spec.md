## ADDED Requirements

### Requirement: Extension Package Surface
The `prometheus-entity-management` repo SHALL ship a `chrome-extension/` package containing a valid Chrome MV3 manifest, a devtools page entry, a content script, a page-injected hook, a panel host, and a bridge module.

#### Scenario: Manifest valid
- **WHEN** the `chrome-extension/manifest.json` is loaded into Chrome via `chrome://extensions` Developer Mode
- **THEN** it MUST load without errors, MUST declare `manifest_version: 3`, MUST declare `devtools_page`, and MUST declare a content script matching `<all_urls>`.

#### Scenario: Package structure
- **WHEN** the package is inspected
- **THEN** the documented `src/` files MUST all exist (devtools-page, panel, content-script, page-hook, bridge/{envelope,page-to-extension,extension-to-page}) and the package MUST build to a loadable extension via `pnpm build`.

#### Scenario: Architecture notes present
- **WHEN** the package is inspected
- **THEN** `chrome-extension/docs/architecture-notes.md` MUST exist with at minimum: the bridge data-flow diagram, MV3 manifest rationale, security model, and web-search references (URLs + fetch dates) for the three pre-flight queries.

### Requirement: Page-Side Hook
The host application's `EntityExplorerFab` (from change 10) SHALL install a `window.__PROMETHEUS_DEVTOOLS_HOOK__` global in dev mode.

#### Scenario: Hook installed
- **WHEN** `<EntityExplorerFab>` mounts in dev mode
- **THEN** `window.__PROMETHEUS_DEVTOOLS_HOOK__` MUST be defined with the documented surface (`isExtensionPresent`, `onMessage`, `postMessage`, `getSnapshot`, `getEventBuffer`).

#### Scenario: Hook absent in production
- **WHEN** `<EntityExplorerFab>` mounts under production build
- **THEN** `window.__PROMETHEUS_DEVTOOLS_HOOK__` MUST NOT be defined; tree-shaking removes the installation code.

#### Scenario: Presence signalling
- **WHEN** the content script detects the hook on a page
- **THEN** it MUST dispatch a `__PROMETHEUS_DEVTOOLS_PRESENT__` CustomEvent on the page, and the hook MUST set `isExtensionPresent = true` upon receiving it.

### Requirement: Bridge Envelope
All page-↔-extension messages SHALL use a typed envelope with a `__PROMETHEUS_DEVTOOLS_HOOK__: true` discriminator and a `nonce` for request/response matching.

#### Scenario: Envelope discriminator
- **WHEN** the content script or extension receives any `window.postMessage` event
- **THEN** it MUST ignore messages whose payload does not contain `__PROMETHEUS_DEVTOOLS_HOOK__: true`.

#### Scenario: Origin verification
- **WHEN** the content script receives a hook message from the page
- **THEN** it MUST verify `event.source === window` AND ignore messages from cross-origin iframes (no `parent` / `top` propagation).

#### Scenario: Envelope types complete
- **WHEN** the bridge types are inspected
- **THEN** they MUST cover: `page→ext/init`, `page→ext/changeSet`, `page→ext/event`, `ext→page/promoteToCanonical`, `ext→page/clearBuffer`, `ext→page/registerSubscriber`.

#### Scenario: Nonce matching
- **WHEN** the extension sends a request-style envelope expecting a response
- **THEN** the envelope MUST carry a unique `nonce`; the page-side handler MUST echo the same `nonce` in its response; un-matched responses MUST be dropped.

### Requirement: Panel Reuse via DevtoolsDataSource
The Entity Explorer panel components from change 10 SHALL accept a `DevtoolsDataSource` prop or context value so the same components render in both the in-app FAB host and the extension panel.

#### Scenario: In-app data source
- **WHEN** the panel renders inside `<EntityExplorerFab>` in the host app
- **THEN** the data source MUST be the in-process implementation reading from `useGraphStore` + the in-process event bus.

#### Scenario: Extension data source
- **WHEN** the panel renders inside the extension's `panel.tsx`
- **THEN** the data source MUST be a bridge-backed implementation that calls into the content script for graph snapshots and subscribes to envelope-relayed events.

#### Scenario: Same components, same behavior
- **WHEN** either data source is in use
- **THEN** the rendered tabs (Tree / Inspector / Events / Stores / Duplicates) MUST present identical interfaces with the same row content and filters; the difference is the latency and origin of the data, not the UX.

### Requirement: Promote-to-Canonical Bridging
The extension SHALL be able to trigger the host app's "Promote to canonical" mutation via the bridge.

#### Scenario: Promote envelope
- **WHEN** a user clicks "Promote to canonical" on a duplicate row in the extension panel
- **THEN** the extension MUST send an `ext→page/promoteToCanonical` envelope; the page-side handler MUST call into the host app's engine via the same path used by the in-app button (change 10).

#### Scenario: Confirmation
- **WHEN** the host app completes the promotion
- **THEN** it MUST send a `page→ext/event` envelope containing the resulting `ChangeSet` so the extension panel reflects the new canonical state.

### Requirement: Security Boundaries
The extension SHALL enforce documented security boundaries.

#### Scenario: Origin restriction at the bridge
- **WHEN** the content script forwards a page message to the extension background
- **THEN** it MUST attach the originating page URL; the devtools panel MUST be able to filter / display the origin so an operator can see what page they're inspecting.

#### Scenario: No cross-tab leakage
- **WHEN** the extension is open on tab A and the host app on tab B emits events
- **THEN** tab A's panel MUST NOT receive tab B's events; the bridge MUST scope by tabId.

### Requirement: Pre-flight Research Doc
The implementation SHALL produce `chrome-extension/docs/architecture-notes.md` BEFORE production code, summarising web-search references and design decisions.

#### Scenario: File at HEAD
- **WHEN** the change is applied
- **THEN** the notes file MUST exist with sections: bridge data-flow diagram, MV3 manifest rationale, security model, and URLs + fetch dates for the three required web-search queries ("Chrome MV3 devtools panel patterns 2026", "react-devtools bridge architecture", "postMessage origin validation security best practices").

### Requirement: Production Build Excludes Extension Code
Production builds of `prometheus-entity-management` SHALL NOT ship any code from the `chrome-extension/` package or include the page-side hook installation.

#### Scenario: Bundle inspection
- **WHEN** a production bundle is analysed
- **THEN** no symbols from `chrome-extension/src/*` MUST appear; the `__PROMETHEUS_DEVTOOLS_HOOK__` installation code MUST be tree-shaken (guarded by `process.env.NODE_ENV !== "production"`).
