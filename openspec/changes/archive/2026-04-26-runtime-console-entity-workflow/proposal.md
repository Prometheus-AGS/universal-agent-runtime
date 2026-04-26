# Runtime Console Entity Workflow

## Why
The frontend currently has useful admin screens, but it does not yet behave as a
live agent operations cockpit. UAR needs parity with librefang's compact
registry/detail UX while exposing runtime-specific state: runs, tool calls,
approvals, artifacts, memory, provider health, model routing, AG-UI/A2UI events,
and Anthropic/OpenAI compatibility diagnostics.

OpenSpec and the Prometheus KBD process also need to work together across
Codex, Claude Code, Cursor, and OpenCode so implementation state survives tool
handoffs.

## What Changes
- Establish OpenSpec as the shared spec/change system and KBD as the phase
  execution state machine.
- Mirror workflow state into Surreal Memory MCP while keeping
  `.kbd-orchestrator/` authoritative.
- Extend the frontend entity graph to cover runtime entities in addition to
  static admin entities.
- Route realtime run/protocol/provider updates through stores and the entity
  graph before rendering.
- Add a librefang-inspired runtime console shell with dense navigation,
  command palette, registry/detail views, and protocol inspection surfaces.
- Improve UI support for `liter-llm` provider routing, Anthropic prompt caching,
  AG-UI, and A2UI.

## Impact
- Frontend users get live operational visibility instead of mostly static admin
  pages.
- Agent tools share consistent planning state across OpenSpec, KBD, and memory.
- Provider/protocol compatibility becomes testable from the UI.

## Non-Goals
- This does not make UAR a pixel-for-pixel librefang clone.
- This does not bypass `liter-llm` with direct provider calls.
- This does not make Surreal Memory the source of truth for workflow state.
