## Why

`docs/ARCHITECTURE.md`'s D-D bullet described `kreuzberg` as tracking
`branch = "main"` and implied `surreal-memory` was SHA-pinned — the
reverse of live `Cargo.toml` state at the time (`kreuzberg` is tag-pinned;
`surreal-memory` was the one floating). The prior phase
(`uar-dependabot-remediation-2026-07`) found this during its reflection
but didn't fix it (Goal 4 NOT MET), seeding this phase. Now that
`pin-surreal-memory-to-sha` has landed, `surreal-memory` really is
SHA-pinned, so this correction can state the truth cleanly rather than
describing an interim state.

While verifying the correction, `docs/DEPENDENCY_MANAGEMENT.md`'s own
"Current Pinned Versions" table was found to have drifted too: `rmcp` and
`prometheus_parking_lot` both show stale `rev` values (bumped in later
phases without the table being updated), and `surreal-memory` showed a
`rev` value even before this phase's change — meaning that line was
aspirational, not descriptive, of the actual `branch = "main"` state at
the time.

## What Changes

- `docs/ARCHITECTURE.md`'s D-D bullet corrected: `rmcp`, `surreal-memory`,
  `prometheus_parking_lot` are SHA-pinned; `kreuzberg` is tag-pinned
  (`v4.9.8`); none float on a branch. Notes that `surreal-memory` was
  moved off `branch = "main"` in this same phase.
- `docs/DEPENDENCY_MANAGEMENT.md`'s "Current Pinned Versions" table
  re-verified against live `Cargo.toml` and corrected on all 3 entries
  that had drifted (`rmcp`, `surreal-memory`, `prometheus_parking_lot`) —
  only `kreuzberg`'s tag was already accurate.

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "Architectural Decision Record
  Accuracy" requirement (a decision record's factual claims about pinned
  dependency state SHALL be re-verified against live manifest state, not
  assumed correct, whenever a related change lands).

## Impact

- **Affected code**: `docs/ARCHITECTURE.md`, `docs/DEPENDENCY_MANAGEMENT.md`
  — documentation only, no code or build impact.
- **Runtime UX / provider compatibility / realtime state**: none.
- **KBD workflow state**: `progress.json` for
  `uar-post-dependabot-followup-2026-07` updated to DONE for this change
  once verified.
