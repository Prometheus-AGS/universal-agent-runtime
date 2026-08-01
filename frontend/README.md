# UAR Frontend

React 19 + Vite + Tailwind CSS 3 web UI for the Universal Agent Runtime. Built to `../static/` and served identically in browser, Docker, and the Tauri webview by the Axum binary.

## Design tokens — KnowMe standard (ported)

This UI renders the **KnowMe** design-token set (`docs/knowme-ui-ux-standard.md` in the KnowMe monorepo). Values live in `src/index.css` as HSL channels consumed by `tailwind.config.ts` via `hsl(var(--token))`.

| Token | Dark | Light |
|---|---|---|
| `--background` (canvas) | `#0B0F14` | `#F7F7F8` |
| `--chrome` (sidebar/header) | `#111620` | `#FFFFFF` |
| `--surface` | `#161D29` | `#FAFBFC` |
| `--card` (raised) | `#1C2535` | `#FFFFFF` |
| `--card-hov` | `#202B40` | `#F2F4F7` |
| `--muted` | `#253044` | `#F2F4F7` |
| `--foreground` / `--fg-sub` / `--fg-faint` | `#E8EDF3` / `#A7B0BC` / `#6B7280` | `#0B0F14` / `#4B5563` / `#6B7280` |
| `--primary` / `--ember` | `#FF6A3D` | `#E04E28` |
| `--ember-soft` (selection) | `#2B1A18` | `#FBE8E1` |
| `--cyan` (reasoning/citations/streaming) | `#00C2DC` | `#0891B2` |
| `--success` / `--warning` / `--destructive` | `#22C55E` / `#F59E0B` / `#EF4444` | `#16A34A` / `#D97706` / `#DC2626` |

Radius base `--radius: 0.75rem` (scale sm 0.6×, md 0.8×, lg 1×, xl 1.4×, 2xl 1.8×, 3xl 2.2×, 4xl 2.6×). Fonts: Geist Variable (body, bundled via `@fontsource-variable/geist`), Space Grotesk (display), JetBrains Mono (meta/code). `.eyebrow` = mono 10px/600/0.12em uppercase ember label.

**Flat 2.0 is enforced by CSS, not convention**: `--border: transparent`, a global `border-transparent` base rule, `box-shadow: none` on `.aui-root`, and markdown border neutralization. Do not add `border-*`, `divide-*`, or `shadow-*` utilities to product UI — regions separate by background color alone. The `high-contrast` theme is the one deliberate exception (visible lines are the accessibility feature there).

Themes: `dark` (default), `.light`, `.high-contrast`, `system` — managed by `src/stores/theme-store.ts` (`uar-theme` localStorage key, class on `<html>`).

## Recorded decisions (uar-ui-token-convergence, 2026-07-21)

1. **Tailwind 3.4 kept** — token *values* were ported from KnowMe's Tailwind 4 `@theme` source; the mechanism here stays `hsl(var())` channels. Token-value syncs are manual until a shared token package exists; the table above is the contract.
2. **TanStack Query scoped exemption** — `@tanstack/react-query` remains in this frontend. KnowMe's `preserve-entity-management` constraint forbids it in the KnowMe app; this vendored submodule is a deliberate, documented exception. Entity sync already flows through `@prometheus-ags/prometheus-entity-management` (workspace package) + SSE.
3. **Admin CRT theme exception** — `/admin` routes use the scoped `:root[data-admin-theme="terminal"]` phosphor/terminal aesthetic (see `../docs/admin-aesthetic-spec.md`). It never bleeds into product surfaces, which render the KnowMe tokens.

## Commands

```sh
pnpm install          # from repo root (workspace)
pnpm -C frontend dev  # Vite on :8080, proxies /api to UAR_BACKEND_URL (default 127.0.0.1:1906)
pnpm -C frontend build
pnpm -C frontend test
pnpm -C frontend storybook
```
