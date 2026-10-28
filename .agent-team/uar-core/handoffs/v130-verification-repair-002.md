# v1.3.0 verification fixture repair002 handoff

Phase: `uar-post-lifecycle-verification-readiness`; Execute. Operator gate: CLOSED.
Authorized scope: remaining repair2of2 source and authority authoring, then show
exact approvals. No remaining automatic repair allowance. This handoff grants no
verification, signing, adoption, publication, paid call, or remote-action authority.

## Files and source identity

- `tests/post_lifecycle_main.rs`: 39380 bytes; SHA-256 e3398d86968b210d7061b3fd767baa00f6b504447339ba1bba28fa5ed8869272.
- `.agent-team/uar-core/handoffs/v130-verification-repair-002.md`: this handoff.
- Existing repair001 durability import/match changes were already present and
  preserved. No Python, production API, manifest, contract, inventory, canonical
  projection, or evidence files were edited by this writer.

## Observed failure and minimal repair

Retained packet002 receipts under
`.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-002/`
show every case BLOCKED with `request_transport_failed`; teardown contains one
owned peer launch, no main launch, peer exitCode null/success false after a
successful SIGTERM command, five `request_transport_failed` cleanup failures,
and process-count mismatch. Source previously used `self.request(...).await?`
inside every poll, ending startup on its first transport failure despite its
manifest attempt budget. The receipts do not prove a startup timing root cause.
The lead separately observed stale peer output paths pointing into001 and owns
binding new outputs to fresh003; this writer did not edit those paths.

Exact new poll flag: `startupTransportRetry: true`. The lead confirmed it will
appear only on two setup readiness GET polls, preserving15peer/30main attempts
and existing1000ms sleeps. The driver passes an explicit setup context only
while executing setupSteps. Opt-in is rejected outside setup and for methods
other than GET. The expanded request determines main/peer routing, and that exact
target must be in the owned child map. Before and after each opted-in request,
`Child::try_wait` retains early exit/wait-error receipts and immediately blocks
startup, including an early zero exit. Each request error retains process,
attempt, attempt bound, control, and exact error classification. Only exact
`request_transport_failed` retries; request_timeout, body errors, malformed
requests, and other failures still propagate. Runtime/scenario/restart polls and
all unflagged requests retain immediate transport failure. No attempt, sleep,
HTTP timeout, restart budget, probe, or provider call was added.

Cleanup uses successful launch history, never current process-map absence.
Only direct main-target request cleanup steps are skipped when main has never
launched, with explicit `cleanup_step` / `NOT_APPLICABLE` /
`main_never_launched` events retaining method and path. The five reviewed DELETE
steps therefore cannot issue HTTP calls for resources this attempt could not
create. If main was ever launched, every existing cleanup obligation remains.
Setup failure is retained explicitly in teardown, case failures remain BLOCKED,
and process-count mismatch remains BLOCKED. No failed outcome becomes PASS.

Owned SIGTERM, awaited child exit,30-second grace, nonzero exit rejection, and
forced-exit failure criteria remain unchanged. Exit receipts now retain PID,
control-command success, and formatted exit status in addition to exit code,
including after forced cleanup; a null exit code no longer discards signal
status. The existing Python peer installs SIGTERM handlers and performs
server_close with joined request threads. It was inspected and not edited.
Graceful shutdown remains a runtime claim requiring later authorized evidence.

## Guards and limits

The retry opt-in responds to the observed first-error startup abort and is
limited by setup stage, GET method, existing child ownership and attempt bounds.
The cleanup guard responds to the five observed requests after no main launch.
Early child-exit receipts prevent the bounded retry from concealing a child
failure; richer teardown receipts address the observed null exit code. No
unrequested guard or adjacent cleanup was added. Failed startup can still
leave teardown BLOCKED; this repair deliberately retains that evidence.

## Commands and unverified work

All commands were classified as source inspection or authorized source/document
writing. Inspections used `cat`, `sed`, `rg`, `rg --files`, and
`git diff -- tests/post_lifecycle_main.rs` (plain source diff only). Filesystem
reads inspected the waypoint, AGENTS.md, versions.toml, Cargo.toml, Rust rules,
gotchas, Rust router/workspace/best-practices/async references, packet002 retained
receipts, prior manifest setup/cleanup, and the existing peer shutdown code.
`apply_patch` authored the Rust repair; Python authored this handoff and recorded
the source bytes/hash above without importing or executing project code.

No test, lint, formatter, compiler, Cargo command, build, probe, policy validator,
automated review/screening, diff check, signing, adoption, or publication ran.
No new tests were authored. Compilation, startup success, runtime behavior,
resource cleanup and graceful child exit remain unverified under CLOSED.
Mandatory independent acceptance/review is deferred under the operator lock.

## Lead continuation

Root owns the new manifest, task-specific spec/contract documentation, refreshed
whole-phase inventory, authority reconciliation and approval packet. Bind the
exact source identity above; enable the agreed flag only on the two setup polls;
retain original budgets and strict exit criteria. Reconcile the complete ledger
and waypoint before presenting exact proposed commands to the operator. Wait
for explicit named command approval. Any subsequent edit requires a renewed
inventory. No automatic repair remains after this repair2of2; a future failure
must be retained and surfaced for operator direction.
