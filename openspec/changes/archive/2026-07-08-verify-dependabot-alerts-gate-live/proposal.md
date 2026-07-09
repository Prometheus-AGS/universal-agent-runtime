## Why

`add-dependabot-alerts-ci-gate` (this phase, already archived) added a new
`dependabot-alerts-gate` job to `security-audit.yml`, verified only via
local dry-run against this session's own `gh` credentials — not the
`secrets.SUBMODULES_TOKEN` the job actually uses in CI, and not on the
real GitHub Actions platform. Per this project's own `CI Trigger Actually
Fires` requirement (`dependency-security-posture` capability, added
`uar-post-dependabot-followup-2026-07`), a CI change SHALL NOT be
considered verified until observed firing for real — this change fulfills
that existing requirement for the new job specifically.

## What Changes

- Confirm the 2 Round 1 changes (already pushed to `main`) trigger a real
  `security-audit.yml` run via `gh workflow run` (`workflow_dispatch`).
- Inspect the run's `dependabot-alerts-gate` job specifically: does
  `secrets.SUBMODULES_TOKEN` have sufficient scope to read the Dependabot
  alerts API from inside Actions, or does the fail-loud preflight check
  fire (meaning the token-source decision needs revisiting)?
- No code changes are anticipated unless the live run surfaces a real bug
  — if so, document findings and fix in this same change per this
  project's established practice (see `push-and-verify-security-audit-workflow`
  precedent, which found and fixed 2 real Vite regressions mid-verification).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `dependency-security-posture`: extends the existing `CI Trigger
  Actually Fires` requirement with a scenario specific to jobs whose
  correctness depends on a credential's *runtime scope* (not just its
  presence) — a local dry-run with a different credential (as
  `add-dependabot-alerts-ci-gate` used) does not confirm the real
  secret's scope is sufficient; this change closes that verification gap
  for real.

## Impact

- No source files, unless the live run surfaces a real bug requiring a
  fix (would be disclosed in `findings.md` if so).
- Confirms whether `verify-dependabot-alerts-gate-live`'s own scope
  (real-CI confirmation) is achievable, or surfaces a genuine blocker
  (insufficient token scope) that becomes a follow-up for the next phase.
