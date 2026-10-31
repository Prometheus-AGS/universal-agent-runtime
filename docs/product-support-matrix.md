# Product Support Matrix

This is UAR's public release contract. The canonical, linted data is
[`product-support-matrix.json`](product-support-matrix.json); its schema is
[`product-support-matrix.schema.json`](product-support-matrix.schema.json).
“Cataloged” never means “certified.” Maturity labels and historical gate names do not certify a new source tree or artifact. The matrix contract revision remains `1.0.0`; the Rust product is **1.4.0-rc.1**, and SDK/package/schema versions are independent. `verified_at` retains the historical inventory date; the October 31 claim review is not a verification date.

The first selected production contract is **Linux amd64 container APIs**, with SurrealDB 3.3.0 default persistence and PostgreSQL 18’s explicit restrictions. Its actual image/source/configuration and applicable API/recovery rows remain **unverified** pending #465. Other platforms/native archives and WASM/AOT execution are not promoted by this selection. #462’s library baseline on `5aa36a40` is exact-source developer evidence, not container acceptance.

## Release bundles

| Bundle | Status | Feature set |
|---|---|---|
| Minimal headless | Stable | `minimal` → `surreal-backend` (Cargo default; not distributed as the customer archive) |
| Server full | Preview, candidate qualification pending | Authoritative server/sidecar distribution: Minimal + local models, Cedar, quality, documents, telemetry, A2A, API docs, admin UI, WASM |
| Desktop full | Preview | `desktop-full` → Server Full + `tauri` |
| Server + Postgres | Preview | `postgres-backend` (implies `sqlx`) |
| In-memory development | Development only | `in-memory-backend` without defaults |

The unintegrated `memory-palace` extension is not a UAR feature. Model
regeneration is an explicit maintainer operation (`cargo run --manifest-path
tools/uar-model-builder/Cargo.toml`) and is excluded from release bundles.
Native archives and the container package `server-full` together with the built
React assets. The `minimal` profile exists for dependency-constrained headless
deployments and compile-time boundary verification.

## Provider tiers

| Tier | Promise | Evidence |
|---|---|---|
| 1 | Named capability evidence on its recorded source/configuration | Historical named OpenAI chat/stream/tools/structured and Anthropic chat/stream/tools records; local FastEmbed records. No blanket current-artifact or vision certification |
| 2 | Native registry and basic chat/stream compatibility | Registry resolution and provider health tests |
| 3 | Catalog metadata only | Schema/catalog validation; no execution promise |

The compiled provider catalog is discovery metadata. Counts are not certified provider totals. Exact capability scope, historical dates and current qualification limits live in the JSON contract.

The bounded #463/#464 ordinary-chat slice is source-authored and **NOT_EXERCISED**: last-current-user inline PNG/JPEG, at most 8 images, existing 10 MiB whole HTTP envelope, bounded 100 MiB decoder operation, supported generic-compatible adapter only. Remote URLs, image uploads, audio/video and broad multimodal client replay are excluded. Actual mixed/image-only/streaming/history/routing evidence remains pending. Existing direct Anthropic image conversion is implemented but independently unqualified; it must not be described as universally text-only or as certified vision. A deterministic peer proves bytes/dispatch, never image understanding. A real image has no guaranteed fit from JSON token counting alone; unproven strict/finite accounting must refuse. The authored decoder admits two workers without a waiting queue and requests cancellation after ten seconds; it joins running work before returning a timeout, so this is not a ten-second response SLO. The 100 MiB limit is per operation, not process RSS. These controls also remain unexercised.

## Persistence authority

SurrealDB is authoritative in the stable default server bundle. PGlite is the
browser/desktop cache for local threads and messages; runtime entity events
reconcile the reactive graph, and server entity versions win conflicts while
unsent local drafts remain client-owned. PostgreSQL 18/pgvector authority is Preview and requires the Postgres feature gate. [Persistence capabilities](operations/persistence-capabilities.md) defines its explicit unsupported teams/follow-ups/schedules/observers/checkpoint operations. Memory remains an independent service on both persistence lanes; persistence writes refuse with `memory_service_required`.

## Routing and tools

Catalog filtering describes declared metadata. Actual capability/availability/policy dispatch is Preview while #464’s primary/retry/fallback/graph cases remain unexercised on the selected container. A route-query answer is advisory and never grants credentials or destination scope. New admission preserves exact targets, enabled/cooldown state and captured identities; these authored changes are not certified by this prose. Adaptive learned routing is Experimental. MCP-discovered and native tools use the same governed
`execute_tool` action with `Allow`, `RequireApproval`, and non-overridable
`Deny`, plus audit events. WASM tool execution is Preview and disabled unless
`wasm-runtime` is enabled.

## Platforms

Web is Stable. Desktop and native WASM are Preview. Mobile is Experimental.
Browser-side WASM execution is Unsupported; browsers render validated,
declarative A2UI artifacts and do not execute arbitrary artifact code.
Linux amd64 container APIs are the selected qualification lane. Native archives, ARM64, Windows and mobile remain unqualified unless separately selected; they are not mandatory gates for this first contract. Bundled WASM toolchains are not AOT execution evidence (#467). Historical browser/desktop maturity does not qualify their workflows for the new API-only promotion.

## Execution and recovery guarantees

These are source-defined boundaries and retained obligations, **not current-container pass receipts**. #465 binds promoted rows to an exact runtime/image/configuration/store and named upgrade pair. #466 maintains this single summary alongside the existing operational contracts.

| Interface/profile | Acknowledgement, history and recovery boundary | Effects and evidence limit |
|---|---|---|
| Ordinary OpenAI/Anthropic response | HTTP/stream acknowledgement is not universal durable acceptance of every inbound byte. Warm session history is distinct from governed checkpoint continuation; generic client reconnect is not a resumable executor. | Adapter-specific cancellation/usage and mounted embeddings still need #469–471 evidence. New image history remains unexercised. |
| Native runs, AG-UI/A2UI, A2A durable bindings on supported stores | Run IDs/bindings correlate owned execution. Bounded cursors may use active history and durable fallback; missing retained history must be explicit. No unlimited replay promise or arbitrary cross-protocol equivalence. | Event replay does not re-execute tools. A2UI new-run retries may duplicate actions; 64-run restatement limits and #468 residuals remain. Independent pinned-peer/transport evidence stays in #465. |
| Protected checkpoint/approval, durable Surreal only | Atomic protected envelope/claim and readback, current owner/policy/resource admission and single-use continuation apply. Unknown predecessor/effect/admission outcomes prevent reassignment; abrupt recovery requires its separately qualified exclusion mechanism. | [Fresh resume](operations/fresh-process-resume.md), [approval recovery](operations/approval-recovery.md) and [supervisor recovery](operations/supervisor-recovery.md) govern. PostgreSQL/unsupported profiles refuse. Process replacement alone is not trusted exclusion. |
| Full harness, all stores | `process_ephemeral`, `unsupported_after_restart`, `steer_supported:false`; bounded process-owned observations do not become durable from native history. | No full-harness durability rewrite or recovery certification is implied. Expired/old-epoch controls cannot authorize new effects. |
| Teams/instances, Surreal only | Settled artifacts remain evidence; uncertainty remains reserved-unknown and cannot trigger automatic redispatch or silent liability release. PostgreSQL team/observer/schedule capabilities remain unsupported. | Historical C54 recovery is source-bound; control eligibility and unknown accounting disposition remain #475. |
| Memory and upgrades | Memory is an independent service-owned store, not another run-recovery backend. Runtime/schema/wire/SDK versions are distinct. | No universal exactly-once, automatic memory-injection or cross-version execution guarantee. Named migration/rollback and runtime/SDK upgrade pairs remain unqualified until their receipts. |

A source correction or successful build never promotes a recovery feature. Historical failed/blocked/partial receipts, including acknowledged browser loss and unknown-effect liabilities, remain applicable obligations until exact replacement evidence is recorded.

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


The capability/dispatch source slice also restricts a final send to the selected registered model's output ceiling when known, including ordinary fallbacks. It adds this bound by minimum to the existing typed cap and any existing destination budget contract. It does not create a real-image counting contract or certify context fit. The new fallback/public-path outcome remains NOT_EXERCISED until its exact container fixture receipt exists.
