# Findings: triage-unassigned-unmaintained-warnings

## Disposition summary

| Crate | Advisory | Path | Disposition |
|---|---|---|---|
| `instant` | `RUSTSEC-2024-0384` (unmaintained) | `notify` 7.x → `notify-types` 1.0.1 | **Fixed** — bumped `notify` "7" → "8"; `notify-types` 2.0.0 dropped `instant` for `web-time`. |
| `bincode` | `RUSTSEC-2025-0141` (unmaintained) | `burn-core` → `burn` (always-compiled, not feature-gated) | **No fix exists, disclosed** — bincode's maintainers permanently ceased development after a doxxing/harassment incident; no version will ever be patched. Fixing requires `burn` to migrate serialization backends — outside UAR's control. |
| `paste` | `RUSTSEC-2024-0436` (unmaintained) | `kreuzberg`→`biblatex` AND `burn`-family crates (multiple independent paths) | **No single fix point, disclosed** — stable, simple proc-macro crate, no unsound behavior reported; two unrelated upstream owners would each need to move off it. |
| `ttf-parser` | `RUSTSEC-2026-0192` (unmaintained) | `kreuzberg` → `lopdf` | **No fix exists through current kreuzberg, disclosed** — same `lopdf` dependency already covered by `kreuzberg-reachable-vulns` (prior phase): no kreuzberg release through `v5.0.0-rc.35` resolves it. |
| `number_prefix` | `RUSTSEC-2025-0119` (unmaintained) | `indicatif` → `hf-hub` → `fastembed` → `mempalace-core` → `surreal-memory` | **Too deep to control, disclosed** — 4 hops beyond `surreal-memory` (which is itself first-party-adjacent); none of the intermediate crates are controlled by UAR or, as far as traced, by `surreal-memory`'s maintainers either. |
| `rustls-pemfile` | `RUSTSEC-2025-0134` (unmaintained) | `microsandbox-network` (behind optional, off-by-default `sandbox-microsandbox` feature) | **Not reachable by default, disclosed** — same disposition class as `hickory-proto` (prior phase). |
| `proc-macro-error2` | `RUSTSEC-2026-0173` (unmaintained) | `microsandbox` → `oci-spec`/`sea-orm-macros` (same optional feature) | **Not reachable by default, disclosed** — already surfaced in the prior phase's `grcov-toolchain-refresh` findings; formally disclosed in docs here. |
| `scc` | `RUSTSEC-2026-0205` (unsound) | `serial_test` (`[dev-dependencies]`) | **Dev-only, disclosed** — never ships in the release binary. |
| `atomic-polyfill` | `RUSTSEC-2023-0089` (unmaintained) | none found (`cargo tree -i --target all --all-features` → empty) | **Orphaned lockfile entry, disclosed** — same class as `quinn-proto` (prior phase); likely self-prunes on a future full `cargo update`. |

## Why `notify` was the only actionable fix

Every other crate's flagged dependency is either (a) a permanently
abandoned upstream project with literally no patched version that could
ever exist (`bincode`), (b) owned by a third-party crate UAR doesn't
control and which has independent reasons not to have moved off it yet
(`paste`, `ttf-parser`), (c) buried 4+ hops into a dependency chain no
party in this investigation controls (`number_prefix`), (d) gated behind
an optional feature that's off by default (`rustls-pemfile`,
`proc-macro-error2`), (e) dev-only (`scc`), or (f) not actually present in
the resolved dependency graph at all (`atomic-polyfill`). `notify` was
different: it's a normal, always-compiled dependency with a stable major
release already available that demonstrably drops the flagged transitive
crate, and UAR's own call site uses only long-stable core API.

## Verification

- `cargo check --lib --tests`: clean (same pre-existing warnings as
  baseline).
- `cargo test --lib`: 387/388 pass (1 pre-existing ignore) — unchanged.
- `cargo clippy --lib`: 499 warnings — unchanged, zero new.
- `cargo audit`: warnings count dropped from 9 to 8 (`instant` cleared);
  vulnerabilities count unchanged at 11 (all pre-existing, disclosed).
- `cargo tree -i instant`: no match — fully eliminated.
- `notify` resolved version confirmed `8.2.0` in `Cargo.lock`.
