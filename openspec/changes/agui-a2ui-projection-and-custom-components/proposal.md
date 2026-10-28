> **Superseded 2026-10-27.** This change is being rebuilt as change 3 of the
> re-scoped AG-UI/A2UI work, on top of `a2ui-surface-state-patches` (change 1)
> and a durable thread-scoped surface store (change 2). Its A2UI state-patch
> fix has moved to change 1. Do not implement it from this text.

## Why

AG-UI clients that use UAR's dedicated agent route (`/api/uar/ag-ui/agents/{id}`) cannot use A2UI: the route drops every A2UI surface, input-request and state event, and it has no way to send a user's A2UI action back (catalog I19). Separately, the A2UI component set is closed to 9 built-ins, so operators and hosts cannot add components, and the shipped entity components (`+entities`) are client-only and rejected by the server. Both block the AG-UI/A2UI work the operator has made the next priority.

## What Changes

- **A2UI state patch fix:** withdrawn 2026-10-27 and moved to `a2ui-surface-state-patches` (change 1 of the re-scope). This change will be rebuilt on it.
- **Dedicated AG-UI route, outbound:** project A2UI surfaces, input requests and state patches with the chat adapter's event names (`CUSTOM uar.artifact.available`, `CUSTOM uar.artifact.input_required`, `STATE_DELTA`), carrying wire thread/run identity, omitting native run ids, and starting each run from a `STATE_SNAPSHOT` built from the thread's earlier runs, so continuations see the source run's surfaces. Input requests are display-only on this route.
- **Dedicated AG-UI route, inbound:** accept an A2UI action as a new AG-UI run on the same thread (`forwardedProps.uar.a2ui_action`). The route finds the declaring run and prepares the continuation (including host fields) before reserving, then reserves the binding and executes exactly one continuation with the reserved run id; the action is echoed right after `RUN_STARTED`. Actions must name a declared action event, not any string on the surface.
- **Template components (server-expanded):** operators define named templates made of the 9 built-ins. Before validation, UAR expands every template use into built-ins, so published surfaces stay on the certified baseline catalog and every renderer, including third-party AG-UI clients, draws them with no new code.
- **Host components (extension catalogs):** hosts declare versioned component definitions in seed files that are authoritative at startup (upgrades land, removed entries disappear), in declared catalogs `urn:uar:a2ui:catalog:1+<name>`. A per-session surface catalog index records each surface's catalog; extension components are accepted only for surfaces indexed in a catalog that contains them, and unindexed surfaces are validated as baseline, so built-in surfaces behave as today. Props must fit a closed, non-executable schema subset (closed objects, event-only actions). The full `+entities` catalog (7 components) is seeded. Every admission path validates, including saved surface templates (by their own catalog) and the surface test trigger; presentation output stays baseline-only.
- **Agent discovery:** a new read-only `a2ui_catalog` tool returns current templates and host components with props at call time; the `a2ui_render` tool's schema accepts extension items generically and validates them against the registry using the run's owner and session.
- **Client registry:** the React renderer resolves host components through `registerA2uiComponent(catalogId, name, impl)`; anything unimplemented fails closed with a named diagnostic. Lit and Svelte fail closed.
- **BREAKING (spec-level):** the `a2ui-uar-renderer` "exactly 9 components" requirement is restated: the certified baseline stays exactly 9, and extension catalogs are opt-in per surface.

## Capabilities

### New Capabilities
- `ag-ui-agent-endpoint`: the dedicated AG-UI agent route's event projection (including A2UI surface, input and state events) and its acceptance of A2UI actions as same-thread runs.
- `a2ui-component-registry`: template and host component definitions, server-side template expansion, catalog-scoped validation, the props subset, authority and agent discovery.

### Modified Capabilities
- `a2ui-uar-renderer`: the 9-component baseline requirement is restated for extension catalogs, and the renderer gains host-component registration with fail-closed handling.

## Impact

- **Runtime UX:** AG-UI clients on the dedicated route can render, act on and continue A2UI surfaces. Operators gain reusable template components that render everywhere; hosts gain extension components. Unimplemented host components fail closed with a named diagnostic.
- **Provider compatibility:** none; no model or provider path changes.
- **Realtime state:** the dedicated route gains `STATE_SNAPSHOT`/`STATE_DELTA`, and each action is echoed as a delta on its continuation. The chat baseline gains `actions:{}`. Actions on surfaces from before a restart are refused, because surface replay and the catalog index are in memory.
- **Code:** `src/uar/api/agui.rs`, `src/uar/api/adapters.rs`, `src/uar/api/sse.rs` (baseline), `src/uar/api/routes.rs` (`RunApiState`) and the 5 `tests/*.rs` files that construct it, `src/server.rs` (state wiring, startup seed); `src/uar/a2ui/routes.rs`, `src/uar/a2ui/mod.rs`, new `src/uar/a2ui/actions.rs` and `src/uar/a2ui/component_registry.rs`, `src/uar/a2ui/protocol.rs`, `src/uar/a2ui/presentations.rs`, `src/uar/a2ui/design_systems/`, `src/uar/api/settings.rs` (reusable admin-key check), `src/uar/runtime/native_skills/a2ui_render.rs`, `native_skills/mod.rs` and a new `a2ui_catalog` tool, `src/uar/runtime/a2ui_output.rs`; a governance permit for `a2ui_catalog`; `frontend/packages/a2ui-uar`, `a2ui-lit`, `a2ui-svelte`, `frontend/src/platform/a2ui/protocol.ts`; `docs/protocols/a2ui-profile.md`, `docs/protocols/ag-ui-profile.md`.
- **Security:** with `settings_mutation_auth_required=false` (as on the owned test instance), any loopback caller can create templates, including an agent with a loopback-capable tool such as `terminal_exec`; templates can only produce built-ins. Deployments granting such tools must run with the flag on. The render tool's compiled schema becomes more permissive; strict checks move into its context-aware execution.
- **KBD workflow state:** yes. Planned in KBD child phase `agui-a2ui-projection-and-custom-components` under `uar-post-lifecycle-verification-readiness`; the operator adopted new catalog IDs (D-4); revision 4 proposes splitting the rejections into I23-I25 (denominator 107, decision D-6). The child phase's KBD state must be reconciled to one directory with registered tasks before Execute.
