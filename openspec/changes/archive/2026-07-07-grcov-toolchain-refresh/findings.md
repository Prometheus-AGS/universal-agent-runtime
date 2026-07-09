# Findings: grcov-toolchain-refresh

## Disposition summary

| Finding | Crate(s) | Path | Disposition |
|---|---|---|---|
| Unmaintained/unsound, 18 warnings per assessment | `atty`, `failure`, `ansi_term`, `proc-macro-error` | Exclusively via `grcov` 0.8.19 → `cargo-binutils`/`tabled` → (`clap` 2.34.0, `rustc-cfg`, `tabled_derive`) | **Fully eliminated** — `grcov = "0.8"` removed entirely from `Cargo.toml`'s `[dev-dependencies]`. `grep -rn "use grcov\|grcov::"` found zero call sites anywhere in the repo; the coverage tool actually used (`tools/coverage.sh`, `tools/test-all.sh`, 3 CI workflows) is a separately `cargo install`'d CLI binary, entirely decoupled from this workspace's `Cargo.lock`. Confirmed via `cargo tree -i` post-removal: `grcov`, `atty`, `failure`, `ansi_term`, `proc-macro-error`, `cargo-binutils`, `tabled`, `tabled_derive`, `rustc-cfg` are all absent. |
| `paste` — listed in the plan as expected to clear | `paste` 1.0.15 | `kreuzberg`/`biblatex`, `burn-ndarray`/`burn-train` | **Not affected — plan correction.** `cargo tree -i paste` shows it's pulled in via `kreuzberg` and `burn`, not `grcov`. Left unchanged, disclosed here rather than silently claiming a fix that didn't land. |
| `instant`, `number_prefix`, `scc`, `ttf-parser` — mentioned in assessment's 18-warning tally | various | `notify` (`instant`), `liter-llm`→`tokenizers`→`hf-hub`→`indicatif` (`number_prefix`), `serial_test` (`scc`), `kreuzberg`→`lopdf` (`ttf-parser`, already covered by `kreuzberg-reachable-vulns`) | Out of scope — none attributable to `grcov`. Not touched by this change. |
| `proc-macro-error2` (unmaintained) | `proc-macro-error2` 2.0.1 | `microsandbox` (behind the optional, off-by-default `sandbox-microsandbox` feature) → `oci-spec`/`sea-orm-macros` | Pre-existing (confirmed via `git diff Cargo.lock` — untouched by this change), unrelated to `grcov`. Not assigned to any of this phase's 8 changes; disclosed here since it surfaced during this change's `cargo audit` re-run, same treatment as `hickory-proto` in `direct-network-facing-vulns` (same optional feature family). |

## Verification

- `cargo check --lib --tests`: clean (same pre-existing unrelated warnings).
- `cargo test --lib`: 387/388 pass (1 pre-existing ignore) — unchanged.
- `cargo clippy --lib`: 499 warnings — unchanged, zero new.
- `cargo audit`: `Cargo.lock` shrank by 476 lines (25 insertions, 501
  deletions in the diff); `grcov`/`atty`/`failure`/`ansi_term`/
  `proc-macro-error` no longer appear. `proc-macro-error2` (distinct crate,
  pre-existing) still listed as expected.

## Round 1 checkpoint

This closes Round 1 (all 5 Rust changes: `kreuzberg-reachable-vulns`,
`surreal-memory-transitive-vulns`, `direct-network-facing-vulns`,
`first-party-direct-dep-hygiene`, `grcov-toolchain-refresh`). Shared
checkpoint per `plan.md`: `cargo audit` re-run (above), `cargo test --lib`
full suite green, `cargo clippy --lib` zero new warnings — all satisfied
cumulatively across the 5 changes.
