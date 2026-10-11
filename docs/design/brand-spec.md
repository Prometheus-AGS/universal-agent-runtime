# Brand spec — Universal Agent Runtime operator shell

Source of truth: the incumbent token system already shipped in the UAR frontend
(`frontend/src/shared/theme/tokens.css`, `frontend/src/index.css`) and the
documentation site (`website/src/css/custom.css`), plus the brand marks in
`website/static/img/brand/`. These are an **established world**; this prototype
inherits it rather than inventing a replacement. The one deliberate change is
structural (navigation and panel system), not visual identity.

In one sentence: a dark, Flat 2.0 operator console on a four-step navy surface
ladder, where a single ember accent marks the current selection and the primary
action, Space Grotesk carries headings, Inter carries UI text, and JetBrains
Mono carries identifiers, addresses, and numbers.

## Six core tokens (dark, the default scene)

| Token | OKLch | Hex origin | Role |
|---|---|---|---|
| `--bg` | `oklch(16.7% 0.012 254)` | `#0B0F14` canvas | window background, thread canvas |
| `--surface` | `oklch(20.0% 0.022 264)` | `#111620` chrome | sidebar, header, right panel (chrome step) |
| `--fg` | `oklch(94.4% 0.010 253)` | `#E8EDF3` | primary text |
| `--muted` | `oklch(75.4% 0.020 256)` | `#A7B0BC` | secondary text, labels, captions (AA on every ladder step) |
| `--border` | `transparent` | — | Flat 2.0: regions separate by fill, never by stroke; kept for the seed's contract |
| `--accent` | `oklch(70.4% 0.192 37)` | `#FF6A3D` ember | selection, primary action, focus ring |

Extended ladder (derived from the same source, used for fills in place of
borders): raised `oklch(26.4% 0.033 262)` `#1C2535`, hover `oklch(28.9% 0.042 263)`
`#202B40`, input/skeleton `oklch(30.9% 0.039 262)` `#253044`,
ember-on text `#0B0F14`, ember-soft selection surface `oklch(23.9% 0.028 27)` `#2B1A18`.

State colors (semantic only, never decorative): success `#22C55E`,
warning `#F59E0B`, error `#EF4444`, streaming/reasoning cyan `#00C2DC`.

Light scene (mirrors `.light` in tokens.css; preserved for the system-theme
toggle): canvas `#F7F7F8`, chrome `#FFFFFF`, surface `#FAFBFC`, hover `#F2F4F7`,
fg `#0B0F14`, muted `#4B5563`, ember `#B93C1C` (darkened to hold 4.5:1 on white).

## Type

- Display: `"Space Grotesk", "Inter Variable", system-ui, sans-serif` — weights 500–700, tracking -0.02em on headings.
- Body / UI: `"Inter Variable", "Inter", system-ui, sans-serif` — 13–15px in chrome, 15–16px in the thread.
- Mono: `"JetBrains Mono", ui-monospace, Menlo, monospace` — model addresses (`provider/model`), run ids, token counts, prices, timestamps; tabular numerals on.

Fonts are localized under `assets/fonts/` (Inter variable from `@fontsource-variable/inter` 5.3.0,
Space Grotesk and JetBrains Mono latin variable subsets, all SIL OFL 1.1) so the prototype runs
offline and inside a Tauri bundle. Inter is the body font by decision of 2026-10-05, matching the
binding `docs/knowme-ui-ux-standard.md`; the shipped frontend still loads Geist and switches in the
first implementation PR (see `react-implementation-plan.md`).

## Observed rules of the visual language

1. **Four-step ladder, no strokes.** canvas → chrome → surface → raised. A region is a fill, a control is a fill one step up, hover is one more step. Dividers are not drawn; spacing and fill do the separating. (Data-viz axes are the documented exception.)
2. **One accent, two jobs.** Ember marks the current selection in nav/lists and the single primary action in view. It never tints backgrounds for decoration; `ember-soft` is the only tinted surface and it means "selected".
3. **Motion is state.** 120ms fast / 200ms base / 320ms slow, `cubic-bezier(0.2, 0, 0, 1)` standard and `cubic-bezier(0.16, 1, 0.3, 1)` for emphasis (panel slide-in). No load choreography.
4. **Mono means measurement.** JetBrains Mono appears only on identifiers, addresses, prices and counts, never as a "technical" costume on labels.
5. **Radius 12px (0.75rem) on cards and panels, 8px on controls, pill only on small status chips.** Focus ring is a 2px ember ring with a 2px canvas gap.
6. **Phase colors are reserved.** Context grey, skill amber, memory violet, retrieval blue, reasoning indigo, tool green, generate ember — used only on run-phase indicators in the execution record.
