# Durable local runtime and recovery

## Ownership

UAR owns TeamInstance, AgentInstance, TeamTask, TaskAttempt, InboxMessage, TaskLease and TeamEvent. A collaboration service admits bounded turns through the existing kernel; it does not introduce another model/tool execution loop. The Boss presents and controls this state through its trusted host bridge. BossFang retains ownership of a workflow that delegates a UAR task. Fabric transports events, scoped memory supplies context, and neither schedules UAR tasks.

## Task state machine

| Current state | Allowed next state | Required reason |
|---|---|---|
| queued | ready, failed, cancelled | Dependencies resolved; unrecoverable validation; cancellation before effects |
| ready | running, awaiting_approval, failed, cancelled | Atomic admission; approval prerequisite; admission failure; cancellation |
| running | ready, awaiting_input, awaiting_approval, reconciling, succeeded, failed, cancelled | Bounded turn yields/retries; waiting; uncertainty; settled result |
| awaiting_input / awaiting_approval | ready, reconciling, failed, cancelled | New input/current authorization; unresolved effect; denial; cancellation |
| reconciling | ready, succeeded, failed, cancelled | Recorded evidence settles effect/ownership uncertainty |
| succeeded / failed / cancelled | none | Terminal task states are immutable |

An attempt can fail and permit a new attempt while its task remains nonterminal. Retrying a terminal task requires a new task linked to the original; this resolves the distinction between immutable terminal tasks and repeated attempts. CancelRequested is a separate durable flag, not immediate terminal success. A cancelled task MUST have no unsettled owned effect.

Team/member lifecycle is inactive, ready, active, suspended, draining or stopped. Suspend rejects new execution while retaining state; drain stops new work and settles active attempts; stop requires a declared drain/cancel policy. Restart restores identity but never resurrects old run authority. Closing a client or stream does not stop a team.

## Admission and claims

The store transaction checks task revision/owner/workspace, claims the next ownership epoch, reserves the root budget and writes the dispatch intent/outbox event atomically. Only one mutating turn may be active per AgentInstance. A worker MUST present the current epoch at state commit and protected effect admission. Expiry or supersession fences old owners. Task publication is committed before dispatch; dispatch acknowledgment is separately recorded.

A crash before commit leaves no claimed work. A crash after commit before dispatch leaves a replayable outbox intent. A crash after dispatch with missing result enters recovery; it does not blindly dispatch again. The selected backend MUST prove CAS, uniqueness, atomic state/outbox and durable commit behavior at I2. A backend lacking them cannot advertise durable local teams. This document does not certify any configured database.

## Messaging and waits

Commands/messages have stable idempotency IDs scoped to authenticated owner and operation. Reuse with different content is a visible conflict. The durable inbox commit precedes the accepted receipt; delivered means made available to the recipient; processed means consumed by a turn. None means task completion. A terminal task event proves completion.

Queue-only messages do not activate idle agents. Triggering task delivery schedules a fresh admitted turn. Active agents receive queued input at kernel-supported boundaries; delivery cannot mutate another agent's current private context in place. Broadcast captures the authorized recipient set and produces per-recipient receipts; later membership cannot widen a prior broadcast. Replays recheck disclosure authorization.

Joins persist dependency IDs and conditions. Waiting coordinators release execution slots; wake-up coalesces repeated notifications into one eligible turn. Child/subteam outputs belong to their task contracts and are joined only under authorized context projection. A stopped root run does not erase the durable team inbox; the collaboration layer starts fresh bounded runs rather than modifying the existing kernel's terminal-root authority rules.

## Scheduling, routing and budgets

Eligibility precedes ranking: owner/workspace, required capabilities, skill/tool availability, context grants, supported model and current policy/budget. Then prefer explicit operator choice, declared role, available capacity, cost class and stable ID. A model may suggest a routing decision; only UAR admission can claim work. Overrides cannot bypass eligibility. Persist safe exclusion reasons and binding/model revisions.

Admission is round-robin across owners and then teams, FIFO within priority classes. Interactive cancel/status bypass model execution slots. Implementations MUST define bounded aging or quotas so priority classes do not starve lower-priority admitted work; policy changes are inspectable. Default host ceilings: four concurrent turns/team, eight overall, sixteen members across a root graph, nesting depth three, 1,000 queued tasks/team. Descendants may narrow ceilings only. Existing kernel limits also apply.

Reserve aggregate token/cost/time ceilings before dispatch; reconcile actual usage and release unused reservation after settlement. Unknown provider usage retains a conservative reservation until resolved or an authorized ledger adjustment is recorded. No pricing estimate grants unlimited spending. At exhaustion, stop new admissions and settle/cancel active work according to current policy. Resource limits and queued reasons remain visible.

## Recovery and uncertainty

Recovery loads committed instance/binding/task/inbox state, fences old attempts, reconciles effects and creates fresh admissions. Pending approval intent can survive; obsolete live-run approval tokens cannot. Recheck current policy, grant revocation and exact tool arguments after a wait. A read-only artifact may be reused if still authorized; a lost issue-create response requires connector lookup/manual reconciliation with the original effect identity.

Uncertain effect outcomes remain reconciling until evidence or an authorized operator decision resolves them. Record who decided, evidence and residual risk. Do not promise exactly-once remote effects. Automatic retry is permitted only when the effect's actual idempotency contract or reconciliation result proves it safe. Cancellation after an external effect may require reporting the completed effect; it cannot undo it by changing local state.

## Capacity targets

I2 measures an 8-core/16-GiB host with32 idle teams,8 real-provider turns,1,000 queued tasks and100,000 retained events. Targets: local control API p95<250ms; committed state visibility p95<500ms; replay1,000 retained events<2s; cancel admission<250ms; restore32 idle teams<5s. Measure provider latency separately and record backend/payload/hardware. These are approved targets, not measured results. A missed target requires an explicit decision rather than a silent change to the published claim.
