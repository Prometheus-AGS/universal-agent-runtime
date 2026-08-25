## 1. Accessibility and State Contract

- [x] 1.1 Add focused tests for provider-specific labels and descriptions, status/alert semantics, clean/dirty/saving action states, refresh protection, and browser unload; verify the new assertions fail against the current panel.

## 2. Accessible Provider Controls

- [x] 2.1 Extend shared settings primitives with backward-compatible label, description, control-id, action-name, and live-region semantics; verify shared primitive and provider panel tests pass.
- [x] 2.2 Name provider cards and controls, connect stale-model guidance, and keep both simple and searchable picker variants accessible; verify the focused provider panel test passes.

## 3. Dirty-State Protection and Responsive Layout

- [x] 3.1 Derive provider modified state from the existing dirty map, disable Save while clean, disable Refresh while dirty or busy with an explanatory status, and install the dirty-only browser-unload guard; verify focused state tests pass.
- [x] 3.2 Stack provider fields at narrow widths while preserving the two-column desktop composition and containing long controls; verify the settings structure gate and responsive source assertions pass.

## 4. Change Verification

- [x] 4.1 Run TypeScript, lint, focused settings tests, and strict OpenSpec validation for `provider-settings-accessibility-dirty-state`.
- [x] 4.2 Run the frontend phase-completion build and test suite, the single post-edit Impeccable detector, and fresh-context adversarial review; record any baseline or review finding before closeout.
