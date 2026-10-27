# Complete r5 fixture bootstrap and ownership correction

KBD remains Execute in the isolated repair child, supporting Kubo Execute/FIR-9. Ordinary verification is CLOSED after the first r4 command failed. No further gate, timing experiment, probe or debugger run is authorized.

## Assessment and analysis

The exact approved r4 command compiled in4m38s, then completed827 library cases in53.31s:823 passed,3 failed and1 existing git/build fixture ignored. The four corrected contract tests passed; the previous stack overflow did not recur. Source drift was empty. Neither selected integration executable nor command2/3 ran. Preserve the raw receipt and logs unchanged.

All three lifecycle failures hit `bounded("host lifecycle setup", Self::start())` at ten seconds. Each emitted namespace/database selection but never the subsequent provider-connected or manager-classifier messages. In `src/uar/persistence/providers/surreal.rs`, the constructor's retained initializer selects namespace/database, awaits `host::migrate`, then delivers the provider. Thus the logs place failure before provider construction completed; they do not prove authentication, request preparation or token-fit assertions were reached.

The absence of another stack overflow is not runtime proof of the hosted-root heap correction: these three cases stopped before reaching that root path. Only the four contract-test passes and the reported aggregate are established by this attempt.

The newly introduced aggregate ten-second timeout abandons the constructor waiter. Its retained initializer can safely finish and close an undelivered client only while the runtime and temporary directory remain alive. Panicking from that wrapper ends the test runtime and drops its directory first. The two transaction-drop messages are consistent with this teardown; they do not establish a migration deadlock or a correct larger time limit. Existing `lifecycle_owned_resources.rs` and `ingestion_admission.rs` fixtures await full cold construction before timing lifecycle operations. The construction/panic ownership pattern in `embedding_lifecycle_failover.rs` retains acquired resources outside fallible work and finalizes before reporting errors.

Source inspection also found an unexecuted fixture assertion comparing distinct identity namespaces: delegation contract ownership uses the verified subject from the durable root thread; accounting uses the complete credential-principal storage key. Preserve both production namespaces, and compare each against its proper owner binding.

## Entire source batch — one Rust file

The existing specialist owns exactly `src/uar/runtime/thread/service_tests.rs`. Root owns this plan, design/task/proposal records, append-only history and final inventory. No other production, provider, migration, API/schema, catalog, dependency or route edit is included.

1. Await complete cold provider construction before applying the existing root-ready and operation deadlines. Remove the newly introduced aggregate ten-second wrapper; do not replace it with a guessed larger bootstrap timeout. Preserve actual migrations and durable SurrealKV storage, with no cached/pre-migrated substitute or in-memory downgrade.
2. Hold the temporary directory and every acquired provider/manager/peer/root resource in a construction/cleanup owner outside fallible setup and scenario futures. Catch setup and scenario errors/panics, perform cleanup, then surface the original failure. The three complete existing scenarios must run through this ownership boundary. Do not suppress failures or convert incomplete cleanup to success.
3. During root readiness, observe both the driver's notification and root-task completion. Report an actual early root failure directly instead of masking it as a readiness timeout. Keep the existing ten-second model-ready/operation deadlines and five-second shutdown assertions; no increased budget or deadline is part of this fix.
4. Separate resource cleanup from success assertions. Cancel/join the exact root and captured host, close manager lifecycle/history, stop/join the owned peer, and request acknowledged provider closure before removing temporary storage. Attempt subsequent owned cleanup even if an earlier step fails. Preserve the original failure alongside cleanup diagnostics. If closure cannot be acknowledged, retain the exact owned directory and report incomplete cleanup; never delete it or claim release based solely on a canceled waiter.
5. Compare `remote.contract.owner_id` to `service.inner.root.owner_id`, the verified subject/root binding. Keep durable accounting owner assertions against the complete principal key. Preserve exact contract-digest binding, authentic finite reservations, restored capacity, unchanged pending model liability, zero peer execution, durable child cancellation and repeated shutdown checks. No fabricated terminal usage or weakened acceptance oracle.

The existing hosted-root boundary and all four passing r4 contract fixes remain intact. Complete all fixture behavior, regression source and required documentation before stopping writers and renewing the full inventory/canonical boundary. Any additional path need or required source item must return to root before editing.

## Causal and operational limits

No source evidence establishes migration duration, identifies an outstanding migration statement, or distinguishes slow concurrent bootstrap from a stall. The current constructor exposes no initializer cancellation/join handle. Through this API, a strict finite bootstrap deadline plus guaranteed initializer release cannot be promised. This correction follows the established uninterrupted cold-construction pattern and explicitly retains that limitation; all actual operation and shutdown bounds remain in force. Bootstrap may wait if migration stalls. Separately authorized timing/diagnostic evidence would be needed to resolve that uncertainty; this plan grants none.

Actual protected request fit beneath the unchanged1000-token ceiling, runtime correctness and all r5 cleanup outcomes remain unverified. One pre-existing ignored provisioning case remains unverified, not silently passed. After all code/docs are integrated and writers stopped, capture the complete source manifest and typed canonical reconciliation, then present exact commands for renewed operator instruction. No testing, builds, lint/format/compiler/diff checks, validators, probes, publication or Kubo cumulative gate may run in between.

## Complete source handoff

Every planned source item above is implemented in the one assigned fixture file. All three existing named scenarios use the retained setup/scenario cleanup boundary, genuine root-ready result selection and proper identity namespace comparisons. A fallback drop retains unacknowledged storage as well; absence of a delivered provider handle never invents a close acknowledgement. The specialist stopped writing, reported no known source item or extra path need, and ran no verification. The complete integrated source/documentation set has29 paths; root records its exact manifest and canonical boundary separately. No production, provider, migration, dependency, catalog, route, schema or API changed in this batch. The cold-bootstrap and runtime limits above remain in force.
