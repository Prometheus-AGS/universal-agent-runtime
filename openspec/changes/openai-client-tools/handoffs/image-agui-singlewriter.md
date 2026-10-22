# Caller-tool handoff acknowledgment serialization — task 13.30

Source-only implementation; verification gate CLOSED. No tests, checks, builds,
scripts, network calls, or runtime probes were executed by this writer.

## Observed failure and scope

Published-image gate001 passed 94/95 core cases. The AG-UI partial-rejection
setup emitted `host_cleanup_unconfirmed` for native source run
`f936321b-4e89-4b15-aafc-3b035e6c1830`. A separate mixed host/caller case for
`7fcf1355-d842-4776-9f6c-5720496e3286` logged the same confirmation failure while
another observer published success for the same calls. Competing local
confirmers disagreeing is observed; the original internal failure stage is not
proven. Native gate018 subsequently passed all nine groups without a stage warning.

## Files changed

- `src/uar/runtime/manager.rs`: each of the three `RunStreamState` construction
  paths now creates a lifecycle-owned `Arc<Mutex<()>>`. The crate-private
  `caller_handoff_confirmation_lock` helper clones that run's lock while reading
  the active-run map, without holding the map lock while waiting for confirmation.
- `src/uar/api/client_tools.rs`: `confirm_handoff_for_run` obtains an owned guard
  before loading the durable continuation and retains it through validation,
  actual producer/graph joins, settlement CAS, and the losing-CAS reread.

The existing authentication/binding, validation, joins, CAS, fixed-stage logging,
and original error text remain in force. If no live source exists, a durable row
already marked `source_settled` still succeeds; an unsettled row still reaches
the existing `source_run_not_found` failure rather than acquiring acknowledgment.
No retries, fake settlement, additional tasks, or persistence changes were added.

## Mechanism and uncomfortable limits

This serializes cooperating local confirmers for one retained source run. It does
not prove an upstream storage defect. The proposed SurrealKV oracle-publication
versus read-visibility explanation remains an inference, not an observed stage:
<https://github.com/surrealdb/surrealkv/blob/v0.21.4/src/commit.rs>.

Cancellation drops the request's mutex guard. Scoped persistence may retain an
already-dispatched CAS after that drop, so the guard does not promise serialization
through an unknown canceled operation's eventual outcome. Durable CAS and the
existing fail-closed reread remain authoritative. This adds no cross-process lock
and makes no stronger exactly-once claim.

Compilation and production-path behavior of this patch remain unverified. The
lead owns independent source review, renewed inventory, and any authorized gate.
