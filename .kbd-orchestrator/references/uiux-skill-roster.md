# UI/UX skill roster — Universal Agent Runtime

Project-scoped roster of skills + research targets that the agent
consults before writing UI/UX code in this repo. Referenced from the
`uiux-routing` fenced region in `CLAUDE.md` / `AGENTS.md` (managed by
`/kbd-inject-agent-rules --pack uiux-routing`).

Last fetched: 2026-08-24

## Tier 1 — Always consult

For every non-trivial UI/UX change in this repo, run both of these
through the relevant components / pages before writing code.

### UI/UX Pro Max — `nextlevelbuilder/ui-ux-pro-max-skill`

- **Source**: https://github.com/nextlevelbuilder/ui-ux-pro-max-skill — anchor: `UI/UX Pro Max`
- **Skill site**: https://ui-ux-pro-max-skill.com/
- **Canonical local skill**: `.agents/skills/ui-ux-pro-max/SKILL.md`
  (repository-owned; tool-specific entries link back to this payload).
- **Reproducibility**: `skills-lock.json` records the GitHub source and
  computed payload hash; the vendored subtree includes the upstream MIT
  license.
- **What it gives**: 79 searchable styles (50 active), 192 product
  palettes and reasoning profiles, 74 font pairings, 119 UX guidelines,
  105 curated icons, 17 motion presets, 25 chart types, and 22 stack
  catalogs.
- **Use it to**: select palette + font pairing for new features,
  validate contrast/spacing/touch targets, lift accessibility (ARIA,
  focus states), pick chart types.
- **How to consult it**: read the local `SKILL.md` query contract first.
  Use `--design-system` for new pages or system-wide direction, one
  explicit `--domain` for a targeted concern, and `--stack` for detected
  implementation-stack guidance. Verify the result category and retry
  once with a narrower query when results are empty or off-topic.

### Impeccable — `pbakaus/impeccable`

- **Source**: https://github.com/pbakaus/impeccable — anchor: `Impeccable`
- **Plugin Hub**: https://www.claudepluginhub.com/plugins/pbakaus-impeccable
- **Shape**: 1 skill, **23 commands** under `/impeccable/*`.
- **Modes**: `brand` (design IS the product) and `product` (design
  SERVES the product).
- **Command catalogue** (full set — run the relevant subset per task):
  - `/impeccable audit` — accessibility, performance, responsive
  - `/impeccable critique` — UX review: hierarchy, clarity, emotional
    resonance
  - `/impeccable polish` — final pass before shipping (alignment,
    spacing, micro-details)
  - `/impeccable distill` — strip to essence; remove complexity that
    doesn't earn its place
  - `/impeccable bolder` — amplify safe / boring designs
  - `/impeccable quieter` — tone down overly aggressive designs
  - `/impeccable animate` — add purposeful motion (not decorative
    jitter)
  - `/impeccable colorize` — introduce strategic color to monochrome
  - `/impeccable normalize` — align with design-system standards
  - `/impeccable harden` — error handling, i18n, edge cases
  - `/impeccable typeset` — type pairing + scale
  - `/impeccable layout` — composition / grid
  - `/impeccable delight` — purposeful micro-interactions
  - (additional commands documented in the upstream catalogue at
    https://deepwiki.com/pbakaus/impeccable/2.3-skills-and-commands-catalog — anchor: `Skills and Commands Catalog`)

## Tier 2 — Stack-specific

Consult per-stack when relevant.

### Vercel — React Best Practices
- **Source**: https://vercel.com/ (Claude Code skill)
- **Use it to**: composition patterns, server vs client component
  boundaries, performance defaults, image / font primitives.

### Vercel — Composition Patterns
- Same family as above; pair them.

### Vercel — React Native Skills
- For mobile / Tauri-bridge work.

### Vercel — Web Design Guidelines
- Layout, hierarchy, motion, accessibility defaults.

### Anthropic — frontend-design
- **What it gives**: pushes Claude toward distinctive, intentional UI
  design.

### Anthropic — ux-designer
- **What it gives**: UX-engineer reviewer persona.

## Tier 3 — Pre-work research

Run these BEFORE planning UI/UX changes:

1. **Memory recall** — invoke `/kbd-memory-recall` to populate
   `prior-context.md` for the active phase. This surfaces prior UI/UX
   decisions captured in surreal-memory (the recall hook is auto-fired
   on `assess:before` per change 6).

2. **Web search for runtime devtools** — search for "runtime dev tools
   on a web page best practices" before designing the in-app entity
   explorer (change 10). Cover: floating action button patterns, panel
   docking, event-log UX, multi-store visualisation, React Devtools as
   reference.

3. **Web search for Chrome MV3 devtools panel** — search for "Chrome
   MV3 devtools panel patterns" and "react-devtools bridge architecture"
   before scaffolding the browser extension (change 11). Cover:
   manifest v3 background service worker, content-script ↔ devtools
   bridge via `window.postMessage`, devtools_page lifecycle, panel
   reuse between in-app and extension.

## How this roster gets used

The `uiux-routing` fenced region in `CLAUDE.md` / `AGENTS.md` (managed
by `/kbd-inject-agent-rules --pack uiux-routing`) enumerates the
seven-step discipline AI tools follow before writing UI/UX code. This
file is the reference for **what** to consult; the fenced region is
the reference for **when** and **in what order**. UI/UX Pro Max means
the tracked local skill at `.agents/skills/ui-ux-pro-max/SKILL.md`, not
an assumed global or machine-local installation.

## Refreshing this roster

```sh
/kbd-inject-agent-rules --pack uiux-routing --refresh
```

That probes each source URL for its anchor keyword and warns on
changes; rule wording is updated by hand based on the warnings.
