## 1. Investigation

- [x] 1.1 Run a live `npm audit --json` re-check against the
      assessment-era snapshot — found 15 findings (11 moderate, 4 high),
      all with `fixAvailable: true` (semver-compatible, no `--force`
      needed for any).
- [x] 1.2 `npm audit fix --dry-run` to confirm no finding requires a
      breaking/major bump before applying for real.
- [x] 1.3 Trace the `chevrotain`/`langium`/`@mermaid-js/parser`/`mermaid`
      chain — confirmed it's a single vulnerable `lodash-es` resolution
      propagating through 5 packages, not 5 independent issues.

## 2. Apply the fix

- [x] 2.1 `npm audit fix` (no `--force`).
- [x] 2.2 Confirm `package.json` has zero diff (lockfile-only change).

## 3. Verify

- [x] 3.1 `npm audit` re-run — 0 vulnerabilities.
- [x] 3.2 Root dev tools sanity check post-fix: `eslint`, `tsc`,
      `playwright`, `prettier`, `tailwindcss` — all respond correctly.
- [x] 3.3 `bun run build` (== `pnpm -C frontend build`) — succeeds, no new
      errors vs. baseline.

## 4. Update docs and KBD state

- [x] 4.1 Add a disposition note to `docs/DEPENDENCY_MANAGEMENT.md` for
      the npm-root remediation.
- [x] 4.2 Update
      `.kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json`
      (`change_status.npm-root-remediation` → DONE, `changes_completed`
      incremented, `next_change` → `frontend-npm-remediation`).
