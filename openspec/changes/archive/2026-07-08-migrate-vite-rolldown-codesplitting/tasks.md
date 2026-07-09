## 1. Config migration

- [x] 1.1 Rename `build.rollupOptions` to `build.rolldownOptions` in `frontend/vite.config.ts`.
- [x] 1.2 Replace the `manualChunks(id) {...}` function with `codeSplitting.groups: [{ name(moduleId) {...} }]`, preserving the exact same match logic for `vendor-react`, `vendor-assistant`, `vendor-query`, and `vendor-hljs`.

## 2. Verification

- [x] 2.1 Run `pnpm run build` in `frontend/` and confirm the same 4 vendor chunk files are emitted (by name/pattern) as before the change.
- [x] 2.2 Confirm `chunkSizeWarningLimit: 1100` still applies and no new build warnings appear.
- [x] 2.3 Confirm `git status --short` shows only `frontend/vite.config.ts` changed.
