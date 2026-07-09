## 1. Investigation

- [x] 1.1 Run a live `pnpm audit --json` re-check against the
      assessment-era snapshot — found 11 findings (4 high, 4 moderate, 3
      low): `vite` (direct), `undici` (via `jsdom`), `js-yaml` (via
      `eslint`), `esbuild` (via `vite` and, separately, `tsup`).
- [x] 1.2 For each transitive finding, check whether the parent's own
      declared range already permits the patched version before deciding
      an override is needed.
- [x] 1.3 `pnpm why esbuild` to trace the dual-resolution — found `tsup`
      (via `bundle-require`) pins `esbuild` to exactly `^0.27.0` with no
      compatible patched version in range.

## 2. Apply the fix

- [x] 2.1 `pnpm update vite` — resolved to `7.3.6` within the existing
      `^7.3.1` range.
- [x] 2.2 `pnpm -r update js-yaml undici` — both resolved within their
      parents' already-declared ranges (`@eslint/eslintrc`'s `^4.1.1`,
      `jsdom`'s `^7.25.0`).
- [x] 2.3 Add a single `pnpm-workspace.yaml` override for `esbuild`,
      pinned to the exact patched version (`"0.28.1"`), not an open-ended
      range.
- [x] 2.4 **Incident + correction**: an initial `pnpm audit --fix` run
      auto-generated an open-ended `vite` override that resolved to a
      major-version bump (`vite@8.1.3`) — caught via `pnpm install`'s
      dependency-diff output, reverted, and redone via 2.1-2.3 instead.

## 3. Verify

- [x] 3.1 `pnpm audit` — 0 vulnerabilities (down from 11).
- [x] 3.2 `pnpm -C frontend build` — succeeds, no new errors.
- [x] 3.3 `bun run typecheck` — clean.
- [x] 3.4 `bun run lint` — confirmed the 140 errors are pre-existing
      (verified via `git stash` + reinstall showing the identical count
      without this change's fix applied), not a regression.

## 4. Update docs and KBD state

- [x] 4.1 Add a disposition note to `docs/DEPENDENCY_MANAGEMENT.md` for
      the frontend npm remediation.
- [x] 4.2 Update
      `.kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json`
      (`change_status.frontend-npm-remediation` → DONE, `changes_completed`
      incremented, Round 2 marked complete, `next_change` →
      `sdk-typescript-lockfile-and-ci-audit-fix`).
