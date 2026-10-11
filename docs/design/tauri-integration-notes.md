# Tauri integration notes — UAR operator shell

How the prototype's shell maps onto the existing `src-tauri/` build, and what to change there
before relying on the desktop layout. Observations about the current config come from
`src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json` and `frontend/src/components/Titlebar.tsx`.

## 1. Window

| Setting | Today | Change | Why |
|---|---|---|---|
| `width` × `height` | 800 × 600 | 1280 × 800 | 800px is under the 900px mobile breakpoint, so the desktop app launches in the phone layout with a bottom tab bar. |
| `minWidth` × `minHeight` | unset | 960 × 600 | Keeps the sidebar rail + thread + inspector composable; below 1180px the inspector becomes a slide-over anyway. |
| `decorations` | `false` with hand-drawn traffic lights | macOS: `decorations: true`, `titleBarStyle: "Overlay"`, `hiddenTitle: true`; Windows/Linux: keep `decorations: false` with custom controls | Native traffic lights, native double-click-to-zoom, and the sidebar runs the full window height under them (the ChatGPT / Claude / Codex look). |
| `title` | "KnowMe — Universal Agent Runtime" | "Universal Agent Runtime" | Template leftover. |
| `identifier` | `com.tauri.dev` | `ai.prometheusags.uar` (or the org's reverse-DNS) | Required for signing, updater, and keychain entries. |
| `center` | unset | `true` | First launch lands centered. |
| `security.csp` | `null` | `default-src 'self'; connect-src 'self' http://127.0.0.1:* ws://127.0.0.1:*; img-src 'self' data:; font-src 'self'` | Fonts are bundled locally (see `assets/fonts/`), so no remote origins are needed. |

## 2. Titlebar and drag region

The prototype adds `.is-tauri` to `<html>` when `window.__TAURI__` or `window.__TAURI_INTERNALS__`
is present, which sets `--titlebar-h: 38px` and shows a `.titlebar` strip with
`data-tauri-drag-region`. In the React app this becomes:

- macOS (Overlay): no custom strip. Give `.sidebar-top` `padding-top: 28px` so the brand clears the
  traffic lights, and mark `.sidebar-top` and `.topbar` with `data-tauri-drag-region` so the empty
  chrome drags the window. `trafficLightPosition` can be set to `{ x: 14, y: 16 }` to align with the
  32px icon buttons.
- Windows/Linux: keep a 38px strip with minimize / maximize / close drawn by the app (the existing
  Segoe glyph buttons are fine) and `data-tauri-drag-region` on the strip.
- `-webkit-app-region: drag` in CSS is a hint only; Tauri needs the data attribute.

Capabilities: `core:default` does not include window mutation. Add to `capabilities/default.json`:

```json
"permissions": [
  "core:default",
  "core:window:allow-start-dragging",
  "core:window:allow-minimize",
  "core:window:allow-toggle-maximize",
  "core:window:allow-close",
  "core:window:allow-set-title",
  "global-shortcut:allow-register",
  "store:default",
  "tray:default"
]
```

## 3. Shortcuts

Browser-level `keydown` handling (what `assets/shell.js` does) works inside the webview, but on
macOS the menu bar should own the accelerators so they show in menus and survive focus in iframes.
Register through Tauri's `Menu` API with the same bindings the prototype uses:

| Binding | Action | Where |
|---|---|---|
| ⌘N / ⌘⇧O | New chat | File menu |
| ⌘K | Command palette | Edit menu |
| ⌘B | Toggle sidebar | View menu |
| ⌘\ | Toggle inspector | View menu |
| ⌘⇧D / ⌘⇧E / ⌘⇧B | Inspector: Run / Files / Browser | View menu |
| ⌘, | Settings | App menu (macOS convention) |
| ⌘/ | Shortcut sheet | Help menu |
| ⌥Space | Quick entry (global) | `tauri-plugin-global-shortcut`, user-rebindable in Settings → Desktop |

Menu events post to the webview with `emit("uar:menu", { action })`; the shell's existing
`uar:action` listener handles the rest, so web and desktop share one dispatcher.

## 4. Persisted layout

The prototype keeps `sidebarCollapsed`, `panelOpen`, `panelTab`, `panelW` and `theme` in
`localStorage["uar.shell.v1"]`. In the app this is a Zustand `ui-store` slice with the
`persist` middleware; in Tauri back the storage with `tauri-plugin-store` so the layout survives
webview cache clears and is shared with the quick-entry window. Keep the key versioned.

## 5. Inspector as a pop-out window

The inspector's "Open in its own window" button maps to `WebviewWindow` creation with a
`?pane=files&thread=<id>` route. The main window then hides its panel and the state slice records
`panelDetached: true`. Closing the child window re-attaches. This is the Claude Code pop-out /
dock-back behaviour and it needs no new UI beyond the button already in the panel head.

## 6. Browser tab

The "Browser" inspector tab is a sandboxed page view. In Tauri use a child `Webview` positioned over
the panel body (`Webview::new` with `WebviewBuilder::new("inspector-browser", WebviewUrl::External(url))`)
rather than an `<iframe>`, so cross-origin pages, cookies and devtools behave. Keep the URL bar in
the host UI; the webview only renders the page. Governance: route the URL through the same Cedar
policy that gates `web.fetch` so the agent cannot open what a policy denies.

## 7. Theme

`data-theme="dark|light"` on `<html>` drives all tokens. On desktop, default to `system` and
subscribe to `tauri::window::Theme` changes (`onThemeChanged`) to flip the attribute; the user's
explicit choice in Settings → Appearance overrides it. The high-contrast scheme already in the
frontend tokens can be a third value of the same attribute.

## 8. Fonts

The React app currently links Space Grotesk, Inter and JetBrains Mono from Google Fonts in
`frontend/index.html`, which fails offline and is blocked by a strict CSP. Bundle the three
`.woff2` files from `assets/fonts/` through `@fontsource-variable/inter`,
`@fontsource-variable/space-grotesk` and `@fontsource-variable/jetbrains-mono`, replace the
existing Geist import (Inter is the body font by decision of 2026-10-05), and remove the
`<link rel="preconnect">` tags.

## 9. Order of work

1. Window config, identifier, capabilities (one PR, unblocks everything else).
2. Titlebar overlay on macOS; keep the custom strip on Windows/Linux.
3. Shell layout port: single sidebar + inspector region in `app-shell.tsx`, `--panel-w` and
   `--sidebar-w` tokens, `ui-store` persistence.
4. Menu accelerators and ⌥Space quick entry.
5. Inspector pop-out, then the child-webview browser tab.
