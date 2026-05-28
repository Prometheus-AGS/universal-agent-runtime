# Spec: chrome-mv3-extension-architecture

## Goal
Document the Chrome MV3 architecture pattern needed to surface
`DevtoolsEventBus` events in a Chrome DevTools panel.

## Requirement 1 — Message Bridge Architecture

The Chrome MV3 extension must implement a 4-layer message bridge:

```
[Inspected Page]           [Content Script]          [Service Worker]       [DevTools Panel]
DevtoolsEventBus  ──postMessage──>  relay listener  ──chrome.runtime.sendMessage──>  display
```

Layer responsibilities:
1. **Inspected Page**: The host app calls `registerStore()` and emits
   `DevtoolsEvent` objects. A content-script injection must subscribe to
   the bus's `subscribe()` method OR listen to a `window.postMessage`
   channel the page exposes.
2. **Content Script**: Receives events from the page via `window.addEventListener("message")`
   and forwards to the service worker via `chrome.runtime.sendMessage`.
3. **Service Worker (background)**: Routes messages from content scripts to any
   open devtools panels via `chrome.runtime.connect` port messaging.
4. **DevTools Panel**: Opened via `chrome.devtools.panels.create()`; receives
   events and renders the EntityExplorerPanel in a React root.

### Scenario: Panel opens after page load
Given: The inspected page already has a running DevtoolsEventBus
When: The user opens the DevTools panel
Then: The panel receives a replay of the ring buffer (up to 500 events) on connect

### Scenario: Panel closed
Given: DevTools panel is closed
When: The inspected page emits events
Then: Events are silently dropped (no memory accumulation in the service worker)

## Requirement 2 — Page-side Bridge Injection

The content script cannot directly access in-page JS objects (different worlds).
Two options:
- **Option A** (preferred): Inject a tiny `page-bridge.js` script into the
  `MAIN` world via `chrome.scripting.executeScript` with `world: "MAIN"`.
  This script patches `window.__entityExplorerBus` = the registered bus, then
  calls `subscribe()` and posts each event via `window.postMessage`.
- **Option B**: The library exposes a `window.postMessage` channel natively
  (add a `enableWindowBridge()` call to `EntityExplorerProvider`).

Option B is cleaner for the library — no MAIN-world injection needed. The
content script just listens for `{ type: "__entity_explorer_event__", payload }`.

## Requirement 3 — MV3 Constraints

- Service worker is ephemeral (sleeps after 30s of inactivity in MV3)
- Use `chrome.devtools.inspectedWindow.eval()` for synchronous page queries
- `chrome.runtime.connect()` ports keep the worker alive while open
- DevTools panel page is a full HTML page; React and the library can be
  bundled directly into it (no CDN)

## Requirement 4 — DevtoolsEventBus adaptation

For the extension context, `createDevtoolsEventBus` needs no changes.
The panel-side `EntityExplorerProvider` uses the same bus, fed by events
arriving over the Chrome messaging bridge instead of direct `registerStore` calls.

A thin `createExtensionBus()` factory wraps `createDevtoolsEventBus` and
subscribes to `chrome.runtime.onConnect` / port messages to inject events.
