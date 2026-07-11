# KBD Analyze — uar-final-production-hardening-2026-07

Date: 2026-07-11
Mode: stack specified (React 19/TypeScript frontend; Rust/Axum backend)
Input: `assessment.md`, current source, OpenSpec/KBD progress, official protocol and testing documentation
Lifecycle note: this analysis was explicitly requested while the phase was already at execution step 4/9. It supersedes the narrow remaining-course assumptions; Spec/Plan must reconcile it before more implementation is treated as release-complete.

## Executive decision

Keep **React as UAR's primary and authoritative first-party interface**. Do not migrate the application shell or administrative UI to HTMX or Web Components. Remove present-tense claims that UAR is HTML/HTMX/Web-Components-first, avoids SPA frameworks, or runs identically on web/desktop/mobile. Historical design documents may remain only when clearly labeled as superseded, archival, or exploratory.

The shortest credible path to GA is not another visual polish pass. It is a **specification-to-behavior certification program** with four coupled tracks:

1. establish one truthful React architecture and migrate every live page to it;
2. give every advertised surface an executable functional contract against real backend behavior;
3. align AG-UI/A2UI implementations with current protocol contracts while retaining React rendering;
4. rewrite public and operational documentation from verified code and release evidence.

No UI framework replacement is recommended. The project already has the appropriate verification stack: Vitest, React Testing Library, user-event, Playwright, Zod, Zustand, React Router, and an entity graph. Add MSW only if the first page-contract migration proves the existing hand-written fetch mocks cannot be shared cleanly.

## Current architecture findings

The repository rules require:

`React Component → Hook/View Model → Store/Entity Domain → Service → API`

Current live UI violates that contract in multiple load-bearing places:

| Surface | Current evidence | Required correction |
|---|---|---|
| A2UI Testing | Component imports `postA2uiTestTrigger` service and `useThreadRegistryStore` directly. | Create `useA2uiTesting` hook backed by an A2UI store/domain action; component only renders state and invokes actions. |
| Runtime Console / Cockpit / Protocols / Runs / Approvals | Page imports entity store directly, calls `fetch`, and performs graph mutations. | Create runtime-console hooks and store/domain actions; move approval HTTP to service; make replay/live feeds one normalized store boundary. |
| Providers | Page imports fetcher, optimistic helpers, and provider services. | Hook exposes load/configure/default/remove; store/entity action owns service and optimistic transaction. |
| Models | Page imports fetcher and provider services and coordinates mutation state locally. | `useModelsPage` hook backed by model/provider store/domain actions; preserve UI-only filtering/selection locally. |
| Knowledge | Existing `useKnowledgePage` hook imports services directly, violating the Hook → Store boundary. | Introduce knowledge store/entity actions; hook only subscribes and exposes actions. |
| Settings | Component uses hooks, but `useSettings` imports a service directly; the 3,336-line page concentrates unrelated namespaces. | Move persistence into a settings store; split page by settings feature namespace without changing routes or behavior. |
| AG-UI chat interface | Stream store owns substantial protocol parsing; Runtime Console mirrors synthetic entities. | Define one typed AG-UI adapter and conformance fixtures; both chat and console consume the same normalized events. |
| Agent selector/session config | Components call `fetch` or services directly. | Move session agent-config and capability persistence into store actions. |
| Tool detail/agent builder/editor | Components import services or call `fetch`. | Add owning feature hooks/stores; keep components pure. |

This is not cosmetic debt. It explains why pages can render successfully while mutations, synchronization, and error handling silently fail.

## Product surface certification matrix

Each live route needs a spec mapping and a behavior test covering load, success mutation, API failure, empty state, permission/auth failure, and live update where applicable.

| Surface | Definition of “works to specification” | Required test level |
|---|---|---|
| A2UI Testing | Loads real catalog/schema; selects active run; validates payload; triggers server action; resulting A2UI surface renders through the same React renderer used in chat; action response reaches the run; malformed/unsupported components fail safely. | Vitest integration + Playwright real-backend round trip + protocol fixtures. |
| AG-UI interface/chat | Ordered lifecycle/text/tool/state events; cancel/resume; approval; reconnect/replay; snapshot/delta semantics; errors terminate visibly; no duplicate events after reconnect. | Adapter contract tests + Playwright BDD with stub LLM/backend. |
| Runtime Cockpit | Real runs, steps, tool calls, provider health, and current activity update without refresh; absent concepts are removed rather than simulated. | Store/adapter tests + live SSE Playwright test. |
| Protocols view | Shows actual AG-UI events and A2UI surfaces with run/thread correlation and protocol/version metadata. | Replay fixture tests + live stream E2E. |
| Runs/Approvals | Inspect navigation works; approve/deny persists exactly once; Cedar hard deny cannot present an approve action; timeout/error states surface. | Integration + E2E. |
| Settings | Every displayed control maps to a real schema/config key; save/reload round-trips; secrets are write-only/redacted; restart-required keys are labeled; invalid values are rejected; env precedence is documented. | Namespace contract suite generated from settings schema + focused E2E. |
| Providers | Catalog versus configured state is distinct; configure/default/remove round-trips; secrets never echo; health/failure status is real. | Store/service contracts + E2E mutation path. |
| Models | Catalog filters are correct; configured models mutate provider config; default selection routes a real request; compare uses real metadata and identifies unknown data. | Domain tests + routing integration + E2E. |
| Knowledge | Create KB, upload, index, search ranked result, use in chat, delete, and failure/retry all work with real embeddings. | Existing backend integration + React page E2E + chat BDD. |
| MCP Health/Tools | Discovered servers/tools, health transitions, execute path, and transport errors are real; no static health indicators. | Service/store tests + backend failure simulation. |
| Agents/Skills/Compiler/Memory/Auth/Costs/Credentials | CRUD/actions persist, errors surface, and realtime changes reconcile without manual reload. | One behavior contract per advertised action plus critical E2E journeys. |

## AG-UI and A2UI technical direction

### AG-UI

Adopt the official event model as the conformance reference, not necessarily as a runtime dependency. AG-UI distinguishes lifecycle, text, tool, state snapshot/delta, message snapshot, raw, and custom events. UAR should:

- declare the AG-UI version/profile it implements;
- map every UAR normalized event to a typed AG-UI event or documented custom event;
- replace deprecated `THINKING_*` events with `REASONING_*` if present;
- preserve ordering and IDs, tolerate out-of-order delivery, and support snapshot recovery;
- use one adapter for both Chat and Runtime Console;
- publish fixtures and an interoperability matrix.

Adding `@ag-ui/core` is optional. First compare its current schemas with UAR's Rust event contract. Adopt it only if it eliminates duplicated TypeScript types without forcing a second client state model.

### A2UI

A2UI is declarative UI data, not an HTMX rendering prescription. The official current production family is v0.9.1 and v1.0 is a candidate. UAR should choose and document one supported version before GA. The React renderer must:

- accept only validated messages and approved component catalogs;
- render native UAR React components;
- reject unknown components/properties safely;
- keep structure and data-model updates distinct;
- support progressive surface updates and action responses;
- never execute model-provided HTML or JavaScript;
- share the same renderer between chat artifacts and A2UI Testing.

Do not claim A2UI compliance from a schema list and trigger endpoint alone. Certification requires official conformance fixtures or UAR-owned fixtures derived from the declared version.

## Documentation truth program

### Canonical present-tense product statement

Use this architecture consistently:

> UAR ships a React 19 + TypeScript first-party web and administrative interface backed by Rust/Axum APIs and typed streaming events. The React application is used for web deployments and as the frontend of the Tauri desktop shell. Mobile support is experimental until platform-specific packages and tests are published. A2UI is rendered as validated declarative data through approved React components; AG-UI is the event protocol connecting the React client to agent runs.

### Files that must be corrected

- `README.md`: remove HTML-first/no-heavy-SPA and identical web/desktop/mobile claims; replace “142+ providers” with catalog breadth plus support tiers; describe React, A2UI, AG-UI, persistence authority, and release maturity accurately.
- root `package.json`: replace stale description, ISC license, and framework claims; align version/license with Cargo and release decision.
- `docs/ARCHITECTURE.md`: make React the primary UI; distinguish AG-UI transport from A2UI rendering; replace “tools always on” with policy-selected exposure; distinguish catalog routing from adaptive routing.
- `docs/STATE_MANAGEMENT.md`: rewrite obsolete HTMX lifecycle around React hooks, Zustand/entity graph, services, PGlite, and SSE.
- `docs/COMPREHENSIVE_TESTING_INFRASTRUCTURE_SUMMARY.md`: remove claims of HTMX/Web Components and unverified “production-ready” certification.
- `docs/full-implementation/A2UI + AG-UI.md`: mark as historical/exploratory or replace with a short supersession banner; do not let thousands of lines of no-React/HTMX design read as current architecture.
- Tauri/mobile/code-interpreter/realtime docs: classify implemented, preview, planned, and remote-fallback behavior separately.
- assessment and implementation-summary documents: retain as dated records but add “historical; not current product truth” banners where search results can mislead.

### Documentation source-of-truth controls

1. Add a compact `docs/product-support-matrix.md` for UI, protocols, providers, features, persistence, and platforms.
2. Add `docs/frontend-architecture.md` containing the mandatory layer flow and ownership examples.
3. Generate API/config references from schemas where feasible; hand-write only semantics and operations.
4. Add CI grep/link gates for prohibited present-tense claims (`HTML-first`, `no React`, `runs identically`, unqualified `142+ providers`, unsupported “production-ready”). Exclude explicitly marked historical archives.
5. Require every public claim to point to an executable release gate or be labeled preview/experimental.

## Build-versus-adopt decisions

### Adopt existing repository tools

- **Playwright Test** for functional page journeys, real-backend/stub-provider flows, browser errors, and web-first assertions.
- **Vitest + React Testing Library + user-event** for page-level integration at the hook/component boundary, testing behavior by accessible roles rather than internal state.
- **Zod and existing Rust schemas** for boundary validation; generate or share contracts where practical rather than duplicating interfaces.
- **Existing Zustand/entity graph** as application state, after enforcing ownership and eliminating component/service shortcuts.

### Conditional adoption

- **MSW** for reusable success/error/latency handlers across Vitest page contracts. Adopt only after a pilot on Providers or Settings demonstrates lower duplication than existing mocks.
- **`@ag-ui/core`** for TypeScript event schemas if a compatibility spike proves a one-to-one mapping with UAR events. Otherwise use it as conformance reference only.

### Build in UAR

- feature-owned stores/hooks for each administrative domain;
- one AG-UI normalization/conformance adapter;
- one validated React A2UI renderer/catalog and Testing-page view model;
- a UI capability/spec matrix and release certification suite;
- documentation claim inventory and automated truth gates;
- backend-specific semantics such as Cedar hard deny, provider support evidence, settings schema coverage, and persistence authority.

### Reject

- replacing React with HTMX/Web Components;
- introducing a second generic admin framework;
- Storybook as the primary certification mechanism (useful later for components, insufficient for end-to-end state/data behavior);
- snapshot-only or visibility-only tests as evidence that a feature works;
- parallel “v2” pages while old routes remain live.

## Recommended execution architecture

The existing nine-change plan is too narrow. Preserve completed work, but replan the remainder into the following risk-ordered program:

### Wave 0 — Freeze truth and inventory

1. Declare React as primary UI in an ADR and support matrix.
2. Inventory every live route, user action, backing endpoint, store/service owner, spec, and test.
3. Classify each item: stable, preview, experimental, internal, remove.
4. Convert all unsupported visible controls/panels into either a real contract or removal decision.

Exit: no live route or control lacks an owner, endpoint, maturity label, and acceptance test identifier.

### Wave 1 — Architecture boundary enforcement

1. Add ESLint/CI import-boundary rules for component/hook/store/service layering.
2. Migrate Runtime Console, A2UI Testing, Providers, Models, Knowledge, Settings, agent selector, and remaining direct-fetch components feature by feature.
3. Do not change visuals and data architecture in the same change unless behavior requires it.

Exit: no component calls `fetch`, imports services, or performs store mutation logic; no hook imports services; only stores/domain actions import services.

### Wave 2 — Protocol and state correctness

1. Specify and test the UAR normalized-event → AG-UI mapping.
2. Select A2UI v0.9.1 or v1.0 candidate explicitly; recommendation for GA today: v0.9.1 stable, with v1.0 experimental.
3. Unify chat and console event ingestion; implement replay/snapshot recovery.
4. Make Cedar hard deny non-overridable before approval UI certification.
5. Publish persistence authority/sync/conflict rules.

Exit: protocol fixtures pass in Rust and TypeScript; live replay and approval semantics pass E2E.

### Wave 3 — Surface-by-surface functional certification

Execute vertical slices in this order:

1. Providers → Models → Settings (configuration and routing foundation).
2. Knowledge → Chat/AG-UI → A2UI Testing (core customer journey).
3. Cockpit/Protocols/Runs/Approvals (operability and governance).
4. Agents/Skills/Tools/MCP/Compiler/Memory/Auth/Costs/Credentials.

For each slice: service contract, store tests, hook/page integration tests, real-backend Playwright journey, error/empty/auth coverage, docs update, OpenSpec verification.

Exit: every stable capability has a passing executable acceptance scenario; experimental routes are labeled or hidden.

### Wave 4 — Documentation and release truth

1. Rewrite canonical docs from the support matrix and certified behavior.
2. Mark or quarantine stale historical documents.
3. Align release workflow, version, SECURITY policy, package metadata, artifacts, SBOM/signatures/provenance, and platform matrix.
4. Run clean-machine/offline and candidate-tag certification.

Exit: public claims, packages, docs, runtime version, and immutable release evidence agree.

## Acceptance gates for “all inconsistencies cleared”

- Zero layering violations under automated import/fetch gates.
- Zero visible placeholder controls or unsupported panels on stable routes.
- Every live stable route has at least one successful and one failure-path behavior test.
- Providers/models/settings/knowledge/chat/A2UI/AG-UI/cockpit journeys pass against a real UAR server with deterministic provider fixtures.
- AG-UI and A2UI version/profile statements match passing fixtures.
- README, package metadata, architecture, state, testing, deployment, security, and support matrix agree.
- Search for prohibited stale claims returns only explicitly marked historical documents.
- Release workflow uses the same Node/pnpm and supported Cargo matrix as CI.
- Candidate tag produces tested artifacts, checksums, SBOM, provenance, and signatures.
- Offline locked build and advertised-platform startup smoke tests pass.

## Research evidence

- Playwright recommends user-facing locators and retrying web-first assertions; this directly addresses the repository's visibility-only test weakness: https://playwright.dev/docs/best-practices
- React Testing Library's principle is to test the way users use the product, supporting page-level behavior contracts: https://testing-library.com/docs/react-testing-library/intro/
- Vitest recommends MSW for request mocking, including fail-on-unhandled-request configuration: https://vitest.dev/guide/mocking/requests
- AG-UI defines an event-driven, transport-agnostic frontend/agent protocol and typed lifecycle/tool/state patterns: https://docs.ag-ui.com/concepts/architecture and https://docs.ag-ui.com/concepts/events
- AG-UI explicitly distinguishes itself from generative UI specifications such as A2UI: https://docs.ag-ui.com/concepts/generative-ui-specs
- A2UI defines declarative UI data rendered through native client components; v0.9.1 is current production and v1.0 is candidate: https://a2ui.org/

## Research budget

- Tier 1 GitHub searches: 6 attempts (three malformed-field/network attempts, three successful read-only queries returning no useful candidate set).
- Tier 2 official documentation: covered through official protocol/testing documentation returned by web research; no Context7 connector was available.
- Tier 3 registry queries: 5 attempted; local npm invocation returned no usable metadata, so current repository-pinned versions were used without making new-version claims.
- Tier 4 web queries: 8 (hard cap reached). Research stopped at the cap.
- Elapsed analysis remained within the 20-minute research budget for external candidate research; local code inspection continued as project evidence.

## Open questions for Spec/Plan

1. A2UI GA profile: v0.9.1 stable (recommended) or v1.0 candidate with an explicit preview label?
2. Should historical mega-documents remain searchable in place with supersession banners, or move under `docs/archive/`? Recommendation: move when links can be updated mechanically; banner dated assessments.
3. Should MSW be added after a one-page pilot? Recommendation: decide from measured fixture reuse, not preference.

These questions do not contest the stack choice: React is operator-specified and already implemented.
