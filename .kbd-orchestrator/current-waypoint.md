# Current Waypoint — universal-agent-runtime

- **Phase:** uar-security-deps-and-hygiene
- **Status:** executing (Round 1: 6/6 ✅; Round 2: 2/2 ✅)
- **Progress:** 8 of 10 changes (latest: `38f285b`). GitHub alert count: 78 (down from 96 at phase start; critical 5→3, high 17→16)
- **Next pending change:** `rmcp-pin-bump` (Round 3, own checkpoint)
- **Exact next command:** `/kbd-execute uar-security-deps-and-hygiene` for Round 3, then Round 4 (`surrealdb-upgrade`, highest blast radius, last)
- **Recommendation source:** `.kbd-orchestrator/phases/uar-spec-v2-and-polish/reflection.md`'s 2026-07-04 addendum (rescoped after a Dependabot backlog was found via post-reflection research); confirmed against direct inspection in `assessment.md`; sequenced by risk in `plan.md`

## Round 1 results (6 of 6 done)

- `dependabot-yml`: new `.github/dependabot.yml`, 4 ecosystems.
- `fix-uar-integration-test`: `Skill` struct literal fixed via
  `..Default::default()`.
- `fix-bdd-test-path`: nested `#[path]` resolution fixed by moving the
  prefix onto the outer `mod live`.
- `artifact-refiner-gate-decision`: D-E decision record written
  (`.kbd-orchestrator/references/artifact-refiner-gate-decision.md`) —
  gate formally retired, no tool available in this environment.
- `npm-deps-triage`: **both** alerts fully resolved (better than
  `plan.md`'s disclosed risk of a dead end). `jsonwebtoken` was actually
  a Rust alert against `tools/uar-jwt-proxy` (bumped `9`→`10`,
  `Cargo.lock` now unified on `10.4.0`). `dompurify` traced to a
  completely unused `@types/dompurify` devDependency (removed).
- `fix-waypoint-stage-schema`: after being surfaced as a cross-repo
  blocker, the user chose "fix at the source." Fixed
  `write-position-reminder.sh` **and** `write-session-summary.sh`
  (identical bug found in the second file) in the separate
  `prometheus-skill-system` repo — `.stage // "unknown"` →
  `.stage // .status // "unknown"`. Rebased cleanly onto 20 unrelated
  upstream commits (none conflicting), pushed as `91006b8`. Verified
  against a synthetic `.status`-only waypoint (no `.stage`) — the real
  regression scenario, not just the happy path.
- Checkpoint: `cargo check --workspace` clean, `cargo test --lib`
  363/363, frontend `tsc --noEmit` unchanged at 17 pre-existing errors.

## Why this phase, and why rescoped

`uar-spec-v2-and-polish` closed 7/7 changes, G4+G5 both MET. Its own
"Next Phase Focus" recommended a `uar-hygiene-and-bench-validation`
phase (QA gate automation, 2 broken test files, `cargo bench`). Before
starting that, the user asked for research into whether anything should
change the plan. It did: GitHub had been flagging **96 open Dependabot
alerts** (5 critical, 17 high, ~4 months accumulated, no
`dependabot.yml` automation) on every push all phase, never
investigated because it wasn't in that phase's declared scope. The two
that matter most are directly production-relevant:

- **`surrealdb`** pinned `=3.0.5` (crates.io has `3.2.0`) — high-severity
  HTTP RPC session-hijack + privilege-escalation CVEs. `surreal-backend`
  is UAR's **default** feature.
- **`rmcp`** pinned via git rev, behind upstream `HEAD` — high-severity
  DNS rebinding in its Streamable HTTP transport. Core, non-optional MCP
  SDK.

This directly undercuts D-D ("dependency pins are deliberate, not
debt"), which `uar-spec-v2-and-polish`'s own CH-19 re-affirmed without
checking whether the pinned versions carry known, fixed-upstream CVEs.

## Goals (see `phases/uar-security-deps-and-hygiene/goals.md` for full detail)

- **G1 (P0, primary): security dependency triage & upgrade.** Triage
  the 5 critical + 17 high alerts; upgrade `surrealdb`; bump the `rmcp`
  pin; disposition `wasmtime` (opt-in feature, lower priority) and the
  `failure` crate (dev-only via `grcov`, no exposure); triage npm-side
  alerts (`dompurify`, `jsonwebtoken`, etc.); add `.github/dependabot.yml`.
- **G2 (P1, secondary — carried from `uar-spec-v2-and-polish`): hygiene
  & validation.** Automate the artifact-refiner QA gate (or explicitly
  drop it — 4th+ phase as debt); fix `tests/uar_integration.rs` +
  `tests/bdd.rs` pre-existing compile failures; run `cargo bench` on
  `benches/hot_path.rs`; fix `write-position-reminder.sh`'s
  `.stage`/`.status` schema mismatch at the source.

## Planned change order (see `plan.md` for full detail — all 10 are independent, ordered by risk not dependency)

- **Round 1 (parallel, low risk)**: `dependabot-yml` ✅, `fix-uar-integration-test` ✅, `fix-bdd-test-path` ✅, `artifact-refiner-gate-decision` ✅, `npm-deps-triage` ✅, `fix-waypoint-stage-schema` ✅ — **all 6 done**
- **Round 2 (parallel)**: `wasmtime-disposition` ✅ (bumped 41→46 per user request, fixed the resulting Context-trait break at 6 call sites), `run-hot-path-bench` ✅ (executed for the first time — see Round 2 results below)
- **Round 3 (sequenced, own checkpoint)**: `rmcp-pin-bump` (next)
- **Round 4 (sequenced, own checkpoint, last, highest blast radius)**: `surrealdb-upgrade`

`surrealdb-upgrade` and `rmcp-pin-bump` carry real regression risk and each get their own dedicated test-suite checkpoint, not bundled with the smaller Round 1/2 items.

## Round 2 results (2 of 2 done)

- `wasmtime-disposition`: bumped `wasmtime`/`wasmtime-wasi` 41.0.3→46
  (user asked for latest). Fixed the resulting break (`wasmtime::Error`
  no longer implements `std::error::Error`) by swapping to wasmtime's
  own `wasmtime::error::Context` trait at 6 call sites. 367/367 tests
  green with the feature enabled.
- `run-hot-path-bench`: `cargo check --benches` + `cargo bench --bench
  hot_path` both run for the first time ever. All 4 benchmarks
  microsecond-scale or better, no red flags. Baseline recorded in
  `benches/hot_path.rs`'s own doc comment.

## Decisions carried forward (still load-bearing)
- D-A: RAG hardened in-process; Knowledge Service extraction deferred
- D-B: MemPalace stays off
- D-C: LibreFang integration scoped to UAR side
- D-D: dependency pins deliberate — **under active re-examination this
  phase** for `surrealdb`/`rmcp` specifically, given known fixed-upstream
  CVEs on the currently-pinned versions

## Carried-over debt (see progress.json for full list; G1/G2 above absorb most of it)
- 17 pre-existing `bun run typecheck` errors (unrelated to recent work)
- CH-06 per-agent/per-task budget configuration surface (global-only today)
- CH-08 activation-outcome correlation (recall wired; outcome half unsolved)
- Durable cost/spend history for CH-07 dashboard
- `main()` always loads full `AppConfig` before dispatching any
  subcommand, so the config-light `compile`/`eval` subcommands need a
  minimal persistence config they don't otherwise use (found while
  building `uar-spec-v2-and-polish`'s CH-15 `compile` subcommand;
  not yet in this phase's own goals — candidate for a future pass)

## Prior phase archive

- **`uar-spec-v2-and-polish`** (2026-07-04): 7/7 changes, G4+G5 MET.
  See `.kbd-orchestrator/phases/uar-spec-v2-and-polish/reflection.md`
  (and its addendum) for full detail, including the sycophancy
  self-check (score 0.018, no phase inversion detected).
- **`uar-next-harness`**: 16/24 changes, G1-G3 MET, G4-G5 deferred to
  `uar-spec-v2-and-polish`. See its own `reflection.md`.
