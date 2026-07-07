## Why

`cargo audit` reports 18 unmaintained/unsound warnings attributable to an
old toolchain chain (`cargo-binutils` → `clap` 2.x → `atty`/`ansi_term`;
`tabled` → `tabled_derive` → `proc-macro-error`), all pulled in via the
`grcov = "0.8"` entry in `Cargo.toml`'s `[dev-dependencies]`. Investigation
found this dependency is dead weight: nothing in the repository imports
`grcov`/`grcov::` as a library (`grep -rn "use grcov\|grcov::"` finds zero
call sites), and the actual coverage tool used in CI/local scripts
(`tools/coverage.sh`, `tools/test-all.sh`, `.github/workflows/tests-full.yml`,
`comprehensive-tests.yml`, `release.yml`) is a separately-installed CLI
binary via `cargo install grcov` — entirely decoupled from this workspace's
`Cargo.lock`. The dev-dependency entry provides no value and only exists to
compile an old, vulnerable-warning-laden chain on every `cargo check`/`test`.

## What Changes

- Removed the unused `grcov = "0.8"` line from `Cargo.toml`'s
  `[dev-dependencies]` entirely (mirrors the `testcontainers` removal
  precedent from `direct-network-facing-vulns`) — a complete fix, not a
  version bump or disclosure, since the dependency itself is unused.
- This eliminates `atty`, `failure`, `ansi_term`, `proc-macro-error`, and
  their now-orphaned exclusive transitive chain (`cargo-binutils`,
  `rustc-cfg`, `tabled`, `tabled_derive`, `clap` 2.34.0) from `Cargo.lock`.
- **Correction vs. the plan's framing**: the plan listed `paste` among the
  crates this change would clear. Traced via `cargo tree -i paste`: it's
  pulled in via `kreuzberg`/`biblatex` and `burn-ndarray`/`burn-train`, not
  `grcov` at all — unaffected by this change, disclosed here rather than
  silently claiming a fix that wouldn't actually land. Likewise `instant`
  (via `notify`), `number_prefix` (via `liter-llm`/`tokenizers`/`hf-hub`/
  `indicatif`), `scc` (via `serial_test`, a real direct dev-dependency),
  and `ttf-parser`/`memmap2` (via `kreuzberg`, already addressed/disclosed
  in `kreuzberg-reachable-vulns`/`first-party-direct-dep-hygiene`) are all
  unrelated to `grcov` and out of this change's scope.

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "Unused Dev-Dependency
  Elimination" requirement (verify a flagged dependency has zero library
  call sites before assuming a version bump is the right fix; remove
  entirely when confirmed unused). Otherwise no other spec-level
  requirement changes.

## Impact

- **Affected code**: `Cargo.toml` (removed `grcov` line), `Cargo.lock`
  (grcov + its exclusive transitive chain removed).
- **Runtime UX / provider compatibility / realtime state**: none.
- **CI/tooling**: none — `tools/coverage.sh`, `tools/test-all.sh`, and the
  CI workflows all invoke `grcov` as an independently `cargo install`'d CLI
  binary, never as a linked library dependency of this workspace.
- **KBD workflow state**: `progress.json` for
  `uar-dependabot-remediation-2026-07` updated to DONE for this change;
  Round 1 (all 5 Rust changes) now complete.
