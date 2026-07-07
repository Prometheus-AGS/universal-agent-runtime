## 1. Reachability / usage investigation

- [x] 1.1 Grep all `serde_yml::` call sites in `src/` — found exactly 3:
      `Cargo.toml` declaration, `src/uar/compiler/parser.rs`
      (`deserialize_section`), `src/uar/runtime/skills/storage/filesystem.rs`
      (`parse_skill_file`, `serialize_skill_to_md`). No other usage.
- [x] 1.2 Confirm `libyml` is pulled in only transitively via `serde_yml`
      (`cargo tree -i libyml` before the swap) — no direct dependency on it.
- [x] 1.3 Re-run `cargo audit` and grep for `anyhow`/`memmap2` — confirmed
      neither appears at the currently pinned versions (`anyhow` 1.0.103,
      `memmap2` 0.9.11). The assessment's 2026-06-20/25 unsoundness reports
      do not apply here; no fix needed.
- [x] 1.4 During the `cargo audit` re-run, noticed `quinn-proto` 0.11.14
      (`RUSTSEC-2026-0185`) still listed in `Cargo.lock` but
      `cargo tree -i quinn-proto --target all --all-features` resolves to
      nothing — orphaned lockfile entry, not actually in the resolved
      dependency graph. Confirmed `reqwest`'s enabled features
      (`json`, `stream`, `rustls-tls-native-roots`, `multipart`) don't
      include HTTP/3 either way. Disclosed in `proposal.md`; not assigned
      to any of this phase's 8 changes, left as-is (likely self-resolves on
      a future full relock).

## 2. Apply the fix

- [x] 2.1 `Cargo.toml`: `serde_yml = "0.0.12"` → `serde_norway = "0.9.42"`.
- [x] 2.2 `src/uar/compiler/parser.rs`: `serde_yml::from_str` →
      `serde_norway::from_str`, `serde_yml::Error` → `serde_norway::Error`
      in the mapped error closure (all section-deserialize call sites).
- [x] 2.3 `src/uar/runtime/skills/storage/filesystem.rs`:
      `serde_yml::from_str`/`serde_yml::to_string` →
      `serde_norway::from_str`/`serde_norway::to_string`.
- [x] 2.4 `cargo check` to regenerate `Cargo.lock` — confirm `serde_yml`
      and `libyml` both fully absent afterward
      (`cargo tree -i serde_yml` / `cargo tree -i libyml` → no match).
- [x] 2.5 Incidental: `build.rs`'s `which()` — `.map(...).unwrap_or(false)`
      → `.is_ok_and(...)` (clippy lint surfaced during verification, not
      part of the dependency swap itself).

## 3. Verify

- [x] 3.1 `cargo check --lib --tests` — clean.
- [x] 3.2 `cargo test --lib` — 387/388 pass (1 pre-existing ignore), no
      regression vs. the baseline from the prior 3 changes.
- [x] 3.3 `cargo clippy --lib` — 499 warnings, same as baseline, zero new.
- [x] 3.4 `cargo audit` — confirmed `serde_yml`/`libyml` no longer appear;
      `anyhow`/`memmap2` confirmed absent (no regression).

## 4. Update docs and KBD state

- [x] 4.1 Add a disposition note to `docs/DEPENDENCY_MANAGEMENT.md` for
      the `serde_yml` → `serde_norway` swap (mirrors the pattern used for
      the prior 3 changes) plus the `quinn-proto` orphaned-lockfile
      observation.
- [x] 4.2 Update
      `.kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json`
      (`change_status.first-party-direct-dep-hygiene` → DONE,
      `changes_completed` incremented, `next_change` → `grcov-toolchain-refresh`).
