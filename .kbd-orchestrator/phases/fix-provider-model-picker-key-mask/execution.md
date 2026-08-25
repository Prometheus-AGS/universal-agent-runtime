# EXECUTION: fix-provider-model-picker-key-mask

Project: universal-agent-runtime
Date: 2026-08-25
Selected backend: openspec
Dispatched to: Codex (current session)
Backend rationale: The change is spec-backed, has four ordered tasks, and must retain KBD task hooks and progress synchronization; `/kbd-apply` is the canonical driver.
Backend entrypoint: `/Users/gqadonis/.codex/skills/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/fix-provider-model-picker-key-mask/plan.md`

## EXECUTION SCOPE

- `provider-model-picker-key-mask`: Bound provider model selection and preserve exact-length credential masks.
- Operator amendment: persist the UI-design skill precedence in `AGENTS.md` and the active OpenSpec change before resuming task 3.2.

## DISPATCH CONTRACTS

- `provider-model-picker-key-mask` → Codex through `/kbd-apply`
  - Entry: list and begin exactly one pending OpenSpec task, implement and verify that task, then end it before beginning the next.
  - Model class: medium from the reviewed plan; effective frontier fallback because `.kbd-orchestrator/project.json` has no `model_policy`.
  - Concrete model: GPT-5 (current Codex session).
  - Model rationale: Four tasks cross one bounded React/settings-API boundary with prior patterns and no unresolved design decisions.
  - Progress file: `.kbd-orchestrator/phases/fix-provider-model-picker-key-mask/progress.json`
  - Handoff: `/kbd-apply` updates the OpenSpec checklist, KBD projection, hooks, and waypoint at each task boundary.

## APPROVAL GATES

- NONE — all edits and local verification are within the user-requested repository scope.

## FALLBACK CONDITIONS

- Stop if `/kbd-apply` no longer detects OpenSpec, the apply instructions become blocked, or a task requires behavior outside the approved spec/design.
- Do not invoke bare `/opsx:apply`; preserve KBD ownership of the loop.

## VERIFICATION REQUIREMENTS

- Backend T0/T1: `cargo check --locked --no-default-features --features server-full`; `cargo test --locked --no-default-features --features server-full settings_api_masks_and_preserves_sensitive_values`.
- Frontend T0/T1: `pnpm -C frontend test src/features/settings/ui/panels/ai-settings-panels.test.tsx`; `pnpm -C frontend typecheck`; `pnpm -C frontend lint`.
- Contract: `pnpm -C frontend settings:structure`; `openspec validate provider-model-picker-key-mask --strict`.
- Phase T2: `pnpm -C frontend build`; `pnpm -C frontend test`; `cargo fmt --all -- --check`; `cargo test --locked --no-default-features --features server-full`.

## PROGRESS LEDGER

- PENDING `provider-model-picker-key-mask` — Codex via `/kbd-apply` (0/4 tasks).

## OUTPUTS

- OpenSpec change `openspec/changes/provider-model-picker-key-mask/`.
- Source, focused tests, verification evidence, and KBD review receipts.

## BLOCKERS

- NONE.

## REFLECTION HANDOFF

- Compare the reviewed plan with delivered source, record exact test outputs and any unverified tier, and lead with any delta or residual mask-sentinel limitation.

## EXECUTION READY
