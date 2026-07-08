## Why

`surreal-memory` is the only one of D-D's 4 pinned git dependencies
(`rmcp`, `surreal-memory`, `kreuzberg`, `prometheus_parking_lot`) still on
a floating `branch = "main"` rather than a fixed commit/tag. Every
`cargo update` (routine or accidental) can silently pull in whatever
upstream `main` currently points to — this already had a real effect in
`uar-dependabot-remediation-2026-07`'s `surreal-memory-transitive-vulns`
change, where `ammonia`/`crossbeam-epoch` advisories became reachable
through `surreal-memory` → `surrealdb-core`. The user was asked directly
(via `AskUserQuestion` during this phase's planning) whether to pin to a
SHA, re-affirm the float, or defer the decision — they chose to pin to a
SHA, matching the pattern already used for `rmcp` and
`prometheus_parking_lot`.

## What Changes

- `Cargo.toml`: `surreal-memory`'s git dependency changed from
  `branch = "main"` to `rev = "f9ab1c29944b86d44c23ea0e6192fa3d39acbde8"`
  (re-verified as `main`'s current HEAD via `git ls-remote` both at
  planning time and immediately before this change's execution — no
  drift).
- `Cargo.lock` regenerated scoped to just this manifest edit.
- No application code changes — pinning to the branch's current HEAD
  resolves to the same commit that was already in use, so no crate
  version changes are expected as a side effect.

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "Floating Git Dependency
  Resolution" requirement (a git-pinned dependency intended to be
  reproducible SHALL be pinned to a fixed commit or tag, not a floating
  branch, unless a floating pin is an explicit, documented choice).

## Impact

- **Affected code**: `Cargo.toml`, `Cargo.lock`.
- **Runtime UX / provider compatibility / realtime state**: none — the
  resolved commit is unchanged (pinning to current HEAD of the branch
  already in use), so no behavior change is expected.
- **KBD workflow state**: `progress.json` for
  `uar-post-dependabot-followup-2026-07` updated to DONE for this change
  once verified. `fix-d-d-pin-characterization` (next change) depends on
  this landing first so its wording reflects the new pin state.
