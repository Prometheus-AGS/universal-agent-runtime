# Execution — `prometheus-package-integration`

**Date started:** 2026-05-26
**Tool:** claude-code (`/kbd-execute`)
**Backend selected:** OpenSpec (scaffolds already present at `openspec/changes/<change-id>/`)
**Strategy:** sequential, one change at a time, gated by user approval between large changes. Small/independent changes (e.g. #1) execute directly. Risky structural changes (e.g. #2 live-bus, #6 WASM runtime, #7 pnpm migration, #13 Dockerfile) pause for review before proceeding to the next.

## Dispatch contract

For each change in the ordered list:

1. Emit `Starting change <N> of 14: <change-id>`.
2. Implement per `openspec/changes/<change-id>/proposal.md` and `tasks.md`.
3. Tick boxes in `tasks.md` as work lands.
4. Build + targeted test pass.
5. Update `progress.json.changes_completed`.
6. Run artifact-refiner QA when the change touches ≥3 files and isn't doc-only.
7. Emit `Completed change <N> of 14: <change-id>`.
8. Either auto-continue to next change (small/safe) or stop and await user direction (large/structural).

## Auto-continue allowlist

These changes are small and self-contained — execution proceeds without pausing:

- `fix-kb-document-count`
- `add-skill-kind-and-origin`
- `builtin-skills-ui-affordance` (after upstream lands)

All others require explicit user go-ahead per turn.

## QA gate

`artifact-refiner` is **not** currently installed as an invocable skill in this workspace. Per-change validation will be performed inline (compile + targeted test). QA gate will be re-introduced when `/refine-validate` lands.

## Change cursor

- **Next:** `fix-kb-document-count` (change 1 of 14)
