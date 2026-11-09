# Product Support Matrix

This is UAR's public release contract. The canonical source data is
[`product-support-matrix.json`](product-support-matrix.json); its schema is
[`product-support-matrix.schema.json`](product-support-matrix.schema.json).
“Cataloged” never means “certified.” Maturity labels and historical gate names do not certify a new source tree or artifact. The matrix contract revision remains `1.0.0`; the Rust product is **1.4.0-rc.3**, and SDK/package/schema versions are independent. `verified_at` retains the historical inventory date; the November 2 source claim update is not verification. The [RC3 pilot guide](releases/v1.4.0-rc.3.md) records completed and open qualification separately. Full #462–475 qualification remains in progress.

The first selected production contract is **Linux amd64 container APIs**, with embedded SurrealDB 4.0.0 on owned SurrealKV volumes by default and a restricted PostgreSQL 18 lane. The [embedded upgrade qualification](operations/embedded-surrealdb-4-upgrade.md#qualification-scope) covers a specific candidate and a pinned 3.3.0 migration baseline. Final release image, configuration, and applicable API and recovery qualification remain pending under #465. Other platforms, native archives, and WASM/AOT execution remain outside this contract. Gate 156 records the three-profile clean-source baseline on `a9533ff6`. The [pilot guide](releases/v1.4.0-rc.3.md#evidence-and-remaining-qualification) identifies the tested predecessor and remaining final obligations.

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

## Build and verification features

| Cargo feature | Classification | Scope |
|---|---|---|
| `tract-embeddings` | Internal | CPU-only local embedding backend selected by Intel macOS sidecar packaging. Intel macOS is outside the selected Linux amd64 qualification. |
| `afc-c06-gate` | Development only | Agent Fabric C06 acceptance target. |
| `afc-c07-gate` | Development only | Agent Fabric C07 acceptance target, which includes the C06 feature. |
| `test-probes` | Development only | Test probe executable and integration targets that require it. |
| `bauar-native-admission-gate` | Development only | Separate full-feature native admission acceptance artifact. |

The development-only flags listed here are outside the product bundles.

## Provider tiers

| Tier | Promise | Evidence |
|---|---|---|
| 1 | Named capability evidence on its recorded source/configuration | Historical named OpenAI chat/stream/tools/structured and Anthropic chat/stream/tools records; local FastEmbed records. No blanket current-artifact or vision certification |
| 2 | Native registry and basic chat/stream compatibility | Registry resolution and provider health tests |
| 3 | Catalog metadata only | Schema/catalog validation; no execution promise |

The compiled provider catalog is discovery metadata. Counts are not certified provider totals. Exact capability scope, historical dates and current qualification limits live in the JSON contract.

Provider totals, when reported, must derive from the `providers` map in the
pinned Liter catalog at `vendor/git/liter-llm/crates/liter-llm/schemas/catalog.json`.
This contract asserts no fixed numeric total. A list of certified named
provider/capability/source combinations is separate from that metadata count.

The bounded #463/#464 ordinary-chat slice accepts last-current-user inline PNG/JPEG, at most 8 images, existing 10 MiB whole HTTP envelope, bounded 100 MiB decoder operation, supported generic-compatible adapter only. Remote URLs, image uploads, audio/video and broad multimodal client replay are excluded. Earlier mounted Linux gates observed mixed and image-only transport, streaming, retained history, constrained dispatch, and malformed-image refusal on their recorded images. Existing direct Anthropic image conversion is implemented but independently unqualified; it must not be described as universally text-only or as certified vision. A deterministic peer proves bytes/dispatch, never image understanding. A real image has no guaranteed fit from JSON token counting alone; unproven strict/finite accounting must refuse. The authored decoder admits two workers without a waiting queue and requests cancellation after ten seconds; it joins running work before returning a timeout, so this is not a ten-second response SLO. The 100 MiB limit is per operation, not process RSS.

## Persistence authority

SurrealDB is authoritative in the stable default server bundle. PGlite is the
browser/desktop cache for local threads and messages; runtime entity events
reconcile the reactive graph, and server entity versions win conflicts while
unsent local drafts remain client-owned. PostgreSQL 18/pgvector authority is Preview and requires the Postgres feature gate. [Persistence capabilities](operations/persistence-capabilities.md) defines its explicit unsupported teams/follow-ups/schedules/observers/checkpoint operations. Memory remains an independent service on both persistence lanes; persistence writes refuse with `memory_service_required`.

## Routing and tools

Catalog filtering describes declared metadata. Actual capability, availability and policy dispatch is Preview. Earlier mounted Linux gates exercised selected eligibility, cooldown, inherited fallback and explicit-target refusal. Genuine-provider E04 and full final-artifact qualification remain open. A route-query answer is advisory and never grants credentials or destination scope. New admission preserves exact targets, enabled/cooldown state and captured identities; these authored changes are not certified by this prose. Adaptive learned routing is Experimental. MCP-discovered and native tools use the same governed
`execute_tool` action with `Allow`, `RequireApproval`, and non-overridable
`Deny`, plus audit events. WASM tool execution is Preview and disabled unless
`wasm-runtime` is enabled.

## Platforms

Web is Stable. Desktop and native WASM are Preview. Mobile is Experimental.
Browser-side WASM execution is Unsupported; browsers render validated,
declarative A2UI artifacts and do not execute arbitrary artifact code.
Linux amd64 container APIs are the selected qualification lane. Native archives, ARM64, Windows and mobile remain unqualified unless separately selected; they are not mandatory gates for this first contract. Bundled WASM toolchains are not AOT execution evidence (#467). Historical browser/desktop maturity does not qualify their workflows for the new API-only promotion.

## Execution and recovery guarantees

Contract revision **2026-11-02**. This is the canonical interface table; the JSON
`execution_recovery` entries carry the same source-defined boundaries. Every row
is **unverified for the current whole-set source and selected installed image**.
No reusable session/run ID alone proves crash recovery. The final [bounded
checklist](releases/v1.4.0-final.md) binds promoted rows to exact source, artifact,
configuration/store, protocol peer and a named upgrade pair.

| Interface/profile | Admission and crash boundary | Replay/history and gaps | Effects, retries and operator responsibility |
|---|---|---|---|
| Ordinary OpenAI/Anthropic without stable turn identity | Legacy request acknowledgement is not durable intent acceptance. Stored session history does not guarantee restart continuation. | Response streaming is not an unlimited replay log. Canonical owned session history can disclose unavailable/truncated metadata. | No automatic resend of an uncertain request; adapter cancellation does not settle usage. Protocol/SDK support is route-specific. |
| Native chat with stable turn identity | Owner/session/client-turn/message identity and semantic digest reserve the original run before dispatch on a capable durable store. Lost acknowledgement reconciles that original receipt. | [Chat recovery](operations/chat-turn-recovery.md) returns actual run state or unknown/admission-unconfirmed/unavailable. Canonical typed content, deterministic legacy IDs and bounded provenance remain distinct from live execution. | Same intent returns the same run; changed meaning conflicts. Client persists intent before POST; missing receipt never authorizes uncertain repost. History/approval read projection grants no permission. |
| Native run event APIs | Owner-qualified persisted run records where supported; process lane acknowledgement alone is not universal crash acceptance. | Ordered retained event IDs/cursors, configured retention and durable fallback where available; expired/missing/gapped history is explicit. Replay redelivers observations, never execution. | Original canonical tool/admission/accounting IDs retain uncertainty. Reconnect is not permission to repeat an external effect. |
| AG-UI | Durable owner/agent/thread/wire-run binding on a capable store, with predecessor checks. Same binding reattaches rather than creating a fresh run. | Bounded run projection keeps original profile IDs/sequence; cursor gap/expired history is disclosed/refused. No arbitrary cross-protocol replay equivalence. | Current approval/host policy remains authoritative. Independently pinned-client conformance is still unverified; optional features are not inferred from the protocol name. |
| A2UI logical actions | Updated clients persist interaction identity. Exact owner/thread/generation/Button/action/digest receipt reserves the original continuation before dispatch. | [Action recovery](operations/a2ui-interaction-recovery.md) reads original intent. Up to64 prior runs/48 surfaces; expired/truncated/unavailable authority refuses new action. | Same logical intent/new wire ID reconciles; changed payload conflicts. Legacy no-ID requests are `retry_protected:false`. Uncertain creator never redispatches. Single-host publication lock is not a multiwriter guarantee. |
| A2A | Exact owned task binding on a capable store; advertised Agent Card capability and selected transport govern admission. | Read/reconnect exact task only within advertised transport/retention. Explicit gaps and old epochs cannot become new execution authority. | Cancel intent is not confirmed cleanup. Unknown effects/usage remain original liabilities. Independent pinned-client/optional-feature conformance remains unverified. |
| MCP client/tool bridge | Run-owned resource, exact tool/schema/principal plus current Cedar/host approval before effect. Client tools do not imply a full MCP-server product. | Canonical tool/elicitation receipts have their own owner/retention. Transport reconnect does not imply automatic tool resume. | Only supported protected continuation/current policy can proceed. Unknown outcome needs original authoritative receipt. Independent counterpart/version/transport results remain unverified. |
| Protected checkpoint/approval, Surreal only | Atomic protected envelope/claim/readback and current owner/policy/resources. Abrupt replacement requires trusted execution exclusion; process restart alone does not prove it. | Protected retention differs from public history; single-use continuation and incompatible/missing state refuse. | Unknown predecessor/admission/effects block takeover; no fabricated approval. [Fresh resume](operations/fresh-process-resume.md), [approval](operations/approval-recovery.md), [supervisor](operations/supervisor-recovery.md). PG unsupported. |
| Full harness, all stores | `process_ephemeral`; restart recovery is `unsupported_after_restart`; `steer_supported:false`. | Bounded process/runtime-epoch cursors only. Durable native sessions do not promote harness history. | Expired/old-epoch controls cannot authorize effects. No persistence redesign or restart certification is implied. |
| Teams/instances, Surreal only | Catalog-owned original attempt/root/fence/member/binding/reservation identities. Settled outputs remain immutable across recovery. | Original attempts/artifacts/control receipts retain their scope and epochs; unknown effects never redispatch. | [Manual reconciliation](operations/team-control-reconciliation.md) uses exact trusted provider/effect evidence. Generic provider settlement does not clear team capacity: `automaticTeamProjection:false`. Gates 154 and 157 exercised C36 and C54 on the pre-bump Linux image from `a9533ff6`. PG teams/follow-ups/schedules/observers unsupported. |
| Independent memory service | Complete owner, current version/deletion/TTL and operator-configured finite cosine cutoff. Missing cutoff refuses automatic injection; tools remain independent. | [Injection provenance](operations/memory-injection-provenance.md) distinguishes preparation/exclusion/unavailability from final provider submission. Memory records do not resume runs. | C39 covers historical tool lifecycle. Earlier mounted synthetic cases separately observed final provider input and exclusion. Fresh real-model recall remains unverified. No ambient team memory or lexical fallback. |

Runtime, schema, protocol and SDK versions have separate identities. No row
promises arbitrary cross-version continuation: the selected upgrade pair must
exercise migration/refusal, backup/restore and rollback with current owner,
epoch and capability rules. Retention exhaustion or incompatible state is a
refusal/recovery obligation, never implicit permission to retry effects.

A source correction or successful build never promotes a recovery feature.
Historical failed/blocked/partial receipts retain their original source/artifact/
profile and denominator; corrected prose is not replacement evidence. Lost
results and unknown effects remain explicit until authoritative disposition.

## Version, telemetry and publication evidence

`Cargo.toml` defines the Rust product at `1.4.0-rc.3`; `versions.toml` defines
dependency decisions/pins. The JSON `version_inventory` records actual independent
packages: private root tooling/frontend/SDKs `1.0.0`, documentation package
`0.0.0`, desktop shell `0.1.0`. These are not product support or compatibility
certifications. Do not force unrelated package releases into the Rust version.

[Telemetry ownership](operations/telemetry-ownership.md) preserves host-owned
subscriber/recorder globals, reports unavailable scrape/export states and does
not treat exporter attachment as delivery or a hard flush guarantee. Runtime
capacity, reliability distributions and comparative quality are unverified;
[comparison scope is NONE](operations/runtime-measurement-scope.md), including
Rig. Helper benchmarks, deterministic fixtures and real-model outcomes retain
distinct methodology, durability and glue-overhead limits.

The checked-in deployment workflow builds/publishes container images and
sidecars as deployment execution. Repository policy keeps routine product tests,
lint/type/security checks and release certification local. Existing local
checksum/SBOM/provenance/signing scripts describe a procedure, not evidence that
the selected downloadable artifact carries authentic signatures. Publication,
installed execution and signed-artifact verification are separate final results.

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


The capability/dispatch source slice also restricts a final send to the selected registered model's output ceiling when known, including ordinary fallbacks. It adds this bound by minimum to the existing typed cap and any existing destination budget contract. It does not create a real-image counting contract or certify context fit. Earlier mounted fallback receipts retain their original image and fixture identities. Final certification remains separate.


### rc.2 integration evidence boundary — 2026-11-01

This readiness candidate incorporates committed rc.2 source from af37c2f834f7b34f511ecc93f408e80205c94229. Imported macOS receipts retain their original source/configuration/artifact scope; they do not qualify the current integrated tree or Linux amd64 container lane. The optional loopback owner remains disabled by default and is not enabled for the trusted gateway fixture. Compiler owner migration is preserved; applying and verifying it against the selected store is unverified. Existing restricted PostgreSQL and process-ephemeral FullHarness recovery claims remain unchanged.
