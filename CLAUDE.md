
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an agentic streaming LLM application that combines Rust (Axum + Leptos) with HTML-first frontend technologies (HTMX, Web Components, Alpine.js). The application is designed to be tool-first, streaming-native, and Tauri-compatible for web/desktop/mobile deployment.

## Development Commands

### Rust Backend
```bash
# Run the server in development mode
cargo run

# Build the Rust application
cargo build --release

# Run tests
cargo test

# Check for linting issues (extensive clippy configuration)
cargo clippy

# Format Rust code
cargo fmt
```

### Frontend Assets
```bash
# Build all frontend assets (TypeScript, CSS, WASM)
bun run build

# Development mode with file watching
bun run dev

# Type check TypeScript without emitting
bun run check

# Lint TypeScript files
bun run lint

# Format frontend code
bun run format
```

### Individual Asset Building
```bash
# Build TypeScript only
bun run build:ts

# Build CSS with Tailwind
bun run build:css

# Copy WASM files from dependencies
bun run copy:wasm
```

## Architecture Overview

### Core Technologies
- **Backend**: Rust with Axum web framework, Leptos for SSR
- **Frontend**: HTMX 2.0.8, Web Components (TypeScript), Alpine.js
- **Streaming**: Server-Sent Events (SSE) with normalized event model
- **Tools**: MCP (Model Context Protocol) via rmcp Rust SDK
- **Styling**: Tailwind CSS (ShadCN-inspired design system)

### Key Architectural Patterns

#### Event-Driven Streaming Architecture
The application uses a normalized event model for LLM interactions that supports:
- Token streaming (`message.delta`)
- Tool call streaming (`tool_call.delta`, `tool_call.complete`)
- Tool results (`tool_result`)
- Error handling (`error`)
- Completion signaling (`done`)

All events are mirrored into AG-UI-style events (`agui.*`) for future compatibility.

#### MCP Tool Integration
- Tools are discovered dynamically from `mcp.json`
- Supports both stdio and HTTP-based MCP servers
- Tools are namespaced automatically (e.g., `time::now`, `tavily::search`)
- Server controls all tool execution (not the model)

#### HTML-First UI Philosophy
- Uses HTMX for navigation and server interaction
- Web Components provide client-side programmability
- Alpine.js handles local UI state only
- Progressive enhancement over heavy SPA frameworks
- Identical UI across web/desktop/mobile via Tauri compatibility

### Key Components

#### Rust Backend (`src/`)
- `main.rs`: Entry point, Axum server configuration
- `lib.rs`: Core application logic and orchestrator
- Session management via `SessionStore`
- LLM orchestration via `Orchestrator`
- MCP tool registry via `McpRegistry`

#### Frontend (`web/components/`)
- `<chat-stream>`: Main streaming chat interface
- `<chat-messages>`: Message container and management
- `<chat-tool-call>`: Tool call visualization
- Other specialized components for code blocks, Mermaid diagrams, etc.

#### Static Assets (`static/`)
- `main.js`: Compiled TypeScript bundle
- `app.css`: Compiled Tailwind CSS
- `*.wasm`, `*.data`: PGLite WebAssembly files

## Configuration Files

### MCP Tools Configuration (`mcp.json`)
Define MCP servers for tool discovery:
```json
{
  "mcpServers": {
    "time": {
      "command": "npx",
      "args": ["-y", "@mcpcentral/mcp-time"]
    },
    "tavily": {
      "url": "https://mcp.tavily.com/mcp/?tavilyApiKey=${TAVILY_API_KEY}",
      "env": {
        "TAVILY_API_KEY": "${TAVILY_API_KEY}"
      }
    }
  }
}
```

### Environment Variables
Set up the following in `.env` (copy from `.env.example`):
- `UAR_LLM__MODEL`: Default model in `provider/model` format (e.g. `openai/gpt-4o`). See models.dev.
- `UAR_LLM__API_KEY`: API key for the selected provider.
- Provider-specific shortcuts: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GROQ_API_KEY`, etc.
- `TAVILY_API_KEY`: For web search functionality via MCP.
- `CREDENTIAL_ENCRYPTION_KEY` (optional): Enables multi-tenant provider credentials. When set (32 ASCII bytes or 64 hex chars), users may store their own provider API keys encrypted at rest (AES-256-GCM); requests resolve per-user keys via the scoped chain `session → agent → user → system → env`. Leave unset for single-tenant: provider keys come from the env/config values only (unchanged behavior).
- Backward-compatible: `LLM_API_KEY`, `LLM_BASE_URL`, `LLM_MODEL` (still supported, lower priority).

LLM configuration precedence (highest → lowest):
1. CLI args (`--llm-model`, `--llm-api-key`, `--llm-base-url`)
2. `UAR_LLM__*` env vars
3. Legacy `LLM_*` env vars
4. Provider-specific keys (`OPENAI_API_KEY`, etc.)
5. `llm:` section in `config.yaml`
6. Compiled defaults

## Important Development Patterns

### Streaming Implementation
- All LLM interactions default to streaming mode
- Events flow: LLM → Orchestrator → SSE → Web Components
- Protocol-agnostic design supports OpenAI Chat Completions, Responses, and compatible backends

### Tool-First Design
- Tools are non-optional and always available
- Server maintains MCP client connections
- Tool calls are deterministic and server-controlled
- Dynamic tool discovery at startup

### Component Architecture
- Web Components consume typed SSE events
- Components have clear lifecycle hooks and boundaries
- Zero framework lock-in approach

### Tauri Compatibility
- No CDN scripts (all assets served locally)
- No API keys in browser
- Same codebase for web/desktop/mobile
- SSE works identically in webview

## Code Quality Standards

The project uses extensive Rust linting (see `Cargo.toml` lints section) including:
- Clippy with pedantic and performance lints
- Custom restriction lints for better code quality
- Structured logging with `tracing`
- `mimalloc` for performance optimization

## Key Files to Understand

- `src/main.rs`: Server setup and routing
- `src/lib.rs`: Core orchestration logic
- `web/main.ts`: Frontend entry point
- `web/components/chat-stream/chat-stream.ts`: Main streaming interface
- `mcp.json`: Tool configuration
- `package.json`: Frontend build scripts
- `Cargo.toml`: Rust dependencies and linting configuration

This architecture represents a modern approach to building AI applications that prioritizes tool use, streaming interactions, and clean separation between server logic and client presentation.

## Worktree convention

Git worktrees for this repository are created under **`~/.claude/worktrees/`** — never inside the repo working tree. The repo's own `.claude/` directory holds checked-in tool configuration (`settings.local.json`, `commands/`, `skills/`) that is read by Roo, Cursor, Codex, OpenCode, and Claude Code; putting worktrees alongside that config collides namespaces, confuses tooling, and risks accidental deletion of real configuration during cleanup.

Always create a new worktree with:

```bash
scripts/worktree-new.sh <name> [--base <ref>]   # creates ~/.claude/worktrees/<name>
scripts/worktree-list.sh                        # show worktrees under that root
scripts/worktree-rm.sh <name> [--force]         # remove a worktree under that root
```

The helper refuses any path that would land inside the repo tree and seeds the new worktree's `.claude/settings.local.json` from the current checkout so per-tool permissions follow you.

Existing in-repo worktrees under `.claude/worktrees/` are intentionally **not relocated** — the convention applies to every worktree created from now on. The KBD orchestrator surfaces the active worktree path via `/kbd-status` and warns when the current checkout is outside `worktreeRoot` (configured in `.kbd-orchestrator/project.json`).

## Prometheus Base Rules Set

> Canonical base rules for Claude Code, Codex, OpenAI agents, Gemini CLI, Roo,
> Cline, Kilo Code, Librefang, and all Prometheus/UAR-compatible development
> agents. **Mirrored in [AGENTS.md](AGENTS.md) — update both files together.**
> These define how agents reason, code, modify files, and interact with human
> operators. The auto-managed "Think-first principles" block below is a subset;
> this section is the full canonical set. Per **Rule 26**, project-specific
> sections may add *stricter* requirements but may not contradict these.

### Reasoning & execution

**1. Think Before Coding** — Do not assume; do not hide confusion; surface
tradeoffs before implementation. State assumptions explicitly. If uncertain,
ask. If multiple interpretations exist, present them. If a simpler approach
exists, say so. If something is unclear, stop and ask.

**2. Simplicity First** — Write the minimum code that solves the problem. No
features beyond what was requested, no speculative abstractions, no unnecessary
configurability, no unrequested future-proofing, no overengineering. If 50 lines
solve it, do not write 200.

**3. Surgical Changes** — Touch only what is necessary. Do not refactor or
reformat unrelated code. Match existing conventions. Remove only artifacts you
created. Mention unrelated issues; do not fix them unless asked.

**4. Goal-Driven Execution** — Define success criteria first. Convert vague
requests into testable outcomes. Verify completion; run tests where available.
Stop only when success criteria are satisfied.

**5. Truth Over Fluency** — Never prefer a confident answer over a correct one.
Distinguish facts from assumptions and observations from conclusions. State
uncertainty explicitly. Do not invent APIs, functions, files, packages,
commands, or behavior. If something is unknown, say so plainly.

**6. Evidence Before Conclusions** — Cite evidence, show the reasoning path,
explain tradeoffs and why alternatives were rejected. Prefer primary sources,
source code, tests, official docs, or direct observation over guesses.

**7. Preserve User Intent** — Optimize for the user's actual goal, not your
preferences. Do not silently expand or reduce scope. Clarify when requirements
conflict. Preserve the user's architectural direction unless told otherwise.

**8. Minimize Irreversible Actions** — Before destructive or hard-to-reverse
actions: confirm intent, explain consequences, prefer reversible approaches,
create rollback paths. Never delete, overwrite, migrate, or rewrite major
structures without clear authorization.

**9. Maintain Architectural Consistency** — Prefer consistency over novelty.
Follow existing architecture, patterns, naming, and state-management
conventions. Avoid introducing new frameworks without justification. No one-off
architectural exceptions.

**10. Keep Context Explicit** — Never rely on hidden assumptions. State
dependencies, constraints, and limitations. Record decisions and important
reasoning in the appropriate project file. Make implicit contracts explicit.

**11. Architecture Before Code** — Before implementation, identify affected
subsystems, data flow, interface contracts, persistence/UI/security/runtime
impact, and the testing strategy. Do not start coding until the architecture is
understood.

### Standards, state & portability

**12. Open Standards First** — Prefer open, portable, ecosystem-agnostic
standards: MCP, OpenAI-compatible APIs, A2A, AG-UI, A2UI, HTMX, WASM Component
Model, JSON Schema, OpenAPI, GraphQL (where appropriate), PostgreSQL-compatible
storage, IPFS-compatible distribution (where appropriate). Avoid vendor lock-in
unless explicitly required.

**13. No Hidden State** — Business state must live in explicit, inspectable
systems: databases, event streams, explicit stores, durable queues, documented
runtime state containers. State must not hide inside UI components, untracked
globals, implicit caches, framework magic, or agent-only memory without
persistence/auditability.

**14. Cross-Platform Parity** — Feature proposals must consider web, mobile,
desktop, local execution, cloud execution, and offline/degraded operation where
relevant. Do not trap the platform in a single runtime, framework, vendor, or
deployment model unnecessarily.

**35. Prefer Deterministic Systems** — Where possible prefer deterministic
behavior: IDs, allocation, ordering, retries, replay, and explicit conflict
resolution. Non-determinism must be intentional and documented.

**36. Local-First When Practical** — Prefer architectures that run locally and
sync outward: local execution/storage, offline-capable workflows, syncable
state, portable runtimes, edge-compatible agents. Cloud is allowed but avoid
unnecessary cloud dependence.

**37. Runtime Portability Matters** — Design for cloud, local, mobile, browser,
edge, WASM, and containerized execution. Avoid coupling business logic to a
runtime unless required.

### Architecture & layering

**15. Feature-Based Clean Architecture Required** — Organize codebases around
features/domains/bounded contexts rather than technical layers:

```
src/
├── features/
│   ├── customers/
│   │   ├── components/  hooks/  stores/  services/
│   │   ├── types/  schemas/  pages/  tests/
│   ├── orders/
│   └── billing/
├── shared/
├── core/
└── infrastructure/
```

Organize by business capability first. Avoid global component dumping grounds.
Keep feature logic in the owning feature. Shared code must be genuinely
reusable. Cross-feature dependencies must be explicit. Business logic belongs to
the feature domain, not the UI.

**16. Strict Layering Is Mandatory** — Enforce clear boundaries with one
direction of flow:

```
UI Components → Hooks/View Models → State Stores → Services/Repositories/APIs → External Systems
```

Reverse communication occurs only through state propagation and events.
*Allowed:* UI→Hook, Hook→Store, Store→Service, Service→API. *Not allowed:*
UI→API, UI→Service, UI→Database, Hook→API, Hook→Service, Component→Store-mutation
logic.

**17. UI Components Must Remain Pure** — Components only render, handle
interaction, layout, styling, and accessibility. They must not fetch data, call
APIs/services, perform business logic, manage persistence, or run workflow
logic. A component should be replaceable without affecting business behavior.

**18. Hooks/View Models Coordinate UI State** — Hooks connect UI to stores and
handle UI-state composition, UI-derived calculations, and presentation logic.
They must not call APIs/DBs directly, implement persistence, or contain domain
rules. (React Hooks, Flutter Controllers, Riverpod Notifiers, Vue Composables.)

**19. Stores Own Application State** — Stores are the single source of truth:
they call services, coordinate loading, manage optimistic updates, maintain
cache, and expose reactive state. Stores must not contain UI rendering logic.
(Zustand, Riverpod, Redux Toolkit, MobX, Signals.)

**20. Services Own External Communication** — Services handle API calls, DB
access, MCP/agent communication, external integrations, and file I/O. They must
be reusable, testable, and framework-independent where possible; they must not
render UI, manage component state, or contain presentation concerns.

**21. State Changes Must Be Reactive** — Propagate state changes through the
framework's native reactive mechanism (Zustand/Riverpod/Signals/Rx/Vue/Svelte).
Avoid manual refresh calls, hidden mutable state, direct component manipulation,
or imperative UI synchronization. The UI reacts automatically.

**24. Consistency Across Languages** — The architecture is identical regardless
of language; only the technology changes:

```
React:      Component → Hook        → Zustand Store → Service → API
Flutter:    Widget    → Controller  → Riverpod      → Service → API
Rust HTMX:  Template  → Handler     → Store/Domain  → Service → Repository
Vue:        Component → Composable  → Store         → Service → API
Svelte:     Component → View Model  → Store         → Service → API
```

**28. No Untouchable Framework Magic** — Do not introduce systems that force
case-by-case reasoning around hidden behavior: opaque caches, hidden global
state, framework-owned business logic, state trapped in component tiers, magic
side effects, uninspectable runtime behavior. Prefer predictable, explicit,
inspectable architecture.

**38. UI Is a Projection of State** — The UI must not be the source of truth. UI
renders state and submits intent; domain logic validates intent; durable systems
persist state; events describe changes. No business rules that exist only in
frontend components.

**39. Artifacts Must Be Structured** — Prometheus artifacts must be typed,
versioned, inspectable, portable, renderable across supported hosts, compatible
with agent workflows, and safe to persist and replay. Do not invent ad hoc
artifact formats when a formal schema exists.

### Dependencies & typing

**22. Dependency Versions Must Be Verified** — Before adding libraries,
frameworks, SDKs, runtimes, build tools, or infra: verify current compatible
versions against official docs/repos/compatibility matrices and existing
dependencies. Never assume versions or reuse stale training-era examples when
current info is available.

**23. Web Verification Before Dependency Introduction** — When internet access
is available, search for the latest stable version, known compatibility issues,
breaking changes, migration requirements, and security advisories. Priority:
official docs → official repo → release notes → vendor migration guides.
Community sources supplement, never replace, authoritative ones.

**27. No Silent Dependency Introduction** — Before adding a dependency, check
existing deps and prefer existing project tools. Explain why it is needed. Avoid
large deps for small tasks, architecture-conflicting deps, and lock-in.

**29. Strong Typing Required** — Use strong types wherever the language supports
them. No implicit/unnecessary `any`, no untyped business objects, no stringly
typed domain models when proper types are possible. Prefer generated types from
schemas. Keep API contracts typed and versioned.

### Process, safety & completion

**25. Human Override Always Exists** — Every automated decision must support
inspection, auditability, override, recovery, manual correction, and human
escalation. Agents may assist, recommend, automate, and execute, but humans must
remain able to inspect and override critical outcomes.

**26. Repo-Level Rules Override Base Rules Only When Explicit** — These are base
rules. Project-specific `CLAUDE.md`, `AGENTS.md`, README, architecture docs, or
task instructions may add stricter requirements, and may override these only
when explicit and non-contradictory with safety, correctness, and user intent.

**30. Tests Are Part of Completion** — Implementation is not complete until
verified. Where available, run unit/integration tests, type checks, linters, and
build checks; add tests for new behavior; update tests when behavior
intentionally changes. If tests cannot be run, state why.

**31. Prefer Small, Reviewable Changes** — Keep commits focused and diffs small.
Avoid broad rewrites and unrelated cleanup. Separate mechanical changes from
behavioral changes. Explain what changed and why.

**32. Preserve Existing Behavior** — Do not break existing behavior unless the
task requires it. First identify current vs desired behavior and compatibility
impact; update tests and docs; call out breaking changes clearly.

**33. Security Is Not Optional** — Always consider authentication,
authorization, input validation, output escaping, secrets handling, tenant
boundaries, data leakage, prompt injection, tool-execution boundaries, and
dependency risk. Never log secrets, tokens, credentials, private keys, or
sensitive user data.

**34. Agent Actions Must Be Auditable** — For agentic systems, preserve an audit
trail: user request, agent decision, tool calls, inputs, outputs, files changed,
external effects, errors, and human approvals where required. Agentic execution
without auditability is not acceptable.

**40. Stop When Done** — Do not keep expanding after the goal is satisfied. When
done: summarize what changed, summarize how it was verified, and list remaining
risks or follow-ups. Do not perform extra work unless asked.

<!-- agent-rules:start v1 -->
## Agent rules

> Auto-managed by `/kbd-inject-agent-rules`. Re-running the skill
> overwrites everything between the `agent-rules:start` / `agent-rules:end`
> markers. Edit the cache at
> `kbd-process-orchestrator/skills/kbd-inject-agent-rules/references/rules-cache.md`
> if you need to change the content.

### Think-first principles (Karpathy)

1. **Think Before Coding** — State assumptions explicitly, surface
   ambiguity, present tradeoffs, ask for clarification rather than
   guessing silently.
2. **Simplicity First** — Write the minimum code that solves the
   problem; no speculative features.
3. **Surgical Changes** — Touch only what the request requires.
4. **Goal-Driven Execution** — Operate against concrete success
   criteria, not step-by-step micro-instructions.

### Workflow principles (Claude Code, Boris Cherny)

1. **Plan Mode First** — Iterate the plan until it's right; only then
   auto-accept edits.
2. **CLAUDE.md as accumulated knowledge** — Long-lived project rules;
   accumulate constraints and lessons over time.
3. **Verification + feedback loops** — Give the agent a way to verify
   its work (2-3× quality bump).
4. **Code quality matters for AI too** — Partially-migrated codebases
   confuse models. Finish migrations.

Verbatim sources + fetch dates in
`kbd-process-orchestrator/skills/kbd-inject-agent-rules/references/rules-cache.md`.
<!-- agent-rules:end -->

<!-- uiux-routing:start v1 -->
## UI/UX work routing

Before writing or modifying any UI/UX code in this repo, the AI agent
**MUST** follow these steps in order. The roster of skills + source
URLs is cached at
`.kbd-orchestrator/references/uiux-skill-roster.md` (refreshable via
`/kbd-inject-agent-rules --pack uiux-routing --refresh`).

1. **Memory consult.** Run `/kbd-memory-recall` (default-on via
   `assess:before` hook) to populate `prior-context.md` with prior
   UI/UX decisions in surreal-memory.
2. **UI/UX Pro Max analysis.** Run the design-system + audit pass on
   target components / pages. Pull palette + font + spacing + a11y
   recommendations from its database.
3. **Impeccable commands.** Always run `/impeccable audit` +
   `/impeccable critique`. Then run the work-specific commands —
   `/impeccable polish` before shipping, `/impeccable distill` when
   simplifying, `/impeccable animate` when adding motion,
   `/impeccable harden` for edge-case + i18n, etc.
4. **Anthropic skills.** Consult `frontend-design` + `ux-designer`
   for intentional design + UX-engineer review perspective.
5. **Vercel skills.** Consult React Best Practices + Composition
   Patterns. For the entity-explorer panel and Chrome extension work
   specifically (changes 10 + 11), **also web-search**: "runtime
   devtools page best practices" AND "Chrome MV3 devtools panel
   patterns" / "react-devtools bridge architecture".
6. **Summarise.** Write a one-paragraph distillation of the relevant
   best practices for this specific task. Reference the roster
   entries you actually consulted.
7. **Only then write code.** The summary above is the prompt context
   for the implementation step.

This routing block is auto-managed; re-run
`/kbd-inject-agent-rules --pack uiux-routing` to update. See
`kbd-process-orchestrator/skills/kbd-inject-agent-rules/SKILL.md` for
the `--pack` flag and the fenced-region machinery.
<!-- uiux-routing:end -->
