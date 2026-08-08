# C-14d handoff

C-14c leaves the frontend ready for the existing `base-ui-verification` change.

## Verified entry state

- The C-10 shared application shell is the only navigation and responsive shell.
- `frontend/src/admin/` is empty and all retained configuration routes compose directly from feature roots.
- The A2UI tester remains development-only under `features/a2ui`; MCP health is under `features/tools`.
- Direct TanStack Query, highlight.js, and Radix declarations are gone; retained Radix packages are transitive.
- The exact §6.3 boundary matrix passes production source and rejects all ten negative fixtures.
- Focused Vitest passes 26/26; affected Playwright smoke passes 6/6; manifest build passes; initial JS is 231,433/250,000 gzip bytes.
- The C-14c protected-path hash matches its entry baseline.

## C-14d work

Continue the existing `base-ui-verification` task list as the final C-14 wave gate. Reconcile its historical paths and counts against the migrated feature tree before editing, then run the full frontend suite and the verification matrix assigned by that change. Do not recreate the inner admin shell or direct Radix declarations to satisfy stale path-oriented tasks; translate those checks to the feature-owned equivalents while preserving their behavioral intent.

The active repository contains unrelated dirty protected paths. Preserve them and do not stage, commit, reset, or rewrite them.
