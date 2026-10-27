# Bounded iOS same-item correction handoff

Phase: uar-post-lifecycle-verification-readiness; task2.9. Operator verification
lock: CLOSED. This is source authoring, not runtime acceptance. No test, build,
format/lint/typecheck, compile/import probe, review gate, service, provider,
simulator, retry or acceptance command was run. No reviewers were delegated.
The lead owns exact source identity/inventory and repair16 charging under the
unchanged global28 cap; this worker did not claim a newly computed identity.
Other writers share the checkout; no unrelated source was reverted.

## Delivered files

- scripts/acceptance/ios/runner.py: explicit six-item manifest, seed-to-restore
  dependencies per repetition, initial failure/evidence retention, ordinary
  failure collection, owned process/private-input cleanup and settled same-epoch
  budget boundary, at most one actual-authority correction plus exact same-item
  rerun, actual refreshed source/product/installed-app admission, immutable live
  helpers/authority/config, separate exclusive attempt/result directories,
  durable correction charges, final source identity, truthful PASS/FAIL/BLOCKED
  summaries/exit codes. No automatic repair/build/store restore operation.
- scripts/acceptance/platform_suite.py: run_ios writes private collection control
  from existing shared count/cap/epoch and frozen authority; reads ordinary
  nonzero collection receipts; recovers durable correction charges in finally
  even if child/stop cleanup fails; propagates safety and source-rebind stops.
- scripts/acceptance/ios/receipt.schema.json: collection result schema with six
  explicit items, final identities, correction accounting, PASS-only predicate
  constraints, and honest FAIL/BLOCKED receipt shapes.
- scripts/acceptance/ios/README.md and scripts/acceptance/platform_suite.README.md:
  new result/control/actual-authority contracts, effects, ownership boundaries,
  failure caveats and unchanged denominator documented.
- scripts/acceptance/tests/test_ios_failure_policy.py: five UNEXECUTED regression
  tests for stable item/dependency identity, unresolved failure retention,
  mixed-source blocking, complete coherent denominator, immutable input rejection.

## Mechanism and source-inspected reasoning

The observed review finding was an unhandled nonzero XCTest command escaping the
entire runner. Initial attempts now preserve invocation-result/native-original/
terminal receipts and logs/xcresult under fresh attempt scopes; code zero plus
missing/malformed/wrong-identity native output is an item failure, not success.
A malformed native receipt may continue only when cleanup is independently
confirmed. UNKNOWN cleanup/ownership/accounting stops the collection and suite.
The uncomfortable limitation: an early thrown native failure may produce no
shutdown receipt. If simctl cannot confirm owned termination either, later items
remain blocked. No fabricated cleanup proof is accepted.

Native Swift selects Documents/store-N. Each attempted item freezes its exact
owned pre-state and retains a private copy when a directory exists, bounded by
10000entries/16MiB-per-file/64MiB-total and the original wall deadline. The runner
never deletes/restores the database. Actual host correction must specifically
admit source inventory refresh, fresh artifact build, owned-app installation
preserving the same container, and exact baseline restoration. It must return
the existing failure_sweep self-sealed receipt bound to this run/item/failure,
plus the iOS extension documented in README. Self sealing is not authorization.
There is no repair-command dispatcher. The runner verifies real restored store
bytes, frozen/new source identity, distinct admitted artifact root/product tree,
and installed app bytes. Unchanged old XCTest products cannot certify new source.
Only an exact unchanged owned native result may be removed after retaining its
original bytes, to permit the same native filename on the single rerun.

Each restore requires PASS for that repetition's seed. Unresolved seed means
BLOCKED restore; independent later stores may run only after cleanup and settled
same-live-epoch accounting. Initial receipts remain retained after a rerun.
Mixed successful identities are BLOCKED. A coherent new-source iOS wave cannot
make old-source macOS/Linux results coherent: the suite propagates the final
source and marks requires_platform_rebind/BLOCKED.

Shared correction count/cap is inherited after macOS/Linux. A consumed authority
receipt writes correction-consumed-N.json before refresh admission, so rejected
refresh still consumes the attempt. Parent recovery runs even if stopping the
child raises. Prior durable proxy consumption and all epoch caps remain intact.
No counters are reset. This authoring does not assert runtime correctness of that
accounting, command termination, simulator install identity, copying or linkage.

## Remaining lead integration and unverified status

Preparation's future suite ios object must supply result_root as a NEW direct
absolute child of private_scope, separate from artifact_root. Future direct run
must add --results; optional --collection-control is a private host input. An
optional correction_authority reference must already be frozen in the fixture;
no populated authority or host correction is generated by this worker. Existing
private-config schema is unchanged because no fields were added to that input.

All source and tests are UNVERIFIED. Only filesystem source reads and authorized
writes were used. Python authoring snippets only modified text and never imported
or executed repository modules. No command output establishes syntax, behavior,
acceptance, package validity or whole-phase completion. There were no accidental
verification commands. This handoff is ready for the lead's implementation
inventory and proposed refreshed approval, not for gate execution.
