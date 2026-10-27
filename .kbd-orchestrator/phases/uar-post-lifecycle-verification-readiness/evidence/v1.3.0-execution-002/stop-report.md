# UAR1.3.0 packet002 — stopped at first failure

Operator approved exact A2/F2/V2 signing/adoption and serial packet. Public-key SSH-agent signing succeeded. Actual adoption succeeded for6a6103741503fe6fd74ae1614b2c00f1b6d7dcd5f3666df4b02bd031fb65d5c6,plan110,bindingrevision4. Published77446974/tag/image remain unchanged. Local candidate includes exactly approved repair001; repair1of2 applied, no second repair.

## Actual command results

- FixtureA directory preparation:exit0. FixtureB owned pinned SurrealDB up:exit0.
- Main002 launcher accepted all32068bound inputs, manifestSHA652e63b50009c9d7534c7489ea5f55e85ce341865ce66b23d9f4fa9d1aebc074.
- Declared cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime:PASS,exit0,4.227144222997595s.
- Declared cargo test --locked --no-default-features --features server-full --test post_lifecycle_main -- --test-threads=1:FAIL,exit101,31.106773384002736s. Cargo compilation finished24.97s;1test executed and failed,0passed. The seven prior compile errors did not recur.
- FixtureD stop andEdown:bothexit0. Exactowned container/network removed, diskdata/evidence retained. No namespace/volume/cache purge or unrelated cleanup.

## Failure and evidence limits

All10scenario receipts areBLOCKED with request_transport_failed. Retained process events show only peer launch1,pid83913; main never launched. Setup failed before scenario behavior. The peer was signaled and reaped; teardown records signalExit0,exitCodenull,successfalse and process_nonzero_teardown. It is not a successful graceful teardown. Five main cleanup requests also failed transport; expected process counts mismatched. Runner outcomeBLOCKED, completeScenarioAndTeardownSummaryfalse.

Both peer stdout/stderr logs are empty. Source inspection finds poll action returns the first request transport error immediately via self.request(...).await? rather than continuing the declared bounded polling attempts. This explains why a connection failure can terminate setup; exact cause of peer unavailability/startup timing is not proven by empty logs. Do not claim an application runtime defect or a successful readiness/teardown control.

Warnings remain for patched SurrealDB unused OnceLockExt, IngestionAdmission missingDebug and sidecar AsyncFd::newdeprecation. None caused this test failure. No warning repair is included.

Per explicit instruction, no repair002, retry or latergate ran. Issue426, sixprofiles, freshRust/codeacceptancereviews, screening, sevenOpenSpecvalidations, canonical5.1/5.2completion, archives, closure and reflection areNOT_RUN. GateCLOSED. Any minimal nextrepair needs explicitauthorization, renewed wholephaseinventory/signature/commandinstruction and the one remaining repair allowance. All14exitobligations and originalepochlimits remain; no automaticextension/reset or acceptancewaiver.

The uncomfortable fact: compilerrepair succeeded but the authored fixture never reached mainstartup. The published image and successful binarybuild cannot certify any runtime scenario or profile. Preserve main-boundary-v1.3.0-001 and002 receipts separately.
