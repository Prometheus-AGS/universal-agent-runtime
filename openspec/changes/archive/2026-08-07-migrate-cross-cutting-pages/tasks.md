## 1. Reconcile the absorbed change

- [x] 1.1 Replace the stale entity-store migration proposal with the C-10 app-shell scope and explicitly defer those migrations to C-14.
- [x] 1.2 Resolve cand-011 in favor of Base UI `Autocomplete` + `Dialog` for the new app command palette while preserving existing `cmdk` consumers.
- [x] 1.3 Complete the required UI/UX routing, current-library research, architecture analysis, and binding design distillation.

## 2. Establish one navigation model and shell state boundary

- [x] 2.1 Add a typed `app/shell/nav-destinations.ts` inventory with work, Configure, and system projections plus deterministic route matching.
- [x] 2.2 Extend the UI store and `useUiState` hook with serializable rail-collapse, command-palette, and mobile-sheet state/actions.
- [x] 2.3 Add focused tests for destination matching and shared hook/store transitions.

## 3. Build the responsive application shell

- [x] 3.1 Add the expanded/collapsed desktop navigation rail with work and Configure groups, accessible icon-only labels, and text-plus-color readiness.
- [x] 3.2 Add the compact top bar, four-target mobile tab bar, and id-based Base UI mobile sheet host.
- [x] 3.3 Add the inventory-derived breadcrumb header, skip navigation, and single main-content composition.
- [x] 3.4 Add the Base UI command palette with filtering, keyboard auto-highlight, global Control/Meta+K access outside editable controls, and close-on-navigation.
- [x] 3.5 Wire `App.tsx` to the kebab-case shell tree and retire the superseded `AppShell.tsx` and `navigation.ts` files without changing feature routes.

## 4. Install the delivered UAR brand

- [x] 4.1 Copy `docs/ui/logo/` into `frontend/public/brand/` without operating-system metadata.
- [x] 4.2 Add `shared/ui/uar-logo.tsx`, migrate current React brand consumers, and retire `KnowMeLogo.tsx`.
- [x] 4.3 Update favicon and manifest references to delivered UAR assets and add deterministic asset/markup coverage.

## 5. Verify and close C-10

- [x] 5.1 Add focused interaction coverage for desktop collapse, compact navigation, Configure-sheet routing, breadcrumbs, command access, readiness labels, landmarks, and accessible names.
- [x] 5.2 Pass frontend typecheck, lint, architecture boundaries, Flat 2.0, focused tests, strict OpenSpec validation, and diff-integrity checks.
- [x] 5.3 Complete the manual audit/critique/polish fallback, artifact refinement, and isolated adversarial review; remediate actionable findings.
- [x] 5.4 Run OpenSpec verification, record canonical C-10 completion, sync the capability, and archive the change before starting C-11.
