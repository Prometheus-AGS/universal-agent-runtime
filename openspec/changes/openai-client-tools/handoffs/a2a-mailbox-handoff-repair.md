# A2A caller-tool handoff and registration ordering repair

Phase: Execute. This teammate performed source inspection and authorized writing
only. No execution, tests, checks, imports, probes, builds, or network calls ran.

## Observed failures and source causes

Gate007 matrix records A2A 9 passing and 10 failing scenarios. The nonstreaming
loop, two-round, and parallel-false result submissions receive task-state
conflicts. Streaming emits Working then stream-unavailable. The required case
returns Internal error. Shutdown retains three `a2a-durable-projection` tasks
after actors released and is force-killed; cleanup is not verified.

Two source defects are demonstrated:

1. `subscribe_durable` attempted run subscription immediately after actor enqueue,
   then treated absent run events as a conflict. Actor enqueue precedes manager
   run registration (already documented by the durable projection observer).
   Thus no channel yet is not evidence that an observation channel was lost.
2. Local task projection could advertise pending caller-tool InputRequired after
   manager producer settlement while `Entry.turn` still retained the mailbox
   completion. `resume_client_tools` and `send_internal` reject that exact state
   because another actor turn remains live. The advertised handoff was earlier
   than the actual local actor handoff boundary.

## Change inventory

Only `src/uar/api/a2a/thread_service.rs` changed in this cohort:

- `persist_entry_projection` projects a pending caller batch only after
  `entry.turn` is absent. While it remains live the task stays Working and has
  no public caller-tool extension. This also avoids invoking manager producer
  join while the Entry lock is held before the actor mailbox is consumed.
- `get_durable` uses the local get's caller-tool projection, rather than
  independently overriding a still-working local snapshot with InputRequired.
  Cold-process projection retains the existing durable acknowledgment path.
- The owning completion observer publishes a committed-work notification only
  after actual mailbox consumption and successful settled projection. It does
  not notify on every pass, preventing a notification feedback loop.
- `subscribe_durable` installs retained registration and committed-work receivers
  before initial run lookup, waits registration/settlement when the actor has
  not yet registered, and subscribes to the exact run once available.
- `wait_durable_until_settled` has the same retained-registration ordering.

Actual closed source/lag errors remain errors. The prior lifecycle-admission
ordering and completion-receiver restoration remain intact. No InputRequired
state is fabricated, no durable claim bypassed, and no stream failure relabeled
as successful completion. JSON-RPC streaming code was inspected but unchanged.

## Remaining uncertainty

The exact source of gate007's three shutdown-observer stalls is not established.
The earlier premature projection could await producer join and root shutdown
under Entry lock before actor mailbox completion. The new ordering removes
that path, but source inspection alone cannot attribute the observed forced
shutdown to that path. Required-case Internal error is likewise not proven to
share a cause beyond the observed lifecycle/projection state. Do not claim all
ten failures fixed from this source edit.

Compilation, A2A real continuation and concurrent replay, streaming event order,
required-choice behavior, and graceful resource teardown remain unverified.
Lead owns independent source review and the next explicitly authorized real
gate. This handoff is the only additional file; no unrequested guard was added.

## Gate008 correction: stale local binding revision

Gate008 still reports A2A 9 passing/10 failing. The earlier registration repair
allowed the streaming loop to reach handoff, but both loop continuations still
received `-32004`. Therefore the preceding section does not establish a complete
repair. Normal Completed handling does **not** set `entry.closed`; the branch
does so only when artifact evidence fails. A stopped actor mailbox would return
a host error, not the observed task conflict. Those hypotheses are not adopted.

The deterministic source defect is revision ownership:

1. `get_durable` invokes local `get`, whose `persist_entry_projection` commits
   a binding revision and synchronizes `Entry.durable`.
2. The same `get_durable` then adds `uar.approvals` to a separate task snapshot
   and CAS-commits another revision outside Entry, without updating the local
   expected binding revision or task metadata.
3. On the next read the local projection removes that metadata, then the outer
   projection adds it again. On continuation, `send_internal` instead clones
   the stale local revision. Persistence compares that expected binding with
   the current row and returns false, which is the explicit `TaskError::Conflict`
   / `-32004` path before actor dispatch. A durable result claim can already
   have been made at this point.

The new source correction remains entirely in `thread_service.rs`:

- Local approval metadata now commits under Entry lock through
  `persist_entry_projection`, sharing serialization with mailbox projection
  and new-turn admission. It updates Entry's task and expected revision together
  and avoids overwriting a newer local execution snapshot with the earlier
  out-of-lock snapshot. Typed pending-approval status remains supported.
- Before new-turn CAS, load the authoritative current same-owner/agent/task
  binding under Entry lock. Require unchanged run/process and no cancellation;
  preserve current context, contract, task history, and metadata before adding
  the new request. The actual CAS still arbitrates admission. Changed run,
  cancellation, or a genuine lost CAS still fails closed.
- Caller-tool results reach their locator lookup before the InputRequired
  admission guard. Completed receipts use the common exact-result/schema/history
  validation and read-only replay. Pending work still requires InputRequired,
  no local live turn, and exact source-run match **before** any claim.
- Batch lookup is full-principal/tenant/agent scoped. Its session must match
  the actual `ActorCollaboration::spawn_a2a_session` context derivation,
  `a2a-context-{A2aBinding::storage_key_for(owner,agent,context)}`. A replay's
  saved Task id and context must match the requested binding. A batch from
  another task cannot return its receipt under this task's identity.
- The projection observer checks lifecycle cancellation at loop top. Its CAS
  conflict path sets `first_snapshot=true`, previously skipping the select that
  contained its only cancellation check. Sustained conflicts could therefore
  starve shutdown. The check occurs between completed operations, never cancels
  an accepted write/join, and retains the outer completion receiver restoration.

Gate008 again retains three projection observers at manager drain. This matches
the named cancellation-starvation scenario, but logs contain no stacks mapping
those jobs to the conflict branch. Attribution remains unproven. The observed
negative-stream terminal GetTask host failures remain unlocalized; no catch,
default task, swallowed observation error, or invented success was added.

No other production file, manager helper, or JSON-RPC handler changed. No
verification or runtime execution ran in this correction. Lead owns independent
review and the next authorized real gate; compilation, successful continuation,
duplicate replay, negative-stream reads, and graceful shutdown remain unverified.

## Fresh-agent claim coordination

Added the lead-requested `resolve_registered_agent(agent)` lookup in
`resume_client_tools` after the non-completed batch's active-state/mailbox guards
and before shared `prepare` can claim it. A deleted registered agent therefore
fails before the durable claim. Completed receipt replay skips this lookup.
The actual fresh artifact/policy handling before effects belongs to the runtime
writer's `actor_host.rs` change; this teammate did not edit that file or add
another policy rule. No execution or verification ran. This is the final owned
source edit for this coordination packet.

## Build009 compiler correction

Lead reports build009 exited 101 with E0631 at the fresh-agent lookup: its
`AgentStoreError` could not be passed directly to the `TaskError::Host` variant,
which accepts `anyhow::Error`. Changed only that conversion to
`map_err(|error| TaskError::Host(error.into()))`. No behavior or additional guard
changed. This teammate ran no compiler, check, test, or other verification;
successful compilation remains for the lead's next authorized build.

## Gate009: mixed transcript projection root cause

Lead reports gate009 90/95, A2A 14/19 and clean candidate join exit 0. Earlier
shutdown corrections therefore have real clean-exit evidence for this gate;
that does not certify subsequent source edits.

The two race/disconnect scenarios failed **before** racing or disconnecting.
Their `prepared_results` helper configures a mixed host/caller turn when a real
host tool is supplied. Raw captures returned HTTP 408 after 30 seconds, not a
generic A2A host error. Candidate run
`bceb420c-a9ae-4bb5-ab68-ae31561a8f93` reached provider execution, successful real
MCP host effect, and MCP shutdown at 19:51:42.813 before the timeout.

Exact source cause: `project_settled_client_tools` read pending calls from
`batch.canonical_transcript.last().tool_calls`. Orchestrator lines 3250–3289
append executed host-tool results to the canonical transcript before persisting
the pending batch. Thus mixed canonical history ends in a tool result, and this
projection always returned Conflict. The owning observer interpreted Conflict
as a lost CAS and immediately retried. Public handoff never became readable.
Pure caller turns end in assistant calls, explaining the seven passing ordinary
A2A cases before the mixed scenarios.

Production edit in `thread_service.rs` only: project public calls from the final
assistant record in `visible_transcript`, then require the exact unique ID set
and cardinality to equal `pending_call_ids`. Preserve canonical hidden host
calls/results unchanged. Malformed public evidence returns a static host error,
not a retryable CAS conflict; neither raw payload nor SQL is logged. These
checks protect the actual caller/host disclosure boundary encountered here.

The negative stream pair has a distinct fixture interpretation defect. Raw A2A
responses show successful JSON-RPC GetTask results with `TASK_STATE_FAILED` and
`Agent execution failed`, not GetTask Host errors. Driver.send(stream=True)
uses `require(not failed(current), "A2A stream terminal task read failed")` after
GetTask. `failed` includes a legitimate terminal failed-task result, so this
raises before `provider_failure_case` can assert the expected public failure.
Lead was asked to have the setup owner distinguish RPC/HTTP read failure from
the returned failed/canceled task state and preserve that state for the negative
assertion. No production catch, forged task, or speculative diagnostic was
added for this case.

The mixed projection correction is source-only and unverified. No test, build,
check, import, query probe, network call, or runtime command ran by this teammate.
Lead owns independent source review and the next authorized integration gate.
