# C-15 accessibility baseline

Run date: 2026-08-08

## Observed failures before implementation

- The shared `*:focus-visible` rule in `frontend/src/index.css` renders `ring-2`, not the
  binding 3px ember indicator.
- Source inspection found 28 component/feature declarations that can weaken focus to a
  1px or 2px ring/outline, including the agent selector, settings schema editors, provider,
  knowledge, tools, compiler, memory, models, skills, and shared sidebar controls.
- Six primitive stories and one A2UI entity story disable the fail-closed Storybook
  accessibility test through two `a11y.test = off` declarations.
- The shell owns the application `<main>`, while chat, settings, and the unused shared
  sidebar primitive also render nested `<main>` elements. Integrated axe determines which
  live routes reproduce the duplicate-landmark defect.

## Existing positive controls

- The shell contains a skip link, a banner, named desktop and compact navigation
  landmarks, and one shell-owned main landmark.
- Source inspection found 24 reduced-motion, landmark, live-region, or status references.
- The shared theme includes an explicit `prefers-reduced-motion: reduce` rule.
- Readiness state includes human-readable status text in expanded and compact variants;
  coloured dots are marked decorative.

This baseline records source observations only. Browser axe, computed focus/contrast,
keyboard, target-size, and overflow outcomes belong to the certification receipts.
