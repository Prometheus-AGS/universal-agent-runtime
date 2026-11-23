## Why

The operator decided on 2026-11-19 that "the system level" for model selection is the
registry default provider plus that provider's `default_model`, and that an agent's
explicit model always beats it (`.prometheus/decisions.md`, entries "The system-level
model is the registry default provider" and "An agent's explicit model always beats the
system default"). Source today does not give the operator a control that matches:

- The only live way to change the default is `POST /api/uar/providers/{id}/default`
  (also mounted at `/api/providers/{id}/default`, `src/server.rs:2532,2680`; handler
  `src/uar/api/providers.rs:367-399`). It persists `llm.default_provider` and then calls
  `registry.set_default`. The route takes only a provider id. The model half is the
  provider's `default_model`, which is written by a separate `PUT /providers/{id}`
  (`providers-api.ts:57-68`).
- `llm.default_provider` is read back only at boot (`settings/provider_sync.rs:21-23`).
  Editing it through the generic settings API changes nothing until restart while the
  API reports the edit as effective (analysis.md delta 5).
- `run_policy.global.model` is a second, hidden system level. Global scope is applied
  first in `resolve_run_policy` (`domain/policy.rs:510-566`, model applied at :538) and
  seeds the chain before the registry default is considered, because the registry
  default is only backfilled when the resolved route is empty
  (`runtime/manager.rs:11345-11356`). No console control writes it.
- The one control that sets a default model, the Models page star
  (`features/models/ui/models-page.tsx:603-611`, `models-store.ts:111-121`), writes only
  the provider's `default_model`, and the Models page has no navigation entry
  (`pages/admin-page.tsx:32-52`, `nav-destinations.ts`).
- After a default change, nothing tells the operator which agents will not follow it.
  The Agents list classifies an explicit model with `policy.provider.default` only
  (`agents-page.tsx:206-225`), but the runtime also honors a model carried in the
  `uar.run_policy` agent extension (`domain/policy.rs:304-305`).
- Sub-agent model inheritance is unspecified (`multi-agent-orchestration` is silent).
  Code resolves a child's model as extension model, then the child artifact's own
  default, then the parent's effective model (`runtime/thread/policy_intersection.rs:582-590`).

Without this change the most visible operator action, "change the model for the whole
system", has no reachable, truthful control, and a polished picker on the wrong write
path would do nothing until restart.

## What Changes

Build items 12, 13 and 16 of the phase analysis.

- **System default picker on the live path.** One picker chooses an enabled provider and
  one of its models. Saving writes the provider's `default_model` (only when it differs)
  and then calls the live default route. It never writes `llm.default_provider`,
  `llm.model` or `run_policy.global.model`. The Models-page star and the Providers-page
  "make default" action call the same single action.
- **Full-catalog scale.** The picker lists the whole catalog (8,170 provider-qualified models) through the shared virtualized `CatalogModelPicker`, with non-usable rows shown but not selectable.
- **Reachable from primary navigation.** The picker lives on the Providers destination
  (already in primary navigation) and has a command-palette entry.
- **Retire `run_policy.global.model` from model selection.** The resolver ignores a
  `model` carried by the Global scope. Other Global fields keep working.
- **Default route rejects a disabled provider** (409), because a disabled default provider
  leaves no usable system default (`src/llm/registry.rs:551-563`) and the route accepts it today.
- **Agent wins; show who is pinned.** Precedence stays Turn, Conversation, Agent, system
  default. When the default changes, the console lists the agents that keep an explicit
  model, using the same rule as the runtime (artifact default or `uar.run_policy`
  extension model).
- **Sub-agent inheritance rule (pending operator confirmation).** An agent or
  sub-agent's explicit model wins over conversation and system defaults. A sub-agent
  without one inherits its parent's effective model.

Coordinated by reference only (other writers own the internals):

- `uiux-realtime-topics-and-replica-refresh`: publishing the default change to open
  clients and refreshing the registry default on every replica. This change defines no
  topic or replica behavior.
- `uiux-settings-truthful-status-and-removals`: removing the `llm.default_provider`
  settings control, rejecting settings-API writes to that key (so the row is written only by
  the live default route), and the `models[0]` banner on the Providers page.
- `uiux-schema-controls-and-pickers`: the shared picker component this change reuses.

**Operator-visible cost.** Changing the system default never reaches agents that pin a
model, and a default change is not a force. A deployment that depends on
`run_policy.global.model` (an embedded host that wrote it through the SDK) loses that
route; see design.md "Embedded hosts".

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `provider-model-settings-certification`: ADDED requirements for the system default
  model definition and write path, picker reachability, and retirement of
  `run_policy.global.model` from model selection.
- `session-configuration`: ADDED requirements for model precedence against the system
  default, pinned-agent visibility after a default change, and sub-agent inheritance
  (pending operator confirmation).

## Impact

- Backend: `src/uar/domain/policy.rs` (resolver), `src/uar/settings/manager.rs`
  (run-policy schema note), `src/uar/api/providers.rs` (default route rejects a disabled
  provider), possibly an agent read projection.
- Frontend: providers feature (system default panel and domain action), command palette,
  agents list pinned status, Models-page star routing.
- Wire: no new endpoints; one added rejection (409 for a disabled provider on the default route). `PUT /providers/{id}` and `POST /providers/{id}/default` are
  reused as they are.
- No dependency pin changes. No vendored code.

## Related open changes

| Open change | Overlap | Suggested disposition |
|---|---|---|
| `embedded-run-policy-and-admin-surface` | Its `run-policy-resolution` spec has a scenario "Global default model is applied on the embedded runtime" (`specs/run-policy-resolution/spec.md:13-19`). This change retires that behavior. 21 of 22 tasks are done. | Continue for the admin surface; supersede the Global-model scenario. Operator decides in Plan (decision 7). |
| `effective-policy-model-backfill` | Backfills the route from the registry default; this change keeps that behavior and relies on it. All tasks done. | Continue; no conflict. |
| `explain-agent-model-status` | Agents list three-way status; this change reuses it and corrects the "explicit" rule to include the extension model. All tasks done. | Continue; archive after boundary, with this change's pinned-agent rule applied on top. |
| `provider-meta-singleton` | Introduced the `ProviderMeta` entity this change reads (`default_id`, `default_model`). Done. | Archive (already shipped). |
| `migrate-models-page-direct-and-redesign` | Touches the Models page whose star control this change reroutes. | Continue; coordinate with `uiux-settings-truthful-status-and-removals` on whether the Models page stays. |
| `make-config-authoritative-on-boot` | Re-seeds `llm.default_provider` from YAML on every boot, which would override the live default. Contradicts the DB-wins decision. | Supersede (owned by the config boot-safety change); archive after Plan confirms. |
| `embedded-conversation-policy-surface` | Embedded conversation policy writes; does not touch the system default. | Continue; no conflict. |
