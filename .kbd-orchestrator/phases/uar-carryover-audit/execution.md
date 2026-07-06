# Execution — uar-carryover-audit

**Backend:** `openspec` (per `.kbd-orchestrator/project.json`'s
`specSystem`, consistent with every prior phase).

**Dispatch contract:** single-tool (`claude-code`), no multi-tool
handoff — 4 changes, executed as 3 rounds per `plan.md`:

- **Round 1** (zero risk): `fmt-drift-cleanup`
- **Round 2** (batched, one shared checkpoint): `ch06-wire-agent-cost-budget`,
  `ch08-activation-outcome-correlation`
- **Round 3** (own dedicated checkpoint — touches persistence):
  `ch07-durable-cost-history`

QA gate: artifact-refiner remains formally retired
(`uar-security-deps-and-hygiene`'s `artifact-refiner-gate-decision`).
Verification method: direct `cargo check`/`cargo test --lib`/`cargo
clippy` execution, plus a live-server smoke check for Round 3 given it
touches the persistence layer (matching this project's established
pattern for persistence-layer changes, e.g. `surrealdb-upgrade`).

`openspec/changes/<id>/proposal.md` + `tasks.md` written per-change at
execute time, per established practice.

EXECUTION READY
