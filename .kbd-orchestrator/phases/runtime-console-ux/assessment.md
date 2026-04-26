# Runtime Console UX Assessment

## Recommendation
Proceed, but treat the work as a staged live runtime-console redesign rather
than a cosmetic UI clone.

## Current Gaps
- The frontend has admin pages, but not a unified live operations cockpit.
- Entity graph integration exists, but typecheck failures show the package
  contract is not yet aligned.
- Runtime concepts such as runs, run steps, tool calls, approvals, artifacts,
  provider health, route decisions, AG-UI events, and A2UI surfaces are not yet
  consistently modeled as frontend entities.
- OpenSpec existed but still had placeholder project context.
- KBD process files were present only as incomplete placeholders in tool skill
  folders.
- Surreal Memory MCP existed in the runtime but was not registered as a
  workflow-state mirror in `mcp.json`.

## Target State
- OpenSpec defines the change and project context.
- KBD tracks phase state across Codex, Claude Code, Cursor, and OpenCode.
- Surreal Memory mirrors phase/task state for recovery and lookup.
- The UI behaves like a dense live runtime console inspired by librefang.
- Realtime state enters stores/services first, normalizes into the entity graph,
  then renders through hooks/components.
