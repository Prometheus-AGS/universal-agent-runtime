# Execution — uar-security-deps-and-hygiene

**Date:** 2026-07-04
**Backend:** `openspec` (OpenSpec is available at the project root and is
this project's established backend for every prior phase).

## Dispatch contract

Single-tool execution (`claude-code`), same as every change in
`uar-spec-v2-and-polish`. Each change follows this project's established
convention (not the generic template's "pre-scaffold all `openspec/changes/`
dirs during planning"): implement + verify the change, **then** write its
`openspec/changes/<id>/proposal.md` + `tasks.md` retroactively as part of
that change's own commit — proposal.md documents what was actually built
and how it was verified, not a plan for what will be built.

Per-change QA gate: the artifact-refiner tool is confirmed unavailable in
this environment (candidate change #5, `artifact-refiner-gate-decision`,
addresses this directly). Until that decision lands, every change in this
phase is verified via `cargo check`/`cargo test`/`cargo clippy` (Rust) or
direct inspection (docs/config-only changes) instead of the formal QA gate
— consistent with how every change in the prior 4+ phases has actually
been verified. No change in this phase will be silently marked DONE without
an explicit verification note in its own commit message and `progress.json`
entry.

## Round order (from `plan.md`, unchanged)

1. **Round 1** (parallel, low risk, one shared checkpoint at the end):
   `dependabot-yml`, `fix-uar-integration-test`, `fix-bdd-test-path`,
   `fix-waypoint-stage-schema`, `artifact-refiner-gate-decision`,
   `npm-deps-triage`
2. **Round 2** (parallel, one shared checkpoint): `wasmtime-disposition`,
   `run-hot-path-bench`
3. **Round 3** (own dedicated checkpoint): `rmcp-pin-bump`
4. **Round 4** (own dedicated checkpoint, last, highest blast radius):
   `surrealdb-upgrade`

## First change

`dependabot-yml` — config-only, no code touched, no test-suite checkpoint
needed beyond confirming the YAML is valid and matches GitHub's schema.
