## Why

`vite.config.ts` uses the function form of `build.rollupOptions.output.manualChunks`,
added last phase as a stopgap when Vite 8 (Rolldown) removed the object
form. That function form is itself deprecated in favor of Rolldown's
native `codeSplitting` API — leaving it in place means migrating twice
instead of once, and risks breaking silently on a future Vite release
that drops the deprecated form entirely.

## What Changes

- Rename `build.rollupOptions` → `build.rolldownOptions` in `vite.config.ts`.
- Replace the `manualChunks(id) {...}` function with
  `codeSplitting.groups: [{ name(moduleId) {...} }]`, preserving the
  exact same 4 vendor-chunk match logic (`vendor-react`, `vendor-assistant`,
  `vendor-query`, `vendor-hljs`).
- No behavior change intended — same chunk groupings, same
  `chunkSizeWarningLimit`.

## Capabilities

### New Capabilities

- `frontend-build-tooling`: covers the frontend production build's chunk-splitting configuration, currently just the Rolldown migration; expected to gain more requirements as the build pipeline evolves.

### Modified Capabilities

(none)

## Impact

- `frontend/vite.config.ts` only.
- Verified via `pnpm run build` producing the same chunk groupings.
- No runtime, API, or realtime-state impact — build-time config only.
- KBD workflow state: belongs to phase `uar-security-audit-alerts-gate-2026-07`;
  updated via `/kbd-apply`, not manually.
