## Context

Provider configuration, model selection, and settings determine how every routed request executes. Their React pages currently mix presentation state with graph hydration, service calls, optimistic mutations, and persistence error handling. This violates the canonical Component → Hook → Store → Service → API direction and makes configuration-to-routing behavior difficult to certify.

## Goals / Non-Goals

**Goals:**

- Make domain stores the only frontend owners of provider/model/settings I/O and mutation state.
- Keep hooks as subscription/action façades and pages focused on presentation state.
- Preserve entity-graph reactivity while providing deterministic optimistic rollback.
- Certify successful, failed, empty, invalid, secret-redacted, and real routed-request behavior.

**Non-Goals:**

- Redesign the provider, model, or settings interfaces.
- Add a second configuration API or bypass liter-llm routing.
- Treat catalog presence as provider support certification.
- Move transient filters, dialog state, drafts, or compare selections into application stores.

## Decisions

### Domain stores own I/O and graph reconciliation

Dedicated Zustand stores call the existing typed services, publish loading/error/mutation state, and reconcile successful responses into the entity graph. Hooks subscribe to both stores and graph projections. Keeping the graph as the normalized read model avoids duplicating durable domain records in page state; keeping action status in Zustand makes failures and retries reactive.

Direct page/service calls and service-importing fetcher modules were rejected because both skip the mandatory store boundary.

### Draft settings remain a presentation cache

The existing per-namespace form cache remains responsible only for unsaved draft values, conflicts, and presentation continuity. The settings domain store owns load/save, server validation, optimistic graph updates, and rollback. Moving drafts into the durable entity graph was rejected because unsaved user intent must not masquerade as persisted configuration.

### Provider model mutations use the provider resource

The backend exposes configured models and `default_model` through `PUT /api/uar/providers/{id}` rather than a separate model mutation endpoint. The model store therefore snapshots the provider, applies an optimistic full-resource update, persists it, and reconciles the returned provider. This preserves the server contract and provides deterministic rollback.

### Certification combines store tests, API tests, and a real-server journey

Vitest covers store success/failure/rollback and UI-derived catalog behavior. Rust tests cover settings validation and redaction at the API boundary. Playwright exercises configure → default model → route against a real UAR server and stub provider. Mock-only browser evidence was rejected because it cannot prove persisted routing behavior.

## Risks / Trade-offs

- [Entity graph and mutation-status store can drift] → Every successful mutation reloads or reconciles the authoritative server response; failures restore captured graph snapshots.
- [Secrets could leak through settings reloads or errors] → Assert redacted API payloads and avoid storing submitted plaintext in graph or error state.
- [Large settings surface makes hand-authored coverage incomplete] → Generate namespace round-trip cases from the settings schema/type metadata.
- [Real-server E2E can be slower or flaky] → Reuse the deterministic stub LLM and explicit readiness checks already used by live integration tests.

## Migration Plan

1. Introduce provider, model, and settings store actions behind existing services.
2. Replace page and hook I/O with subscription/action façades while preserving UI-only state locally.
3. Add store/API/E2E certification and remove the migrated allowlist entries.
4. Run the full frontend, Rust API, browser, boundary, and OpenSpec gates.

Rollback is a focused revert to the previous page-owned calls; no persistence schema or external API changes are introduced.

## Open Questions

- Whether later work should consolidate provider and model mutation status into a single configuration bounded-context store after certification evidence establishes stable ownership.
