# Plan — `prometheus-package-integration`

**Date:** 2026-05-26
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec (detected at `openspec/`)
**Assessment input:** `.kbd-orchestrator/phases/prometheus-package-integration/assessment.md`

---

## Decisions locked

1. Replaces `runtime-provider-protocol-hardening` as current waypoint.
2. Frontend: **pnpm workspaces** (migrating from single-package bun).
3. Skill model: **extend `Skill` with `kind: Wasm | Manifest | Native`** (single unified type) — plus a separate `origin: Builtin | User` flag for deletion guard.
4. Skill-system submodule path: `crates/prometheus-skill-system/`.
5. Default-assistant binding for builtin skills: **opt-in per agent** (registered at startup, referenced explicitly).

## New requirements added with /kbd-plan invocation

- **No stale data anywhere** across views (toolbars, dropdowns, headers, lists). Cross-view propagation is mandatory.
- **Realtime change notifications from the database**. SurrealDB's native **Live Queries** (`.select().live()` returning `Action::Create | Update | Delete` notifications) are the chosen primitive; Postgres backend will mirror via `LISTEN/NOTIFY` triggers when re-enabled. ([Live Queries — Rust SDK docs](https://surrealdb.com/docs/sdk/rust/concepts/live))
- **WASM plugin model** for first-class skills via the **WebAssembly Component Model + WIT**. UAR already has `wasmtime`/`wasmtime-wasi` 41 as optional deps under the `wasm-runtime` feature; we now formalize a `uar:skill@0.1.0` WIT world and load components dynamically via `wasmtime::component`. ([wasmtime::component docs](https://docs.wasmtime.dev/api/wasmtime/component/index.html))
- **Binary instance de-duplication**: before UAR launches child binaries (surreal-memory-server CLI variant, external MCP servers, etc.), it must detect already-running instances (port probe + health check + optional pidfile) and reuse them. No double-running expensive processes.

These additions raise the change count from the assessment's 5-slice sketch to **14 ordered OpenSpec changes** below. Each change is scoped to land independently; dependencies are encoded in the order.

---

## Ordered change list

| # | Change ID | Title | Agent | Depends on |
|---|-----------|-------|-------|------------|
| 1 | `fix-kb-document-count` | Backend: add `document_count` aggregate to KB list/detail responses | claude-code | — |
| 2 | `surreal-live-query-bus` | Live-query bus + SSE topic stream for cross-view realtime | claude-code | — |
| 3 | `add-skill-kind-and-origin` | Domain: extend `Skill` with `kind` + `origin`; DELETE guard for Builtin | claude-code | — |
| 4 | `binary-instance-discovery` | Probe-then-spawn for child binaries (Surreal, MCP servers, liter-llm) | claude-code | — |
| 5 | `add-skill-system-submodule` | Add `crates/prometheus-skill-system/` recursive submodule + `BuiltinSkillLoader` walks `SKILL.md` | claude-code | 3 |
| 6 | `wasm-component-skill-runtime` | Wasmtime Component Model loader + `uar:skill@0.1.0` WIT world + per-skill instantiation | claude-code | 3 |
| 7 | `frontend-pnpm-workspace-migration` | Convert `frontend/` from bun-single to pnpm workspaces; update `build.rs` | claude-code | — |
| 8 | `add-entity-management-submodule` | Add `frontend/packages/prometheus-entity-management/` recursive submodule; register as workspace member | claude-code | 7 |
| 9 | `configure-entity-engine-and-realtime-bridge` | SPA bootstrap: `configureEngine` + custom `UarRealtimeAdapter` (SyncAdapter) wired to SSE bus from #2 | claude-code | 2, 8 |
| 10 | `migrate-admin-knowledge-to-entity-mgmt` | Pilot migration: KB list, document list, doc-count now driven by `useEntityList` | claude-code | 9, 1 |
| 11 | `migrate-providers-models-agents-to-entity-mgmt` | Rollout: providers, LLM models, agents, skills, settings — all toolbars/dropdowns consume same graph keys | claude-code | 10 |
| 12 | `builtin-skills-ui-affordance` | UI badge + disabled delete for `origin = Builtin`; filter chip for Built-in/User | claude-code | 3, 11 |
| 13 | `dockerfile-multistage-with-submodules` | Multi-stage Dockerfile vendoring both submodules; volumes for derived/user skills + HF cache | claude-code | 5, 8 |
| 14 | `integration-tests-and-docs` | Contract tests for live-query latency, entity rehydration, builtin-delete 409, docker build green | claude-code | all prior |

---

## Per-change synopsis

### 1. `fix-kb-document-count`
Add `document_count: usize` to `KnowledgeBaseResponse` (and DB-layer aggregate query). Both providers (surreal, postgres). Frontend already reads `kb.document_count ?? 0` so the UI requires no changes. Acceptance: A1, A2.

### 2. `surreal-live-query-bus`
- New module `src/uar/realtime/` housing `LiveQueryBus` (tokio broadcast per topic).
- On startup, open `.select().live()` streams against: `knowledge_bases`, `knowledge_documents`, `agents`, `providers`, `models`, `skills`, `settings`.
- Each stream task forwards `Action::{Create,Update,Delete}` + record id + record body into the bus.
- Public SSE endpoint `GET /api/live/{entity_type}` (auth-gated) tails the broadcast; events shaped as `{ "action": "create"|"update"|"delete", "id": "...", "data": {...} }`.
- Postgres backend (when re-enabled) implements the same `LiveQueryBus` trait via `LISTEN/NOTIFY` triggers on the same tables.
- Tests: write a row, assert SSE notification within 200ms.

### 3. `add-skill-kind-and-origin`
- Extend `Skill` (`src/uar/domain/skills.rs`):
  ```rust
  pub enum SkillKind { Manifest, Wasm, Native }
  pub enum SkillOrigin { Builtin, User }
  ```
- Add fields to `Skill { kind: SkillKind, origin: SkillOrigin }`. Default for legacy rows: `kind=Native`, `origin=User`.
- Persistence migration (surreal + postgres) backfills.
- DELETE handler in `SkillService` returns `409 Conflict` for `origin = Builtin`.
- No UI affordance yet (change #12 handles that).

### 4. `binary-instance-discovery`
- New module `src/uar/orchestrator/process_supervisor.rs`.
- For each managed child binary (currently: any MCP stdio servers, `surreal-memory-server` if we ever spawn it standalone, `liter-llm` proxy, `forge` MCP enrichment server) define a `ManagedBinary { name, expected_port, health_url, pidfile_path }`.
- Before spawn: TCP probe → if port open, hit `health_url`; if 200, mark as **adopted** (don't spawn). Otherwise spawn and write a pidfile under `$XDG_RUNTIME_DIR/uar/<name>.pid` (fallback `~/.uar/run/`).
- On shutdown, signal only owned processes (those we spawned), not adopted ones.
- Reuses adopted endpoints via env var injection (e.g. `UAR_MEMORY_MCP_URL`).

### 5. `add-skill-system-submodule`
- `git submodule add git@github.com:Prometheus-AGS/prometheus-skill-system.git crates/prometheus-skill-system` then `git submodule update --init --recursive`.
- New `src/uar/runtime/skills/builtin_loader.rs` walks `crates/prometheus-skill-system/skills/<domain>/<name>/SKILL.md`, parses YAML frontmatter with `serde_yaml`, builds `Skill { kind: Manifest, origin: Builtin, … }`.
- Loader runs at startup, registers all builtins in `SkillService::Local Skills` provider (or a new dedicated `BuiltinSkillsProvider`).
- Logs `Loaded N builtin manifest skills from crates/prometheus-skill-system/skills`.

### 6. `wasm-component-skill-runtime`
- New module `src/uar/runtime/skills/wasm_runtime.rs`.
- Define `wit/uar-skill.wit` with world `uar:skill@0.1.0` exporting `run(input: string) -> result<string, string>` (concrete API to be finalized during implementation — keep room for streaming).
- Use `wasmtime::component::Linker` to instantiate `.wasm` components from `~/.uar/skills/wasm/<name>.wasm` and `crates/prometheus-skill-system/skills/<...>/skill.wasm` (if any author chooses to ship binaries).
- `Skill { kind: Wasm }` dispatch routes through this runtime.
- Behind the existing `wasm-runtime` Cargo feature (already on by default in our release build).

### 7. `frontend-pnpm-workspace-migration`
- Create `pnpm-workspace.yaml` at repo root listing `frontend` and `frontend/packages/*`.
- Convert `frontend/package.json` to a workspace member; pick `bun` or `pnpm` as the script runner — we standardize on `pnpm` per locked decision.
- Update `build.rs` to invoke `pnpm install --frozen-lockfile && pnpm --filter ./frontend build` (replacing the current `bun run build`).
- Document in `AGENTS.md` + `CLAUDE.md`.

### 8. `add-entity-management-submodule`
- `git submodule add git@github.com:Prometheus-AGS/prometheus-entity-management.git frontend/packages/prometheus-entity-management`.
- Frontend `package.json` adds `"@prometheus-ags/prometheus-entity-management": "workspace:*"`.
- Build the package once via the workspace (`pnpm --filter prometheus-entity-management build`).
- Verify Vite resolves the workspace import.

### 9. `configure-entity-engine-and-realtime-bridge`
- New `frontend/src/lib/entity-engine.ts` calling `configureEngine({ defaultStaleTime: 30_000, gcInterval: 60_000, revalidateOnFocus: true, … })` at bootstrap.
- New `frontend/src/lib/realtime/uar-realtime-adapter.ts` — implements the lib's SyncAdapter contract by subscribing to `EventSource(`/api/live/${entityType}`)` (from change #2) and applying `Action::Create/Update/Delete` to the graph via `upsertEntity` / `removeEntity` keyed by `(type, id)`.
- Register the adapter for every entity type the SPA tracks.
- Cross-view propagation guaranteed: any view that reads `useEntity("provider", id)` will re-render when the bus publishes an update.
- This is the change that **kills stale-data-anywhere**.

### 10. `migrate-admin-knowledge-to-entity-mgmt`
- Replace `knowledge-page.tsx` ad-hoc fetcher with `useEntityList("knowledge_base")`.
- Per-KB detail uses `useEntityList("knowledge_document", { kbId })`. The displayed count becomes `list.items.length` — change #1's aggregate is the first-paint fallback before the list resolves.
- Verify upload → indexing → count update happens **without a page refresh** end-to-end (live bus drives it).

### 11. `migrate-providers-models-agents-to-entity-mgmt`
- Provider list, LLM model catalog, Agents page, Skills page, Settings — all moved off bespoke fetchers onto entity hooks.
- Header chips (current agent, current model) read from the same graph entries.
- Acceptance criterion: editing a provider in Admin updates the chat-header model badge immediately, no refresh, no re-fetch.

### 12. `builtin-skills-ui-affordance`
- Skill row shows a Built-in badge when `origin === "Builtin"`.
- Trash icon disabled with tooltip "System skill — cannot be removed" for built-ins.
- Filter chips at top of Skills page: All / Built-in / User.

### 13. `dockerfile-multistage-with-submodules`
**The runtime image doubles as a polyglot build host** so skills/plugins authored in any supported language can be compiled in-container.

Required toolchain (all stages, including runtime):
- **Rust nightly** + `wasm32-wasip2`/`wasm32-wasip1`/`wasm32-unknown-unknown` targets + `cargo-component`.
- **Node.js LTS ≥ 24** with `npm`, `pnpm` (via corepack), and `bun` all present.
- **Python 3.13** with `uv` and `maturin`/`pyo3-build-config` pre-installed.
- **Go (latest)**.
- **wasmtime CLI** for AOT compilation; **Cranelift** is the wasmtime backend (already linked through the `wasmtime` crate at runtime for JIT).

Build stages:
- Stage 1 — `toolchain` (base = ubuntu:24.04): install all five toolchains + `wasmtime`.
- Stage 2 — `builder`: `git submodule update --init --recursive`, `pnpm install --frozen-lockfile && pnpm --filter ./frontend build`, `cargo +nightly build --release --features "memory-palace,wasm-runtime,surreal-memory/embedded"`, AOT-precompile any WASM components shipped in `crates/prometheus-skill-system/skills/**/skill.wasm` via `wasmtime compile -o .cwasm`.
- Stage 3 — `runtime` (FROM toolchain): copy artifacts; keep all build tools resident so the container can also build user-supplied skills at runtime.

Declared volumes:
- `/var/lib/uar/skills-derived/` (writable, derivative artifacts)
- `/var/lib/uar/skills-user/` (user-installed WASM components + `SKILL.md`)
- `/var/lib/uar/cache/huggingface/` (HF model cache)
- `/var/lib/uar/cache/cargo/` (Cargo registry/build cache for in-container builds)
- `/var/lib/uar/cache/pnpm/` (pnpm store)
- `/var/lib/uar/data/` (SurrealKV if running embedded)

Update `docker-compose.prod.yaml` with named volumes + bind-mount option for dev.

### 14. `integration-tests-and-docs`
- Backend: live-bus latency, builtin delete 409, document_count correctness.
- Frontend: contract test that `EventSource` mock drives `useEntity` re-render.
- Container: GH Action runs `docker build` against the multi-stage file.
- Docs: README architecture diagram updated; `AGENTS.md` + `CLAUDE.md` describe the pnpm workspace + skill model + realtime contract.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Live-query streams stall under load (Surreal known to drop streams on connection loss) | Wrap each stream in supervised reconnect loop with exponential backoff; expose stream-health gauge |
| `pnpm install` breaks CI that currently uses `bun` | Keep `bun` as runtime for ancillary scripts; migrate CI step alongside change #7 |
| WIT world churn — early skill authors get stuck | Pin the WIT version at `uar:skill@0.1.0`; document forward-compat policy (additive only until 1.0) |
| Recursive submodule clones in Docker double build time | Use `--depth 1` clones + a layered cache; submodule init runs before `cargo fetch` so it's cacheable |
| Adopted child binaries with mismatched versions cause subtle bugs | The supervisor records the version it adopted; emit a structured warn log if version differs from the version we'd have spawned |
| Migrating Admin pages all at once breaks user flows | Pilot Knowledge page (#10) first; gate the rest behind #10 passing manual QA |

---

## Sources

- [Live Queries in Rust — SurrealDB SDK docs](https://surrealdb.com/docs/sdk/rust/concepts/live)
- [select_live method — SurrealDB Rust SDK](https://surrealdb.com/docs/languages/rust/methods/select-live)
- [wasmtime::component API docs](https://docs.wasmtime.dev/api/wasmtime/component/index.html)
- [Building a Plugin System with the WebAssembly Component Model — Sy Brand](https://tartanllama.xyz/posts/wasm-plugins/)
- [WASI 0.2 and the Component Model status — eunomia](https://eunomia.dev/blog/2025/02/16/wasi-and-the-webassembly-component-model-current-status/)
- [PGlite + ElectricSQL realtime sync — context for entity-mgmt realtime adapter design](https://pglite.dev/docs/sync)

---

## Progress signal

Completed kbd-plan — prometheus-package-integration
