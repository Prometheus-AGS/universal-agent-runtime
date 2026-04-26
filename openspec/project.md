# Project Context

## Purpose
Universal Agent Runtime (UAR) provides a local-first, multi-provider agent
runtime with web UI, OpenAI-compatible APIs, Anthropic-compatible APIs, AG-UI and
A2UI surfaces, MCP integrations, and SurrealDB-backed memory.

The active product direction is to turn the frontend into a live agent
operations console: runs, tool calls, approvals, artifacts, memory, provider
health, model routing, and protocol traces should be visible and update without
manual refresh.

## Tech Stack
- Rust backend using Axum, anyhow, tracing, and cargo.
- React/TypeScript frontend under `frontend/`, built with Bun/Vite.
- Zustand stores, shadcn-style UI primitives, lucide icons, assistant-ui.
- `@prometheus-ags/prometheus-entity-management` for normalized entity graph
  state and realtime UI updates.
- `liter-llm` for all LLM provider/model access.
- OpenSpec and KBD orchestrator for cross-tool planning state.
- Surreal Memory MCP via UAR's in-process `/mcp/memory` endpoint.

## Project Conventions

### Code Style
- Rust uses 4-space indentation, `snake_case` functions/modules, `CamelCase`
  types, `anyhow` for app errors, and `tracing` for structured logs.
- TypeScript uses 2-space indentation and semicolons.
- Prefer existing local components, hooks, stores, services, and entity helpers
  over new abstraction families.

### Architecture Patterns
- Backend sources live under `src/`.
- React app lives under `frontend/`.
- Frontend layering is strict: components -> hooks -> stores -> services.
- Components must not call `fetch`, import Zustand stores directly, or import
  `frontend/src/services/`.
- Runtime UI state should be normalized into the entity graph before it is
  rendered.

### Testing Strategy
- Rust: `cargo test`, `cargo clippy --all-targets --all-features`.
- Frontend: `bun run typecheck`, `bun run lint`, and focused Bun tests.
- User-facing UI changes should be checked in browser at desktop and mobile
  widths.
- Workflow changes should run `openspec validate` and verify KBD state files.

### Git Workflow
Use short-lived feature branches. Do not revert user changes. Keep generated
workflow and UI changes separate enough to review.

## Domain Context
UAR is used alongside librefang and should share its compact, registry-driven,
operator-focused UX patterns where appropriate. The goal is not a pixel clone;
the goal is interaction parity for agent runtime work.

AG-UI/A2UI, Anthropic REST compatibility, OpenAI compatibility, prompt caching,
MCP server health, model routing, and memory traceability are core runtime
features and should be inspectable in the UI.

## Important Constraints
- All LLM access should route through `liter-llm`.
- Preserve the frontend layering rules.
- Keep `.kbd-orchestrator/` as the primary workflow source of truth.
- Treat Surreal Memory MCP as a secondary mirror for workflow state.
- Maintain compatibility with Codex, Claude Code, Cursor, and OpenCode workflows.

## External Dependencies
- `liter-llm` provider catalog and routing.
- OpenAI-compatible REST clients.
- Anthropic-compatible REST clients, including Claude Code-style callers.
- Surreal Memory MCP.
- Tavily MCP for research.
- OpenSpec CLI.
