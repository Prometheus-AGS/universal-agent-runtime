# C-15 responsive certification matrix

Run date: 2026-08-08

Each cell loads `/admin/runtime` with deterministic transport responses, runs axe, checks
the active shell navigation mode, asserts one main landmark, measures shell/content
overlap, rejects page-level horizontal overflow, and measures standalone controls.

| Width | Theme | Shell mode | Axe | Overflow / overlap | Targets | Verdict |
|---:|---|---|---|---|---|---|
| 320 | Light | Compact bottom navigation | 0 violations | Pass | Pass | Pass |
| 320 | Dark | Compact bottom navigation | 0 violations | Pass | Pass | Pass |
| 768 | Light | Compact bottom navigation | 0 violations | Pass | Pass | Pass |
| 768 | Dark | Compact bottom navigation | 0 violations | Pass | Pass | Pass |
| 1024 | Light | Desktop navigation rail | 0 violations | Pass | Pass | Pass |
| 1024 | Dark | Desktop navigation rail | 0 violations | Pass | Pass | Pass |
| 1440 | Light | Desktop navigation rail | 0 violations | Pass | Pass | Pass |
| 1440 | Dark | Desktop navigation rail | 0 violations | Pass | Pass | Pass |

The UAR high-contrast theme also passed axe at 1440 CSS pixels. Desktop and compact
keyboard paths were exercised separately so the same interaction proof was not duplicated
in every matrix cell.

Repeat with:

```bash
pnpm -C frontend test:a11y
```
