## Context

Change 10 ships the in-app explorer + a `DevtoolsDataSource` seam. This change leverages that seam so the same panel components run in a Chrome MV3 extension's devtools panel. The pattern is well-established (React Devtools, Redux Devtools, Apollo Devtools) — we follow conventions rather than invent.

## Goals / Non-Goals

**Goals**
- Working Chrome MV3 extension that loads via `chrome://extensions` and registers a "Entity Graph" devtools panel.
- Page-↔-extension bridge over `window.postMessage` + `chrome.runtime`.
- Panel UI reuse from change 10 via `DevtoolsDataSource`.
- Per-tab scoping (no cross-tab leakage).
- Documented security model.

**Non-Goals**
- No Web Store publication. Dev side-load only.
- No Firefox or Edge port.
- No write-side capabilities beyond "Promote to canonical" (which bridges to the host app's existing engine).
- No data persistence in the extension. Refresh = lose state. Acceptable for dev.
- No production-grade observability (Sentry / etc.).

## Decisions

### D1. MV3 over MV2

MV2 is deprecated and removed from new Chrome installs. MV3 uses service workers instead of background pages but the devtools panel API is identical. We pay the MV3 cost up front.

### D2. Two-stage script injection: content script + page hook

`content_script` runs in an isolated world; the host app's `__PROMETHEUS_DEVTOOLS_HOOK__` lives in the MAIN world. We inject `page-hook.ts` via `chrome.scripting.executeScript({ world: "MAIN" })` from the content script. The content script then bridges between the page (via `window.postMessage`) and the extension background (via `chrome.runtime.sendMessage`).

This pattern is canonical — same as React Devtools, Apollo Devtools.

### D3. Discriminator + nonce + origin

Three layers of message hygiene:
1. **Discriminator**: every envelope carries `__PROMETHEUS_DEVTOOLS_HOOK__: true`. Other postMessages are ignored.
2. **Nonce**: request-style envelopes carry a unique nonce; responses echo it. Drops un-matched responses, prevents replay confusion.
3. **Origin**: content script attaches the page URL on every forward; the panel can display + filter on origin.

### D4. Per-tab scoping via tabId

The background service worker tracks connections per `chrome.devtools.inspectedWindow.tabId`. Messages from tab A never reach a panel inspecting tab B. The bridge envelope carries `tabId` (extension-side) so the routing is explicit.

### D5. DevtoolsDataSource abstraction

Defined in change 10. Two concrete implementations:
- `InProcessDataSource` — reads `useGraphStore` directly, subscribes to in-process bus.
- `BridgeDataSource` — reads via envelope round-trips. `getSnapshot()` is a synchronous-feeling API backed by a request-response handshake on connect.

Panel components consume the abstraction; neither knows whether it's running in-app or in the extension.

### D6. Hook installation is dev-only

The `__PROMETHEUS_DEVTOOLS_HOOK__` is installed inside `EntityExplorerFab`'s effect (which is itself dev-only per change 10 spec). Production tree-shakes the installation. Verifiable via bundle analysis.

### D7. No package interdependency in production

The host app depends on `@prometheus-ags/prometheus-entity-management`. The extension also depends on it (as a sibling pnpm workspace). The host app does NOT depend on the extension — the extension is observe-only from the page's perspective.

### D8. Architecture notes committed before code

Per change 8's routing discipline, `chrome-extension/docs/architecture-notes.md` lands first with the web-search synthesis. The implementation references the doc.

### D9. Stretch scope honesty

This change ships: manifest, bridge contract, scaffolding directories, and architecture notes. It does NOT ship a polished panel UX, a tested extension, or a published Web Store listing. The README documents the side-load procedure for developers.

## Implementation Sketch

### `manifest.json`

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

### `content-script.ts`

```ts
// 1. Inject page-hook into MAIN world.
import { scripting } from "chrome";
chrome.scripting.executeScript({ target: { tabId: chrome.devtools.inspectedWindow.tabId }, world: "MAIN", files: ["page-hook.js"] });

// 2. Relay page → extension.
window.addEventListener("message", (e) => {
  if (e.source !== window) return;
  if (!e.data || e.data.__PROMETHEUS_DEVTOOLS_HOOK__ !== true) return;
  chrome.runtime.sendMessage({ ...e.data, origin: location.href });
});

// 3. Relay extension → page.
chrome.runtime.onMessage.addListener((msg) => {
  if (!msg || msg.__PROMETHEUS_DEVTOOLS_HOOK__ !== true) return;
  window.postMessage(msg, "*");
});
```

### `panel.tsx`

```tsx
import { EntityExplorerPanel } from "@prometheus-ags/prometheus-entity-management";
import { createBridgeDataSource } from "./bridge/extension-to-page";

const dataSource = createBridgeDataSource({ tabId: chrome.devtools.inspectedWindow.tabId });

ReactDOM.createRoot(document.getElementById("root")!).render(
  <EntityExplorerPanel dataSource={dataSource} />,
);
```

### `page-hook.ts` (injected into MAIN world)

```ts
declare global {
  interface Window { __PROMETHEUS_DEVTOOLS_HOOK__?: PrometheusDevtoolsHook; }
}

const subscribers = new Set<(m: BridgeEnvelope) => void>();

window.__PROMETHEUS_DEVTOOLS_HOOK__ = {
  isExtensionPresent: false,
  onMessage(cb) { subscribers.add(cb); return () => subscribers.delete(cb); },
  postMessage(msg) { window.postMessage({ ...msg, __PROMETHEUS_DEVTOOLS_HOOK__: true }, "*"); },
  getSnapshot() { /* dev-mode getter from the host app */ return /*…*/; },
  getEventBuffer() { return /*…*/; },
};

window.addEventListener("__PROMETHEUS_DEVTOOLS_PRESENT__", () => {
  window.__PROMETHEUS_DEVTOOLS_HOOK__!.isExtensionPresent = true;
});

window.addEventListener("message", (e) => {
  if (e.source !== window) return;
  if (!e.data || e.data.__PROMETHEUS_DEVTOOLS_HOOK__ !== true) return;
  for (const cb of subscribers) cb(e.data);
});
```

### `chrome-extension/docs/architecture-notes.md` (committed first)

Sections: bridge data-flow ASCII diagram, MV3 manifest rationale, security model (D3 summary), pre-flight web-search references with URLs + fetch dates, decision log mirroring D1–D9.

## Risks

1. **MV3 API churn.** Chrome's MV3 surface is still evolving; the `scripting.executeScript` shape has changed. Pin to a tested Chrome version range in README.
2. **Hook installation timing.** Page-hook script needs to run BEFORE the host app's first React render so the hook is available when `EntityExplorerFab` checks for it. We rely on `run_at: "document_start"` + `world: MAIN`.
3. **Snapshot cost.** `getSnapshot()` may be large (10k+ entities); transferring on every panel render is expensive. Mitigation: incremental updates via `page→ext/changeSet` envelopes; snapshot only on initial connect.
4. **Cross-tab confusion.** Without strict tabId scoping (D4), an operator inspecting two tabs side-by-side would see merged data. Enforce in the background service worker.
5. **Stretch scope creep.** Easy to over-build. The spec calls out what's in scope (manifest, bridge contract, scaffolding, notes) and what's out (panel UX polish, Web Store, cross-browser, persistence).

## Alternatives Considered

- **MV2.** Rejected — deprecated.
- **WebSocket bridge instead of postMessage.** Rejected — requires a local dev server; postMessage is zero-config.
- **Ship a separate panel implementation.** Rejected — duplicates change 10's work; `DevtoolsDataSource` abstraction lets us share.
- **Cross-browser polyfill (Edge / Firefox).** Rejected for this stretch — focused dev tool, Firefox MV3 isn't yet stable.
