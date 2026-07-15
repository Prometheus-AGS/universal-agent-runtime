## Why

UAR's A2UI renderer has a sound semantic baseline, but its appearance, fallback copy, motion, and failure behavior still depend on the host. Change 21 closes the final Grade-A implementation gap by making every A2UI surface predictably accessible, localizable, themeable, and recoverable across supported hosts.

## What Changes

- Add scoped light, dark, and high-contrast A2UI themes backed by semantic CSS variables and forced-colors fallbacks.
- Add typed i18next resources for English, Spanish, Japanese, and Simplified Chinese, plus explicit LTR/RTL surface direction.
- Add reduced-motion-aware Motion transitions for surface entrance, exit, updates, and streaming state changes.
- Add a renderer-owned error boundary with localized empty/error states and a retry affordance on every surface.
- Associate validation feedback with controls, harden keyboard/focus behavior, responsive wrapping, and touch targets.
- Add axe-core accessibility tests and a path-filtered CI gate for A2UI surfaces.
- Record the Codex harness override and synchronize completion evidence back to KBD.

## Capabilities

### New Capabilities

- `a2ui-ux-2026`: Theme, accessibility, localization, bidirectional layout, motion, and recovery requirements for UAR-rendered A2UI surfaces.

### Modified Capabilities

- `a2ui-react-conformance`: Extend the certified UAR renderer contract with host-independent UX semantics while preserving protocol and action conformance.
- `frontend-validation-gate`: Require automated axe-core coverage for changed A2UI renderer surfaces.

## Impact

- Affects `frontend/packages/a2ui-uar`, its package exports/tests/stories, the frontend theme store/toggle and global token definitions, frontend CI, and KBD execution state.
- Adds verified `motion`, `i18next`, and `axe-core` dependencies; no provider, backend API, or realtime wire-format changes.
- Existing `UarSurface` callers remain source-compatible because all new props are optional and defaults preserve English/LTR behavior.
