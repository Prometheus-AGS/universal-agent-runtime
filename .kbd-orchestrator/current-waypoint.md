# Current Waypoint — universal-agent-runtime

- **Phase:** uar-security-deps-and-hygiene
- **Status:** assessed
- **Progress:** 0 of 10 changes (assessed; not yet planned)
- **Next pending change:** none yet — run `/kbd-plan uar-security-deps-and-hygiene`
- **Exact next command:** `/kbd-plan uar-security-deps-and-hygiene`
- **Recommendation source:** `.kbd-orchestrator/phases/uar-spec-v2-and-polish/reflection.md`'s 2026-07-04 addendum (rescoped after a Dependabot backlog was found via post-reflection research); confirmed against direct inspection in `assessment.md`

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

## Candidate changes (assessed 2026-07-04, see `assessment.md`; not yet planned/sequenced)

- G1: `surrealdb-upgrade` (PARTIAL — pin confirmed stale, not started), `rmcp-pin-bump` (PARTIAL — pin confirmed stale, not started), `wasmtime-disposition` (not started, correctly lower priority), `npm-deps-triage` (STUB — dompurify traced, jsonwebtoken not located), `dependabot-yml` (not started, absence confirmed)
- G2: `artifact-refiner-gate-decision` (confirmed unavailable in this environment, not just unused), `fix-uar-integration-test`, `fix-bdd-test-path`, `run-hot-path-bench`, `fix-waypoint-stage-schema` (all reconfirmed still needed via fresh `cargo check`/inspection)

`surrealdb-upgrade` and `rmcp-pin-bump` carry real regression risk and should each get their own test-suite checkpoint at plan time; the 4 G2 items are small and independent.

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
