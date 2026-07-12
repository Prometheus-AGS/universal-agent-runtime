# Architecture Overview

This document is the canonical system architecture for UAR 0.1.0. Support status is defined only by the [product support matrix](product-support-matrix.md).

## Runtime boundaries

```text
React 19 UI
  -> hooks/view models
    -> stores and entity domains
      -> typed services
        -> Axum REST/SSE transport
          -> RunManager / Orchestrator
             |-> liter-llm provider driver
             |-> governed MCP, native, or WASM tools
             |-> retrieval and document services
             `-> SurrealDB or feature-gated PostgreSQL
```

The default listener is `127.0.0.1:1906`; bind address and port are configurable. Rust/Axum owns authentication, policy, orchestration, persistence authority, and transport. The React application renders state and submits intent; it is not a business-state authority.

## Frontend state and persistence

The mandatory dependency direction is Component -> Hook/View Model -> Store/Entity Domain -> Service -> API. Components do not fetch, import services, or mutate stores directly. Stores own asynchronous state and call thin typed services. See [frontend-architecture.md](frontend-architecture.md).

SurrealDB is authoritative in the Stable default bundle. PGlite persists local threads and messages in supported browser/desktop clients. SSE and realtime adapters normalize versioned server events into entity-graph updates. Server entity versions win reconciliation conflicts; unsent local drafts remain client-owned. PostgreSQL/pgvector authority is Preview behind `postgres-backend`.

## Streaming and rendered artifacts

Provider-specific output is normalized once inside the runtime. Clients may request OpenAI-compatible SSE, AG-UI events, or dual streaming where the endpoint supports it.

- **AG-UI** is a versioned event vocabulary and transport profile for runs, text, tools, state, and lifecycle events.
- **A2UI** is a declarative artifact envelope and rendering profile. The React renderer validates the profile and component schema, then maps data to an allowlisted native component.

AG-UI does not prescribe React components. A2UI does not replace the run/event transport. Model-provided HTML or JavaScript is never executed. See [protocols/ag-ui-profile.md](protocols/ag-ui-profile.md) and [protocols/a2ui-profile.md](protocols/a2ui-profile.md).

## Providers and routing

`liter-llm` supplies the unified provider client. `catalog/provider_catalog.json` is a committed, checksummed snapshot copied into the binary during the build; ordinary builds perform no catalog network fetch. Maintainers refresh it explicitly with `scripts/refresh-provider-catalog.sh`.

The catalog currently describes 269 providers, but metadata presence is not execution certification. Provider promises are tiered:

1. Tier 1: named capability-specific execution evidence.
2. Tier 2: native registry and basic chat/stream compatibility.
3. Tier 3: catalog/schema metadata only.

Routing likewise has explicit levels: catalog filtering, provider availability, and policy constraints are Stable; adaptive learned routing is Experimental. `POST /api/uar/route` returns the best available model that satisfies declared requirements, not a guarantee that every catalog model implements every capability.

## Tools and security

MCP-discovered and native tools enter the same governed `execute_tool` path. Inputs are schema-validated, decisions are audited, and policy yields `Allow`, `RequireApproval`, or non-overridable `Deny`. WASM tools are Preview, disabled unless `wasm-runtime` is enabled, and execute behind WASI capability boundaries. Browser-side arbitrary WASM execution is unsupported.

Edge authentication and authorization belong at Flint Gate when deployed in the Flint platform; UAR still enforces its own runtime/tool policies. Credentials remain server-side and must never be returned through provider configuration APIs or logged.

## Deployment topology

The Stable platform is web/server. Tauri desktop and native WASM are Preview; mobile is Experimental. These targets do not promise identical implementations or certification.

BossFang should currently supervise UAR as an out-of-process local service. This preserves fault, security, dependency, and release isolation. An in-process API becomes sensible only after extracting a small runtime kernel and measuring IPC as a real bottleneck. Flint service ownership is recorded in the [support matrix](product-support-matrix.md#bossfang-and-flint-architecture).

## Testing and release evidence

Verification is layered rather than represented by a blanket production-readiness claim:

- Rust unit/integration gates validate the selected feature bundle.
- Vitest validates React stores, hooks, services, entities, and components.
- Playwright and BDD suites validate supported user flows and streaming behavior.
- `scripts/check-frontend-boundaries.mjs` enforces frontend dependency direction.
- support-matrix and documentation validators prevent claims from exceeding evidence.
- offline packaging verifies two isolated builds from the same complete source bundle.

Run the exact gate listed for a feature or platform in [product-support-matrix.json](product-support-matrix.json). Preview and Experimental rows remain narrower than Stable even when an individual check passes.

## Source layout

| Path | Ownership |
|---|---|
| `src/` | Rust runtime, APIs, orchestration, persistence and policy |
| `frontend/` | React application following strict component/hook/store/service layers |
| `catalog/` | committed provider metadata snapshot and provenance |
| `docs/protocols/` | supported AG-UI and A2UI profiles |
| `tests/` | Rust integration, BDD and browser contracts |
| `scripts/` | architecture, support, docs and release validation |
| `openspec/` | versioned capability and change contracts |
