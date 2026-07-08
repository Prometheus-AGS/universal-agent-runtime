# Findings: fix-d-d-pin-characterization

## Corrections made

| Location | Before | After |
|---|---|---|
| `docs/ARCHITECTURE.md` D-D bullet | Claimed `kreuzberg` tracks `branch = "main"`; implied `surreal-memory` was SHA-pinned | Corrected: `rmcp`/`surreal-memory`/`prometheus_parking_lot` are SHA-pinned, `kreuzberg` is tag-pinned (`v4.9.8`), none float |
| `docs/DEPENDENCY_MANAGEMENT.md` pinned-versions table — `rmcp` | `rev "085470025f690050e8776ffa939e7ba71d3abc01"` (stale) | `rev "26b65b6b88c5552447905923f683b6e4720a5600"` (live) |
| `docs/DEPENDENCY_MANAGEMENT.md` pinned-versions table — `surreal-memory` | `rev "c6f95c905c16907ad58ef9049f32dcc9531d40eb"` (was aspirational — actual state was `branch = "main"` until this phase's `pin-surreal-memory-to-sha`) | `rev "f9ab1c29944b86d44c23ea0e6192fa3d39acbde8"` (matches the new real pin) |
| `docs/DEPENDENCY_MANAGEMENT.md` pinned-versions table — `prometheus_parking_lot` | `rev "32b481d6c5694545d35789894f6feecf5ac4ca3e"` (stale) | `rev "ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0"` (live) |
| `docs/DEPENDENCY_MANAGEMENT.md` pinned-versions table — `kreuzberg` | `tag "v4.9.8"` | unchanged — already accurate |

## Scope note: found more drift than the phase's original goal

The phase's Goal 1 only named `ARCHITECTURE.md`'s D-D bullet. While
proofreading the correction against live `Cargo.toml`, the parallel table
in `DEPENDENCY_MANAGEMENT.md` was checked too (not assumed correct just
because it was updated during the prior phase) and found 3 of its 4
entries stale — `rmcp` and `prometheus_parking_lot` had both been bumped
in later phases (`uar-security-deps-and-hygiene` for `rmcp`, per the
existing comment at `Cargo.toml:151`) without this table being updated.
Fixed all 3 in this same change rather than opening a separate one, per
the `dependency-security-posture` capability's own new requirement (this
change's own delta spec) that a correction should catch parallel drift
found along the way.

## Verification

- All 4 D-D-listed dependencies' pins re-verified directly against live
  `Cargo.toml` via `grep` — both corrected documents now match exactly.
- `git diff --stat` confirms only `docs/ARCHITECTURE.md` and
  `docs/DEPENDENCY_MANAGEMENT.md` were touched — no code or build impact,
  consistent with a docs-only change.
