---
sidebar_position: 1
title: Introduction
---

# Universal Agent Runtime

Universal Agent Runtime (UAR) is a Rust/Axum runtime for governed agent execution, typed streaming, model routing, tools, retrieval, and declarative agent UI. The first-party interface is React 19 + TypeScript.

UAR is version **0.1.0**. The minimal web/server bundle is Stable under the project's evidence policy; server-full, desktop, WASM, mobile, and provider capabilities have narrower statuses. The [product support matrix](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/product-support-matrix.md) is the public release contract.

## Core contracts

| Area | Contract |
|---|---|
| Runtime | Axum REST and SSE APIs listen on configurable port `1906` by default. |
| Frontend | React components call hooks, hooks expose stores, stores call typed services. |
| Streaming | AG-UI is the normalized event transport vocabulary. |
| Artifacts | A2UI validates declarative artifacts and renders an approved React component catalog; it executes no model-provided code. |
| Persistence | SurrealDB is the Stable server authority; PGlite is a local cache reconciled through versioned events. |
| Providers | The committed 269-provider catalog is discovery metadata. Execution support is certified in three evidence tiers. |
| Tools | MCP-discovered and native tools share schema, Cedar policy, approval, hard-deny, and audit controls. WASM tools are opt-in Preview. |

Catalog, availability, and policy routing are Stable. Adaptive learned routing is Experimental. Web is Stable, Tauri desktop and native WASM are Preview, mobile is Experimental, and browser-side arbitrary WASM execution is unsupported.

## Architecture

```text
React -> hooks -> stores/entity graph -> services -> Axum REST/SSE
                                                     |
                    SurrealDB <- runtime/orchestrator -> providers/tools
                        ^                 |
                        `-- versioned events -> PGlite cache
```

## Next steps

- [Installation](./installation)
- [Configuration](./configuration)
- [API reference](./api-reference)
- [Backup and restore](./backup-and-restore)
- [Upgrade guide](./upgrade-guide)
- [Troubleshooting](./troubleshooting)

## Licensing

UAR is licensed under `AGPL-3.0-only`; separate commercial terms are available for AGPL-incompatible usage. See the repository's `LICENSE` and `LICENSE-COMMERCIAL.md`.
