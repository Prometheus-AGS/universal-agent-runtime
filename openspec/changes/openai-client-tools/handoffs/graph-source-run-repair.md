# Graph child pre-admission history repair

Task 13.24 / CT4, Execute. Verification CLOSED. Source inspection and authorized
implementation/documentation only; no checks, tests, formatting, compiler probes,
builds, imports, network calls or runtime commands ran by this teammate.

## Observed failure and exact source cause

Gate012 passes the core/safety/cancellation and ordinary-cold cohorts but all
five warm graph cases fail worker-contact count. `candidate-2.log` records real
spawn approval followed by `AgentNode 'general-purpose' failed:
source_run_not_found`. Root `dcc6da9b-6b87-4668-88a2-34c326813a83` has admitted
child run `1965d856-d083-46bc-b37a-1b013449611c`, but no worker provider call.
Lead reports all five equivalent and cleanup joined exit 0.

The 13.21 thread kernel set `request.inherited_history=None` whenever captured
caller-tool input exists, intending to keep canonical caller history above
generic forked parent history. Manager's trusted-child validation at
`manager.rs:4080–4096` unconditionally requires `inherited_history.is_some()`.
Consequently this path emits `child_bindings_unavailable` and returns before
the manager active-run registration, preserving a Failed mailbox completion.

Graph delegation then attempted `await_settled_run_status(child_run)` before
examining that actual child result. Its manager lookup fails
`source_run_not_found`, masking the captured pre-admission failure. No missing
approval, provider support assumption, or cleanup timeout is needed to explain
this source path.

## Files changed

- `src/uar/runtime/thread/kernel.rs`: always supply inherited history for the
  trusted-child contract. For captured caller input, deserialize its prepared
  canonical `input.history` into `Vec<Message>` with errors propagated; for
  ordinary children retain `turn.messages`. Captured canonical history still
  wins over generic fork history and stays tool-paired. Existing manager
  admission requirements remain unchanged.
- `src/uar/runtime/graph/delegation.rs`: request a caller handoff only for an
  actually Completed child outcome. Failed/canceled/yielded/missing outcomes
  return no handoff so AgentNode's existing outcome handling exposes the real
  terminal result. Completed handoffs retain their actual child settlement and
  exact durable root/child correlation checks. Existing parent/thread cleanup,
  governance and approval ownership are unchanged.

No control/service/manager/orchestrator/persistence/API file changed in this
cohort. This handoff is the only additional file. The history requirement is
an existing trusted-child identity boundary; no unrelated guards or fallbacks
were added. The result check addresses the observed error masking and never
fabricates an empty successful handoff.

## Remaining evidence

Source repair only. Compilation, real worker contact, paired-history continuation,
all five graph protocols, cold graph recovery and cleanup of this changed source
remain unverified. Lead owns isolated source review and the next authorized
whole-phase local gate. Gate012 acceptance does not certify these later edits.
