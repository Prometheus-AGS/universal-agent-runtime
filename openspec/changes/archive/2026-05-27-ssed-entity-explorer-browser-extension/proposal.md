## Why

The phase plan flagged the browser extension as a **stretch / v2** deliverable contingent on user confirmation. The user's `/kbd-execute` directive said "execute … without asking unless there is an error or questions or problems" — that opens the door to scaffolding without an explicit pre-execute confirmation, but the underlying scope (Chrome MV3 + content script + devtools panel + page-↔-extension bridge) is real engineering work.

Treating it as a scaffold: this change ships **the architecture, the manifest, the bridge contract, and the empty panel package**, but defers the substantial React UI to a focused implementation session (mirroring the pattern used for changes 9 and 10). The OpenSpec artifacts here are the contract; the panel UI is the change 10 implementation re-housed in extension chrome.

The motivation from the user: a developer-facing extension that observes any host app importing `@prometheus-ags/prometheus-entity-management` and renders the same five-tab explorer in the browser DevTools alongside Elements / Network / etc. — like React Devtools, but for the entity graph.

## What Changes

### New package — `prometheus-entity-management/chrome-extension/`

Chrome MV3 extension at a sibling package path in the entity-management repo. Structure:

```
chrome-extension/
├── manifest.json                  — MV3 manifest, devtools_page entry
├── package.json                   — its own scripts (vite, tsup)
├── tsconfig.json
├── src/
│   ├── devtools-page.html         — invisible page that registers panels
│   ├── devtools-page.ts           — `chrome.devtools.panels.create(...)`
│   ├── panel.html                 — the panel UI host
│   ├── panel.tsx                  — entry; mounts EntityExplorerPanel from the main package
│   ├── content-script.ts          — injected into every page; relays messages
│   ├── page-hook.ts               — injected via MAIN world; defines window.__PROMETHEUS_DEVTOOLS_HOOK__
│   └── bridge/
│       ├── envelope.ts            — message envelope types
│       ├── page-to-extension.ts   — page-side bridge (used by host app)
│       └── extension-to-page.ts   — extension-side receiver
├── public/
│   └── icons/                     — extension icon assets
└── README.md
```

### Page-side hook — `__PROMETHEUS_DEVTOOLS_HOOK__`

A small page-side global (analogous to React Devtools'  `__REACT_DEVTOOLS_GLOBAL_HOOK__`) that the main package's `EntityExplorerFab` (change 10) installs automatically when the extension is detected. Surface:

```ts
declare global {
  interface Window {
    __PROMETHEUS_DEVTOOLS_HOOK__?: {
      isExtensionPresent: boolean;
      onMessage(cb: (msg: BridgeEnvelope) => void): () => void;
      postMessage(msg: BridgeEnvelope): void;
      getSnapshot(): EntityGraphSnapshot;     // current entities, lists, registered stores
      getEventBuffer(): DevtoolsEvent[];      // tail of the dev-mode event bus
    };
  }
}
```

The extension's content-script polls for the hook on page load; the host app sets `isExtensionPresent = true` when it detects the extension's presence via a CustomEvent.

### Message envelope

```ts
type BridgeEnvelope =
  | { type: "page→ext/init"; tabId?: number; }
  | { type: "page→ext/changeSet"; changes: EntityChange[]; affectedListKeys: string[]; ts: string; }
  | { type: "page→ext/event"; event: DevtoolsEvent; }
  | { type: "ext→page/promoteToCanonical"; entityType: string; entityId: string; chosenStoreId: string; }
  | { type: "ext→page/clearBuffer"; }
  | { type: "ext→page/registerSubscriber"; entityType: string; entityId: string; }   // open Inspector in extension panel
```

All envelopes carry a `nonce` for matching requests/responses.

### Bridge wiring

```
host app page
  └── window.__PROMETHEUS_DEVTOOLS_HOOK__   ← installed by EntityExplorerFab
        ↕ window.postMessage(envelope, '*')
  └── content-script.ts                     ← extension MV3 content script
        ↕ chrome.runtime.sendMessage
  └── devtools-page.ts                      ← extension devtools page
        ↕ chrome.runtime.onMessage
  └── panel.tsx                             ← renders EntityExplorerPanel (from main pkg)
```

`postMessage` origin is verified at both ends; the extension only accepts messages with `__PROMETHEUS_DEVTOOLS_HOOK__: true` discriminator.

### Manifest (MV3 outline)

```json
{
  "manifest_version": 3,
  "name": "Prometheus Entity Devtools",
  "version": "0.1.0",
  "description": "Inspect Prometheus entity graphs from Chrome DevTools.",
  "devtools_page": "devtools-page.html",
  "permissions": ["scripting"],
  "host_permissions": ["<all_urls>"],
  "content_scripts": [{
    "matches": ["<all_urls>"],
    "js": ["content-script.js"],
    "run_at": "document_start"
  }],
  "icons": { "16": "icons/16.png", "48": "icons/48.png", "128": "icons/128.png" }
}
```

### Panel reuse

`panel.tsx` imports `EntityExplorerPanel` (and friends) from `@prometheus-ags/prometheus-entity-management`. The panel UI is identical to in-app (change 10) — only the data source differs (bridge envelopes instead of in-process subscriptions). A `DevtoolsDataSource` abstraction (added in change 10's `src/devtools/data-source.ts`) lets the same components work against either source.

### Package isolation

The extension is its own pnpm workspace package. It depends on `@prometheus-ags/prometheus-entity-management` for the panel components and on `react` / `react-dom`. The host app does NOT take a dep on the extension package; the extension imports the host's published bundle.

### Pre-flight (per change 8 routing discipline)

- Web search "Chrome MV3 devtools panel patterns 2026".
- Web search "react-devtools bridge architecture".
- Web search "postMessage origin validation security best practices".
- Document findings in `chrome-extension/docs/architecture-notes.md`.

### Non-changes

- **No host app modification** beyond change 10's hook installation. The extension is a passive observer.
- **No write capability** on the page except for the explicit "Promote to canonical" envelope (which goes through the host app's engine, same as change 10's in-app button).
- **No published Chrome Web Store release.** Side-loaded for dev only at first.
- **No Firefox / Edge port** in this change.

## Capabilities

### New Capabilities

- `entity-explorer-browser-extension`: A Chrome MV3 extension that observes any page importing `@prometheus-ags/prometheus-entity-management`, communicates via a `__PROMETHEUS_DEVTOOLS_HOOK__` page-side global + `postMessage` bridge, and renders the same five-tab Entity Explorer panel in Chrome DevTools that change 10's `<EntityExplorerFab>` renders in-app. Reuses `EntityExplorerPanel` from the main package via a shared `DevtoolsDataSource` abstraction.

### Modified Capabilities

- `entity-explorer-fab-panel` (change 10): gains a `DevtoolsDataSource` abstraction so the same panel components serve both in-app and extension data sources. The in-app default and behavior is unchanged.

## Impact

- **Risk**: Medium-high. Browser extension surface is a long-lived maintenance burden (manifest changes, permission policy shifts, panel API differences across Chromium versions).
- **Affected files**:
  - `prometheus-entity-management/chrome-extension/` (new package).
  - `prometheus-entity-management/src/devtools/EntityExplorerFab.tsx` (change 10) — gains hook installation logic.
  - `prometheus-entity-management/src/devtools/data-source.ts` (new in change 10's tasks; this change finalises the contract).
- **Cross-repo**: Just `prometheus-entity-management` for the extension itself; the panel reuse means the entity-management repo carries both halves.
- **Reversibility**: Trivial — delete the `chrome-extension/` package and the hook installation block; rest of the explorer keeps working.
- **Stretch status**: This change ships **the manifest, bridge contract, and scaffolding only**. Panel UI work IS change 10. Production-readiness of the extension (Web Store publication, cross-browser polyfill, etc.) is reserved for a later phase.
