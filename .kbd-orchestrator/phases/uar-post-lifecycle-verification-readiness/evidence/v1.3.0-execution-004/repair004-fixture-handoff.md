# Packet004 fixture source handoff

Phase: uar-post-lifecycle-verification-readiness / Execute. Verification CLOSED.
Scope: repair004 preparation only. No verification or service launch occurred.
Others share the checkout; only tests/post_lifecycle_main.rs and this handoff
were written by this teammate. Lead owns manifest amendments and shared records.

## Source implementation

tests/post_lifecycle_main.rs now permits explicitly enabled startupTransportRetry
on a scenario GET /health only for an owned main whose launch count exceeds one.
Setup GET behavior remains supported. Existing owned-process exit observations,
maximum 30 attempts, one-second delay, transport-only retry, HTTP status/body
condition, and all scenario assertions remain in place. This guard traces to
packet004 LIAB-02 request_transport_failed immediately after its second launch.
No request replay for mutation or arbitrary scenario GET is enabled.

## Exact manifest edits for lead

1. In scenarios.LIAB-02.steps and scenarios.HISTORY-01.steps, set
   startupTransportRetry=true on the poll with control
   main_responds_after_restart. Keep attempts=30 and the existing GET /health,
   HTTP200, /status=ok oracle. Health requires no bearer: middleware explicitly
   bypasses identity admission for /health. Adding auth is unnecessary.
2. In scenarios.KB-01.steps, change the two collection POST paths with controls
   create_kb_ab and create_kb_separate from /api/uar/knowledge-bases/ to
   /api/uar/knowledge-bases. Preserve bodies,201 expectations, captures, and
   all later document/search/isolation assertions. Server mounts the collection
   through nest; existing real sidecar fixtures use the slashless collection
   path (tests/sidecar_session_principal/isolation.rs and
   tests/sidecar_launch_security/features.rs). The retained create_kb_ab404 has
   a103-byte body; server::api_route_not_found emits the corresponding generic
   API404 envelope. Body plaintext was not retained, so body identification is
   source-based inference, not a recovered response.
3. Preserve processes.main.expectedStarts=3, both KB cleanup204 assertions,
   and every current oracle. No cleanup404 acceptance or launch-count reduction.

## Failure chain and limits

Packet004 teardown records a successful first main exit, second launch, then
LIAB-02 transport failure. Unlike setup, this scenario readiness poll lacked
startupTransportRetry. ERROR-01 and ERROR-02 subsequently failed transport;
ERROR-02 did not create {fixtureId}-open-success. HISTORY-01 create_one selects
that exact registered agent. routes::resolve_run_agent returns404
run_agent_not_found when it is absent. This explains HISTORY404 and why its
planned third launch was never reached. The middleware maps auth failures to
401/403/503, not404; missing workspace is not the source-supported404 cause.
KB collection404 prevents its two KB resources from being created; cleanup
therefore encounters404 for their prebound IDs. Strict cleanup expectations
must remain until successful fixture creation/removal is exercised.

HISTORY still depends on ERROR-02's registered success agent. The minimal repair
retains this existing dependency; if ERROR-02 fails later, HISTORY can still
fail secondarily. No independent scenario redesign is introduced.

## Commands and unverified state

All emitted commands were source inspection (cat, sed, rg, and Python reading
retained JSON), plus the authorized apply_patch implementation/document write.
No Cargo command, test, build, lint, formatter, compiler/import probe,
git diff --check, service start, review-driven check, sign or adoption ran.
No acceptance credit follows. Compilation, restart readiness, the slashless KB
collection, history persistence, cleanup, and exact three starts remain
unverified pending the lead's renewed inventory and explicit operator approval.
No dependencies, unrelated production logic, new scenarios or guards without
an observed failure were added.
