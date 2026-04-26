## 1. Baseline And Scope

- [x] 1.1 Re-run `bun run lint` from `frontend/` and confirm the current errors and warnings match the KBD assessment.
- [x] 1.2 Re-run `bun run typecheck` from `frontend/` to confirm the pre-change typecheck baseline remains green.

## 2. Unused State And Prop Cleanup

- [x] 2.1 Remove or use the stale `initialUrl` value in `frontend/e2e/chat-agent-selection.spec.ts`.
- [x] 2.2 Remove unused `defaultId` and `catalog` props from `ProviderDetail` in `frontend/src/admin/pages/providers-page.tsx` or use them in visible provider diagnostics.
- [x] 2.3 Remove the unused `_threadId` binding in `frontend/src/features/chat/capability-toggles.tsx` unless the component needs it for a verified behavior.

## 3. React Effect Safety

- [x] 3.1 Refactor `frontend/src/components/model-selector.tsx` so model loading does not synchronously set state inside an effect solely to initialize request state.
- [x] 3.2 Refactor `frontend/src/features/chat/agent-selector.tsx` so agent loading does not synchronously set state inside an effect solely to initialize request state.
- [x] 3.3 Refactor `frontend/src/features/chat/capability-toggles.tsx` so skills, tools, and knowledge base lists are derived safely from `agentConfig` or updated without violating `react-hooks/set-state-in-effect`.
- [x] 3.4 Refactor `frontend/src/pages/chat-page.tsx` so thread changes reset agent config without a synchronous effect state update violation.

## 4. Fast Refresh Warning Cleanup

- [x] 4.1 Resolve `react-refresh/only-export-components` warnings in shared UI component modules by moving reusable constants/helpers to non-component modules or adding narrow documented exceptions for accepted UI-library variant exports.
- [x] 4.2 Resolve Fast Refresh warnings in chat feature modules without weakening runtime console or chat behavior.

## 5. Verification

- [x] 5.1 Run `bun run lint` from `frontend/` and verify it exits 0 with no ESLint errors or warnings.
- [x] 5.2 Run `bun run typecheck` from `frontend/` and verify it exits 0.
- [x] 5.3 Run `openspec validate frontend-lint-zero-warning` and verify the change remains valid.
- [x] 5.4 Update `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` to mark `frontend_lint_zero_warning` verified after lint and typecheck pass.
