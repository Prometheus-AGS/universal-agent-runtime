# Assessment — a2ui-surface-architecture

KBD Assess, 2026-10-28. Read-only source and document assessment. No code,
builds or tests were run (verification gate CLOSED). Detailed evidence lives
in three companion files in this directory:

- `upstream-research.md`: A2UI, AG-UI and CopilotKit read at pinned commits
  `ccf5dc20`, `dab9c95f` and `4693af4d`, plus local `@a2ui/web_core` 0.12.0.
- `uar-surface-consumers.md`: every UAR producer and consumer of surface state.
- The San Saba workspace documents cited in section 3. The sibling projects
  were summarized by a read-only agent and are cited in section 3.2.

## Prior context

`prior-context.md` recalled three lessons and one reflection. One applies:

- The reflection of `uar-uiux-full-migration-2026-08` says its first goal
  "conflicted with already-complete, conforming A2UI assets". That is a warning
  against replacing working A2UI pieces. Here it applies to the chat path, which
  already renders correctly with `web_core` per artifact.

The knowledge gaps listed there concern the earlier UI migration (style
exceptions, coverage, assistive-technology certification). They are unrelated
to this assessment.

## 1. Objectives

| Id | Objective | Source |
|---|---|---|
| O1 | Live | accepted 2026-10-28 |
| O2 | Recoverable after reload or reconnect, on any device or console | accepted 2026-10-28 |
| O3 | Interactive across turns | accepted 2026-10-28 |
| O4 | Extensible: one catalog definition | accepted 2026-10-28 |
| O5 | Safe on a shared server | accepted 2026-10-28 |
| O6 | Full applications built only from A2UI over AG-UI | operator scope decision, `goals.md` |
| O7 | Personalized real-time morphing | operator scope decision |
| O8 | Upstream standards alignment | project rule: open standards first |
| O9 | Can implement a specification like `sansaba-workspace` | operator scope decision |

## 2. Upstream facts (from `upstream-research.md`)

1. **The renderer owns A2UI surface state.** The server sees the data model only
   when an action is sent (A2UI v0.9.1 #L324, #L491, #L593).
2. **A2UI defines no snapshot, resync or reconnect message** (v0.9, v0.9.1 and
   v1.0). Re-creating an existing surface is an error (v1.0 #L178).
3. **The de-facto mapping is an activity.** `@ag-ui/a2ui-middleware` carries
   A2UI as `ACTIVITY_SNAPSHOT` `activityType:"a2ui-surface"`, with the
   cumulative operation list and `replace:true`. The client reduces it with
   `web_core` (middleware `index.ts` #L494-L497, #L1323-L1377; CopilotKit
   `A2UIProvider.tsx` #L53-L91).
4. **Actions travel in run input** as
   `RunAgentInput.forwardedProps.a2uiAction.userAction`
   `{name, surfaceId, sourceComponentId, context, timestamp}`, and start a new
   run (middleware `types.ts` #L120-L147).
5. **AG-UI 1.0 rules that apply:**
   - activity events are standard, and activity messages are stripped from run
     input (`activity.mdx` #L73-L78);
   - SSE cannot resume (`http-sse.mdx` #L51-L55);
   - replays must be restated as snapshots (`lifecycle.mdx` #L190-L202);
   - `STATE_*` is application state that returns to the agent;
   - `CUSTOM` must not carry semantics that standard events own
     (`passthrough.mdx` #L62-L65).
6. **`web_core` 0.12.0 limits.** It has no state-to-messages serializer. It has
   depth and index limits, but no limits on bytes, component count or
   data-model size.
7. **Official renderers.** React, Lit, Angular and Flutter (GenUI) are stable
   for v0.9.1. Compose is alpha. SwiftUI is planned, with code in the repo.
   There is no Kotlin renderer in the A2UI repo.

**Unverified:** whether the middleware or CopilotKit restores surfaces after a
reload, and whether the data model is forwarded with actions.

## 3. San Saba requirements

### 3.1 sansaba-workspace (read by the lead)

- **Three planes on one AG-UI channel.**
  - **Conversation:** episodic, triggered by a message, with inline A2UI.
  - **Application (center):** durable, triggered by `mount`, `navigate` or
    `Button → Action → A2A task`, rendered as an *assembled A2UI surface* in
    layout regions.

  Sources: `docs/SSW-ARCH-001-architecture-spec-and-plan.md` §1.2;
  `docs/operating-philosophy.md` §3.
- **Surfaces bind to durable state, not to one run.** "Bind the surface to
  state, not to a single run (runs are episodic writers)"
  (`operating-philosophy.md` §4.2). The README retires "binding the center
  surface to an agent run's event stream" (`README.md`, Mini App contract).
- **The Shared Entity-Graph Pattern.** A normalized reactive entity graph
  (`prometheus-entity-management` 3.0.0-alpha) is the single source of truth.
  Human edits, agent AG-UI/A2A writes, embedded-app bridge messages and
  realtime sync all land in the same `(type,id)` node. Server truth is in
  Postgres, synced by `flint-realtime-fabric` or Electric/PGlite (SSW-ARCH-001
  §1.3, §3.2).
- **Assembly.** flint-forge `POST /a2ui/v1/surfaces/assemble` assembles a
  surface from a component library and parameters on `mount`/`navigate`. Data
  props are "resolved through the entity graph" (SSW-ARCH-001 §3.3(A)). The
  Mini App manifest (`prometheusags.ai/v1alpha1`) declares `surface.kind`,
  `componentLibrary`, `initialComponent`, and enforced deny-by-default
  `capabilities` for entities, agents and components (§3.4).
- **The catalog.** Forge serves an a2ui.org v0.9.1 catalog with 55
  `definitions`, including `DataTable`, `DataGrid`, `DatePicker`, `Form`,
  `NavBar` and `Pagination` (`docs/SSW-A2UI-011-catalog-conformance.md`).
- **Chat artifacts.** The C-29 pre-generation gate arrives as an AG-UI
  `CUSTOM` A2UI artifact, and its response writes to the graph (SSW-ARCH-001
  §3.3(D); SSW-A2UI-011).
- **Access model.** Verified `admin`/`user` roles, a tenant from verified
  claims, and PostgreSQL RLS. No Cedar or Keto for Sansaba. Every trigger
  carries the same verified identity (`operating-philosophy.md` §4.4;
  SSR-P3-Shell-Spec SH-3).
- **UAR status in San Saba (SSW-ARCH-001 §0.1).** "Its current user claims omit
  `tenant_id`, its shared-secret authentication is incompatible with this BFF's
  RS256 identity, and its generic A2A task store is not tenant-partitioned.
  Consequently UAR-backed Mini Apps stay unavailable."
- **Per-persona views.** S-1 deal-state checklists give "each persona a
  filtered view of what needs them now". State changes fan out to an inbox,
  email and channels within one minute (SSR-P3-Shell-Spec §2.3, C-19, SH-5).
- **The A2UI harness.** It must be a schema registry with validation and hot
  reload, without redeploying (SH-2, R-5).

### 3.2 Sibling projects (read-only agent summary)

Root aliases: DGA `document-generation-agent`, SSR `ssr-frontend`, GOT
`gotenberg`.

- **document-generation-agent (Mastra).**
  - It emits A2UI as `CUSTOM A2UI_ARTIFACT`. The client replies with
    `context.a2uiResponse` (DGA/README.md:41-50).
  - It sends `STATE_SNAPSHOT` only when the client sends state; there is no
    delta and no resume (DGA/docs/clients/agui-custom.md:10-27, 272-285).
  - Renders are synchronous: 2-8 s, returning a 10-minute SAS URL. No progress
    events exist (SSR/docs/document-generation-end-to-end.md:77, 354-356,
    1020-1028).
- **ssr-frontend (Next.js 16).**
  - About 50 dense list, detail, report and admin routes
    (admin-dashboard-navigation.md:21-87), and a 10-panel acquisition editor
    sharing one state (acquisition-architecture-complexity.md:20-37).
  - Real-time is "SSE feeds invalidation, not data", followed by a refetch
    (BIBLE.md:380-445).
  - It is moving to the PEM graph: 16 ms coalescing, optimistic writes with
    rollback, an edit buffer with a dirty flag, and save before destructive
    transitions (acquisition-architecture-complexity.md:629;
    refactoring-methodology.md:165-172; letter-agreements.md:137).
  - Governed document edits use revision/hash compare-and-swap and
    authoritative reload (DGA/docs/runbooks/governed-document-knowledge.md:3-7,
    190-193).
  - WCAG AA (PRODUCT.md:91-94). Offline is out of scope for phase 3.0.
- **gotenberg.** A PDF conversion service with no UI. A webhook module suggests
  async conversion, but its use is undocumented.

**Conflict recorded.** The ssr-frontend Phase 3 spec requires "Cedar policy on
every tool call" (phase3 spec:221). The workspace documents, which are later
and govern the container, say no Cedar or Keto is deployed for Sansaba. The
workspace is taken as current. The operator should confirm.

### 3.3 Requirements on an A2UI/AG-UI runtime (consolidated)

| Id | Requirement | Evidence |
|---|---|---|
| SR1 | Durable application surfaces that outlive runs and navigation | SSW-ARCH-001 §1.2; README contract |
| SR2 | Surface data resolved from domain entities, with multi-tab and multi-user invalidation | SSW-ARCH-001 §1.3, §3.2; SSR BIBLE.md:380-445 |
| SR3 | Server-side assembly from a registered library plus parameters on mount/navigate, with a hot-reloadable, validated catalog | SSW-ARCH-001 §3.3(A); SH-2 |
| SR4 | Episodic chat A2UI artifacts with human-in-the-loop responses that block actions | SSW-ARCH-001 §3.3(D); DGA precheck schema |
| SR5 | Verified identity and claim-derived tenant on every trigger; tenant isolation by RLS | operating-philosophy §4.4; SH-3 |
| SR6 | Deny-by-default capabilities per Mini App (entities, agents, components), enforced at the server and the renderer | SSW-ARCH-001 §3.4 |
| SR7 | Per-persona views and state-driven notifications | Shell spec §2.3, C-19, SH-5 |
| SR8 | Job status and durable results for renders and reports | DGA/SSR end-to-end docs |
| SR9 | Drafts, dirty state, optimistic writes, compare-and-swap on agent edits | SSR refactoring-methodology; DGA governed knowledge |
| SR10 | Dense screens: about 50 routes and a 10-panel editor | SSR admin-dashboard-navigation; complexity doc |
| SR11 | Accessibility (WCAG AA) for generated surfaces | SSR PRODUCT.md:91-94 |

## 4. UAR current state (from `uar-surface-consumers.md`)

| Gap | Evidence | Objectives hit |
|---|---|---|
| G1 State patches encode messages, so they fail to apply and are last-message-wins | `realtime.rs:69-97`; `sse.rs:79`; `agui-adapter.ts:72` | O2 |
| G2 Surface state is scoped to a run and kept in memory; continuations start empty | `realtime.rs:121-163`; `manager.rs:3096-3189` | O2, O3, SR1 |
| G3 Routes 404 once a run leaves the active map, although durable history holds the events | `tests/run_retention/runs.rs:81-92` | O2 |
| G4 A2UI is carried in `STATE_*` and `CUSTOM`, not AG-UI activities; the UAR AG-UI profile excludes activity events | `adapters.rs:208-219, 332-338`; `docs/protocols/ag-ui-profile.md:33` | O8 |
| G5 The dedicated AG-UI route emits no A2UI and accepts no action (I19) | `agui.rs:627-657` | O6, O8 |
| G6 No catalog object; server and client definitions differ; closed 9-component enum | `protocol.rs:12-17, 68-80`; `uar-basic-catalog.ts:32` | O4, SR3 |
| G7 The action gate uses a prefix match on unescaped ids, lets deleted surfaces pass, and matches names anywhere | `routes.rs:837-848` | O5, O6, SR6 |
| G8 No admission caps on bytes, depth or count | `protocol.rs` (no limits); body limit 10 MB (`server.rs:2898`) | O5 |
| G9 No deterministic action handlers: every action continues the agent run | `routes.rs:883` | O6 |
| G10 No push channel per thread or session; streams end at the terminal event | `routes.rs:873-989` | O6, O7 |
| G11 No entity binding for surface data | not found | O6, O7, SR2 |
| G12 San Saba blockers: no `tenant_id` claim, auth that doesn't fit RS256, A2A tasks not partitioned by tenant | SSW-ARCH-001 §0.1 | O9, SR5 |
| G13 Two client reducers (`web_core` and `reduceA2uiMessage`); the console throws on action entries | `run-trace-projection.ts:39, 312-357` | O2 |

What works today:

- Chat renders correctly from artifacts, with a fresh `web_core` processor per
  artifact (`a2ui-artifact-block.tsx:212-293`).
- Durable run history stores every event.
- `parse_message` filters executable content.

## 5. Knowledge gaps

- How upstream restores A2UI activities after a reload. The code was not found.
- Whether real AG-UI clients use the persistent `connect()` path. Upstream's JS
  default throws `ConnectNotImplementedError`.
- How the PEM graph and the A2UI data model connect: San Saba says data props
  are "resolved through the entity graph", but no implementation has rendered a
  live assembled surface yet (SSW-A2UI-011, "Carried forward").
- The `tenant_id` and RS256 blockers are recorded by San Saba as of
  2026-07-29. They were not re-checked against current UAR source.

Each gap can be offered to the operator as `/learn-goal <topic>`. None was run.

## 6. Addendum — prometheus-entity-management 4.2.0 (operator instruction, 2026-10-28)

The operator directed that UI/UX testing take PEM 4.2.0 into account. The
embedded React application should support it, and it handles both React and
Flutter A2UI rendering. Full findings are in `pem-4.2.0.md`. In summary:

- **Published.** npm `latest` is 4.2.0 for `a2ui-react`, `entity-graph-core`
  and `prometheus-entity-management`. pub.dev has `entity_graph_flutter` 4.0.0.
  The repository's registry-status file is stale.
- **A long-lived runtime.** PEM `a2ui-react` runs one application-owned
  official `MessageProcessor`. It consumes AG-UI `a2ui-surface` activities,
  processes batches atomically, accepts A2UI v1.0-RC and v0.9.1, and allowlists
  catalogs. Actions pass a default-deny entity-graph policy with tenant
  authorization and approval.
- **Snapshots reset surfaces.** A `replace:true` activity deletes and recreates
  the surfaces of that activity, data model included. Incremental activity
  messages do not.
- **Compaction is possible.** Its preflight rebuilds live surfaces as messages,
  which shows that compaction from official state is feasible.
- **Flutter rendering is example-level.** It is GenUI 0.10.2 in the
  `flutter-riverpod` showcase. The bridge contract does not claim Flutter
  rendering.
- **New UAR gaps.**
  - **G14.** PEM is pinned at 3.0.3 (`frontend/package.json:46-47`).
  - **G15.** UAR's local `@prometheus-ags/a2ui-react` 0.10.1 collides with
    PEM's package of the same name.
  - **G16.** UAR's chat renderer and console do not use a long-lived runtime.
