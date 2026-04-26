# Tasks

## Workflow Foundation
- [x] Refresh OpenSpec for Codex, Claude Code, Cursor, and OpenCode.
- [x] Populate OpenSpec project context.
- [x] Install KBD process orchestrator skills for Codex, Claude Code, Cursor, and OpenCode.
- [x] Create `.kbd-orchestrator/` state for the `runtime-console-ux` phase.
- [x] Register Surreal Memory MCP as the secondary workflow-state mirror.

## Frontend Foundation
- [x] Fix current frontend typecheck failures in entity schema/hooks and icon props.
- [x] Add runtime entity types for runs, steps, tool calls, approvals, artifacts,
      memory events, AG-UI events, A2UI surfaces, route decisions, and provider health.
- [x] Add realtime ingestion helpers that normalize SSE/AG-UI/A2UI/provider
      updates into the entity graph.

## Runtime Console UX
- [x] Add compact runtime shell structure inspired by librefang.
- [x] Add command palette and registry/detail navigation.
- [x] Add runtime cockpit, protocol inspector, provider routing, memory, and
      approvals views.
- [x] Verify desktop and mobile rendering in browser.

## Validation
- [x] Run `openspec validate runtime-console-entity-workflow`.
- [x] Run frontend typecheck.
- [ ] Run frontend lint/tests cleanly.
- [x] Run backend tests relevant to Anthropic/OpenAI compatibility and knowledge search persistence.
