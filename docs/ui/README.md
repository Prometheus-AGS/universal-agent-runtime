# UAR UI/UX design package

Self-contained design artifacts for the Universal Agent Runtime React 19 app.
Open any `.html` file directly in a browser — no build step, no server.

## Designs

| File | What it is |
|---|---|
| `UAR Console.dc.html` | Desktop console: chat with run-trace waterfall, event timeline, inspector, Cherry-Studio-style composer toolbar, knowledge screen, runtime settings deck |
| `UAR Mood Board.dc.html` | 14 panels covering every major use case — chat, providers, credentials + tests, knowledge, chunking lab, agents, compiler, A2UI, skills, context, web, MCP, built-in tools, per-agent override matrix |
| `UAR Shell & Mobile.dc.html` | App container + navigation (expanded/collapsed rail, three-zone frame, command palette) and 15 iPhone frames covering every view |
| `UAR Logo Options.dc.html` | Three logo directions (Slash Gate, Run Trace, Aperture) |
| `UAR Logo Assets.dc.html` | Delivered logo set, in situ, with usage rules |
| `uar-logo-assets.html` | Static version of the logo sheet (no runtime required) |

## Engineering

| File | What it is |
|---|---|
| `uar-frontend-migration-plan.md` | Ten-phase migration to React 19 · Vite 8 · Tailwind 4 · PEM · PGlite · AG-UI · A2UI, with the Flat 2.0 rules, the markdown render contract, and the complete chunk catalog. Point codex at this file. |

`support.js` and `ios-frame.jsx` are runtime dependencies of the `.dc.html`
files and must stay beside them.

## Logo — Slash Gate

`logo/` holds 10 SVGs and 18 PNGs.

- `uar-mark-{dark,light}.svg` — transparent mark, per background
- `uar-mark-mono-{white,charcoal}.svg` — single ink
- `uar-appicon-{dark,light}.svg` — squircle tile
- `uar-favicon-{dark,light}.svg` — below 24px: node dropped, strokes thickened
- `uar-wordmark-{dark,light}.svg` — horizontal lockup (live text; outline to paths for print)
- `logo/png/uar-appicon-{dark,light}-{512,256,128,64,32,16}.png`
- `logo/png/uar-mark-mono-{white,charcoal}-{512,256,128}.png`

Rules: clear space equals one slash width. Ember is the left slash, deep ember
the right — never inverted. The node stays neutral. No rotation, outline, or glow.

## Visual system

Charcoal ladder `#0B0F14` canvas / `#0F1620` chrome / `#141C26` surface /
`#1A2432` raised, one ember accent (`#FF6A3D` dark, `#E04E28` light), ember-soft
`#2A1B17` for selection. Space Grotesk for names, Inter for UI, Roboto for prose,
JetBrains Mono for machine facts and `// eyebrows`. Values match
`frontend/src/index.css` in the runtime repo.

Run-phase colors: context `#A7B0BC`, skills `#F59E0B`, memory `#8B5CF6`,
retrieval `#60A5FA`, reasoning `#2563EB`, tools `#22C55E`, generation `#FF6A3D`.
