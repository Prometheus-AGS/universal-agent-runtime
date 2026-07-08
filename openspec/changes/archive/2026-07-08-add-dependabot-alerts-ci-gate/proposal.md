## Why

`security-audit.yml` runs `cargo audit` + 3x `npm`/`pnpm audit` on a weekly
schedule, but has no check against GitHub's own Dependabot/GHSA alert
feed. The prior phase (`uar-post-dependabot-followup-2026-07`) proved this
is a real, exploitable gap: 2 real CVEs (`cmov` CVE-2026-50185,
`opentelemetry_sdk` CVE-2026-48504) were caught only by a manual
`gh api dependabot/alerts` check triggered by a confusing push-time
vulnerability count — not by any CI job. `cargo audit`'s RustSec database
lags GitHub's GHSA database for at least one already-disclosed dependency
(`hickory-proto`) in this project's own history. Without an automated
check, this class of gap recurs silently every week the scheduled scan
runs green while a real, disclosed CVE sits unflagged.

## What Changes

- Add a new `dependabot-alerts-gate` job to `security-audit.yml` that
  calls `gh api repos/{owner}/{repo}/dependabot/alerts` and fails the
  job when an **open** alert isn't already disclosed in
  `docs/DEPENDENCY_MANAGEMENT.md`'s known-advisory sections.
- The job reads `secrets.SUBMODULES_TOKEN` (reused per this project's
  standing token, already scoped broadly enough for private submodule
  cloning across every workflow) rather than the default `GITHUB_TOKEN`,
  which cannot read this endpoint under any `permissions:` grant — a
  hard GitHub Actions platform limitation, not a config gap. The job
  fails loudly with a clear diagnostic message if the token lacks the
  needed scope (401/403), rather than silently passing or skipping.
- Update `docs/DEPENDENCY_MANAGEMENT.md` to document the new automated
  check, replacing the existing "check manually" language with
  "checked automatically by CI; manual check remains useful between
  runs."

## Capabilities

### New Capabilities

(none — this extends the existing `dependency-security-posture` capability)

### Modified Capabilities

- `dependency-security-posture`: adds a new requirement that GitHub's
  Dependabot/GHSA alert feed is checked automatically in CI as a
  complement to `cargo audit`/`npm audit`/`pnpm audit`, since those
  tools have been shown to under-cover vs. GitHub's own alert database.

## Impact

- `.github/workflows/security-audit.yml`: new job, ~30-40 lines.
- `docs/DEPENDENCY_MANAGEMENT.md`: prose update describing the new
  automated check.
- No runtime/application code affected — CI and docs only. No frontend,
  API, or realtime-state impact.
- KBD workflow state: this change belongs to phase
  `uar-security-audit-alerts-gate-2026-07`; `progress.json` and the
  waypoint are updated on completion per the standing KBD/OpenSpec
  bridge (`/kbd-apply`), not manually.
