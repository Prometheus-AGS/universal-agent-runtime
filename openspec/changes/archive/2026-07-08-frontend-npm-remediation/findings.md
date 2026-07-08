# Findings: frontend-npm-remediation

## Live audit findings and disposition

| Package | Severity | Path | Fix |
|---|---|---|---|
| `vite` | high + moderate | direct devDependency, `^7.3.1` | `pnpm update vite` → `7.3.6` (within existing range; `package.json` floor tightened to `^7.3.6`) |
| `undici` (×7 findings: 3 high, 2 moderate, 2 low) | high/moderate/low | `packages/prometheus-entity-management` → `jsdom` (`^29.1.1`, already latest) → `undici` | `pnpm -r update undici` → `7.28.0` (within `jsdom`'s own `^7.25.0` range) |
| `js-yaml` | moderate | `.` → `eslint` → `@eslint/eslintrc` → `js-yaml` | `pnpm -r update js-yaml` → `4.3.0` (within `@eslint/eslintrc`'s own `^4.1.1` range) |
| `esbuild` | low | `.` → `vite` → `esbuild` (resolves fine); **also** `packages/prometheus-entity-management` → `tsup` → `bundle-require` → `esbuild` (pinned `^0.27.0`, no compatible patched version in range) | `pnpm-workspace.yaml` override `esbuild@>=0.27.3 <0.28.1: "0.28.1"` (pinned to the exact patched version) |

## Incident: an open-ended override caused an unintended major bump

The first attempt used `pnpm audit --fix`, which auto-generates
`pnpm-workspace.yaml` overrides. Its `vite` override
(`vite@>=7.0.0 <=7.3.4: ">=7.3.5"`) has **no upper bound** — `pnpm install`
resolved it to `vite@8.1.3`, a major-version jump far beyond the intended
patch fix. This was caught immediately from `pnpm install`'s own
dependency-diff output (`- vite 7.3.3` / `+ vite 8.1.3`) before committing
anything. Reverted via `git checkout -- package.json pnpm-lock.yaml
pnpm-workspace.yaml` and redone deliberately:

1. `pnpm update vite` (no override) → resolved cleanly to `7.3.6`, still
   within `^7.3.1`.
2. `pnpm -r update js-yaml undici` (no override) → both resolved within
   their respective parents' already-declared ranges.
3. For `esbuild`, traced the dual-resolution via `pnpm why esbuild`:
   `vite@7.3.6`'s own range (`^0.27.0 || ^0.28.0`) already permits
   `0.28.1`, but `tsup@8.5.1` (latest available) pins `esbuild` to
   exactly `^0.27.0` — no override-free path exists for that leg. Added
   a single override pinned to the **exact** patched version
   (`"0.28.1"`), not an open range, specifically to avoid repeating the
   `vite` mistake.

## Why the `esbuild` override is safe

`bundle-require`'s own `esbuild` requirement is a loose peer dependency
(`>=0.18`), and `tsup` only uses `esbuild`'s transform/build API (bundling
`packages/prometheus-entity-management` at devtime) — never its dev-server
mode, which is the actual vulnerable code path in the advisory
(`GHSA-g7r4-m6w7-qqqr`, Windows-only arbitrary file read via
`esbuild serve`). Verified the frontend build still succeeds end-to-end
with the override in place (see Verification below).

## Verification

- `pnpm audit`: "No known vulnerabilities found" (was 11: 4 high, 4
  moderate, 3 low).
- `pnpm -C frontend build`: succeeds, 5719 modules transformed, no new
  errors (same pre-existing Tailwind/PGlite warnings as baseline).
- `bun run typecheck` (`tsc -b`): clean, no errors.
- `bun run lint`: 215 problems (140 errors) — confirmed via `git stash` +
  reinstall that this is the **pre-existing** baseline (React Hooks rules
  in application source, e.g. `react-hooks/set-state-in-effect` in
  `use-mobile.tsx`), completely unrelated to `vite`/`esbuild`/`undici`/
  `js-yaml`. Not a regression from this change; not addressed here
  (out of scope for a dependency-security change).
