# UAR operator shell — design reference

High-fidelity, clickable HTML prototypes for the redesigned UAR web UI and Tauri desktop shell. The
shell is a single collapsible left sidebar plus a tabbed right inspector, matching the AI chat and agent
tools the redesign studied. This folder is the design reference for the React 19 frontend in
`frontend/`. The pages are static, open straight from the file system and need no build.

Start with `index.html`, the launcher for every screen.

## Screens

| Area | File | What it covers |
|---|---|---|
| Chat | `chat-workspace.html` | The shell: sidebar, thread with execution record and approvals, composer, inspector (Run · Files · Browser · Plan). Conversation target: orchestrator with routing on or off, or a pinned agent. `?new=1` opens the new-chat chooser, `?agent=<id>` pins an agent, `&project=<id>` puts the chat in a project |
| Workspace | `projects.html` | **Proposed.** Projects: work board, chats, files and linked knowledge, agent and tool ceiling, instruction stack, memory, members, budget |
| | `knowledge.html` | Knowledge bases: create, upload and index documents, retrieval tester, embedding-mismatch guard, re-embed |
| | `agents.html` | Agents: compile from a description, model and fallbacks, knowledge, skills, tools and MCP, A2UI libraries, AG-UI access |
| | `a2ui-registries.html` | A2UI catalogs, components, presentation templates, design systems, agent bindings, surface preview |
| | `runs.html` | Runs with sub-agent nesting; trace, approvals, admission evidence, usage, checkpoint resume |
| Settings | `models-and-providers.html` | liter-llm providers and models: roles, connection test, model table, routing, add-provider flow |
| | `tools-and-mcp.html` | Native tools, MCP server health, per-tool approval tiers |
| | `skills.html` | Skills, matching tester, required-tool availability, evolution under Cedar |
| | `knowledge-and-memory.html` | Knowledge defaults and agent memory |
| | `governance.html` | Enforcement state, Cedar request tester, approvals, guardrails, budget, access |
| | `appearance.html` | Dark, light, high contrast and system themes; text size, density and motion |
| | `desktop.html` | Tauri-only: in-process runtime and port fallback, sidecars, window, logs, build identity |

## Documents

- `ui-research-findings.md` — pattern survey, audit of the current frontend, decisions, and the
  **backend deltas** the prototypes depend on (§5).
- `project-management-design.md` — the proposed Projects feature: scoping and precedence rules, domain
  types, PostgreSQL migration, SurrealDB schema, `PersistenceLayer` additions, Cedar, HTTP API, web client
  and rollout.
- `proposed-schema/` — the Projects migration as standalone files:
  `20261006000000_projects.sql` (for `migrations/`) and `projects.surql` (for `migrations/surrealdb/`).
  They are **not** applied by the runtime until they are moved there and wired into the Surreal provider.
- `agentic-workspace-shell-pattern.md` — the chat-plus-canvas three-pane shell (Navigator · Thread · Canvas): its name and sources, the rules, and how UAR, PatientGPT and the San Saba Workspace apply it, including mini apps on the right-hand canvas.
- `react-implementation-plan.md` — how these prototypes become React 19 code in `frontend/`: tiers, the `@prometheus-ags/uar-ui` package, shell, routes, features, gates, delivery order, and the decisions of 2026-10-05 (Inter body font, library package, boundary allowlist).
- `brand-spec.md` — token bindings from `frontend/src/shared/theme/tokens.css` and the Flat 2.0 rules.
- `tauri-integration-notes.md` — window, titlebar, capabilities and menu notes for the desktop build.

## Shared assets

- `assets/shell.css`, `assets/shell.js` — tokens, components and shell behaviour (sidebar, inspector,
  command palette, theme and appearance preferences). Framework-neutral, so each piece maps onto a CSS
  Module and a Zustand slice.
- `assets/settings.css`, `assets/settings.js` — the layer used by settings and workspace pages: one route
  table renders both sidebars, plus the icon sprite and shared controls.
- `assets/fonts/` — Inter, Space Grotesk and JetBrains Mono, latin variable subsets (SIL Open Font
  License). `assets/brand/` — UAR marks from `website/static/img/brand/`.

## Reading the prototypes honestly

- All data is sample data. Spend figures are labelled as samples.
- Wherever a screen needs something the backend does not provide yet, it shows a **Backend gap** note.
  The same gaps are collected in `ui-research-findings.md` §5 and, for Projects, in
  `project-management-design.md`.
- Flows show both success and failure paths with simulated latency. Nothing calls the runtime.

## Provenance

Drafted in an Open Design project and copied here on 2026-10-05. Edit the prototypes in the design
project and copy them again rather than editing the copies in place, so the two don't drift.
