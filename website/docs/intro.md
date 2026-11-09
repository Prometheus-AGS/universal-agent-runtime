---
sidebar_position: 1
title: Introduction
---

# Universal Agent Runtime

Universal Agent Runtime (UAR) is a Rust/Axum runtime for governed agent execution, typed streaming, model routing, tools, retrieval, and declarative agent UI. The first-party interface is React 19 + TypeScript.

The Rust product is **1.4.0-rc.3**, a release candidate. SDK/package/schema versions
are independent; private root tooling/frontend/SDK packages currently retain
1.0.0, the documentation package 0.0.0 and desktop shell 0.1.0. The distributed
server/sidecar product is `server-full`; the dependency-light `minimal` profile
is separate. The [support matrix](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/product-support-matrix.md)
is the public contract. The first selected lane is Linux amd64 container APIs
behind a trusted host or gateway, with owned-volume embedded SurrealDB 4.0.0
on SurrealKV by default and a restricted PostgreSQL 18 lane. Use the
[RC3 user guide](./release-candidate.md) for pilot setup, supported workflows and
artifact-specific evidence. Whole-release qualification remains open.

## Core contracts

| Area | Contract |
|---|---|
| Runtime | Axum REST and SSE APIs listen on configurable port `1906` by default. |
| Frontend | React components call hooks, hooks expose stores, stores call typed services. |
| Streaming | AG-UI is the normalized event transport vocabulary. |
| Artifacts | A2UI validates declarative artifacts and renders an approved React component catalog; it executes no model-provided code. |
| Persistence | SurrealDB is the Stable server authority; PGlite is a local cache reconciled through versioned events. |
| Providers | The pinned Liter provider map is discovery metadata; derive counts from that map. Historical named capability evidence and current qualification are separate. |
| Tools | MCP-discovered and native tools share schema, Cedar policy, approval, hard-deny, and audit controls. WASM tools are opt-in Preview. |

Catalog filtering describes discovery. The runtime applies capability, policy and
budget checks to actual dispatch. The RC3 user guide records the scope of the
controlled dispatch evidence. Adaptive learned routing
is Experimental. Historical Web maturity does not qualify current browser
workflows; desktop/native WASM, mobile and other architectures are not selected
for this first API promotion. Browser arbitrary WASM execution is unsupported.

Use the [canonical interface recovery table](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/product-support-matrix.md#execution-and-recovery-guarantees)
and [finite final checklist](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/releases/v1.4.0-final.md).
Full harness remains `process_ephemeral`, `unsupported_after_restart`,
`steer_supported:false`. Independent memory selection and team reconciliation
have their own owner, relevance and uncertainty limits; no global exactly-once
effects or automatic cross-version recovery is claimed. The selected synthetic
baseline uses a zero-error threshold. Capacity, latency targets and runtime
comparisons remain outside that evidence.

## Architecture

```text
React -> hooks -> stores/entity graph -> services -> Axum REST/SSE
                                                     |
                    SurrealDB <- runtime/orchestrator -> providers/tools
                        ^                 |
                        `-- versioned events -> PGlite cache
```

## Next steps

- [Use the RC3 pilot](./release-candidate.md)
- [Installation](./installation)
- [Configuration](./configuration/intro.md)
- [API reference](./api-reference)
- [Backup and restore](./backup-and-restore)
- [Upgrade guide](./upgrade-guide)
- [Troubleshooting](./troubleshooting)

## Licensing

All code — the runtime server and the SDKs under `sdks/` — is `MIT`; documentation is `CC-BY-4.0`. There is no commercial license to buy. See the repository's `LICENSE` and `LICENSE-CC-BY-4.0.md`.
