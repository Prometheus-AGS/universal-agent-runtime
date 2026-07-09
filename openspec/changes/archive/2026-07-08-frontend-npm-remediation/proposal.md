## Why

A live `pnpm audit` against `frontend/package.json`/`frontend/pnpm-lock.yaml`
found 11 findings (4 high, 4 moderate, 3 low) — including Dependabot's
original alerts plus the assessment's net-new `undici` GHSA. All 11 had a
semver-compatible patched version available; none required a `pnpm-fix`
override that would force a breaking major bump.

## What Changes

- **`vite`** (direct devDependency, `^7.3.1`): high (`server.fs.deny`
  bypass) + moderate (`launch-editor` NTLMv2 hash disclosure), both fixed
  by `>=7.3.5`. Bumped via `pnpm update vite` to `7.3.6` (latest 7.x,
  patch-level within the existing `^7.3.1` range — `package.json`'s
  declared floor was tightened to `^7.3.6` by pnpm, not a manual edit).
- **`undici`** (transitive, via `packages/prometheus-entity-management`'s
  `jsdom` devDependency): 3 high + 2 moderate + 2 low findings, all fixed
  by `>=7.28.0`. `jsdom`'s own declared range (`^7.25.0`) already permits
  the patch — bumped via `pnpm -r update undici` to `7.28.0`, no override
  or `jsdom` version change needed.
- **`js-yaml`** (transitive, via `eslint`'s `@eslint/eslintrc`): moderate
  (quadratic-complexity DoS via repeated merge-key aliases), fixed by
  `>=4.2.0`. `@eslint/eslintrc`'s own declared range (`^4.1.1`) already
  permits the patch — bumped via `pnpm -r update js-yaml` to `4.3.0`, no
  override needed.
- **`esbuild`** (transitive, dual-resolved): low (arbitrary file read via
  the dev server on Windows), fixed by `>=0.28.1`. `vite@7.3.6`'s own
  range (`^0.27.0 || ^0.28.0`) already permits the patch and resolved
  cleanly to `0.28.1` for that path — but `tsup` (a build-only bundler
  devDependency of `packages/prometheus-entity-management`, used via
  `bundle-require`) pins `esbuild` to exactly `^0.27.0` with no newer
  `tsup` release available to relax it. Added a single
  `pnpm-workspace.yaml` override
  (`esbuild@>=0.27.3 <0.28.1: "0.28.1"`) pinned to the exact patched
  version (not an open-ended range) to force this one path. Verified
  `tsup`'s `esbuild` peer requirement (`>=0.18`) is loose enough to accept
  `0.28.1` safely, and confirmed the frontend build still succeeds after.
- **Correction of a mistake made mid-change**: `pnpm audit --fix`'s
  auto-generated override for `vite` (`>=7.3.5`, no upper bound) resolved
  to `vite@8.1.3` — an unintended **major** version bump. Caught before
  committing (via `pnpm install`'s dependency-diff output), reverted, and
  redone via the narrower, deliberate approach above instead of a blanket
  `--fix`.

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "pnpm Transitive Fix Within
  Declared Ranges" requirement (prefer bumping within a parent
  dependency's already-compatible range over adding an override; when an
  override is the only path, pin it to the exact patched version rather
  than an open-ended range, to avoid an unintended major bump). Otherwise
  no other spec-level requirement changes; this is a lockfile-only
  frontend remediation, no application source changed.

## Impact

- **Affected code**: `frontend/pnpm-lock.yaml`, `frontend/package.json`
  (vite's declared floor tightened `^7.3.1` → `^7.3.6`, no other manual
  edits), `frontend/pnpm-workspace.yaml` (new `overrides` section, 1
  entry: `esbuild`).
- **Runtime UX / provider compatibility / realtime state**: none — all 4
  affected packages are dev/build tooling (`vite`, `esbuild`, `jsdom`'s
  `undici`, `eslint`'s `js-yaml`), not runtime dependencies of the shipped
  application.
- **KBD workflow state**: `progress.json` for
  `uar-dependabot-remediation-2026-07` to be updated to DONE for this
  change once verified; this closes Round 2 (2/2).
