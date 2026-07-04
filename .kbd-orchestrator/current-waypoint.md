# Current Waypoint — universal-agent-runtime

- **Phase:** uar-spec-v2-and-polish
- **Status:** executing
- **Progress:** 6 of 7 changes done. Tranche B (CH-13, CH-17, CH-20) and Tranche C (CH-14, CH-15) both complete.
- **Next pending change:** docs-overhaul-deploy-guide (CH-19) — last change, Tranche D
- **Exact next command:** /kbd-execute for uar-spec-v2-and-polish — land CH-19, then phase is execute-complete
- **Recommendation source:** goals.md, seeded from uar-next-harness's reflection (2026-07-04)

## Change map (G4 → G5)

- G4 Specification & Distribution (P2, primary, sequential chain):
  - CH-12 agent-spec-v2 — DONE (75116e4)
  - CH-13 compiler-v2-stages — DONE (e1532d6)
  - CH-14 conformance-testing — DONE (4b24f01). New conformance.rs drives real ModelRouter::route/PromptDialect::detect/apply_strategy against declared v2 sections. 10/10 new tests, full suite 360/360.
  - CH-15 agent-template-library — DONE (bbe7ec2). 4 templates + `compile` CLI subcommand + CI release job + regression test. Full suite 363/363, integration 56/56.
  - CH-17 eval-targeted-suites — DONE (13326a4)
- G5 Polish & Release (P3, after G4 lands):
  - CH-19 docs-overhaul-deploy-guide — NOT STARTED
  - CH-20 perf-security-load — DONE (369117b + incidental fix e2c82c7). Criterion hot-path benches (not run this session, disclosed), 50-concurrent-agent load test, prompt-injection whitespace-evasion fix + disclosed known-gaps (13/13 guardrails tests green), server.rs split assessment (recommendation only). OpenSpec change dir at openspec/changes/perf-security-load/.

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

## Housekeeping note (2026-07-04)
This file and `current-waypoint.json` had drifted stale — both were still describing
the closed `uar-next-harness` phase / an `assess_pending` status days after assess+plan
actually completed (commit 16c1aa3) and three G4 changes had merged. Corrected via
`/kbd-status` findings; see `progress.json` as the authoritative source going forward.
