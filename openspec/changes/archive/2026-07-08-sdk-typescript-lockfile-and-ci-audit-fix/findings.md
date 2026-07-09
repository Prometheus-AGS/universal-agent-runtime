# Findings: sdk-typescript-lockfile-and-ci-audit-fix

## `sdks/typescript` lockfile + vitest bump

- No lockfile existed at all (`find sdks/typescript -iname "*.lock*"` /
  `package-lock.json` / `pnpm-lock.yaml` / `yarn.lock` → nothing).
- `vitest` was declared `"^2.0.0"` — entirely inside Dependabot's
  vulnerable range (`< 3.2.6` or `>= 4.0.0, < 4.1.0`). Bumped to
  `^4.1.10` (current stable; matches `frontend/`'s vitest line).
- No workspace config links `sdks/typescript` to the root npm project or
  the `frontend/` pnpm workspace — it's a fully standalone package. Chose
  `npm` (matching the root project's own package manager) over `pnpm`,
  since nothing about this package requires pnpm's workspace features.
- Post-bump `npm install` surfaced 1 residual `esbuild` finding
  (`GHSA-g7r4-m6w7-qqqr`, low) via `tsup` (`^0.27.0`, no compatible patch
  in range) — same root cause as `frontend-npm-remediation`'s `esbuild`
  finding. `npm audit fix` could not resolve it (no change after running);
  added `"overrides": { "esbuild": "0.28.1" }` to `package.json`, pinned to
  the exact patched version per the lesson from that prior change's
  open-ended-override incident.
- `npm audit`: 0 vulnerabilities (was 1, after the vitest bump alone).
- `tsc --noEmit`: clean.
- `tsup src/index.ts --format cjs,esm --dts`: succeeds (only a
  pre-existing, unrelated `package.json` `exports` field warning about
  condition ordering — not touched, out of scope for this change).
- `vitest --run`: runs, but exits non-zero with "No test files found" —
  `sdks/typescript` has zero test files (`src/` contains only
  `index.ts`). This is pre-existing (not introduced by the vitest bump)
  and not currently exercised by any CI workflow (grepped
  `.github/workflows/*.yml` — only `template-cleanup.yml` references this
  package's `package.json`, for placeholder-token substitution, never for
  build/test). Disclosed, not fixed — writing a test suite is out of
  scope for a dependency-security change.

## CI audit-trigger fix

- Confirmed `release.yml` does contain a `cargo audit` step (and a `bun
  audit` step) — the doc's claim wasn't fabricated, but the workflow's own
  trigger (`push: tags: v*.*.*` / `release: types: [published]`) has never
  fired in this repo's history (per the assessment's `gh run list
  --workflow=release.yml` check), so the step has never actually executed.
- Added `.github/workflows/security-audit.yml`: weekly cron
  (`0 6 * * 1`) + `workflow_dispatch`, 4 independent jobs (`cargo audit`,
  `npm audit` root, `pnpm audit` frontend, `npm audit`
  sdks/typescript) — deliberately a new file rather than editing
  `release.yml`'s trigger, per this phase's `plan.md` decision.
- `cargo audit`'s job explicitly `--ignore`s exactly the 7 RUSTSEC IDs
  already disclosed with a documented disposition in
  `docs/DEPENDENCY_MANAGEMENT.md`: `RUSTSEC-2026-0118`/`-0119`
  (hickory-proto, not reachable), `RUSTSEC-2026-0187`/`-0194`
  (lopdf/quick-xml via kreuzberg, mitigated), `RUSTSEC-2026-0195`
  (quick-xml, not reachable), `RUSTSEC-2026-0185` (quinn-proto, orphaned
  lockfile entry), `RUSTSEC-2023-0071` (rsa, accepted risk — no fix
  exists). Verified locally: `cargo audit` with these 7 `--ignore` flags
  exits 0; without them, it exits 1 (11 vulnerabilities reported, several
  are the same advisory across multiple resolved crate versions).
  Unmaintained/unsound *warnings* (9 currently, e.g. `paste`, `instant`,
  `scc` — none of them assigned to any of this phase's 8 changes) do
  **not** cause a non-zero exit by cargo-audit's own default behavior
  (confirmed empirically) — they still print for visibility but won't
  make this workflow permanently red.
  - Tried an `audit.toml` config file first (cargo-audit historically
    documents this) — empirically, this installed version (0.22.2) did
    not pick it up from the repo root; used explicit `--ignore` CLI flags
    instead, which are demonstrably deterministic and more visible in a
    PR diff anyway.
- `npm audit` (root) and `pnpm audit` (frontend) need **no** ignore list —
  both changes 6 and 7 already brought them to 0 vulnerabilities; the new
  workflow will fail immediately if that regresses.
- Corrected `docs/DEPENDENCY_MANAGEMENT.md`'s stale claim to describe the
  actual trigger and ignore-list rationale.

## Verification limitation (disclosed)

`schedule`/`workflow_dispatch` triggers can only be exercised for real
once committed to the default branch on GitHub — not something this
session can trigger locally. Verified instead: the workflow YAML parses
correctly (`python3 -c "import yaml; yaml.safe_load(...)"`), and every
job's underlying command was run directly in this environment with the
exact flags the workflow uses, confirming the expected exit codes (all
four: exit 0 / 0 vulnerabilities). This is disclosed as a verification
gap rather than claiming an on-GitHub dispatch test that wasn't actually
performed.
