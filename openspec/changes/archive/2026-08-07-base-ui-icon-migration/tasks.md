# Tasks: base-ui-icon-migration

## 1. Reconcile the regenerated source

- [x] Audit every wrapper in the original 16-file list.
- [x] Confirm `frontend/src` has zero `@radix-ui/react-icons` imports.
- [x] Confirm `frontend/src` has zero direct `radix-ui` imports.

## 2. Close primitives

- [x] Confirm `dialog.tsx` uses Lucide `XIcon` and retains the accessible `Close` name.
- [x] Confirm `sheet.tsx` uses Lucide `XIcon` and retains the accessible `Close` name.
- [x] Verify close-button behavior in a focused primitive test.

## 3. Expansion and direction primitives

- [x] Confirm `accordion.tsx` uses Lucide chevrons.
- [x] Verify accordion expanded/collapsed semantics in a focused primitive test.
- [x] Confirm `navigation-menu.tsx` uses a Lucide directional chevron.
- [x] Confirm `breadcrumb.tsx` uses Lucide navigation and overflow icons.
- [x] Confirm breadcrumb composition remains covered by the focused composition suite.

## 4. Selection primitives

- [x] Confirm `radio-group.tsx` uses Base UI state plus a native filled-circle indicator.
- [x] Verify radio selection semantics in a focused primitive test.
- [x] Confirm `checkbox.tsx` uses Lucide `CheckIcon`.
- [x] Verify checkbox checked-state semantics in a focused primitive test.

## 5. Menu and peripheral wrappers

- [x] Confirm dropdown, context-menu, and menubar wrappers use Lucide indicators.
- [x] Confirm pagination, carousel, command, input-otp, and sidebar use Lucide icons.
- [x] Confirm `resizable.tsx` uses its native handle with no Radix icon dependency.
- [x] Preserve custom SVG brand and provider artwork as a distinct category.

## 6. Remove the unused dependency

- [x] Remove `@radix-ui/react-icons` from `frontend/package.json` with pnpm.
- [x] Regenerate `frontend/pnpm-lock.yaml` without the package.
- [x] Regenerate the root `pnpm-lock.yaml` without the package.
- [x] Confirm source, manifest, and both lockfiles contain no removed-package reference.
- [x] Verify frozen installs from the root and frontend workspace roots.

## 7. Validate the completed change

- [x] Run the focused primitive interaction tests.
- [x] Run frontend typecheck and lint.
- [x] Run architecture-boundary and Flat 2.0 gates.
- [x] Strictly validate the OpenSpec change and confirm the scoped diff.

## Completion Gate

All 28 checkpoints are complete. Application source uses Lucide for generic
interface icons, custom artwork remains intact, and neither maintained dependency
graph carries `@radix-ui/react-icons`.
