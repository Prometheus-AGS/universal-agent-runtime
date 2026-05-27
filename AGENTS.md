
# AGENTS.md - Repository Guidelines

## Build, Lint & Test

- **Rust toolchain**: pinned via `rust-toolchain.toml` to `nightly-2026-05-01` with `wasm32-unknown-unknown`, `wasm32-wasip1`, `wasm32-wasip2` targets pre-added. `rustup` auto-installs on first `cargo` invocation. The Dockerfile `toolchain` stage uses the same `ARG RUST_TOOLCHAIN`. Bump both together via a new KBD change.
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

### Realtime freshness contract — no stale data anywhere

Every entity that flows through REST is mirrored via SurrealDB live queries → SSE on `/api/live/{topic}` → the entity-graph store (`@prometheus-ags/prometheus-entity-management`). The 10 enrolled topics are listed in `src/uar/realtime/mod.rs::EntityTopic::ALL` and `frontend/src/lib/realtime/topics.ts::UAR_TOPICS` (CamelCase must match the entity types registered in `frontend/src/entities/schemas.ts`).

Each Zustand admin hook uses [`useGraphBridge`](frontend/src/lib/realtime/use-graph-bridge.ts) to refresh its store whenever the graph mutates. The flow:

```
DB write → SurrealDB live event → LiveQueryBus → SSE → entity-graph upsert → bridge fires → store.load() → component re-render
```

When adding a new entity surface:
1. Add the topic to `EntityTopic` + register the schema in `entities/schemas.ts`.
2. Either consume directly via `useEntity` / `useEntityList`, **or** keep a Zustand store and call `useGraphBridge(["EntityType"], load)` inside its admin hook.
3. Update [`docs/migration-stale-data-audit.md`](docs/migration-stale-data-audit.md).

High-frequency mutations (skill toggle, agent enable, provider set-default) use **optimistic patches**: the store applies the change instantly and rolls back on server rejection; the SSE bridge reconciles the authoritative state afterward.
