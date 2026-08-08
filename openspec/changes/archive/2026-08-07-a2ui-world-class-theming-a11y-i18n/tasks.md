## 1. Workflow and package contracts

- [x] 1.1 Record the Codex harness override, dual-agent critique, dependency versions, execution scope, and Change 21 KBD status.
- [x] 1.2 Add verified Motion, i18next, and axe-core dependencies and publish package style/i18n/theme exports.
- [x] 1.3 Document compatibility defaults and host integration for theme, locale, direction, retry, and reduced motion.

## 2. Theme and responsive system

- [x] 2.1 Add scoped light, dark, and high-contrast semantic CSS-variable themes plus forced-colors and reduced-motion fallbacks.
- [x] 2.2 Extend the host theme store/toggle for high contrast without breaking persisted light, dark, or system preferences.
- [x] 2.3 Harden A2UI rows, controls, selection cues, focus indicators, and touch targets for narrow layouts and high zoom.

## 3. Localization and direction

- [x] 3.1 Add typed i18next resources for English, Spanish, Japanese, and Simplified Chinese.
- [x] 3.2 Add a per-surface localization context with automatic or explicit LTR/RTL direction.
- [x] 3.3 Replace every renderer-owned user-facing literal with localized copy while leaving agent-authored payload content unchanged.

## 4. Surface lifecycle and motion

- [x] 4.1 Add localized empty and rendering-failure states at every `UarSurface` boundary.
- [x] 4.2 Add boundary reset/retry behavior and keep unsupported components fail-closed.
- [x] 4.3 Integrate reduced-motion-aware Motion transitions for surface entrance, exit, update, and streaming status.

## 5. Accessibility evidence

- [x] 5.1 Associate validation descriptions/errors with TextField, CheckBox, and ChoicePicker controls.
- [x] 5.2 Add keyboard, focus, theme, locale, RTL, narrow-layout, empty/error/retry, and reduced-motion tests.
- [x] 5.3 Add axe-core fixtures covering certified primitives, entity surfaces, themes, locales, and recovery states.
- [x] 5.4 Add representative Storybook stories and a path-filtered CI accessibility gate.

## 6. Completion gates

- [x] 6.1 Pass package typecheck, lint, tests, build/perf checks, and the Impeccable detector.
- [x] 6.2 Pass frontend workspace typecheck, lint, build, and strict OpenSpec validation.
- [x] 6.3 Run Impeccable harden, animate, audit, critique backlog, and polish; resolve all applicable P0/P1 findings.
- [x] 6.4 Apply the project's formally approved direct-validation substitute for retired artifact-refiner, update KBD to 25/25 implementation-complete, and prepare a focused commit/push/PR.
