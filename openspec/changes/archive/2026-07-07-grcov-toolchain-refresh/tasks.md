## 1. Investigation

- [x] 1.1 Confirm `grcov = "0.8"` in `Cargo.toml`'s `[dev-dependencies]` has
      zero library call sites (`grep -rn "use grcov\|grcov::"`).
- [x] 1.2 Confirm the actual coverage tooling
      (`tools/coverage.sh`, `tools/test-all.sh`, 3 CI workflows) invokes
      `grcov` as an independently `cargo install`'d CLI binary, not as a
      linked library dependency.
- [x] 1.3 Check latest `grcov` release (crates.io) for whether it dropped
      the `cargo-binutils`/old-`clap` chain — moot once 1.1/1.2 confirmed
      the dependency itself is unused; a version bump was not the right
      fix.
- [x] 1.4 Trace via `cargo tree -i` which of the plan's listed crates
      (`failure`, `atty`, `ansi_term`, `paste`, `proc-macro-error`) are
      actually exclusive to `grcov` — found `paste` is not (via kreuzberg/
      burn instead); corrected in `proposal.md`/`findings.md`.

## 2. Apply the fix

- [x] 2.1 Remove `grcov = "0.8"` from `Cargo.toml`'s `[dev-dependencies]`.
- [x] 2.2 `cargo check` to regenerate `Cargo.lock` — confirm `grcov`,
      `atty`, `failure`, `ansi_term`, `proc-macro-error`, `cargo-binutils`,
      `tabled`, `tabled_derive`, `rustc-cfg` all absent afterward.

## 3. Verify

- [x] 3.1 `cargo check --lib --tests` — clean.
- [x] 3.2 `cargo test --lib` — 387/388 pass (1 pre-existing ignore), no
      regression.
- [x] 3.3 `cargo clippy --lib` — 499 warnings, same as baseline, zero new.
- [x] 3.4 `cargo audit` — confirmed the grcov-exclusive chain cleared;
      disclosed `proc-macro-error2` as pre-existing/unrelated (confirmed
      via `git diff Cargo.lock`).

## 4. Update docs and KBD state

- [x] 4.1 Add a disposition note to `docs/DEPENDENCY_MANAGEMENT.md` for
      the `grcov` removal + the `paste`/`proc-macro-error2` corrections.
- [x] 4.2 Update
      `.kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json`
      (`change_status.grcov-toolchain-refresh` → DONE, `changes_completed`
      incremented, Round 1 marked complete, `next_change` → Round 2's
      first change).
