## Context

`desktop-data-layer-pglite-oxide` (change 11 of this phase) uses embedded
SurrealDB (surrealkv/kv-rocksdb) as the desktop data layer's baseline
backend, with `pglite-oxide` layered in as an optional, platform-gated
enhancement (PG-wire compatibility, shared SQL schema with web/cloud)
where it's actually usable. `pglite-oxide` 0.5.1 publishes AOT release
assets for `aarch64-apple-darwin`, Linux (x86_64/aarch64), and
`x86_64-pc-windows-msvc`, plus a `portable-wasix` fallback for everything
else — but there is no `x86_64-apple-darwin` AOT asset, and this project's
primary dev machine is an Intel Mac. Whether the `portable-wasix` fallback
actually boots and performs acceptably here is currently unverified; this
spike produces that evidence.

## Goals / Non-Goals

**Goals:**
- Determine, with direct empirical evidence on this machine, whether
  `pglite-oxide`'s `portable-wasix` asset boots a working `PgliteServer`
  on `x86_64-apple-darwin`.
- Record cold-start time and basic query round-trip latency as
  supplementary numbers (not pass/fail gates — there is no comparative
  SLA yet; that's evaluated later during actual desktop integration).
- Feed a resolved PASS/FAIL verdict back into
  `library-candidates.json` (`cand-001`) and `decision-log.md` so
  `desktop-data-layer-pglite-oxide` reads a decided fact, not an open
  question.

**Non-Goals:**
- No production runtime wiring — the spike never touches
  `src/`, `src-tauri/`, or any served code path.
- No repository-trait design work (that's change 11's job once this
  verdict exists).
- No performance tuning or optimization — this measures feasibility, not
  best-case numbers.

## Decisions

**D1 — Spike location: standalone top-level `spikes/` crate, not a
workspace member.**
`spikes/pglite-oxide-intel-mac-spike/` — a throwaway `cargo run`-able
binary crate, given its own empty `[workspace]` table in its own
`Cargo.toml` (per this repo's established pattern for any crate that's
`cd`'d into and built directly — see `vendor/git/liter-llm/Cargo.toml`,
`vendor/git/rust-mcp-filesystem/Cargo.toml`, `src-tauri/Cargo.toml`: an
excluded-but-buildable crate needs its own `[workspace]` table or `cargo
check`/`build` fails with "current package believes it's in a workspace
when it's not," even with `exclude` set at the root). Unlike those
crates, this one is **not** a path dependency of the root crate in either
direction, so it does not need a root `Cargo.toml` `exclude` entry —
Cargo only walks into it if someone `cd`s in and builds directly.
Alternative considered: a `vendor/`-style git submodule — rejected, this
isn't vendoring an external project, it's a few dozen lines written for
this spike alone.

**D2 — What "boots" means.**
Start `PgliteServer` via `pglite-oxide`'s portable-wasix path, run
`CREATE TABLE spike (id INT); INSERT INTO spike VALUES (1); SELECT * FROM
spike;` over the PG-wire connection, and assert the round-trip returns
the inserted row. This exercises the full boot → connect → query → result
path, not just process startup.

**D3 — Verdict recording.**
Append a `## Result` section to this design.md with the PASS/FAIL verdict,
the two measured numbers (cold-start ms, query round-trip ms), the exact
`rustc`/`pglite-oxide` versions used, and any error output on FAIL. This
is the source of truth `desktop-data-layer-pglite-oxide` reads; the
`library-candidates.json`/`decision-log.md` updates promised in the
proposal's Impact section are a copy of this same verdict for
discoverability, not a second source of truth.

## Risks / Trade-offs

- **[Risk] The portable-wasix runtime itself might be unavailable or
  broken in this environment** (missing WASM runtime shared libs, sandbox
  restrictions) → **Mitigation**: any failure mode is itself a valid,
  actionable FAIL result — document the exact error, which is precisely
  the evidence change 11 needs to make its enhancement-gating decision.
- **[Risk] A spike crate with its own `[workspace]` table could be
  mistaken for a real workspace member and accidentally get referenced
  from production code** → **Mitigation**: no path dependency is created
  in either direction; the spike crate is self-contained and disposable.
- **[Risk] Performance numbers from a cold, unoptimized spike could be
  over- or under-read as representative of production performance** →
  **Mitigation**: D2/D3 explicitly frame the numbers as supplementary
  feasibility evidence, not a performance gate; the design doc's language
  makes this framing explicit for whoever reads the verdict later.

## Migration Plan

Not applicable — no production code or data is touched by this change.

## Open Questions

None — the spike's own execution resolves the only open question this
change exists to answer (does portable-wasix pglite-oxide work on
`x86_64-apple-darwin`).

## Result (2026-07-16)

**Verdict: FAIL** — pglite-oxide 0.5.1 does not build at all on
`x86_64-apple-darwin`, so no boot/query timings were obtainable.

- **Toolchain**: `rustc 1.97.0-nightly (f53b654a8 2026-04-30)`, `cargo
  1.97.0-nightly (eb9b60f1f 2026-04-24)`.
- **Dependencies resolved**: `pglite-oxide = 0.5.1`, transitively pulling
  `wasmer-wasix = 0.702.0-alpha.3` and `virtual-net = 0.702.0` (per
  `Cargo.lock`). No `pglite-oxide-aot-x86_64-apple-darwin` package exists
  on crates.io at all (confirmed via the crate's published dependency
  list — only `aarch64-apple-darwin`, `x86_64/aarch64-unknown-linux-gnu`,
  and `x86_64-pc-windows-msvc` AOT crates exist), so the WASIX/`wasmer`
  path is the *only* one this platform can reach — there is no AOT
  fallback to fall further back to.
- **Failure**: `cargo run` fails during dependency compilation, before
  `PgliteServer::temporary_tcp()` is ever called:

  ```
  error[E0004]: non-exhaustive patterns: `NetworkError::MessageSize` not covered
     --> wasmer-wasix-0.702.0-alpha.3/src/net/mod.rs:376:11
      |
  376 |     match net_error {
      |           ^^^^^^^^^ pattern `NetworkError::MessageSize` not covered
  error: could not compile `wasmer-wasix` (lib) due to 1 previous error
  ```

  This is a genuine version-incompatibility bug *between two of
  pglite-oxide's own pinned transitive dependencies* — `virtual-net`
  0.702.0 added a `NetworkError::MessageSize` enum variant that
  `wasmer-wasix` 0.702.0-alpha.3's exhaustive `match` doesn't handle. It
  is not a platform-specific WASIX runtime-support gap and not something
  this spike can work around (no supported path to patch/override a
  transitive dependency's version without vendoring pglite-oxide itself,
  which is out of scope for a spike).
- **Additional context** (not gating, informational): the upstream
  `f0rr0/pglite-oxide` repository's current (unreleased) `main` branch has
  moved on to a ground-up rewrite/rename ("Oliphaunt", alpha, versions
  reset to `0.0.0`), whose own published first-release target envelope
  *also* explicitly excludes `macOS x64` ("The first release intentionally
  does not claim macOS x64..."). This doesn't affect the FAIL verdict
  above (which is about the still-published 0.5.1, not the unreleased
  rewrite) but confirms Intel Mac support isn't coming from upstream in
  the near term either.
- **Conclusion for `desktop-data-layer-pglite-oxide` (change 11)**:
  pglite-oxide is **not usable** on `x86_64-apple-darwin` today, in any
  configuration. Change 11 should treat this platform as SurrealDB-only
  (the baseline, not the enhancement) with no further spike or retry
  needed unless a future pglite-oxide/wasmer-wasix release fixes the
  underlying incompatibility.
