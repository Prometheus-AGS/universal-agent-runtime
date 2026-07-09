# Findings: first-party-direct-dep-hygiene

## Reachability / disposition summary

| Advisory / warning | Crate | Path | Disposition |
|---|---|---|---|
| Unmaintained + unsound | `serde_yml` 0.0.12 | Direct dependency (`Cargo.toml`) | **Fixed** — replaced with `serde_norway` 0.9.42 (actively maintained fork, API-compatible `from_str`/`to_string`) across all 3 call sites (`Cargo.toml`, `src/uar/compiler/parser.rs`, `src/uar/runtime/skills/storage/filesystem.rs`). |
| Unmaintained | `libyml` | Transitive, pulled in only via `serde_yml` | **Eliminated** — confirmed absent from `Cargo.lock` post-swap (`cargo tree -i libyml` → no match); had no other reverse dependency. |
| Unsound (2026-06-25 report per assessment) | `anyhow` 1.0.103 | Direct dependency | **Checked, not applicable** — `cargo audit` does not list `anyhow` at the currently pinned version. The reported unsoundness does not affect 1.0.103. No action needed. |
| Unsound (2026-06-20 report per assessment) | `memmap2` 0.9.11 | Transitive via `kreuzberg` (direct, non-dev) and `grcov`/`symbolic-common` (dev-only) | **Checked, not applicable** — `cargo audit` does not list `memmap2` at the currently pinned version. No action needed. |
| `RUSTSEC-2026-0185` (remote memory exhaustion) | `quinn-proto` 0.11.14 | Listed in `Cargo.lock`, provenance previously suspected via `reqwest` | **Not reachable — orphaned lockfile entry.** `cargo tree -i quinn-proto --target all --all-features` resolves to nothing; it has zero reverse dependencies in the currently resolved graph. Confirmed separately that `reqwest`'s enabled features (`json`, `stream`, `rustls-tls-native-roots`, `multipart`) never activated HTTP/3 either way. Not assigned to any of this phase's 8 changes; disclosed here rather than silently dropped since it surfaced during this change's `cargo audit` re-run. Likely to self-prune on a future full `cargo update`. |

## Why `serde_norway` and not `serde_yaml`

`plan.md` suggested checking whether `serde_yaml` (the crate `serde_yml`
itself forked from, later archived by its original maintainer) was a
viable drop-in. `serde_yaml` is itself archived/unmaintained upstream —
picking it would just trade one unmaintained YAML crate for another.
`serde_norway` is the actively maintained continuation of that lineage
(same `Serialize`/`Deserialize`-driven API), so it was chosen instead;
verified via the 3 call sites that its `from_str`/`to_string` signatures
are drop-in compatible with no call-site logic changes beyond the
crate/error-type name.

## Verification

- `cargo check --lib --tests`: clean (2 pre-existing warnings, unrelated
  to this change).
- `cargo test --lib`: 387/388 pass (1 pre-existing ignore) — unchanged
  from the post-change-3 baseline.
- `cargo clippy --lib`: 499 warnings — unchanged from baseline, zero new.
- `cargo audit`: `serde_yml`/`libyml` no longer appear. `anyhow`/`memmap2`
  confirmed absent (no regression). Remaining 14 advisories are all
  either previously disclosed (kreuzberg's `lopdf`/`quick-xml`, `rsa`,
  `hickory-proto`) or out of this change's scope (`grcov`-toolchain-only
  crates, the orphaned `quinn-proto` entry above).
