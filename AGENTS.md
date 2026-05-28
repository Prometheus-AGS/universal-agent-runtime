
# AGENTS.md - Repository Guidelines

## Build, Lint & Test

- **Rust**: `cargo build`, `cargo clippy --all-targets --all-features`, `cargo fmt`
- **Web**: `bun install`, `bun run build`, `bun run lint`, `bun run format`
- **Test All**: `cargo test`, `bun test web/tests`
- **Single Test**: `cargo test <test_name>`, `bun test <file_pattern>`
- **Clean Build**: ZERO warnings/errors allowed. Fix warnings immediately or use `#[expect(lint, reason="...")]`.

## Code Style & Conventions

- **Rust**: 4-space indent. `snake_case` (fn/mod), `CamelCase` (types). Use `anyhow` for app errors, `tracing` for structured logs.
- **TypeScript**: 2-space indent, semicolons, TS 5.9.3. Prefer Web Components in `web/components/`.
- **Imports**: No glob re-exports (`pub use foo::*`). Use `#[doc(inline)]` for public re-exports.
- **Documentation**: Public items must have `///` docs with `# Examples`, `# Errors`, and `# Panics` sections.

## Architecture & UI

- **Structure**: `src/` (Axum/Leptos SSR), `web/` (TS/Web Components), `static/` (Bundled assets).
- **UI Reference**: `docs/htmx/` for Material 3 Flat 2.0 patterns (borderless, token-based theming).
- **Config**: `.env` (see `.env.example`), `example.config.yaml`, `mcp.json` for MCP tools.
- **LLM**: All LLM access goes through [liter-llm](https://github.com/GQAdonis/liter-llm) — 142+ providers via unified `provider/model` addressing. Set `UAR_LLM__MODEL` and `UAR_LLM__API_KEY` (or a provider shortcut like `OPENAI_API_KEY`). See `example.config.yaml` for the full `llm:` section.
- **Model routing**: Use `POST /api/uar/route` with capability requirements (`needs_tools`, `needs_vision`, `min_context`, etc.) to get the best available model. The catalog is built at compile time from models.dev + liter-llm schemas.

### React frontend (`frontend/`)

Strict layering — do not skip layers:

1. **Components** never call `fetch`, never import Zustand stores directly, and never import `frontend/src/services/`. They only render UI and call **hooks**.
2. **Hooks** only subscribe to stores and expose store actions; they do not call `fetch` or import service modules. Service calls live inside **stores**.
3. **Stores** (`frontend/src/stores/`) hold state and call **services** for HTTP/SSE and other I/O.
4. **Services** (`frontend/src/services/`) are thin wrappers around `fetch` / streams. **Only stores import services** (not hooks or components).

This avoids duplicated data logic, keeps ESLint `react-hooks/*` rules satisfied, and makes testing straightforward.

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
