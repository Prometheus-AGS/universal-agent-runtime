# Current Waypoint — universal-agent-runtime

- **Phase:** uar-spec-v2-and-polish
- **Status:** executed (execute_complete=true)
- **Progress:** 7 of 7 changes done. All of Tranche B/C/D complete.
- **Next pending change:** none — /kbd-reflect is the next KBD step
- **Exact next command:** /kbd-reflect for uar-spec-v2-and-polish
- **Recommendation source:** goals.md, seeded from uar-next-harness's reflection (2026-07-04)

## Change map (G4 → G5)

- G4 Specification & Distribution (P2, primary, sequential chain):
  - CH-12 agent-spec-v2 — DONE (75116e4)
  - CH-13 compiler-v2-stages — DONE (e1532d6)
  - CH-14 conformance-testing — DONE (4b24f01). New conformance.rs drives real ModelRouter::route/PromptDialect::detect/apply_strategy against declared v2 sections. 10/10 new tests, full suite 360/360.
  - CH-15 agent-template-library — DONE (bbe7ec2). 4 templates + `compile` CLI subcommand + CI release job + regression test. Full suite 363/363, integration 56/56.
  - CH-17 eval-targeted-suites — DONE (13326a4)
- G5 Polish & Release (P3, after G4 lands):
  - CH-19 docs-overhaul-deploy-guide — DONE (45e7e37). ARCHITECTURE.md gained 3 narrative sections + Agent Spec v2 & Conformance + Architectural Decisions (D-A/B/C/D). New DEPLOYMENT.md discloses a real AKS-vs-GKE deploy.yml/Helm-chart drift found via git history, rather than a smoothed-over unified story.
  - CH-20 perf-security-load — DONE (369117b + incidental fix e2c82c7). Criterion hot-path benches (not run this session, disclosed), 50-concurrent-agent load test, prompt-injection whitespace-evasion fix + disclosed known-gaps (13/13 guardrails tests green), server.rs split assessment (recommendation only). OpenSpec change dir at openspec/changes/perf-security-load/.

**All 7 changes done — G4 and G5 both fully landed. This phase's execute stage is complete.**

## Decisions carried from uar-next-harness (still load-bearing)
- D-A: RAG hardened in-process; Knowledge Service extraction deferred
- D-B: MemPalace stays off
- D-C: LibreFang integration scoped to UAR side
- D-D: dep unpin REJECTED (pins deliberate + load-bearing)

## Carried-over debt (see progress.json / goals.md for full list)
- Artifact-refiner QA gate automation (carried 4+ phases)
- 17 pre-existing `bun run typecheck` errors (unrelated to this phase's own work)
- CH-06 per-agent/per-task budget configuration surface (global-only today)
- CH-08 activation-outcome correlation (recall wired; outcome half unsolved)
- Durable cost/spend history for CH-07 dashboard
- NEW this phase: `tests/uar_integration.rs` `Skill` struct literal missing 8 fields (pre-existing, unrelated to any tracked change, found while verifying CH-20/CH-14)
- NEW this phase: `tests/bdd.rs` broken nested `#[path]` resolution (pre-existing, unrelated, same discovery)
- NEW this phase: `main()` always loads full `AppConfig` before dispatching any subcommand, so the config-light `compile`/`eval` subcommands need a minimal persistence config they don't otherwise use (found while building CH-15's `compile` subcommand)
- NEW this phase: `benches/hot_path.rs` (CH-20) has never been run via `cargo bench`/`cargo check --benches` in any session — compiles-by-inspection only

## Housekeeping note (2026-07-04)
This file and `current-waypoint.json` had drifted stale — both were still describing
the closed `uar-next-harness` phase / an `assess_pending` status days after assess+plan
actually completed (commit 16c1aa3) and three G4 changes had merged. Corrected via
`/kbd-status` findings; see `progress.json` as the authoritative source going forward.
