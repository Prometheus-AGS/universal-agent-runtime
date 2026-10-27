# Repair009 feature-boundary source plan

## Implementation handoff — supersedes original read-only scope below

The lead accepted this plan and explicitly assigned source-only implementation
of its ten listed source files. Ownership is now released back to the lead.
The ten source edits in the table below are authored as planned: parent
HTTP-module feature gates, shared module declaration, new shared policy module,
stable explicit API reexports, and the six internal caller retargets. There were
no dependency, manifest, lockfile, DTO, handler, server payload, or memory edits.
The eight symbol bodies/literals and environment decisions were moved unchanged.
The only new source prose is the shared module's purpose documentation.

Reconstructible complete patch:
`.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-repair-009-api.patch`.
Exact per-file before/after SHA256 byte receipts:
`.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-repair-009-api-source-identity.json`.
The new shared file records a null before hash. Both artifacts were captured by
the implementation writer while performing the authorized edit; they are source
identity receipts, not executed checks or acceptance. They allow lead-owned
source009 inventory reconciliation without relying on a moving checkout.

Commands since planning: one Python source transformation captured original bytes,
performed these exact edits, wrote the complete unified patch and identity receipt,
and printed the hashes. One failed apply_patch could not match append context
and changed nothing; this subsequent apply_patch records the handoff. No test,
lint, format, compiler, build, source-review gate, import probe, git diff check,
service, or provider command ran. Operator gate remains CLOSED. No new runtime
guard was added; the two cfg attributes trace directly to the retained
Axum/server-only dependency failures. No unrequested product behavior was added.
The separate memory repair, canonical reconciliation, full-phase inventory, and
eventual operator-authorized compilation/runtime acceptance remain outstanding.

## Scope and authority

Read-only architecture/source assessment; no production edit or verification.
Phase: `uar-post-lifecycle-verification-readiness`, stage
`user-outcome-implementation`. Operator verification gate: **CLOSED**.
No compiler, build, test, format, lint, service, provider, review gate, or
`git diff --check` was run or delegated. Commands used only `pwd`, `cat`,
`sed`, `rg`, and `head` for source/log inspection; this document is the only
authorized write. Other writers share the checkout; preserve their edits.

Waypoint read at sourceRevision497 / planRevision133 / implementation144 of168.
Its exactNextCommand still says stopped P0.10 and discuss repair versus scope
amendment. The lead's task packet reports the operator chose immediate repair009;
the lead must reconcile that choice into canonical phase state. This plan does
not open the gate or supersede the waypoint.

Loaded Rust workspace routing, rust-router, Rust best practices, m11 ecosystem,
the Rust project rules, versions.toml, and gotchas. Applied guidance: keep Cargo
features additive; transport adapters cannot be dependencies of shared execution
policy; preserve public paths through explicit reexports; do not add a dependency
to make an optional adapter appear available. No upstream API claim is needed.

## Observed problem

Retained `target/uar-user-outcomes-005/artifacts/logs/rust_sdk.stderr` identifies
the prior SDK build's63 errors. This is historical failure evidence, not a new
execution result. Three source-boundary groups are visible:

1. `src/uar/api/mod.rs` gates almost all Axum adapters on `server`, but leaves
   `delegation_grants` and `collaboration` unconditional. Their implementations
   import Axum and server-only security modules; delegation additionally exposes
   `Router<crate::AppState>`, whose state exists only under `server`.
2. Shared compiler and run-manager production code calls execution flags in
   `uar::api::capabilities`, which is correctly server-gated. The flags are pure
   constants and std environment readers; the module around them is HTTP-specific.
3. Memory service/scoped storage leak optional SurrealDB types. They are a
   separate writer's required repair; solving groups1–2 alone cannot establish
   host-persistence or embedded-mobile compilation.

`Cargo.toml` declares host-persistence with no dependencies, embedded-mobile as
host-persistence, minimal as server + surreal-backend, and server-full includes
minimal. SDK embedded selects runtime host-persistence; its dev dependency adds
in-memory-backend. Neither existing embedded feature should gain server, Axum,
or the built-in SurrealDB backend as a workaround.

## Minimum implementation packet

One feature-boundary writer owns the following files exclusively:

| File | Required production edit |
| --- | --- |
| `src/uar/api/mod.rs` | Add `#[cfg(feature = "server")]` to the two existing `delegation_grants` and `collaboration` declarations. Parent gating excludes their private children together. |
| `src/uar/domain/mod.rs` | Declare a new shared `execution_capabilities` module. |
| `src/uar/domain/execution_capabilities.rs` (new proposed file) | Move the eight constants/functions below unchanged, with source-purpose documentation. No Axum, security adapter, or database imports. |
| `src/uar/api/capabilities.rs` | Replace those definitions with explicit public reexports from the shared module, preserving all old server-facing public paths. Leave HTTP DTOs, vocabulary arrays, handlers, and capability-response assembly here. |
| `src/uar/compiler/collaboration/validation/projection.rs` | Retarget execution-flag references to shared domain module. |
| `src/uar/compiler/collaboration/team_execution/resolution.rs` | Same. |
| `src/uar/compiler/collaboration/workflow_execution/mod.rs` | Same. |
| `src/uar/compiler/collaboration/workflow_execution/progression.rs` | Same. |
| `src/uar/compiler/collaboration/workflow_execution/start.rs` | Same. |
| `src/uar/runtime/manager.rs` | Retarget its two team-execution-enabled calls only. |

The eight moved symbols are `WORKFLOW_EXECUTION_QUALIFIED`,
`workflow_execution_enabled`, `TEAM_EXECUTION_PROFILE`,
`TEAM_EXECUTION_B_CAPABILITIES`, `TEAM_EXECUTION_B_QUALIFIED`,
`TEAM_EXECUTION_B_QUALIFICATION_RECEIPT`, `team_execution_profile_stage`, and
`team_execution_b_enabled`. Preserve literal values, receipt, environment variable
names, and every branch exactly. Keep `WorkflowExecutionCapability` DTO server-side:
no shared caller requires it. Keep AGUI profile constants server-side for the same
reason. This avoids moving unrelated transport response structures.

Use full shared paths or narrow imports consistently with each edited file.
No edits required to server.rs, the HTTP child adapters, SDK feature declarations,
Cargo.toml, Cargo.lock, security modules, or server capability payloads. Existing
server code may retain old API paths because explicit reexports preserve them.

## Consumer assessment before gating

Repository source searches covered `src`, `sdks`, and `tests`, plus the broader
non-vendor Rust source tree. `delegation_grants.rs` publicly exposes only
`build_router`; `openapi::extend_openapi` is crate-visible. Its callers are
`src/server.rs` and server-gated `api/openapi.rs` respectively.
`collaboration.rs` publicly exposes `CollaborationApiState` and `build_router`;
their external-module production consumer is `src/server.rs`. Its HTTP child
modules are private. No SDK source or example imports either API module.
Server composition is already gated in `src/lib.rs`.

Shared execution-flag consumers are exactly the six compiler/runtime files listed
above. Server.rs, HTTP collaboration handlers, and server-only mcp_server.rs use
the API capability surface; they can continue through reexports. No inspected
SDK consumer requires capabilities response DTOs on embedded profiles.

Unknown out-of-repository users cannot be proven absent. Gating these router types
makes their availability match their already mandatory server-only dependencies;
this is the uncomfortable compatibility boundary. Preserve server public paths
and wire behavior, and document that transport adapters require `server` rather
than promising they are usable in a transport-free build.

## Decision and alternatives

Choose module-level transport gating plus extraction of pure execution policy.
Pros: minimum runtime behavior change; one shared policy implementation; no new
dependency; existing server API paths survive. Cons: one new domain module and
six internal caller retargets; uncompiled until operator authorizes a later gate.

Do not ungate the entire API capabilities module: its Axum/security DTOs would
spread the same dependency leak. Do not feature-gate the compiler or manager:
embedded hosts require those services. Do not duplicate flags into an embedded
fallback: it creates divergent execution policy. Do not enable server or
surreal-backend in SDK embedded/mobile profiles to suppress diagnostics.

## Integration handoff and unverified claims

Coordinate one writer for the listed files. The memory writer separately owns
the SurrealDB-specific service/scoped-storage repair and must preserve shared
memory contracts. Required OpenSpec delta and canonical task/phase documentation
remain the lead's ownership; this assessment did not edit them or inspect the
entire phase ledger. No production implementation item is marked complete here.

Implementation exit criteria for this packet: both HTTP modules are server-gated;
the eight shared symbols have exactly one transport-free definition; compiler and
runtime no longer reach these flags through API; server reexports retain previous
paths and semantics; dependency manifests are unchanged. These are source tasks,
not compilation acceptance.

After every phase implementation task and handoff is reconciled, the lead must
record a fresh whole-phase implementation-complete inventory with exact source
identity, present remaining tasks and exact proposed commands, and wait for the
operator's explicit instruction. Existing P0.005 failure receipts remain intact.
No rerun is requested or authorized by this plan. Compilation of host-persistence,
in-memory, embedded-mobile, SDK and server/full, wire compatibility, and memory
behavior remain unverified. No guards or fallback behavior were added.
