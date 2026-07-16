## 1. Spike scaffold

- [x] 1.1 Create `spikes/pglite-oxide-intel-mac-spike/` as a standalone
      binary crate (`Cargo.toml` + `src/main.rs`), given its own empty
      `[workspace]` table (D1) so it is buildable via `cd
      spikes/pglite-oxide-intel-mac-spike && cargo run` without the root
      workspace ever noticing it exists.
- [x] 1.2 Add `pglite-oxide` as a dependency of this crate only — never
      touch the root `Cargo.toml`. NOTE: `pglite-oxide-assets` is not
      added directly; it's pulled transitively via `pglite-oxide`'s
      `bundled` default feature, which is the crate's documented way to
      get AOT assets where available and fall back to the always-present
      `wasmer`/WASIX runtime path otherwise (verified via crates.io
      dependency graph: `wasmer`/`wasmer-wasix` are non-optional deps;
      `pglite-oxide-aot-x86_64-apple-darwin` does not exist as a package
      at all, so no AOT crate can activate on this triple — the WASIX
      path is the only one available here by construction, no feature
      flag needed to force it).
- [x] 1.3 Confirmed `rustc -vV` host triple is `x86_64-apple-darwin` on
      this machine.

## 2. Boot-and-query verification

- [x] 2.1 `src/main.rs` written: starts `PgliteServer::temporary_tcp()`
      (the documented API, confirmed via docs.rs for 0.5.1), timing boot
      from call to server-ready.
- [x] 2.2 `src/main.rs` written: over the resulting PG-wire connection via
      `sqlx::PgConnection`, runs `CREATE TABLE spike (id INT); INSERT INTO
      spike VALUES (1); SELECT id FROM spike;`, timing the round-trip.
- [x] 2.3 Attempted `cargo run`. Result: **FAIL before boot was ever
      reached** — `wasmer-wasix` 0.702.0-alpha.3 (pglite-oxide's own
      transitive WASIX-runtime dependency, unconditionally required since
      no AOT crate exists for `x86_64-apple-darwin`) fails to *compile*:
      `error[E0004]: non-exhaustive patterns: NetworkError::MessageSize
      not covered` in `wasmer-wasix-0.702.0-alpha.3/src/net/mod.rs:376`,
      because the resolved `virtual-net` 0.702.0 added a
      `NetworkError::MessageSize` enum variant that `wasmer-wasix`
      0.702.0-alpha.3's `match` doesn't handle. This is a real upstream
      version-incompatibility bug between two of pglite-oxide's own pinned
      dependencies, not a platform-support gap in pglite-oxide itself and
      not fixable from this spike's side (no path to patch a transitive
      dependency without vendoring it). Toolchain: `rustc 1.97.0-nightly
      (f53b654a8 2026-04-30)`, `cargo 1.97.0-nightly (eb9b60f1f
      2026-04-24)`. Full command + error captured in `design.md`'s
      `## Result` section.

## 3. Verdict recording

- [x] 3.1 Ran the spike (`cargo run` inside
      `spikes/pglite-oxide-intel-mac-spike/`) and captured its output —
      see 2.3.
- [x] 3.2 Appended a `## Result` section to this change's `design.md`
      with the FAIL verdict, resolved dependency versions, the exact
      compile error, and the conclusion for change 11.
- [x] 3.3 Updated `library-candidates.json`'s `cand-001` (verdict
      adopt→adapt, coverage_estimate 0.5→0.3, new evidence entry,
      `open_questions` resolved) and top-level `open_questions`/`totals`
      in `.kbd-orchestrator/phases/uar-hybrid-app-architecture/`.
- [x] 3.4 Appended a decision-log.md entry in the same phase directory
      recording the verdict and its source (this change).

## 4. Cleanup

- [x] 4.1 Decision: **kept** `spikes/pglite-oxide-intel-mac-spike/` as a
      documented reference rather than deleted. Rationale: it's a
      reproducible, self-contained repro case for the real upstream
      `wasmer-wasix`/`virtual-net` bug (design.md's `## Result`); keeping
      it lets anyone re-run `cargo run` after a future pglite-oxide
      release to cheaply re-check whether the incompatibility is fixed,
      without re-deriving the setup. It has no `[workspace]` membership
      and no path dependency to/from any production crate (verified via
      its own `[workspace]` table and `git status` showing it only as a
      new untracked directory, not referenced anywhere else).
- [x] 4.2 Confirmed: `cargo check --locked --no-default-features --features
      server-full` at the workspace root passes clean (`Finished dev
      profile ... in 7m 09s`), unaffected by the new `spikes/` directory —
      proves the spike's isolation requirement (spec: "Spike isolation
      from production code") held in practice, not just in design.
