# UAR core repository map

Evidence snapshot: 2026-09-24. Paths without a root are relative to the UAR checkout. This describes observed source contracts, not write authority over sibling repositories or live teams in them.

## Roster and routing

The roster is `uar-lead`, `uar-runtime`, `uar-providers`, `uar-trust-tools`, `uar-state`, `uar-console`, and `uar-verifier`. Default activation is lead, one relevant specialist, and verifier. Maximum active agents: four, including the lead. Add a second specialist only for a bounded, independently writable surface.

| Role | Route work here | Observed entrypoint evidence |
|---|---|---|
| uar-lead | Scope, OpenSpec/KBD alignment, shared composition roots and final integration | `src/main.rs:20` process entry calls `server::start_server` at line 97; `Cargo.toml`, `versions.toml`, `.kbd-orchestrator/current-waypoint.json` supply dependency/position context |
| uar-runtime | Runs, turns, actors, threads, checkpoints, skills and compiler execution | `src/uar/runtime/manager.rs:345` RunManager; `src/uar/runtime/thread/kernel.rs:1` captured host contract; `src/uar/runtime/native_skill.rs:56` NativeSkill; `src/uar/compiler/service.rs:29` CompilerService |
| uar-providers | Drivers, routing, model capabilities, streaming and normalization | `src/llm/liter_driver.rs:43` LiterLlmDriver and line 532 LlmDriver implementation; `src/llm/{router,registry,orchestrator}.rs` |
| uar-trust-tools | MCP, tools, sandboxing, credentials, security and governance boundaries | `src/mcp/registry.rs:74` NativeTool and line 960 McpRegistry; `src/uar/tools/mod.rs`; `src/uar/security/mod.rs` |
| uar-state | Durable state, scoped memory, retrieval, session persistence and workflow mirroring | `src/uar/memory/service.rs:30` MemoryService; `src/uar/persistence/mod.rs:47` PersistenceLayer; `src/uar/memory/mod.rs:34` memory MCP exposure |
| uar-console | Frontend entity contracts, console routes and desktop integration | `frontend/src/main.tsx:32` createRoot; `frontend/src/App.tsx:29` AppRoutes; `frontend/src/platform/entities/index.ts` typed domain exports; `src-tauri/src/lib.rs:13` desktop run entry, line 97 server startup |
| uar-verifier | Independent artifact review and completed-change evidence | Existing integration paths: `tests/agent_threads.rs`, `tests/typed_turn_assembly.rs`, `tests/tool_call_protocol.rs`, `tests/skill_activation_runtime.rs`, `tests/embedded_sdk.rs` |

The manifest and each task specify exact writable paths. Shared API/domain files, composition roots and dependency pointers require one assigned writer. Runtime and trust specialists coordinate host/kernel changes; `thread/kernel.rs` assigns policy intersection, admission reservations and durable writes to the trusted thread service. The verifier gets requirements and resulting artifacts without generator reasoning and does not review its own production change.

Frontend entrypoints above were inspected directly. Their component behavior, entity subscriptions and browser behavior are not established by the Rust graph. Frontend tasks must load the repository's UI/entity skill instructions before implementation.

## Graph provenance and limitations

Primary artifact: `compass-out/graph.json`. Inspected before source searches; Compass CLI queries preceded broader text searches. Bounded discovery produced fuzzy candidates and ambiguous results, not complete architecture proofs. Actual source/manifests established the boundaries above.

- Schema: `compass.graph/1`; builder: `0.3.28`.
- Source commit: `ba12845138104d3c8c3b8bca8bc7c5be24004e91`, equal to HEAD when inspected. This does not prove a clean working tree.
- Source tree digest: `sha256:87722a8f7734232baaf13dc655e7cb613853b4f2eadab3e4be6a845761f57cd4`.
- Generation ID: `sha256:670405a638d6c0dfbe32ccc865f167bc2b7fd2dd90ab18be2bb5b84001fef010`.
- 71,048 nodes and 126,259 relationships. All relationship evidence records declare `exact`; that is extraction confidence, not runtime validation.
- 1,813 file records: 1,707 Rust records and 106 excluded records without a language. Status totals: 1,572 extracted, 126 generated, 106 excluded, 6 parse failures, 3 partial.
- 70,711 Rust-language nodes plus 337 route nodes without a language. File records are not equivalent to successfully extracted files.
- 389 Rust files under UAR `src/`. Aggregate totals also include 277 file records under `vendor/git/liter-llm`, 497 under skill-system `tools`, 147 under skill-system `skills`, and 140 under skill-system `substrate`. Vendored/imported duplication must not be counted as independent UAR ownership.
- Material parse failures include `src/server.rs` and `src/uar/api/settings.rs`. Other failures concern Liter PHP/FFI sources; partial records concern Liter WASM and skill-system `crates/prometheus-exec/src/main.rs`.
- Rust-only coverage excludes frontend TypeScript, Node scripts, SKILL.md contracts, harness exports, and manifest/config semantics. Feature selection, dynamic dispatch and actual transport connectivity require separate evidence. Sibling checkouts are not independently indexed by this artifact.

## Actual dependencies and ecosystem connections

### Surreal Memory: direct library integration

`Cargo.toml:341` consumes `vendor/git/surreal-memory-server/crates/surreal-memory`, with default features disabled and local embeddings enabled. `src/uar/memory/mcp_server.rs:3` explicitly describes in-process execution; `src/uar/memory/mod.rs:34-50` describes native tools and optional `/mcp/memory` exposure. Scoped context construction and capture live in `context_builder.rs`, `auto_capture.rs` and `scopes.rs`; workflow mirroring lives in `workflow_mirror.rs`.

The sibling `/Users/gqadonis/Projects/prometheus/surreal-memory-server` is a related development checkout, not the path Cargo currently compiles. `versions.toml` records accepted commit `432eaa1ebbef66fc02b9bb1a1e63cc2fdb2149e8`. Upstream changes and UAR consumption/pointer updates are separate explicit task scopes. Route through `uar-state`, consulting trust/tools for exposure and authorization.

### Liter: direct provider dependency

`Cargo.toml:504` consumes `vendor/git/liter-llm/crates/liter-llm` with the full feature set. `.gitmodules` identifies the vendored Liter checkout. `src/llm/liter_driver.rs` implements the actual UAR adapter; this is more than a configured network endpoint. Route through `uar-providers` and follow authoritative pins.

### Skill system and skill pack: artifact/loader integration

`.gitmodules:4-7` maps `crates/prometheus-skill-system` to the Prometheus skill-system repository; root Cargo excludes it as a workspace member. `src/uar/runtime/skills/pack_detection.rs:7-20,141-166` resolves an explicit override, then `$PROMETHEUS_SKILL_SYSTEM_DIR` or `../prometheus-skill-system`, then installed plugins named `prometheus-skill-pack`, then embedded `crates/prometheus-skill-system/skills`.

The sibling `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack` is not the literal default sibling name. Its presence does not prove UAR selected it. `builtin_loader.rs:259` reads a selected pack's `.mcp.json`; the WASM loader defaults to the embedded skills path. This is a skill/artifact/MCP contract, not proof that the whole skill system links into the runtime binary. Route discovery, provenance and compiler issues through `uar-runtime`; privilege policy through `uar-trust-tools`.

### Rust MCP Filesystem: vendored tool and reference checkout

`.gitmodules:14-16` maps `vendor/git/rust-mcp-filesystem` to the GQAdonis fork. `Cargo.toml:13` excludes it from the root workspace. No direct runtime source/root dependency was found on the examined surfaces. UAR's own native file tools live in `src/uar/tools/` and should not be conflated with this standalone server.

The reference checkout `/Users/gqadonis/Projects/references/rust-mcp-filesystem/Cargo.toml` identifies the Rust MCP Stack upstream. Reference, fork and consuming process remain distinct. Route through `uar-trust-tools`; establish the exact consumer before editing either external checkout.

### Compass: project tooling

`.compass/config.toml` includes `**/*.rs`. The `compass` entries in `.codex/config.toml` and `.mcp.json` launch `/usr/local/bin/compass serve --graph /Users/gqadonis/Projects/prometheus/universal-agent-runtime/compass-out/graph.json --transport stdio`. CLI help verified those flags. No Compass UAR Cargo/source dependency was found on the examined surfaces.

`/Users/gqadonis/Projects/references/compass` is a separate reference/development checkout. Native server tool discovery governs available calls; do not infer tool names from the configuration key. All roles can use authorized read-only discovery; route configuration/transport questions through `uar-trust-tools`.

### The Boss and mini: ecosystem consumers, not established UAR peers

No `universal-agent-runtime` or `universal_agent_runtime` references were found in the examined The Boss `src/`, package manifest and submodule manifest. Actual integrations are in `/Users/gqadonis/Projects/prometheus/the-boss/src/main/services/prometheus/workspaceMcp.ts:87-112`: Compass stdio, Rust Filesystem stdio and managed Surreal Memory SSE. Lines 159-174 install mini skills unless a full pack exists, then invoke Compass project installation. This establishes shared tools, not a UAR HTTP/SDK adapter. Its observed Compass `--engine` argument is not a compatibility assertion about the currently installed binary.

The Boss `.gitmodules:1-3` embeds `resources/prometheus-skills-mini`; `scripts/sync-prometheus-skills.ts:16` uses its skills directory. Mini's package manifest describes a Windows-native process-skill port. No direct UAR dependency was found in the examined UAR manifest/source and mini script/package surfaces.

Treat future cross-team cooperation as a proposal until a real team, destination owner, artifact interface and authorized dispatch exist. Workspace adjacency alone establishes none of those. The lead owns cross-repository scoping; relevant specialists own only the explicitly assigned consumer or upstream surface.
