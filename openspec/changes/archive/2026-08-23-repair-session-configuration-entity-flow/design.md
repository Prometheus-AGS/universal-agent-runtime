## Context

See `proposal.md` and the Analyze report. The panel currently owns seven pieces
of session business state in React, calls setters during render, and delegates
save-only behavior to `useChatSessionConfigStore`. `ModelSelector` mounts
`useModelsStore.load`, which fetches both `/api/models` and
`/api/uar/providers`, then publishes one graph update for every catalog model.
The backend already persists `AgentSessionConfig` and resolves it into
`RunPolicy`, but the frontend sends `model_override` while the backend accepts
`model`; the request path also resolves a model before applying the saved
conversation policy.

## Goals / Non-Goals

**Goals:**

- Establish one typed, inspectable authority for committed and draft session state.
- Keep all entity-package access behind the platform facade and domain hooks.
- Bound sheet work to configured provider/model data and make the saved model effective for inference.
- Correct the observed body spacing without redesigning the sheet.

**Non-Goals:**

- Banning React state for transient widget behavior such as whether a popover is open.
- Rewriting every models/providers administration flow.
- Waiting for the upstream atomic-ingestion release before fixing the sheet.
- Adding new session controls or new provider/model semantics.

## Decisions

### Use canonical and draft graph entities for business state

Keep `AgentSession` as the committed entity keyed by `threadId`. Add an
`AgentSessionDraft` contract keyed by `${threadId}:${editorId}` containing the
editable fields, baseline revision, dirty fields, save status, and error. Draft
rows are local graph entities: they are not sent through realtime patches and
cannot replace the canonical row until save succeeds.

Domain hooks under `frontend/src/platform/entities/session-configuration/`
select one field at a time from the graph store and expose event-driven actions
for open, set-field, save, cancel, and cleanup. The sheet shell subscribes only
to status; each control subscribes to its field. This uses Entity Management's
external store for explicit business state while retaining UI-local React state
only for widget mechanics.

Shared canonical patches were rejected because unsaved values could become
visible to other consumers. A component-local form object was rejected because
it would hide business state from the project's chosen authority.

### Register remote entity transports once at application boot

Re-export `registerEntityTransport`, `useEntities`, and the required graph types
from `frontend/src/platform/entities/index.ts`. Register remote transports for
configured Provider, configured Model, and AgentSession alongside the existing
schema registration. Provider and Model transports normalize
`/api/uar/providers`; AgentSession uses GET/POST
`/api/uar/sessions/{id}/agent-config`. The draft schema is registered but has no
remote transport because it is deliberately editor-local.

The configured Model transport flattens only models actually present beneath
configured providers. `ModelSelector` consumes the configured-model domain hook
and never mounts the static catalog store. The administrative catalog can
continue to use its own catalog query until the upstream atomic primitive is
available, but it cannot populate the configured Model authority used here.

### Align the typed session wire contract before wiring save

Add a frontend `AgentSessionConfig` request/response type mirroring the existing
Rust struct: `agent_id`, optional `model`, tools, skills, knowledge bases, MCP
servers, and tool approval. Replace `model_override` with `model`; add GET and
typed POST decoding. Do not retain `context_strategy` fields in the sheet unless
the same change adds them to Rust `RunPolicy`, persistence conversion, request
resolution, and the genuine inference proof. The minimum change is to remove
the currently ignored context block and retain only fields already supported by
the runtime contract.

### Resolve saved policy before the final model route

In the chat handler, resolve the session identifier and effective conversation
policy before final provider/model selection. Explicit turn/request model keeps
its existing highest precedence; otherwise the saved session model precedes the
agent default. The chosen route is then applied consistently to the agent
policy, context construction, telemetry, and the provider request. Reuse the
existing `resolve_effective_run_policy` path rather than adding a second session
map lookup.

The frontend runtime reads the canonical session entity and includes an
explicit model only when the current turn intentionally overrides the saved
session policy. It must not keep copying `agentConfig.model` into every turn in
a way that masks the saved session value.

### Use the existing sheet spacing scale

Apply the same shared horizontal inset used by `SheetHeader` to the body, plus a
bottom inset and existing gap tokens. The design audit resolves the concrete
utility/token before implementation; computed-style evidence compares header
and body insets rather than relying on a guessed device-specific pixel value.

## Risks / Trade-offs

- **Draft rows leak after abrupt unmount** → Cleanup the editor-owned draft on close/unmount and replace it deterministically on the next open.
- **Two endpoints race and overwrite graph rows** → Give configured entities one domain owner and reject catalog hydration into that authority.
- **Saved model is valid syntactically but no longer configured** → Preserve the value as unavailable and require an explicit operator choice; do not silently route elsewhere.
- **Policy reorder changes explicit request precedence** → Lock precedence with focused backend scenarios and a genuine inference route observation.
- **Removing ignored context controls reduces visible surface** → This is preferable to a deceptive control; they can return only with a complete runtime contract.

## Migration Plan

1. Add facade exports, typed contracts, schemas, transports, and domain hooks.
2. Migrate configured provider/model loading to the registered authority.
3. Migrate the panel to canonical/draft hooks and the typed GET/POST contract.
4. Correct effective model resolution and remove or fully implement unsupported fields.
5. Apply the resolved spacing token.
6. Remove the panel's obsolete store/service paths only after all consumers move.
7. Roll back as one vertical slice if functional inference cannot be verified; no persistent schema migration is required because the backend record format remains compatible.
