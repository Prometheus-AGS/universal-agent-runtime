# Universal Agent Runtime

Universal Agent Runtime (UAR) is a Rust/Axum runtime for governed agent execution, model routing, typed streaming, tools, retrieval, and declarative agent UI. Its first-party operator interface is React 19 + TypeScript.

UAR is at version **1.0.0**. The distributed server/sidecar product is the `server-full` bundle; it includes the React application, document intelligence, governance, telemetry, and supported protocol surfaces. The dependency-light `minimal` bundle remains a Stable headless profile, not the packaged customer distribution. See the [product support matrix](docs/product-support-matrix.md) before making deployment commitments.

## What is supported

- OpenAI-compatible and Anthropic execution paths have named Tier 1 capability evidence. Local FastEmbed embeddings are also Tier 1.
- The committed catalog contains metadata for 269 providers. A catalog entry is discovery data, not proof that execution is certified.
- Catalog, availability, and policy routing are Stable. Adaptive learned routing is Experimental.
- MCP-discovered and native tools share schema validation, Cedar policy, approval, hard-deny, and audit controls. Native WASM tools are Preview and opt-in.
- Web is Stable. Desktop/Tauri and native WASM are Preview. Mobile is Experimental. Browser-side arbitrary WASM execution is unsupported.

The machine-readable source of truth is [docs/product-support-matrix.json](docs/product-support-matrix.json).

## Architecture

```text
React component
  -> hook/view model
    -> Zustand store/entity domain
      -> typed service
        -> Axum REST/SSE API on port 1906
          -> run manager/orchestrator
            -> liter-llm / governed tools / persistence
```

The browser consumes normalized runtime events. **AG-UI is the event transport vocabulary**; **A2UI is the validated declarative rendering contract**. A2UI artifacts map to an approved React component catalog and never execute model-provided HTML or JavaScript.

SurrealDB is authoritative in the Stable default server bundle. PGlite is a local browser/desktop cache for threads and messages. Versioned server events reconcile the reactive entity graph; server entity versions win conflicts while unsent drafts remain client-owned.

Read [the system architecture](docs/ARCHITECTURE.md), [frontend ownership rules](docs/frontend-architecture.md), and the [AG-UI](docs/protocols/ag-ui-profile.md) and [A2UI](docs/protocols/a2ui-profile.md) profiles.

## Run locally

Requirements: a current Rust toolchain, Node.js, and pnpm 10.33.0.

```bash
cp .env.example .env
pnpm install --frozen-lockfile
pnpm build
cargo run --bin universal-agent-runtime
```

UAR listens on `127.0.0.1:1906` by default. The port remains configurable through the CLI, environment, or YAML configuration described in [docs/configuration.md](docs/configuration.md).

Useful checks:

```bash
cargo fmt --all -- --check
cargo test --lib --features minimal
pnpm typecheck
pnpm test
pnpm run frontend:boundaries
pnpm run support-matrix:validate
pnpm run docs:validate
```

For a fully disconnected source build, see [docs/build-reproducibility.md](docs/build-reproducibility.md).

## Deployment and integrations

UAR can run as a server, container, or supervised local service. BossFang should currently supervise UAR out of process and use the OpenAI-compatible API first, adding A2A or AG-UI where richer task/event semantics are needed. A linked library should be reconsidered only after a narrow dependency-light kernel is extracted and profiling demonstrates a material IPC bottleneck. The detailed analysis is in the [BossFang integration guide](docs/librefang-integration.md#6-deployment-decision-library-or-supervised-service).

Flint Gate owns edge auth enforcement, Flint Realtime Fabric owns durable realtime distribution, Flint Forge owns RLS-backed data APIs and edge execution, and Flint Platform Agent owns authenticated administration across these services. UAR retains inference, routing, agent execution, and governance ownership.

## Security

Production deployments must configure authentication, non-default secrets, trusted origins, and an explicit tool policy. Tool execution is server-side and auditable; a Cedar `Deny` cannot be overridden by user approval. Never place provider credentials in frontend code or persisted UI state. See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) and [docs/product-support-matrix.md](docs/product-support-matrix.md).

## License

UAR is version 1.0.0 and licensed under `AGPL-3.0-only`. Separate commercial terms are available for AGPL-incompatible use; see [LICENSE](LICENSE) and [LICENSE-COMMERCIAL.md](LICENSE-COMMERCIAL.md).
