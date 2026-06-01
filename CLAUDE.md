
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
