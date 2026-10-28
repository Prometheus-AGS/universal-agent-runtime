# Serial failure collection and bounded corrective continuation

SOURCE AUTHORED ONLY. Operator verification gate CLOSED. This controller and its
tests have not been imported, executed, syntax checked, or run against services.
The active phase is `uar-post-lifecycle-verification-readiness`; this implements
plan task 2.21. No text in this document grants execution or correction authority.

The existing runner keeps first-failure behavior unless the exact approved argv
also includes `--collect-results`. Its existing `--execute`, `--fixture` and
`--evidence` arguments remain required. Collection refuses `--case`: its inventory
always includes all 97 cases, three repetitions, and both durable backends (582
rows). Supporting gates and platform applicability remain separate acceptance
requirements; collection always reports `phase_acceptance: false`.

The uncomfortable constraint: an isolated 582-item sweep starts and cleans a
fresh server/store for every item. This costs more time and startup resources
than the compatibility path. Approvals must budget it explicitly. The controller
never widens an allowance to finish the denominator. A later corrected PASS also
cannot certify earlier results produced by different source/artifact bytes.

## Frozen fixture contract

Add `failure_sweep.py` to `frozen_files` and this object to the composed fixture:

```json
{
  "failure_sweep": {
    "schema_version": 1,
    "prerequisites": {
      "U01.configured-prompt": [],
      "U02.session-continuation": []
    }
  }
}
```

The example deliberately shows only the shape. An executable contract requires
one explicit list for **every** one of the 97 IDs in `required_cases`. Each list
may contain only unique earlier manifest IDs; forward references and cycles are
refused before startup. An empty list declares that the case constructs its own
scenario resources. The currently authored cases call their own configuration
and scenario setup helpers; U02, U07, all U05 profiles, U24, team, knowledge and
capability cases create their own conversations/resources rather than consuming
another case's outputs. Do not add U01 as a blanket dependency: its useful-answer
failure does not establish that another scenario is unavailable. Inference,
provenance, successful terminals, configuration and genuine binding assertions
inside helpers remain intact and are checked on every actual case invocation.

Dependencies refer to the same backend and same repetition. For example,
`U02.session-continuation` depending on `U01.configured-prompt` produces the
concrete prerequisite ID `postgres_pgvector:U01.configured-prompt:1` for its first
Postgres repetition. A failed/blocked prerequisite blocks only its dependents;
other repetitions and the other backend remain independent. Dependency outcomes
include source/artifact identity. A prior-source PASS cannot admit a dependent
on refreshed source. Each attempt also records `packet` and its own
`admission:<backend>` prerequisite outcomes. Admission failure blocks the
remaining items of that backend, then continues the independent backend only
following confirmed owned cleanup.

## Execution and durable evidence

One `PacketProcesses` owner starts once and remains live until sweep termination.
No proxy restart, ledger reinitialization, or epoch reset occurs during a wait or
correction. HTTP/output counters and wall time carry across every item and its
cleanup. The proxy retains its existing durable spend/attempt/token reservations.

Each item receives a distinct `Context` run UUID and empty owned store. The case
retains its manifest repetition number, while `revision_attempt` distinguishes
initial execution (`0`) from its sole corrective rerun (`1`). Failed case state
never becomes the next case's runtime. The exact cleanup observer must confirm
joined owned processes and zero residuals before any continuation, including a
correction wait. Unknown cleanup halts the whole packet because the host/ports
are shared. Exhausted shared resources or unknown packet ownership also halt
independent dispatch. Unstarted rows remain `NOT_RUN` in the original inventory.

After exact-owned item cleanup, collection also waits for every shared forwarding
reservation to leave `reserved`, before a correction request, rerun, or independent
item can proceed. The same gate runs before each dispatch, including the first
dispatch with a borrowed suite owner. `PacketProcesses.settle` waits at most 90
seconds, further bounded by the original collection/packet wall allowance and
absolute epoch deadline. Its callback retains shared request/output ceilings;
the packet Context retains process-output bounds. It sends no HTTP request.

The read-only SQLite transaction reads metadata and reservations from one view.
It binds the frozen proxy source/config, unchanged ready receipt, live owned PID,
nonce, seed, input identity, epoch, and cumulative totals/provider pools. Totals
are reconciled from the existing seed plus every retained reservation. Successful
responses require the proxy's actual usage predicates. A known terminal failed
HTTP response, including an intentional 401, needs no successful-usage envelope
and retains its full reservation, matching proxy settlement semantics. Unknown
accounting, inconsistent totals/ownership, exceeded allowance, or an unconfirmed
drain stops the packet. Exact saturation is still a known settlement; existing
collection limits and proxy admission ceilings govern further work. No ledger
write, refund, retry, redispatch, deletion,
restart, or allowance reset occurs in this gate.

The attempt's original terminal receipt, cleanup confirmation, and initial failure
reference are written before post-cleanup settlement. A settlement failure appends
`sweep-settlement-failure` referencing that terminal receipt, followed by the
packet safety stop. It never replaces the original case failure. Context creation
failure does not falsely report cleanup confirmation.

`Evidence.write` appends private exclusive receipts, fsyncs them, and chains
hashes; the existing runner seals the final manifest. Receipts include the full
inventory, attempt starts, terminal attempts, failed receipt references,
correction requests/consumption, refresh identities, unresolved items, safety
stops and the aggregate. Prior failed attempts remain in `results`/receipts when
a correction succeeds; they are never overwritten on disk. Final inventory
rows represent the latest admitted result of each planned item. Exit codes are
0 PASS, 1 unresolved product FAIL, and 2 BLOCKED/safety stop. Mixed passing source
or artifact identities force BLOCKED and `coherent_full_rerun_required: true`.
A failed aggregate remains FAIL even when it also contains blocked dependents.

## Optional correction authority and live host receipt

Without correction authority, FAIL is logged unresolved and the controller
continues independent items after cleanup. There is no silent same-source retry.
A BLOCKED prerequisite does not receive a product repair slot.

To permit one targeted correction and one same-item rerun per failed item,
`failure_sweep.correction_authority` must reference `{path,sha256}` of an actual
operator approval JSON already present in `frozen_files`. The exact gate approval
must adopt this authority and the trusted host writer/effects. The private,
owner-owned, regular nonsymlink approval contains:

| Field | Meaning |
| --- | --- |
| schema_version | `1` |
| operator_approved | `true`, accompanied by actual external operator adoption |
| authority_id | Nonempty unique approval identity |
| source_identity / artifact_identity | Initial fixture identities |
| epoch_id | The same live budget proxy epoch |
| max_corrections | Integer global correction cap, 1 through 582 |
| wait_seconds | Positive finite per-failure wait, at most 3600 seconds, further limited by shared wall allowance |
| receipt_directory | Existing private owner-owned absolute nonsymlink host inbox |
| refreshable_paths | Nonempty unique list of existing frozen paths the host may refresh |

A JSON flag or self seal is not a signature and does not grant authority by
itself. Frozen byte references plus exact operator adoption establish the local
trust boundary. The controller does not run an agent, repair, build, shell, source
inventory command, artifact preparer, or arbitrary correction command. A trusted
host may perform only separately authorized actions, including any required
build; the phase lock still applies to that host. If no such authority exists,
the failure remains unresolved.

After cleanup, `correction-request` publishes the failed item's ID, the exact
failure receipt `{path,sha256}`, the bounded wait and the expected inbox file
path. The filename is SHA256 of the UTF-8 concrete item ID, without the `sha256:`
prefix, plus `.json`. The trusted host publishes this file atomically, owner-owned
and read-only with no group/other permissions, only after a correction and
artifact refresh. It contains:

| Field | Meaning |
| --- | --- |
| schema_version | `1` |
| authority_id / run_id | Exact frozen authority and current sweep run identity |
| item_id | Exact concrete failed item ID |
| correction_number | `1` for this item |
| failed_receipt | Exact `{path,sha256}` from the observed failure |
| old_source_identity / old_artifact_identity | Identities of the failed attempt |
| new_source_identity / new_artifact_identity | Refreshed identities; both must change |
| targeted_correction_completed / artifact_refresh_completed | Both `true` |
| description | Nonempty explanation of this targeted correction |
| refreshed_fixture | `{path,sha256}` of the private refreshed executable fixture |
| seal | `sha256:` of compact sorted-key JSON of all other receipt fields (`allow_nan=False`, UTF-8, default ASCII escaping) |

The controller consumes at most one receipt for an item and at most the global
cap, records it, admits the refreshed frozen fixture, and reruns that exact item
once in another fresh context. It never retries a failed corrective rerun.
No receipt before timeout means unresolved failure and independent continuation.
Malformed or unauthorized receipts, changed unapproved bytes, missing refreshed
artifacts or uncertain accounting stop the packet conservatively.

Refresh permits only `source_identity`, `artifact_identity`,
`configuration_revision` and hashes in the existing frozen path inventory to
change. All changed paths must be in the approval's `refreshable_paths`. Already
imported Python/MJS helpers, live packet argv inputs and authority bytes stay
immutable even if mistakenly listed as refreshable. Runtime/source/built artifact
and generated observer metadata refreshes must therefore be prepared in place at
approved paths. Top-level argv, provider configuration, allowances, backend
bindings, dependency contracts, schema revision and path sets remain identical.
The genuine observer must still admit the actual new runtime identities. A
controller/helper correction or new artifact path requires a fresh separately
approved packet; this live controller cannot load changed imported code safely.

There is no cross-invocation resume command. The bounded pause keeps the original
budget epoch and conservative reservations alive. A later invocation requires a
new exact approval that reconciles prior consumption rather than resetting it.

## Authored tests

`tests/test_failure_sweep.py` contains contract and controller policy cases for
fixed denominator, dependency isolation, independent continuation, no automatic
retry, cleanup safety stops, and mixed-source refusal. They remain UNRUN under
the CLOSED operator gate. They do not constitute production-path acceptance.

## Borrowed owner API for a serial platform wrapper

`execute_collection(fixture, evidence, run_id, selected, load_cases, admission,
load_fixture, packet_owner=None, shared_state=None)` keeps the standalone default:
one internally created packet owner is started and stopped by the controller.
A platform wrapper may instead supply its already-started `PacketProcesses`
instance and one mutable shared-state dictionary for the whole serial suite:

```python
shared_state = {
    "started": monotonic_suite_start,
    "requests": 0,
    "output_bytes": 0,
    "corrections": 0,
    "max_corrections": 12,
}
```

A borrowed owner requires that dictionary. The collection never constructs,
starts, or stops the borrowed process, and never repeats its configured-service
observer or ready receipt. The wrapper remains the sole packet lifecycle owner
and must stop it in `finally`, including after a collection safety stop. Case
Contexts still perform their own admission and exact owned cleanup.

Each collection uses the shared monotonic start and initial HTTP/output counters,
then writes updated counters back after every case cleanup, including a failed
cleanup. An admitted correction increments `shared_state["corrections"]` before
refresh or rerun. Dispatching a correction requires both the local frozen
authority's remaining cap and the shared remaining cap. Exhausting correction
slots logs failures unresolved and permits independent bounded work; it never
resets the global repair cap or any request/output/wall allowance. Shared service
and budget admission failures still return `safety_stop: true`; the wrapper must
stop further platform dispatch on that signal or unconfirmed cleanup.

The summary reports `borrowed_packet_owner`, local `corrections_consumed`, total
`shared_corrections_consumed`, and `final_source_identity`,
`final_artifact_identity`, `final_fixture_path`, `final_fixture_digest`. The
wrapper owns source/fixture/proxy compatibility admission before the next
platform, all suite-wide source grouping, any finite child process accounting,
and the final packet cleanup receipt. Borrowing an owner does not authorize a
new fixture/proxy association or additional effects. These API policy tests are
authored but remain UNRUN under the CLOSED gate.

`tests/test_packet_settlement.py` additionally authors pending-to-settled draining,
unknown reservation refusal, controlled 401 settlement, missing successful usage,
and bounded-timeout cases. Controller tests author the no-next-dispatch ordering
and preservation of initial failures across settlement failure. SQLite fixtures
mock approval parsing; these tests cannot certify live provider accounting. All
remain UNRUN, including syntax/import execution, while the operator gate is CLOSED.
