## Why

`sdks/typescript/package.json` declared `"vitest": "^2.0.0"` with **no
lockfile at all** — Dependabot's critical alert (`GHSA-5xrq-8626-4rwp`,
arbitrary file read/execute via the Vitest UI server) affects `vitest < 3.2.6`
or `>= 4.0.0, < 4.1.0`, and the entire declared `^2.0.0` range falls inside
that vulnerable window, so this needs a range bump, not just a lockfile
regenerate. Separately, this whole phase exists because
`docs/DEPENDENCY_MANAGEMENT.md`'s claim that "the CI pipeline runs
`cargo audit` as part of the release workflow" turned out to describe a
step that has **never executed** — `release.yml` only triggers on a
version-tag push or published release, and this repo has never cut one.
Without a real routinely-firing audit trigger, this exact situation (a
52+ alert Dependabot backlog accumulating silently) recurs.

## What Changes

- Bumped `sdks/typescript/package.json`'s `vitest` from `^2.0.0` to
  `^4.1.10` (current stable, matching the version line `frontend/` already
  uses) and generated a real `package-lock.json` via `npm install`.
- Surfaced the same `esbuild`-via-`tsup` blocker seen in
  `frontend-npm-remediation` (`tsup` pins `esbuild` to exactly `^0.27.0`,
  no compatible patched release exists) — added an `overrides` entry in
  `sdks/typescript/package.json`, pinned to the exact patched version
  (`"0.28.1"`, not an open range, per the lesson from the prior change's
  incident).
- Added `.github/workflows/security-audit.yml`: a new, dedicated scheduled
  workflow (weekly cron + `workflow_dispatch`), deliberately **not**
  repurposing `release.yml`'s tag/release trigger (per this phase's
  `plan.md` decision — "when we release" and "how often we scan for CVEs"
  are separate concerns). Runs `cargo audit` (root), `npm audit` (root),
  `pnpm audit` (`frontend/`), and `npm audit` (`sdks/typescript/`) as 4
  independent jobs.
- `cargo audit`'s job ignores exactly the 7 RUSTSEC IDs this phase has
  already triaged and disclosed a disposition for in
  `docs/DEPENDENCY_MANAGEMENT.md` (hickory-proto ×2, lopdf, quick-xml ×2,
  quinn-proto, rsa) — a genuinely new, undisclosed advisory will fail the
  job; that's the signal this workflow exists to produce. The 9
  unmaintained/unsound warnings still print but don't fail the job by
  cargo-audit's own default behavior (verified locally).
- Corrected `docs/DEPENDENCY_MANAGEMENT.md`'s stale "release workflow runs
  cargo audit" claim to describe the actual new trigger.

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "Scheduled Audit Trigger
  Independence" requirement (a security-audit CI trigger must fire on its
  own schedule, not be nested inside an unrelated trigger like a release
  pipeline that may rarely or never fire; known/disclosed advisories are
  explicitly ignored so the workflow only alerts on genuinely new
  findings). Otherwise no other spec-level requirement changes.

## Impact

- **Affected code**: `sdks/typescript/package.json` (`vitest` bump,
  `overrides` entry), `sdks/typescript/package-lock.json` (new),
  `.github/workflows/security-audit.yml` (new),
  `docs/DEPENDENCY_MANAGEMENT.md`.
- **Runtime UX / provider compatibility / realtime state**: none —
  `sdks/typescript` is a published client SDK with no runtime dependency
  on the UAR server itself; the CI workflow change has no effect on any
  deployed system.
- **CI verification limitation**: `workflow_dispatch`/`schedule` triggers
  can only be exercised for real once this file exists on the default
  branch on GitHub — verified locally instead (YAML validity, and each
  job's underlying command run directly with the expected exit code:
  `cargo audit` with the ignore list → exit 0; `npm audit` root, `pnpm
  audit` frontend, `npm audit` sdks/typescript → all exit 0 / 0
  vulnerabilities). Disclosed rather than claiming an on-GitHub dispatch
  test that wasn't actually run.
- **KBD workflow state**: `progress.json` for
  `uar-dependabot-remediation-2026-07` to be updated to DONE for this
  change once verified; this is the phase's 8th and final change — the
  whole phase closes out (ready for `/kbd-reflect`) once archived.
