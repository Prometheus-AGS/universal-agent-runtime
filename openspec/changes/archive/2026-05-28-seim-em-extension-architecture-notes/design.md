# Design: chrome-mv3-extension-architecture

## Decision 1 — window.postMessage bridge (Option B)

`EntityExplorerProvider` gains an optional `enableWindowBridge?: boolean` prop.
When true, the provider calls `window.postMessage({ type: "__entity_explorer_event__", payload }, "*")`
for every event the bus receives. This keeps the library self-contained and eliminates
the need for MAIN-world script injection.

The content script:
```js
window.addEventListener("message", (e) => {
  if (e.data?.type === "__entity_explorer_event__") {
    chrome.runtime.sendMessage({ event: e.data.payload });
  }
});
```

## Decision 2 — No service worker event buffering

The service worker does not buffer events. If the DevTools panel is not open,
events are dropped. The ring buffer lives in the panel-side DevtoolsEventBus.
On panel open, the bus is empty and fills from that point forward.

This avoids MV3 worker sleeping issues entirely.

## Decision 3 — Panel bundle

The DevTools panel is a standalone `panel.html` + bundled React app.
It imports `EntityExplorerProvider`, `EntityExplorerPanel` from the library,
and a thin `createExtensionBus()` from a new `src/extension/` directory.

## File layout for W8 scaffold
```
extension/
  manifest.json         (MV3)
  background.js         (service worker — port relay only)
  content.js            (window.message → chrome.runtime relay)
  devtools.html         (devtools page that creates the panel)
  devtools.js           (calls chrome.devtools.panels.create)
  panel.html            (React root for the explorer panel)
  panel.js              (bundled: EntityExplorerProvider + createExtensionBus)
```

## Implementation steps for W7b (docs only)
1. Write `docs/extension-architecture-notes.md` summarizing this design
2. Commit
