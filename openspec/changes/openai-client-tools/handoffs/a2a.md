# A2A caller-tools handoff

Phase: Execute. Verification lock: CLOSED. Source implementation is in progress;
no tests, compiler, formatting, lint, builds, packaging or review gates ran.

## Contract

The private extension URI and metadata key is
`https://prometheus.dev/a2a/extensions/client-tools/v1`.
Its request object contains `tools` (OpenAI function schemas), `tool_choice`
(provider-independent `{type: auto|none|required|named, name?: string}`),
`parallel_tool_calls` and `history` (canonical provider-independent messages).
Assistant calls retain IDs, function names and serialized arguments; result messages
use role `tool`, `tool_call_id` and `content`. The extension grants no host execution.

Official A2A 1.0 JSON-RPC and gRPC already normalize request metadata into
`MessageSendParams`; their core schemas and state enums need no private additions.
The shared task service owns full-principal/agent/task/context correlation.
Caller results require their exact task and context. Approval decisions remain a
separate message, using their existing protected continuation.

## Files owned

- `src/uar/api/a2a/client_tools.rs`: extension parser and common-input conversion.
- `src/uar/api/a2a/mod.rs`: module export.
- `src/uar/api/a2a/thread_service.rs`: declaration ingress and continuation dispatch.
- `src/uar/runtime/actor/system.rs`: compatible caller-tools submission wrapper.
- `src/uar/runtime/actor/messages.rs`: explicit caller-tools mailbox field.
- `src/uar/runtime/actor/agent_actor.rs`: mailbox-to-host forwarding.
- `src/uar/runtime/thread/actor_host.rs`: compatible root request wrapper.

## Outstanding integration

The common persistence/API service is being authored by the persistence teammate.
A2A still needs its exact pending-batch projection and continuation prepare/claim/replay
APIs. Do not advertise the extension until that path is wired. Parent owns
`routes.rs` and its `ResumeRequest` propagation. Existing checkpoint-only resumes
cannot independently prove caller-tool continuation safety.

No acceptance evidence exists while the lock remains CLOSED. Source inspection
commands only were used. No dependency/version changes or commits were made.

## Implemented source integration

- Actor ingress now retains `ClientToolInput`, including trusted `visible_history`,
  through mailbox submission and `RunExecutionRequest`. Existing submissions keep
  their compatible wrappers and supply no caller declarations.
- Durable A2A 1.0 declarations are normalized and validated. Empty extension
  history starts with the current user text; a supplied history is the complete
  canonical input. The runtime does not append the actor placeholder prompt.
- Result messages require exact task/context and source-run batch identity. The
  current authenticated delegation contract is revalidated before claim. Shared
  `api::client_tools::prepare` restores the host transcript, then the same task
  submits a new actor root. `complete` stores the returned task receipt before
  emission. A crash after claim remains fail-closed; it never re-executes a batch.
- Restart continuation reconstructs an actor entry from its owner-qualified durable
  binding. It reloads the current binding after task projection to avoid using a
  revision superseded by that projection.
- Durable pending rows override both task reads and completion persistence to
  `InputRequired`. Metadata includes batch ID, pending calls, exact visible
  history and declaration binding, enabling callers to produce the exact suffix.
- Official discovery advertises the extension only when its task service has
  the run API and both durable storage capabilities. Legacy interfaces explicitly
  reject the extension because they lack the durable continuation contract.

`src/uar/api/a2a/v1.rs` also changed for conditional discovery. The common service,
persistence providers, manager, and runtime are owned by other teammates; their
integration is an explicit dependency. Source state has no compiler or behavioral
acceptance claim. No new guards unrelated to observed caller-result identity,
replay, owner/task isolation or durable storage boundaries were introduced. No
unrequested dependency, version, workflow, or build changes were added.

## Cohort 11.1: settlement precedes caller handoff

The presence of a durable Pending caller batch is now treated as pre-handoff
state. Both owner-qualified `get_durable` projection and actor-entry persistence
call `RunManager::await_settled_run_status` for the exact bound source run before
publishing `InputRequired` or pending-call metadata. The manager joins the admitted
source producer and graph root, including outer cleanup; no transport timeout or
background-detach fallback was added.

Only settled `RunStatus::Done` publishes caller action. Settled `Error` projects
`Failed`, and settled `Cancelled` projects `Canceled`; both remove caller-tool
metadata. Unconfirmed settlement propagates the host failure and exposes no pending
caller action. The common prepare call sites retain the parent-added manager
argument, so batch claiming independently enforces the same settled-source boundary.
Source implementation only. The verification lock remains CLOSED; no check ran.

The settlement projection now records the scoped, monotonic durable acknowledgment
through `api::client_tools::confirm_handoff_for_run` before exposing caller action.
A live predecessor still joins its producer/graph and preserves Error/Cancelled
outcomes. When the live predecessor is absent after restart, the helper alone may
accept its existing durable settlement acknowledgment; an unacknowledged cold
Pending row fails closed. No terminal run-history record is used as cleanup proof.
Verification remains CLOSED; no checks ran for this source change.
