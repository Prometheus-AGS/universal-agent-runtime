# Administration and client contracts

The Boss is the user-facing administration client. Persistent team domain state belongs to UAR. Renderer state may hold selections and unsaved edits, but must not become an independent membership/task/approval database.

## Additive REST resource design

The following are proposed resources, not a declaration of shipped routes. Existing authentication and owner/workspace checks apply to all requests. A state-replacing command includes a stable commandId and expectedRevision; retries return its prior receipt or conflict for changed content.

| Resource | Operations | Persisted result |
|---|---|---|
| /api/v1/team-definitions | list/get/register/version/CAS replace | Immutable descriptor and catalog revision |
| /api/v1/team-instances | list/get/create | Bound team identity and configuration revision |
| /api/v1/team-instances/{id}/members | list/add/drain/remove | Membership revision and affected task disposition |
| /api/v1/team-instances/{id}/tasks | list/get/submit/assign/cancel/reconcile | Task/attempt and command receipt |
| /api/v1/team-instances/{id}/messages | enqueue/get receipts | Durable inbox and delivery state |
| /api/v1/team-instances/{id}/events | authorized cursor stream/snapshot | Projected committed event log |
| /api/v1/team-instances/{id}/diagnostics | read operational posture | Read-only structured status |

Lifecycle commands activate, suspend, drain and stop are explicit. Destructive purge is not part of this profile. List endpoints use opaque stable pagination; filters cannot broaden principal scope. Failure responses distinguish unauthorized, unsupported capability, revision conflict, resource limit, required approval, storage unavailable and uncertain effect. Error envelopes contain stable localized message keys and safe details, never secrets. Endpoint verbs and generated OpenAPI types are finalized in I3 from these contracts.

## Teams workspace in UAR settings

Navigation: Catalog → Installed bindings → Teams → selected team's Members / Tasks / Activity / Approvals / Budgets / Recovery. Distinguish a reusable template from a running/idle instance. Show immutable version/digest, active runtime connection, workspace and effective limits. Choose provider connections and models from resolved catalogs; do not require hand-entered provider/model string syntax.

Create/launch performs definition capability preflight and explains missing requirements. Team task views display queue reason, assigned member, current attempt, routing rationale, dependencies, elapsed time and terminal result. A member and a team are explicit conversation targets. A failed operation retains logs and offers an applicable recovery action; a spinner alone is not completion feedback.

Membership changes show effects on assigned work before applying. Stop, cancel, detach and drain use distinct labels reflecting backend support. Pending human approval displays exact operation/destination/payload with its issuer and current status. Reconnect/restart may refresh a challenge; stale decision controls must visibly expire.

## Settings and translations

Reuse The Boss preference source definitions, generated schemas, typed IPC and protected main-process connection storage. Persist UI choices and credential references, never duplicate UAR business state or expose secret values in renderer preferences. Add migration defaults preserving ordinary single-agent settings. All new labels, descriptions, errors, statuses and accessibility names need real translations across every shipped locale.

Tables and logs must support keyboard navigation, constrained widths, visible focus and accessible progress announcements. Member/task identifiers are available for diagnostics but display labels remain readable. The chosen workspace/team stays visible during concurrent output and surface actions.

## Diagnostics

User-invoked checks distinguish reachable, authenticated, capability-compatible and operational. Show actual UAR endpoint/port, instance identity, storage capability posture, pending recovery, queue depth, active turns, root budget use, event replay lag and model connection errors. Diagnostics do not silently submit paid inference or create issues; when a meaningful operation is requested, show its scope and resulting structured receipt.
