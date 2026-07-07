## Why

`cargo audit` flags `serde_yml`/`libyml` as both **unmaintained and
unsound** — unlike the git-pinned dependencies in `docs/ARCHITECTURE.md`'s
D-D decision (`rmcp`, `surreal-memory`, `kreuzberg`,
`prometheus_parking_lot`), `serde_yml` is a direct, first-party-controllable
dependency in `Cargo.toml`: UAR can simply stop depending on it, no upstream
fix needed. The assessment also flagged `anyhow`/`memmap2` unsoundness
warnings from 2026-06-20/25 as worth re-checking against currently pinned
versions before assuming they still apply.

## What Changes

- Replaced `serde_yml = "0.0.12"` with `serde_norway = "0.9.42"`
  (actively maintained fork with the same `Serialize`/`Deserialize`-based
  API) across all 3 call sites:
  - `Cargo.toml` — dependency declaration.
  - `src/uar/compiler/parser.rs` — agent-spec YAML section
    deserialization (`serde_yml::from_str` → `serde_norway::from_str`,
    `serde_yml::Error` → `serde_norway::Error` in the mapped error type).
  - `src/uar/runtime/skills/storage/filesystem.rs` — skill manifest
    frontmatter parse (`from_str`) and serialize (`to_string`).
  - Verified via `cargo tree -i libyml` that `libyml` (the unsound native
    library pulled in only transitively through `serde_yml`) is now fully
    absent from `Cargo.lock` — not just deprioritized, eliminated.
- Re-checked `anyhow` (currently pinned `1.0.103`) and `memmap2` (currently
  pinned `0.9.11`, pulled via `kreuzberg` and dev-only `grcov`) against a
  fresh `cargo audit` run: **neither is currently flagged** at these pinned
  versions. The 2026-06-20/25 unsoundness reports the assessment surfaced
  do not apply to the versions already in `Cargo.lock` — disclosed as
  checked-clean, no code change needed or made.
- Incidental: fixed a `cargo clippy` `map_err_ignore`-style lint in
  `build.rs`'s `which()` helper (`.map(...).unwrap_or(false)` →
  `.is_ok_and(...)`), surfaced while re-running clippy to verify this
  change introduces no new warnings. Unrelated to the dependency swap
  itself but bundled here since it was caught in the same verification
  pass.

### Out of scope (flagged, not fixed here)

- `atty`/`failure`/`instant`/`bincode` and friends: unmaintained/unsound
  warnings, all pulled exclusively via the dev-only `grcov` /
  `cargo-binutils` toolchain — that's `grcov-toolchain-refresh`'s scope,
  not this change's.
- `quinn-proto` `RUSTSEC-2026-0185`: present in `Cargo.lock` but
  `cargo tree -i quinn-proto --target all --all-features` finds **zero**
  reverse dependencies — an orphaned lockfile entry, not actually reachable
  in the resolved graph. `reqwest`'s enabled features
  (`json`, `stream`, `rustls-tls-native-roots`, `multipart`) confirm HTTP/3
  is not activated, so this was never reachable via `reqwest` either.
  Likely to self-resolve on a future full `cargo update`; not assigned to
  any of this phase's 8 changes, flagging here since it surfaced during
  this change's `cargo audit` re-run rather than silently dropping it.

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "First-Party Direct Dependency
  Currency" requirement (replace unmaintained/unsound direct dependencies
  with maintained equivalents; re-verify assessment-era findings against
  currently pinned versions before acting). This capability itself was
  added retroactively across this phase's first 3 changes once the
  `openspec validate`/archive gap was caught while applying this 4th
  change — see `findings.md`. Otherwise no other spec-level requirement
  changes; this replaces one YAML serialization crate with a drop-in
  equivalent and discloses two non-issues, no UAR-observable behavior
  changes.

## Impact

- **Affected code**: `Cargo.toml`, `Cargo.lock`, `src/uar/compiler/parser.rs`,
  `src/uar/runtime/skills/storage/filesystem.rs`, `build.rs` (incidental
  clippy fix).
- **Runtime UX / provider compatibility / realtime state**: none —
  `serde_norway` is API-compatible with `serde_yml` for the
  `from_str`/`to_string` calls in use; agent-spec YAML parsing and skill
  manifest frontmatter parsing behave identically.
- **KBD workflow state**: `progress.json` for
  `uar-dependabot-remediation-2026-07` to be updated to DONE for this
  change once verified; `quinn-proto` orphaned-lockfile observation noted
  above for a future round's shared `cargo audit` checkpoint, not a new
  tracked change.
