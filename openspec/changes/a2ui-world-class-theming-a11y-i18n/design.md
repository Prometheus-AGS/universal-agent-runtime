## Context

The UAR renderer already maps the certified A2UI catalog to semantic React primitives, but it inherits host tokens, uses English literals for renderer-owned copy, exposes no surface lifecycle boundary, and has no package-local accessibility gate. The implementation spans the renderer package, host theme selection, and CI. The UI remains a restrained operational surface: consistency and trust take priority over decorative redesign.

## Goals / Non-Goals

**Goals:**

- Make the renderer deterministic under light, dark, high-contrast, forced-colors, LTR, RTL, reduced-motion, empty, malformed, and retry states.
- Preserve the existing protocol catalog, typed action flow, and component/hook/store/service layering.
- Keep new APIs optional so existing callers render with English, LTR, and host-compatible defaults.
- Turn the critique's measurable P1 findings into package-local regression tests and CI.

**Non-Goals:**

- Adding new wire component types, changing the A2UI protocol, or executing agent-supplied styles/code.
- Replacing the host design system or redesigning unrelated runtime-console pages.
- Translating agent-authored payload content; only renderer-owned strings are localized.
- Rebuilding citation UX already delivered by Change 13 or Storybook infrastructure owned by Change 25.

## Decisions

### Surface options are a renderer context

`UarSurface` accepts optional theme, locale, direction, retry, and transition props and provides them through a package-local context. This keeps component rendering pure, allows nested primitives to translate renderer-owned text, and avoids coupling the package to the application Zustand stores. A global singleton was rejected because concurrent embedded surfaces can require different locale/theme settings.

### Theme variables are scoped to each surface

The package ships a CSS file whose variables live below `[data-a2ui-theme]`, with explicit light, dark, and high-contrast palettes plus `forced-colors` rules. Components continue using semantic Tailwind tokens, while the surface scope remaps those tokens without changing the host root. Reusing only the host `.dark` class was rejected because it cannot guarantee contrast or embed portability.

### i18next runs per surface through a typed adapter

The package uses verified `i18next@26.3.6` resources for `en`, `es`, `ja`, and `zh`. A small React context exposes a typed key union and resolves `dir="auto"` through `i18next.dir(locale)`, while allowing explicit RTL for future locales. `react-i18next` was rejected as unnecessary weight for a fixed renderer-owned string set.

### Motion is bounded to the surface lifecycle

Verified `motion@12.42.2` supplies `MotionConfig`, `AnimatePresence`, and a keyed surface container for entrance, exit, structural update, and streaming-status transitions. Durations stay in the 150–250 ms product range and `reducedMotion="user"` plus CSS fallbacks guarantee a usable no-motion path. Per-node wrappers were rejected because they would alter certified semantic/flex DOM structure.

### Errors are contained at every surface boundary

A class error boundary catches catalog/schema/render failures and renders a localized `role="alert"` recovery panel. Retry resets the boundary and invokes an optional host callback; changing `resetKey` also clears the fault. Empty surfaces render a localized status instead of `null`. This prevents one malformed agent surface from taking down its host while preserving fail-closed behavior.

### Accessibility is enforced with axe-core plus focused behavior tests

Verified `axe-core@4.12.1` runs directly against rendered fixtures in Vitest, avoiding the stale `vitest-axe` wrapper. Tests also cover keyboard interaction, composed validation descriptions, direction, theme scope, reduced-motion configuration, empty/error/retry states, long localized text, and narrow wrapping. A path-filtered CI job runs the package lifecycle and accessibility suite.

## Risks / Trade-offs

- **Scoped variables can conflict with host utility generation** → ship the CSS entrypoint explicitly, keep selectors below the surface root, and test computed data/theme contracts independently of host root classes.
- **i18next initialization could become asynchronous** → use a synchronous per-locale instance over bundled resources and no remote backend.
- **Animation can obscure rapid streaming updates** → key only meaningful surface versions/statuses, keep durations short, and honor user reduced-motion preferences.
- **Error boundaries cannot catch event-handler errors** → action dispatch remains typed and tested separately; the boundary covers render/update failures, which are the surface-availability risk.
- **High-contrast palettes cannot prove every host override** → constrain certified themes, document that arbitrary host overrides fall outside the guarantee, and add axe/forced-colors regression coverage.

## Migration Plan

1. Add package contexts, resources, styles, boundary, and optional `UarSurface` props.
2. Migrate renderer-owned literals and validation relationships.
3. Extend the host theme selector with high contrast while preserving stored light/dark/system values.
4. Add tests, stories, CI, and documentation; run package and workspace gates.
5. Roll back by removing the optional wrapper/features; the wire protocol and existing call sites remain unchanged.

## Open Questions

None blocking. Future locale additions and custom certified theme registration can extend the typed resources/options without changing this contract.
