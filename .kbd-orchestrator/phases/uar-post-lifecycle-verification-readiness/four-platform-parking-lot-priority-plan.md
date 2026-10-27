# Four-platform verification scope and Parking Lot priority

Operator direction, 2026-10-25: prioritize Prometheus Parking Lot 0.4.0 certification, then functional verification on macOS, Linux, Windows and iOS. Defer Android until those results support a separate decision.

## Scope and preserved evidence

Android is DEFERRED for this round, not passed, removed from the product, or declared unsupported. Preserve S9's missing Android-target OpenSSL failure and its consumed correction. Historical A15 totals remain 18 items: 15 PASS, 1 FAIL, 2 BLOCKED. Any new scoped denominator must be published separately and must not relabel those results. Do not modify signed contracts or immutable receipts in place.

The uncomfortable fact: 0/97 user scenarios have executed. Removing Android does not resolve the shared fixture failure, current-source artifact gaps, Windows execution gap, or missing iOS runtime evidence.

## Dependency prerequisite

Source inspection finds the requested upgrade already authored: Cargo.toml selects local prometheus-parking-lot-core version 0.4.0; versions.toml pins 0.4.0 and commit f24382acd7ba818204e32f4449e9c9c37c52558d; root and SDK Cargo.lock both resolve core 0.4.0; the vendored facade declares 0.4.0. The retained release note identifies an exact official source commit, not a published/tagged release. Do not substitute an uninspected moving main or claim registry publication. A matching version string or previous build does not certify the integration.

## Ordered continuation

1. Finish source-only consistency/provenance assessment of the existing 0.4.0 integration. Make a dependency edit only if an actual discrepancy is found.
2. Prepare one replacement authority packet covering bounded fixture repair, refreshed current-source artifacts/inventory, exact commands and remaining resource accounting. The fixture failure is a null digest on a nonregular inventory row; preserve regular-file source hashing semantics and all partial evidence. Fresh outputs and any JWT replacement must be named explicitly; no silent reuse of expired credentials.
3. Execute the approved Parking Lot integration gate before the broad product scenario sweep. Retain nonzero test counts, features, source identity, real store effects, failures and cleanup. The existing candidate is:

   cargo test --locked --no-default-features --features server-full,test-probes --test ingestion_admission --test durable_ingestion --test lifecycle_owned_resources --test run_retention -- --test-threads=1

   This is a proposed command, not executed or newly approved here. It requires owned temporary embedded stores, isolated disk-backed remote SurrealDB and the existing self-executing durability fixtures. It does not establish PostgreSQL parity under this feature selection. Enumerate the separate PostgreSQL path and fixture in the exact packet before claiming backend parity. No paid embedding provider is required for the controlled admission cases.
4. Run shared user workflows through the real configured provider/model, mounted interfaces and both durable adapters. Retain all 97 case definitions and their existing three-repetition/two-backend requirements unless an explicit platform applicability decision changes an individual row. Do not infer that 582 rows cover four operating systems; publish per-platform evidence separately.
5. Establish functional results for the actual selected platform artifacts. Report PASS, FAIL, BLOCKED and NOT_APPLICABLE distinctly, with applicability grounded in shipped capabilities. Builds and protocol status codes alone do not pass functional workflows.

| Platform | Required functional evidence | Current gap |
| --- | --- | --- |
| macOS | Current-source native artifact; configured prompt/stream, relevant interfaces, durable restart and supported application/SDK paths | Prior builds bind earlier source; shared fixture incomplete |
| Linux | Current-source Linux artifact/container actually started; same supported server workflow and backend outcome predicates | Docker attempt timed out; no image from that attempt and no runtime outcomes |
| Windows | Actual Windows runtime artifact on an identified Windows executor; supported native/client workflows, persistence/restart and path/process behavior | A15 contained no Windows build or runtime gate; executor and exact commands must be established |
| iOS | Shipped embedded integration executing on an identified simulator/device; configured prompt/stream, host persistence/restart and supported multi-call behavior | Prior cross-build only; app/host runner and exact commands must be established |
| Android | Deferred follow-up decision after four-platform results | Retain target OpenSSL failure; no Android repair or gate in this round |

6. Perform independent review of resulting source and actual runtime evidence; report remaining failures before the Android decision. Certify only the exact passing source/artifact/platform/capability combinations. An unavailable Windows or iOS executor remains a visible blocker to that platform's certification.

## Authority and limits

This records the operator's scope and ordering instruction. It does not sign/adopt a replacement contract, authorize an unnamed fixture repair, or restart finalized A15 items. Task2.9 remains incomplete and the product gate remains CLOSED until the exact renewed packet is approved. Preserve all prior consumption, original epoch, 4,000 cumulative calls, 2,400 active minutes, repair floor15/cap28, and independent USD10 Cohere / USD10 OpenAI embedding ceilings. Do not transfer unused one-per-item correction slots to a finalized item. The next packet must explicitly reconcile the operator's one targeted fix and one same-item rerun policy with the old signed first-failure clauses, and log unresolved failures while continuing independent items.

No test, build, service startup, provider request, publication, signature or adoption ran while recording this decision.

## Source-assessment handoff

A separate read-only explorer confirmed root/SDK lock versions and vendored 0.4.0 provenance alignment. No dependency edit is needed to obtain the requested version. The old issue-426-inventory.md and issue-426-source-manifest.json still describe 0.3.0 commit 4b747350c6883497813bed33623a9937ad7a5894 and earlier source hashes. Preserve them as historical snapshots and create a new current-source Issue426 inventory before certification; do not silently replace their provenance. Historical core0.4.0 compilation is supporting evidence only. No command in the proposed integration gate was executed.
