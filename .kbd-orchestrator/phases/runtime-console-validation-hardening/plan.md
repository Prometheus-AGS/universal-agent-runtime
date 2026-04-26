PLAN: runtime-console-validation-hardening
Project: universal-agent-runtime
Date: 2026-04-26
OpenSpec available: YES
Changes to implement: 7

CHANGE LIST (ordered)

1. frontend-lint-zero-warning: make frontend lint a clean gate
   - Scope: frontend | tests
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Details: Fix the current `bun run lint` blockers without suppressing meaningful rules. Remove unused variables, restructure React effects that synchronously set state, and either resolve or explicitly justify `react-refresh/only-export-components` warnings so lint exits with zero errors and zero warnings.

2. runtime-console-live-visual-tests: prove runtime console layout and navigation across desktop/mobile
   - Scope: frontend | e2e
   - Depends on: frontend-lint-zero-warning
   - Recommended agent: Codex or Cursor Agent
   - Est. complexity: M
   - Customer value: HIGH
   - Details: Add Playwright coverage for `/admin` runtime console navigation, command palette, protocol/provider/memory surfaces, and responsive desktop/mobile rendering. Acceptance requires screenshots or assertions showing no incoherent overlap, usable navigation, and stable layout for the librefang-inspired shell.

3. runtime-event-replay-entity-sync-tests: prove live runtime updates enter the entity graph and update UI
   - Scope: frontend | stores | entity graph | e2e
   - Depends on: frontend-lint-zero-warning
   - Recommended agent: Codex
   - Est. complexity: L
   - Customer value: HIGH
   - Details: Add replay fixtures and tests for run, step, tool call, approval, artifact, memory, provider health, model route, AG-UI, and A2UI events. Verify events normalize through entity graph APIs and become visible in runtime console surfaces without manual refresh.

4. surreal-memory-workflow-mirror-tests: prove KBD workflow state mirror behavior
   - Scope: backend | MCP | workflow docs | tests
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Customer value: MEDIUM
   - Details: Implement or document the workflow-state mirror path for project, phase, task, assessment, plan, blocker, and verification records through the UAR `/mcp/memory` endpoint. Add tests or a deterministic script proving create, retrieve, update, and newest-`updated_at` conflict resolution while preserving `source_tool`.

5. openspec-global-validation-cleanup: restore repository-wide OpenSpec validation
   - Scope: openspec
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: S
   - Customer value: MEDIUM
   - Details: Repair `openspec/changes/implement-opencode-suggestions/` requirement deltas so each requirement has SHALL/MUST wording and valid scenarios, or archive/remove the invalid active change through the OpenSpec workflow. Acceptance is `openspec validate --changes` passing.

6. moonshot-provider-status: resolve or explicitly classify Moonshot Kimi k2.6 compatibility
   - Scope: backend | provider registry | UI status | docs/tests
   - Depends on: frontend-lint-zero-warning
   - Recommended agent: Codex
   - Est. complexity: S
   - Customer value: MEDIUM
   - Details: Re-test Moonshot with a valid credential if available. If authentication still fails, update provider diagnostics/UI status so Moonshot is marked credential-blocked rather than silently failing, and record the status in provider compatibility documentation or tests without committing secrets.

7. runtime-console-archive-readiness: verify and archive the runtime console change
   - Scope: openspec | kbd | verification
   - Depends on: frontend-lint-zero-warning, runtime-console-live-visual-tests, runtime-event-replay-entity-sync-tests, surreal-memory-workflow-mirror-tests, openspec-global-validation-cleanup, moonshot-provider-status
   - Recommended agent: Codex
   - Est. complexity: S
   - Customer value: HIGH
   - Details: Run the final gate: `openspec validate --changes`, frontend typecheck/lint/e2e, focused backend tests, and KBD reflection. Then verify and archive `runtime-console-entity-workflow` or leave a narrow documented exception if any external provider credential remains blocked.

EXECUTION ROUND ORDER

Round 1 (parallel): frontend-lint-zero-warning, surreal-memory-workflow-mirror-tests, openspec-global-validation-cleanup
Round 2 (parallel): runtime-console-live-visual-tests, runtime-event-replay-entity-sync-tests, moonshot-provider-status
Round 3 (serial): runtime-console-archive-readiness

COMMANDS TO RUN

/opsx:new frontend-lint-zero-warning
/opsx:new runtime-console-live-visual-tests
/opsx:new runtime-event-replay-entity-sync-tests
/opsx:new surreal-memory-workflow-mirror-tests
/opsx:new openspec-global-validation-cleanup
/opsx:new moonshot-provider-status
/opsx:new runtime-console-archive-readiness

VALIDATION MATRIX

- `bun run lint` must pass with zero errors and zero warnings.
- `bun run typecheck` must remain green.
- `bun run test:e2e` or targeted Playwright suites must cover runtime console desktop/mobile and live-update behavior.
- `openspec validate runtime-console-entity-workflow` must pass.
- `openspec validate --changes` must pass.
- Focused backend provider/knowledge/memory tests must pass.
- Surreal Memory workflow mirror must demonstrate create, retrieve, update, and conflict handling.

TRADE-OFFS AND SCOPE CUTS

- This phase is hardening, not feature expansion. Do not add new runtime console surfaces unless required to test or expose an existing closure gap.
- Moonshot compatibility may close as "credential-blocked" if a valid key is not available; do not spend time reverse-engineering provider auth beyond documented OpenAI-compatible behavior.
- If global OpenSpec validation requires broad unrelated product decisions, isolate the invalid change through the OpenSpec workflow rather than expanding this phase into an unrelated spec rewrite.

PLAN COMPLETE
