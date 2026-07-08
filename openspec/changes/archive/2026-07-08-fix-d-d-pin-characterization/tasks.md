## 1. Investigation

- [x] 1.1 Confirm `docs/ARCHITECTURE.md`'s D-D bullet text at the time
      of this change (already confirmed during phase assessment).
- [x] 1.2 Re-verify all 4 D-D-listed dependencies' actual pins directly
      against `Cargo.toml` (not just re-trust the assessment's earlier
      finding): `rmcp` (`rev`), `surreal-memory` (`rev`, post
      `pin-surreal-memory-to-sha`), `kreuzberg` (`tag`),
      `prometheus_parking_lot` (`rev`).
- [x] 1.3 While re-verifying, checked `docs/DEPENDENCY_MANAGEMENT.md`'s
      "Current Pinned Versions" table against the same live state — found
      `rmcp` and `prometheus_parking_lot` both stale (bumped in later
      phases without the table being updated), and `surreal-memory`'s
      prior `rev` value was aspirational (didn't match the `branch=main`
      state actually in `Cargo.toml` before this phase's fix).

## 2. Apply the fix

- [x] 2.1 `docs/ARCHITECTURE.md`: rewrite the D-D bullet to accurately
      describe all 4 pins (3 SHA-pinned, 1 tag-pinned, none floating),
      noting `surreal-memory`'s move off `branch = "main"` this phase.
- [x] 2.2 `docs/DEPENDENCY_MANAGEMENT.md`: correct all 3 drifted entries
      in the "Current Pinned Versions" table to match live `Cargo.toml`,
      with a note explaining the correction.

## 3. Verify

- [x] 3.1 Proofread both doc sections against live `Cargo.toml` grep
      output — all 4 entries now match exactly.
- [x] 3.2 No code/build impact expected (docs-only) — confirmed via
      `git diff --stat` showing only the 2 doc files touched.

## 4. Update docs and KBD state

- [x] 4.1 (docs already updated in step 2, this is the same task)
- [x] 4.2 Update
      `.kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/progress.json`
      (`change_status.fix-d-d-pin-characterization` → DONE,
      `changes_completed` incremented, `next_change` →
      `triage-unassigned-unmaintained-warnings`).
