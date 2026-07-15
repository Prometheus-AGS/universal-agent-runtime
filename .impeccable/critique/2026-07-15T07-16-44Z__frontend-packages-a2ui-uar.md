---
score: 22
total: 40
p0: 0
p1: 3
method: dual-agent
timestamp: 2026-07-15T07-16-44Z
slug: frontend-packages-a2ui-uar
---
Method: dual-agent (A: change21_design_review · B: change21_detector_review)

# A2UI UAR baseline critique

Design health: 22/40 Nielsen. The renderer has a restrained, credible product baseline with strong semantic primitives, a fail-closed catalog, plain-text rendering, and no deterministic AI-pattern violations. Its primary trust gap is system behavior rather than decoration.

## Priority backlog

- P1: `UarSurface` has no visible empty/error/retry lifecycle; malformed content can blank or escape into the host.
- P1: TextField, CheckBox, and ChoicePicker validation errors are not consistently associated to controls.
- P1: theme behavior is host-coupled; no certified scoped light/dark/high-contrast contract exists.
- P2: non-wrapping rows, 36px controls, and limited narrow/zoom coverage weaken responsive operability.
- P2: renderer-owned copy is English-only and no explicit RTL contract exists.

## Detector and run notes

CLI detector returned `[]` (0 findings, 0 false positives). Accessibility 3/4, performance 3/4, responsive 2/4, theming 3/4, anti-patterns 4/4. Browser visibility and overlay injection were unavailable because the isolated assessment had no browser automation tool and the component package had no supplied navigable URL. No live server or temporary browser artifact was created. Ignore list absent. Assessments remained independent.
