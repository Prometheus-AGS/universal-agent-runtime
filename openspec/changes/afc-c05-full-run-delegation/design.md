## Context

See `proposal.md` for motivation and `specs/full-harness-run-delegation/spec.md` for observable behavior. `/api/uar/runs` already performs definition resolution, service-placement validation, host-resource assembly, and execution through `RunManager`; its SSE route deliberately cancels after the last subscriber disconnects. A2A currently owns a separate in-memory thread task map. Initiative C05 requires a dedicated task authority without a second execution loop or persistence claims.

## Goals / Non-Goals

**Goals:**

- Make an admission retry safe before native execution begins.
- Keep task control owner scoped and expose the same full-harness record through REST and A2A lookup/cancel.
- Describe process-ephemeral recovery limits exactly.

**Non-Goals:**

- Persist task/admission records across process restarts; C06 owns persistence.
- Add mid-run steer support, modify the native loop, change model-provider routes, or replace legacy A2A tasks.
- Change `/api/uar/runs` request or response behavior.

## Decisions

### Add one process-local task authority

`FullHarnessTaskAuthority` owns runtime epoch, owner/workspace/admission reservations, task records, and terminal pruning. The normalized `x-uar-workspace-id` participates in both the admission key and canonical digest, and every REST reconciliation/control request must present the same workspace. The epoch is generated once at startup and embedded in task IDs. Terminal records use the configured run terminal TTL and cap. Admission tombstones retain the task ID and digest after a task receipt expires so an exact key cannot execute again and reconciliation returns `retention_expired`. Alternatives that store records in the existing persistence layer were rejected because C06 owns migrations and restart recovery.

`GET /api/uar/full-harness/v1/capabilities` exposes the current epoch, retention, unsupported-after-restart recovery posture, and unsupported steer posture. The general UAR capability vocabulary advertises `full_harness_delegation_v1` only with this authority present. A client with only an admission ID compares the current epoch to its stored admission receipt before interpreting `task_unresolved`; it never retries execution solely because the prior epoch disappeared.

### Reserve before calling shared run assembly

`POST /api/uar/full-harness/v1/tasks` accepts `admission_id`, `native_task_id`, and the existing create-run fields. It canonicalizes the deserialized request, atomically reserves `(owner, admission_id, digest, task_id, run_id)`, then calls the same run-assembly function as `/api/uar/runs` with the reserved run ID. Exact retries return the stored receipt; digest mismatches return HTTP 409. This closes the lost-response duplicate-execution window without duplicating the loop.

### Freeze the wire contract

Admission request fields are `admission_id: string`, `native_task_id: string`, plus the existing create-run fields: `artifact`, `agent_id`, `deployment_binding_id`, `service_placement`, `input`, `session_id`, `run_credentials`, `mcp_servers`, `tool_admission`, `working_directory`, `reasoning_effort`, `history`, `skill_attachments`, and presentation-negotiation fields.

Task and admission responses use snake_case and return `admission_id`, `task_id`, `native_task_id`, `run_id`, `agent_id`, `workspace_id`, `runtime_epoch`, `revision`, `cursor`, `state`, `retention`, `effective_service_binding`, `diagnostics`, `cancellation`, `detached`, `detached_observers`, `created_at`, `terminal_at`, `expires_at`, `unsupported_semantics`, and `links`. Approval, cancellation, and detach bodies carry `expected_revision`; approval also carries the exact required non-empty `approval_id`, and detach carries the required `observer_id`. `retention` contains `mode: "process_ephemeral"`, `terminal_ttl_seconds`, and `terminal_record_cap`; a zero cap follows the run manager convention and disables cap eviction. `cancellation` contains `requested`, `acknowledged`, `terminal`, and `cleanup_uncertain`. Links identify status, stream, tool-approval, cancel, detach, and steer endpoints.

Errors use `{ "error": { "code", "message", "task_id?", "admission_id?" } }`. Stable codes include `admission_digest_conflict`, `recovery_unsupported`, `retention_expired`, `task_unresolved`, `task_not_found`, `revision_conflict`, `approval_unresolved`, `capability_unsupported`, and existing run-admission codes. A refused admission remains queryable as a `rejected` receipt with diagnostics, while its initial response and exact retry preserve the non-success error envelope.

### Use a non-cancelling observer

`GET /tasks/{task_id}/stream` replays retained `RunManager` history and follows its broadcast receiver without installing the native route's disconnect guard. It preserves existing normalized event IDs and typed stream-gap behavior. Observation never owns run lifetime.

### Project full-harness records through A2A

A2A `tasks/get` and `tasks/cancel` first recognize the full-harness task-ID namespace and delegate to the shared authority. JSON-RPC requires `workspace_id` in those params and `expected_revision` for cancellation. gRPC requires `x-uar-workspace-id` metadata and `x-uar-expected-revision` for cancellation; absent control context is refused. The projection uses the existing A2A `Task` shape; legacy A2A IDs continue through `A2AThreadService`. This adds one lookup authority for new delegated runs while preserving old clients.

### Report cancellation rather than infer completion

Cancel records request time and the run manager's acknowledgement. A monitor of the existing event stream records terminal state; an acknowledged request is never reported as terminal by itself. Cleanup uncertainty is set only from an explicit runtime cleanup-unconfirmed signal. Detach only updates task metadata. Steer returns HTTP 422 with `capability_unsupported`.

## Risks / Trade-offs

- [Process restart loses records] → Encode runtime epoch and return `recovery_unsupported` for another epoch; reserve `retention_expired` for TTL/cap pruning and make retention mode explicit.
- [A terminal event could be pruned before reconciliation] → Monitor each admitted run and snapshot terminal state in the task authority, then apply the declared TTL/cap.
- [Canonical request contains credential material] → Digest in memory only; never log, serialize into receipts, or persist request bytes or digest inputs.

## Migration Plan

Ship the new routes and authority additively. Existing `/api/uar/runs`, model-provider APIs, and legacy A2A tasks retain their behavior. Rollback removes the new versioned surface; outstanding process-ephemeral task records then become unavailable, which is consistent with the advertised retention class.
