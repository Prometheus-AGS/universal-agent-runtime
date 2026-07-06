# Execution — uar-frontend-typecheck-cleanup

**Backend:** `openspec` (per `.kbd-orchestrator/project.json`'s
`specSystem`, consistent with every prior phase in this project).

**Dispatch contract:** single-tool (`claude-code`), no multi-tool
handoff needed — 5 small, independent, frontend-only changes. Per this
project's standing implementation-first/test-at-checkpoints
preference, executed as 2 rounds with 1 shared checkpoint (not 5
individual ones), matching `plan.md`'s own round structure:

- **Round 1** (sequenced first — unblocks the checkpoint tooling
  itself): `fix-root-frontend-script-invocation`
- **Round 2** (batched, one shared checkpoint): the 4 TypeScript-error
  changes (`fix-typecheck-base-ui-select-nullability`,
  `fix-typecheck-resizable-panels-api-drift`,
  `fix-typecheck-recharts-export-drift`, `fix-typecheck-remaining-errors`)
- **Checkpoint**: `bun run typecheck` exits 0

QA gate: this project's `artifact-refiner-gate-decision`
(`uar-security-deps-and-hygiene`, `openspec/changes/artifact-refiner-gate-decision/`)
formally retired the artifact-refiner QA gate — no MCP tool available
in this environment. Verification method for all 5 changes: direct
`bun run typecheck`/`bun run build`/`bun run lint` execution and
inspection, per that decision record.

`openspec/changes/<id>/proposal.md` + `tasks.md` are written per-change
at execute time, per this project's established practice (confirmed
across every prior phase — not pre-scaffolded during planning).

EXECUTION READY
