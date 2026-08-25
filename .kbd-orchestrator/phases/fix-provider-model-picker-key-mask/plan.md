# PLAN: fix-provider-model-picker-key-mask

Project: universal-agent-runtime
Date: 2026-08-25
OpenSpec available: YES
Changes to implement: 1

## CHANGE LIST (ordered)

1. provider-model-picker-key-mask: Bound provider model selection and preserve exact-length credential masks
   - Scope: UI | settings API
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Files: `src/uar/api/settings.rs`; `frontend/src/features/settings/ui/panels/ai-settings-panels.tsx`; `frontend/src/features/settings/ui/settings-primitives.tsx`; new focused test `frontend/src/features/settings/ui/panels/ai-settings-panels.test.tsx`. Reuse `frontend/src/components/ui/select.tsx` without modifying it.
   - Details: Implement the OpenSpec tasks as one vertical slice. Task 1.1 protects the backend secret boundary with focused tests and schema-guided masking/preservation. Task 2.1 adds focused React evidence and switches the provider field to the existing shadcn control. Task 3.1 runs settings structure and strict OpenSpec validation. Task 3.2 runs the frontend and Rust phase-completion suites.
   - Acceptance: Tasks 1.1, 2.1, 3.1, and 3.2 are checked; strict validation passes; enabled provider models are the complete option set; disabled/foreign models are absent; selected ids enter the existing settings draft; mask length equals stored key character count; absent keys have no fabricated mask; and unchanged nested masks cannot overwrite stored credentials.
   - Trade-off / scope cut: Do not add catalog fetching, search, new entity state, dependency changes, or a stored-key reveal path. A replacement key made entirely of asterisks with exactly the existing key length remains indistinguishable from the unchanged mask. Removing that ambiguity requires a later opaque-sentinel or dirty-field contract and is not part of this change.

## EXECUTION ROUND ORDER

Round 1 (sequential): provider-model-picker-key-mask

Within the change:
1. Update `src/uar/api/settings.rs` tests and implementation; run `cargo check --locked --no-default-features --features server-full` and `cargo test --locked --no-default-features --features server-full settings_api_masks_and_preserves_sensitive_values`.
2. Add `frontend/src/features/settings/ui/panels/ai-settings-panels.test.tsx`, update `ai-settings-panels.tsx` and the optional trigger class in `settings-primitives.tsx`; run `pnpm -C frontend test src/features/settings/ui/panels/ai-settings-panels.test.tsx`, `pnpm -C frontend typecheck`, and `pnpm -C frontend lint`.
3. Run `pnpm -C frontend settings:structure` and `openspec validate provider-model-picker-key-mask --strict`.
4. Run `pnpm -C frontend build`, `pnpm -C frontend test`, `cargo fmt --all -- --check`, and `cargo test --locked --no-default-features --features server-full`.

## COMMANDS TO RUN

OpenSpec change already exists and is strictly valid:
`/opsx:apply provider-model-picker-key-mask`

Verification commands are listed in execution order above and are authoritative for this phase.

## OPERATOR SCOPE AMENDMENT — 2026-08-25

During task 3.2, the operator requested a durable UI-design skill precedence.
Add the rule outside managed regions in `AGENTS.md` (which is also exposed by the
`CLAUDE.md` symlink), add the corresponding `agent-ui-design-workflow` OpenSpec
delta, and validate the amended change before resuming phase-completion checks.
The required order is Impeccable, Anthropic `frontend-design`, then UI/UX Pro Max.
The operator subsequently approved two isolated Impeccable critique subagents and
fresh-context adversarial review as the standard UI ideation/evaluation/refactor/
refinement gate; persist that rule and apply it to this change.

## PLAN COMPLETE
