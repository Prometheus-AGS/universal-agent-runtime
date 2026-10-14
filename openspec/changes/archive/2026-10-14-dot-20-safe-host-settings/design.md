# Design

## Context

Issue #379 items 5, 7 and 8 span listener policy, shipped profiles and effective settings. The child source analysis identifies current CORS composition in `src/server.rs`, live provider state in `src/llm/registry.rs`, persisted settings synchronization in `src/uar/settings/provider_sync.rs` and governance snapshot in `src/uar/api/settings.rs`. A list of stored settings is not the effective aggregate. Existing native-service deployment already requires loopback listeners; this change extends that rule to all shipped default paths while preserving deliberate shared-host overrides.

## Goals / Non-Goals

Provide an empty-default explicit browser-origin allowlist, safe installable defaults, and one documented secret-free current effective view that an admin can read with only a Bearer credential. Keep local and shared profiles equally supported. Do not introduce a new provider registry, change a persisted operator selection silently, replace the settings mutation authority, or require a second admin header on the effective read.

## Decisions

1. The first implementation task resolves shipped-profile behavior before editing defaults: no implicit LLM model, clear failure until a model is explicitly configured, local/off memory embedder, loopback native listeners and JWT required in installable configurations. Existing saved operator provider/default selections remain authoritative through upgrade. Shared hosting declares its non-loopback bind and other required overrides explicitly; it retains authentication and tenant isolation. Test-only fixtures are not installable defaults.
2. CORS receives an explicit configured allowlist; an empty list emits no allow-origin header. Origin matching must be against configured values, with no wildcard/reflect-any fallback. The selected configuration key is verified against current config parsing before implementation rather than invented in this plan. Public agent-card reads may be anonymous but do not expand browser-origin access or task authorization.
3. Before implementing the effective read, pin one path and response shape with Kubo. Recommended: `GET /api/uar/settings/effective` and the published `uar-effective-settings.schema.json` with `revision`, `llm.provider_host`, `knowledge_base_embedder.provider_host`, `memory_embedder.provider_host`, plus supported optional governance/native-tools/MCP fields. If a different stable documented shape is chosen, record the Kubo translation and acceptance fixture first. Decide whether `revision` is a monotonic aggregate generation or stable hash and exactly which effective changes alter it. The choice must account for live provider registry, persisted synchronization and governance changes, and must exclude secrets.
4. Authenticate the effective read through the existing JWT verifier and an explicit admin principal claim. A missing/invalid Bearer token returns 401; a valid non-admin token returns 403; a Bearer admin token alone returns the aggregate. Resolve the concrete claim shape against existing identity policy before implementation. No `x-uar-admin-key` or second credential is required for this read. Existing mutation routes retain their current authority until separately changed.
5. Compose one coherent snapshot from the owning authorities or expose an explicit unavailable result if coherence/currentness cannot be established. Do not copy raw settings rows, secret values, credentials or transport recipes into the response. `local`/`off` embedder values are normalized deliberately; their exact mapping is part of the pinned schema contract.

## Risks / Trade-offs

The uncomfortable thing: safe defaults can appear correct in a checked-in YAML file while a packaged profile or environment override still binds publicly, chooses a remote embedder or turns auth off. Final acceptance must inspect the installed artifact and its effective sockets, egress and auth in both local and shared profiles. The effective read can also become stale if it samples persisted rows instead of live routing/governance; the aggregate revision must reflect actual changes.

## Migration Plan

Do not replace existing persisted provider or model choices on upgrade. Native local installs use loopback and auth-on defaults; shared deployment manifests explicitly state wider bind and remain authenticated. Introduce the CORS/defaults production task under an exclusive server/config writer; the effective-settings task follows dot-16 capability inventory and selected Kubo contract. Existing settings mutation and provider synchronization remain authoritative. No new secret store or model-routing engine is created.

## Verification

Author shipped-profile, socket/egress, CORS and schema/auth scenarios during implementation, but run none before the lead records every Dot production item, required document and runner complete. At that boundary, inspect real installed local and shared profiles, including empty/default and allowlisted origins, effective listener/auth/model/embedder behavior, deliberate shared overrides, Bearer 401/403/admin cases, schema fields, revision changes and secret absence. Dot-17 repeats item 8 against the actual published artifact and records its tag and checksum/digest. The operator’s latest direct instruction prohibits all testing until the entire phase is implemented. Necessary native installer-script fixes therefore retain the workflow-policy validator as a mandatory final-boundary check; it is not run early or reported passed. Deployment-only Actions policy remains unchanged.

Sequencing clarification (2026-10-12): task2 composes the existing live provider/configuration/governance authorities after task1; it does not consume new assistant capability descriptors. Dot16 inventories this endpoint once implemented. Removing that false source prerequisite allows parallel work without reducing the response contract or full-phase acceptance. Pinned Kubo schema: `d5dea2115ad59ed7b12df561e5b085b562df8674`, mcp/contract/agentic/uar-effective-settings.schema.json.
