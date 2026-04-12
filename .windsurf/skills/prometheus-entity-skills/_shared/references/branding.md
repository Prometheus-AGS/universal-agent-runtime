# travisjames.ai — branding tokens (reference)

Source of truth in the Vite example: `examples/vite-app/src/index.css`. Use these tokens when generating UI that should match the Prometheus demo / brand.

## Typography — four fonts and roles

| Token | Font stack | Role |
|-------|------------|------|
| **`--font-display`** | Space Grotesk, ui-sans-serif, sans-serif | Marketing-style headings, hero titles, strong display hierarchy. |
| **`--font-ui`** | Inter, ui-sans-serif, sans-serif | Primary UI: navigation, buttons, labels, tables, dense interfaces. Also exposed as `--font-sans` → `var(--font-ui)`. |
| **`--font-body`** | Roboto, ui-sans-serif, sans-serif | Long-form body copy where distinct from compact UI chrome. |
| **`--font-mono`** | JetBrains Mono, ui-monospace, monospace | Code, IDs, JSON, technical snippets. |

Tailwind v4 `@theme` maps the same names for `font-display`, `font-ui`, `font-body`, `font-mono` utilities.

## Light mode — color anchors (HSL triplets)

Neutrals and surfaces:

- **Background**: `--background` `247 247 248` — page canvas (off-white, not pure white).
- **Foreground / primary text**: `--foreground` `11 15 20`.
- **Card**: `--card` `255 255 255`, `--card-foreground` `11 15 20`.
- **Muted surface**: `--muted` `238 240 243`, `--muted-foreground` `75 85 99`.
- **Border / input**: `--border` / `--input` `216 222 230`.

Brand (ember):

- **Primary**: `--primary` `224 78 40` (ember orange) — CTAs, active nav, focus rings.
- **Primary on color**: `--primary-foreground` `255 255 255`.
- **Ring**: `--ring` `224 78 40` — focus visibility aligned with primary.
- **Secondary**: `--secondary` `17 24 39`, `--secondary-foreground` `255 255 255`.
- **Accent**: `--accent` `242 214 207`, `--accent-foreground` `59 20 16`.

Semantic:

- **Destructive** `220 38 38`, **success** `22 163 74`, **warning** `217 119 6`, **info** `37 99 235` (each with matching `-foreground`).

Legacy example aliases (RGB): `--color-ember`, `--color-text-primary`, `--color-surface`, `--color-border`, etc. — use for gradual migration; prefer shadcn semantic tokens for new code.

## Dark mode — color anchors

- **Background**: `11 15 20` — deep charcoal (not pure black).
- **Foreground**: `232 237 243`.
- **Card**: `15 22 32`.
- **Muted**: `20 28 38` / `167 176 188`.
- **Border / input**: `35 48 65`.
- **Primary / ring**: `255 106 61` (brighter ember for dark surfaces).
- **Accent**: `42 27 23` / `255 177 153`.

Legacy dark: `--color-ember` → `rgb(255 106 61)`, surfaces stepped `15 22 32` → `26 36 50`.

## Color usage rules

1. **Ember is sparse** — primary CTAs, active states, focus rings, key highlights. Roughly **90%+** of the UI should sit on the **neutral ramp** (background, card, muted, border).
2. **No full-page `#000` / `#fff`** — use designated background and card tokens.
3. **Do not fill large surfaces** (full cards, banners) with ember; it overwhelms and hurts readability.
4. **Charts** use `--chart-1` … `--chart-5` in order; chart-1 tracks primary brand.

## Component styling patterns

- **Radius**: `--radius` default `0.625rem`; stepped `--radius-sm` … `--radius-xl` for nested components.
- **Selection**: `::selection` uses `color-mix` of primary at ~18% with transparent over `--foreground` text.
- **Scrollbars**: thin (6px), thumb from `rgb(var(--border))`, hover toward muted foreground.
- **Surfaces**: cards and panels use `bg-card`, `text-card-foreground`, subtle borders `border-border`.
- **Density**: prefer `text-sm` / compact padding for data tables; use `font-ui` for controls and `font-mono` for IDs in cells when showing technical data.

When a consuming app does not import this CSS, still apply the **rules** (restrained accent, neutral-first, accessible contrast) with that app’s own tokens.
