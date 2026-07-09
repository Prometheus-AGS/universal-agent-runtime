## 1. Investigation

- [x] 1.1 Confirm `surreal-memory` is the only one of D-D's 4 pinned git
      dependencies on `branch = "main"` rather than a fixed `rev`/`tag`
      (already confirmed during this phase's assessment).
- [x] 1.2 Resolve current `main` HEAD via
      `git ls-remote https://github.com/Prometheus-AGS/surreal-memory-server.git HEAD`
      — done at planning time (`f9ab1c29944b86d44c23ea0e6192fa3d39acbde8`).
- [x] 1.3 Re-verify the SHA immediately before applying — unchanged, no
      drift.

## 2. Apply the fix

- [x] 2.1 `Cargo.toml`: `surreal-memory`'s `branch = "main"` →
      `rev = "f9ab1c29944b86d44c23ea0e6192fa3d39acbde8"`.
- [x] 2.2 `cargo check` to regenerate `Cargo.lock` scoped to just this
      manifest edit (learned from the prior phase's
      `direct-network-facing-vulns` incident — manifest-edit + scoped
      `cargo check`, not a bare `cargo update`, to avoid unrelated churn).

## 3. Verify

- [x] 3.1 `cargo check --lib --tests` — clean.
- [x] 3.2 `cargo test --lib` — no regression vs. baseline (387/388, 1
      pre-existing ignore).
- [x] 3.3 `cargo clippy --lib` — zero new warnings vs. baseline (499).
- [x] 3.4 `cargo audit` — no new findings; confirmed `surreal-memory`'s
      resolved crate versions are unchanged (pinning to current HEAD did
      not shift anything).
- [x] 3.5 Confirmed `Cargo.lock`'s `surreal-memory` entry now shows `rev`,
      not `branch`.

## 4. Update docs and KBD state

- [x] 4.1 Update
      `.kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/progress.json`
      (`change_status.pin-surreal-memory-to-sha` → DONE, `changes_completed`
      incremented, `next_change` → `fix-d-d-pin-characterization`).
