## 1. Searchable Picker Contract

- [x] 1.1 Add focused tests for the 7/8 threshold, label-and-id filtering, no-match copy, duplicate-label disambiguation, and bounded pointer/keyboard selection; verify the focused provider panel test fails for the missing searchable path.

## 2. Provider Model Search

- [x] 2.1 Add a provider-scoped shadcn/Base UI model picker that preserves the simple select below eight options and uses a scroll-bounded combobox at eight or more; verify the focused provider panel and shared primitive tests pass.
- [x] 2.2 Preserve empty and stale model behavior, prevent arbitrary values, and keep the existing provider settings draft path; verify TypeScript and lint pass.

## 3. Change Verification

- [x] 3.1 Run the settings structure gate and strict OpenSpec validation for `provider-model-search`.
- [x] 3.2 Run the frontend phase-completion build and test suite before beginning `provider-settings-accessibility-dirty-state`, recording any baseline failure without changing unrelated code.
