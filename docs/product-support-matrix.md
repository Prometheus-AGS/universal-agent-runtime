# Product Support Matrix

This is UAR's public release contract. The canonical, linted data is
[`product-support-matrix.json`](product-support-matrix.json); its schema is
[`product-support-matrix.schema.json`](product-support-matrix.schema.json).
“Cataloged” never means “certified.” Stable rows have executable gates; Preview
and Experimental rows state narrower expectations.

## Release bundles

| Bundle | Status | Feature set |
|---|---|---|
| Minimal | Stable | `minimal` → `surreal-backend` (default) |
| Server full | Preview | `server-full` → Minimal + local models, Cedar, quality, documents, telemetry, A2A, API docs, admin UI, WASM |
| Desktop full | Preview | `desktop-full` → Server Full + `tauri` |
| Server + Postgres | Preview | `postgres-backend` (implies `sqlx`) |
| In-memory development | Development only | `in-memory-backend` without defaults |

The unintegrated `memory-palace` extension is not a UAR feature. Model
regeneration is an explicit maintainer operation (`cargo run --manifest-path
tools/uar-model-builder/Cargo.toml`) and is excluded from release bundles.

## Provider tiers

| Tier | Promise | Evidence |
|---|---|---|
| 1 | Named, capability-specific execution certification | OpenAI-compatible chat/stream/tools/vision/structured paths; Anthropic chat/stream/tools/vision; local FastEmbed embeddings |
| 2 | Native registry and basic chat/stream compatibility | Registry resolution and provider health tests |
| 3 | Catalog metadata only | Schema/catalog validation; no execution promise |

The embedded 269-provider catalog is discovery metadata, not 269 certified
providers. Exact capabilities, evidence, and last-verified dates live in the
JSON contract.

## Persistence authority

SurrealDB is authoritative in the stable default server bundle. PGlite is the
browser/desktop cache for local threads and messages; runtime entity events
reconcile the reactive graph, and server entity versions win conflicts while
unsent local drafts remain client-owned. PostgreSQL/pgvector authority is
Preview and requires the Postgres feature gate.

## Routing and tools

Catalog, availability, and policy routing are Stable. Adaptive learned routing
is Experimental. MCP-discovered and native tools use the same governed
`execute_tool` action with `Allow`, `RequireApproval`, and non-overridable
`Deny`, plus audit events. WASM tool execution is Preview and disabled unless
`wasm-runtime` is enabled.

## Platforms

Web is Stable. Desktop and native WASM are Preview. Mobile is Experimental.
Browser-side WASM execution is Unsupported; browsers render validated,
declarative A2UI artifacts and do not execute arbitrary artifact code.

## BossFang and Flint architecture

BossFang should currently consume UAR as a supervised local service/sidecar,
starting with the OpenAI-compatible provider URL and adding A2A/AG-UI where
richer task/event semantics are needed. Linking the full monolithic UAR crate
would couple native dependencies, Tokio/process lifecycle, security and crash
domains, and releases. Reconsider an in-process library only after extracting a
narrow dependency-light kernel and profiling proves IPC is material. See the
[BossFang integration guide](librefang-integration.md#6-deployment-decision-library-or-supervised-service).

The sibling ownership contracts are:

- Flint Gate owns external authentication, authorization, and stream enforcement.
- Flint Realtime Fabric owns durable realtime distribution; UAR emits versioned events.
- Flint Forge owns RLS-backed data APIs and edge execution; UAR consumes typed APIs/tools.
- Flint Platform Agent administers UAR and sibling planes through authenticated management APIs.

These are protocol/API boundaries, not invitations to absorb sibling kernels
into UAR. The machine-readable integration rows are required by the matrix
validator.
