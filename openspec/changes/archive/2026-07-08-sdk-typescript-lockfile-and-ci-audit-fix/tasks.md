## 1. Investigation

- [x] 1.1 Confirm `sdks/typescript` has no lockfile at all (`find` for
      `package-lock.json`/`pnpm-lock.yaml`/`yarn.lock`).
- [x] 1.2 Confirm `vitest`'s declared `^2.0.0` range falls entirely inside
      `GHSA-5xrq-8626-4rwp`'s vulnerable window (`<3.2.6` or
      `>=4.0.0 <4.1.0`) — a range bump is required, not just a lockfile
      regenerate.
- [x] 1.3 Check `release.yml`'s actual trigger condition and confirm (per
      the assessment's `gh run list --workflow=release.yml`) it has never
      fired — the doc's claim about "CI runs cargo audit" is technically
      present-in-file but never-executed-in-practice.
- [x] 1.4 Test whether cargo-audit's `audit.toml` config auto-discovery
      works with the installed version (0.22.2) — it did not; use
      explicit `--ignore` CLI flags instead.

## 2. Apply the fix

- [x] 2.1 `sdks/typescript/package.json`: bump `vitest` `^2.0.0` →
      `^4.1.10`.
- [x] 2.2 `npm install` in `sdks/typescript/` to generate a real
      `package-lock.json`.
- [x] 2.3 Add `"overrides": { "esbuild": "0.28.1" }` (exact version, not
      an open range) to resolve the residual `tsup`-pinned `esbuild`
      finding.
- [x] 2.4 Add `.github/workflows/security-audit.yml` (weekly cron +
      `workflow_dispatch`, 4 jobs: cargo audit, npm audit root, pnpm audit
      frontend, npm audit sdks/typescript).
- [x] 2.5 Correct `docs/DEPENDENCY_MANAGEMENT.md`'s stale claim about
      `release.yml` running `cargo audit`.

## 3. Verify

- [x] 3.1 `npm audit` in `sdks/typescript/` — 0 vulnerabilities (was 1).
- [x] 3.2 `tsc --noEmit` — clean.
- [x] 3.3 `tsup` build — succeeds.
- [x] 3.4 `vitest --run` — runs; exits non-zero on "no test files found"
      (pre-existing, disclosed, not exercised by any current CI workflow).
- [x] 3.5 New workflow YAML parses correctly (`python3 -c "import yaml;
      yaml.safe_load(...)"`).
- [x] 3.6 Each job's underlying command run directly, matching the
      workflow's exact flags: `cargo audit` with the 7-ID ignore list →
      exit 0; `npm audit` (root) → 0 vulnerabilities; `pnpm audit`
      (frontend) → 0 vulnerabilities; `npm audit` (sdks/typescript) → 0
      vulnerabilities. `workflow_dispatch`/`schedule` firing on GitHub
      itself disclosed as unverifiable from this local session.

## 4. Update docs and KBD state

- [x] 4.1 `docs/DEPENDENCY_MANAGEMENT.md` updated (Security Advisories
      section corrected + new `sdks/typescript` disposition section).
- [x] 4.2 Update
      `.kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json`
      (`change_status.sdk-typescript-lockfile-and-ci-audit-fix` → DONE,
      `changes_completed` → 8/8, phase ready for `/kbd-reflect`).
