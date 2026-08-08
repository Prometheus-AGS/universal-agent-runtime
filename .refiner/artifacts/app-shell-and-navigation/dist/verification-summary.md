# C-10 Artifact Refinement Verification

## Scope

Artifact `app-shell-and-navigation` is a `direct:code` refinement of the top-level UAR shell. Its blocking authority is the C-10 `frontend-app-shell` OpenSpec delta plus the vendored UAR UI standard and migration plan.

## Delta found during refinement

The first rendered audit found two gaps not exposed by the DOM tests: high-contrast brand color selection and compact command/theme controls below the 44px interaction target. The exact delivered wordmark/app-icon vector geometry now renders inline with explicit light, dark, and high-contrast palettes, and the two compact controls are 44px without changing their desktop sizes. Retiring the old PascalCase shell and KnowMe logo also resolved two filename-gate entries, reducing the Flat 2.0 baseline from 391 to 389.

## Deterministic evidence

- `pnpm -C frontend typecheck`: pass.
- `pnpm -C frontend lint`: pass.
- `node scripts/check-frontend-boundaries.mjs`: pass, zero production violations.
- `node scripts/check-flat2-style.mjs`: pass, 389 tracked legacy violations and zero new violations.
- Four focused files: pass, 28 tests.
- `openspec validate migrate-cross-cutting-pages --strict`: pass.
- Static source census: no retired `KnowMeLogo`/`KnowMeWordmark` imports, no `cmdk` or legacy command-wrapper import under `frontend/src/app/shell/`, and no `.DS_Store` under `frontend/public/brand/`.

## Rendered audit, critique, and polish

- 1440px desktop: expanded rail, breadcrumb, command trigger, theme access, readiness, and the working surface render with clear Flat 2.0 hierarchy.
- 901px desktop: the rail is visible and measures exactly 240px expanded or 60px collapsed; compact navigation is absent.
- 900px compact: the rail is absent, compact navigation is visible, feature content remains mounted once, and document width equals viewport width.
- 390px and 320px compact: all four bottom targets measure 44px, document width equals viewport width, and header controls remain readable without collision.
- Keyboard focus on the rail control computes to a 3px ember outline. The shell contains one `main` landmark and the compact presentation has no focusable hidden rail.
- The Base UI command palette and Configure sheet were opened, visually inspected after transitions, and exercised through normal React Router navigation.
- The first isolated review passed with 0 critical / 5 warning / 3 suggestion findings. Direct keyboard, open-dialog suppression, system breadcrumb, compact current-state, favicon fallback, manifest-purpose, and design-artifact corrections were adopted. The registry warning was disproved by the referenced prior artifact on disk.
- The corrected review blocked with 1 critical / 7 warning / 0 suggestion findings. The real duplicate-current-route defect, shortcut focus behavior, collapsed status role, favicon ordering, and nested metadata filtering are corrected. The cumulative assistant-ui finding belongs to the already-reviewed C-03b diff and is excluded from the final C-10 packet; the maskable warning remains open because no delivered maskable asset exists.
- The route-corrected review passed with 0 critical / 6 warning / 2 suggestion findings. Modal-only suppression, same-shortcut close, native editable behavior, and two truthful install-icon sizes are now covered. The shared theme already enforces 1ms reduced-motion transitions, and checked-in `static/` regeneration remains deferred to the Wave 4 boundary after C-12.
- The next review blocked on favicon precedence: the unconditional fallback followed the media-specific icons and therefore won modern selection. The fallback now comes first. Its concrete warnings are also resolved: valid `contenteditable` values remain unintercepted, compact modal state closes at 901px, delivered wordmark typography renders in the host font context, the About heading is announced once, brand variants no longer double-download, and the shortcut label matches the platform. A subsequent receipt misclassified documented Base UI Enter behavior as absent; installed Base UI docs and the passing Enter navigation test prove `Autocomplete.Item.onClick` covers both pointer and highlighted-keyboard activation. Runs/About remain available from the compact header palette, Configure uses a generic current-state token, and the UAR touch icon is 180px.

## Constraint evaluation

- `c10-single-responsive-shell`: satisfied by inventory tests and exact rendered measurements.
- `c10-base-ui-and-layering`: satisfied by source imports, boundary validation, shared hook/store tests, and palette/sheet interaction tests.
- `c10-brand-and-accessibility`: satisfied by brand inventory/markup tests, DOM landmark/readiness coverage, Flat 2.0, and rendered focus/compact inspection.
- `c10-change-gates`: satisfied for the C-10 implementation tier. Full tests and production build remain correctly deferred to the Wave 4 boundary after C-12.

## Known local runtime condition

The local Vite preview ran without the backend on port 1906. Expected API proxy connection failures produced the intended `Unreachable` status and were not treated as shell console regressions. No client-side warning or error was present in the controlled browser log.
