# Admin Aesthetic Specification — Terminal / CRT

**Phase:** `direct-entity-migration-models`
**Status:** authoritative — every redesigned admin page MUST conform to this contract
**Driven by:** `frontend-design` (Anthropic) + `impeccable` (product mode) + `ui-ux-pro-max` Claude Code skills

---

## 1. Aesthetic direction

Retro-futuristic / terminal. CRT phosphor callbacks. Operators-in-front-of-an-instrument feel. The runtime is named "Universal Agent **Runtime**" — the admin surface should look like a runtime, not a SaaS dashboard.

Reference moods: vintage terminal emulators (xterm, st), early UNIX tools, mission-control panels, modern terminal aesthetic crates like `gum` / `bubbletea`. NOT brutalist (too harsh), NOT cyberpunk (too kitsch), NOT editorial (too refined).

---

## 2. Banned defaults

The following are **never** used in newly authored admin CSS:

- `font-family.*(Inter|Roboto|Arial|Space Grotesk|system-ui)` — too generic, AI-slop signature
- `outline: none` — accessibility violation; if focus styles are custom, declare them
- Purple gradients (`linear-gradient.*purple`, `--purple-*`) — AI-slop signature
- Lavender, "tech blue" (#3B82F6 family) — overused
- Drop shadows `0 1px 2px rgba(0,0,0,0.05)` — generic ShadCN default; we use thin lines instead

---

## 3. Tokens (added to `frontend/src/index.css`)

Tokens are scoped to `:root[data-admin-theme="terminal"]` so they don't bleed into the chat surface, which retains the existing UAR Dark theme.

```css
:root[data-admin-theme="terminal"] {
  /* Surfaces */
  --terminal-bg: 165 18% 5%;          /* #0a0d0c near-black with green tint */
  --terminal-surface: 168 14% 8%;     /* #0e1311 card */
  --terminal-fg: 155 14% 80%;         /* #c8d4cf cool gray, slightly green */
  --terminal-fg-dim: 155 10% 55%;     /* #7e8d88 secondary text */

  /* Accents — CRT phosphor */
  --phosphor: 140 100% 75%;           /* #7fffa1 phosphor green */
  --phosphor-glow: 140 100% 65%;      /* #4fff7d brighter for focus */
  --amber: 35 100% 64%;               /* #ffb347 CRT amber, warnings */
  --signal-red: 0 100% 68%;           /* #ff5d5d errors */

  /* Lines */
  --terminal-line: 155 14% 80% / 0.18;  /* thin lines */
  --terminal-line-strong: 155 14% 80% / 0.34;

  /* Type — display + body share family */
  --font-mono-display: "Berkeley Mono", "JetBrains Mono", ui-monospace, monospace;
  --font-mono-body: "Berkeley Mono", "JetBrains Mono", ui-monospace, monospace;

  /* Density */
  --size-body: 13px;
  --size-chip: 12px;
  --size-headline: 20px;
  --leading-body: 1.5;
  --leading-headline: 1.2;
}
```

---

## 4. Component contract

Every redesigned page MUST use these shared components (authored in change-4 `migrate-compiler-page-direct-and-redesign`):

| Component | Path | Use for |
|---|---|---|
| `<EmptyFrame>` | `frontend/src/components/admin/empty-frame.tsx` | empty list/detail states; renders ASCII border + slot + one action |
| `<LoadingCursor>` | `frontend/src/components/admin/loading-cursor.tsx` | inline loading; renders `▍` glyph flickering at 600 ms |
| `<ErrorBar>` | `frontend/src/components/admin/error-bar.tsx` | error states; red bar pinned top of card with mono code prefix `ERR-<CODE>` |

Until change-4 lands, the placeholders in `models-page` (change-3) inline these patterns and the change-4 extraction is a structural refactor that doesn't move the visual contract.

---

## 5. Motion

- **Transitions:** ≤180 ms, `ease-out`. No fade-throughs.
- **Page transitions:** 4 px upward slide + opacity, 180 ms.
- **Loading cursor blink:** 600 ms period (250 ms on, 350 ms off).
- **Scanline animation:** optional; if present, 8 s linear translateY loop over the entire admin surface, 4 % opacity.

No spring physics. No exit animations longer than the entrance.

---

## 6. Accessibility

- **Focus ring:** `outline: 2px solid hsl(var(--phosphor-glow)); outline-offset: 2px;` on every interactive element.
- **Tab order:** logical, top-to-bottom, left-to-right. Verified per page with `keyboard.press("Tab")` Playwright loop.
- **Color contrast:** body text vs surface ≥ 7:1 (WCAG AAA).
- **Reduced-motion:** `@media (prefers-reduced-motion: reduce)` disables scanline animation and slide transitions.

---

## 7. Required states per page

Every redesigned page MUST render correctly in all four states. Playwright captures the loaded state; the others are documented in `.kbd-orchestrator/phases/direct-entity-migration-models/audits/<page>.md`.

1. **Loading** — `<LoadingCursor>` inline beside the section header.
2. **Empty** — `<EmptyFrame>` with one CTA.
3. **Loaded** — the normal view.
4. **Error** — `<ErrorBar>` pinned at top of the affected card.

---

## 8. PR review checklist (per redesigned page)

- [ ] No banned fonts in newly authored CSS (`grep -nE "(Inter|Roboto|Arial|Space Grotesk|system-ui)" frontend/src/admin/`)
- [ ] No `outline: none` (`grep -nE "outline:\\s*none" frontend/src/admin/`)
- [ ] No purple gradients
- [ ] Loading state uses `<LoadingCursor>`
- [ ] Empty state uses `<EmptyFrame>`
- [ ] Error state uses `<ErrorBar>`
- [ ] Focus ring visible at 2 px phosphor-green on every interactive
- [ ] `data-admin-theme="terminal"` attribute is set when route starts with `/admin`
- [ ] Playwright screenshot present in `screenshots/<page>.png`
- [ ] All transitions ≤ 180 ms

---

## 9. Reference invocations

When working on a redesigned page, invoke:

- `Skill: frontend-design` for the overall aesthetic discipline
- `Skill: impeccable` in **product mode** for the dashboard/admin-tool guidance
- `Skill: ui-ux-pro-max` for palette/font/UX pattern lookups (if installed)

Audits land at `.kbd-orchestrator/phases/direct-entity-migration-models/audits/<page>.md`.
