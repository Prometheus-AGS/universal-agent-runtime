# Design

## Workflow State
`.kbd-orchestrator/` is authoritative for phase state. Surreal Memory MCP mirrors
compact workflow records keyed by project, phase, task, and timestamp. Records
must include `source_tool` so Codex, Claude Code, Cursor, and OpenCode handoffs
can be audited.

## Runtime State
Runtime UI state should be normalized into the frontend entity graph before
rendering. New runtime entities should represent runs, run steps, tool calls,
approvals, artifacts, memory events, AG-UI events, A2UI surfaces, model route
decisions, and provider health.

## UX Direction
The shell should follow librefang's strengths: compact top-level navigation,
command search, dense registries, detail panes, breadcrumbs, sticky context, and
theme tokens. The implementation should stay domain-specific to agent runtime
operations.

## Protocol Compatibility
The console should expose compatibility diagnostics for Anthropic `/v1/messages`,
OpenAI-compatible APIs, AG-UI, A2UI, MCP health, prompt caching, and `liter-llm`
model routing.
