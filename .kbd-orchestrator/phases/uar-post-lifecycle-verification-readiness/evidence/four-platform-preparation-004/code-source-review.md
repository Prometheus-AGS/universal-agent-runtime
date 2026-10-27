# A19 / P16.004 code source review

Status: **BLOCKED_SOURCE_ONLY**. Stopped at the first required unresolved blocker, as assigned.

Phase: `uar-post-lifecycle-verification-readiness`, task 2.9.
Verification gate: **CLOSED**.
Supplied source identity: `sha256:c2d37cfee61c535fd569c89b8a515f685c05092bb4cf0a4a3f7987f10d8c8888`.
The identity was read from supplied current artifacts; it was not independently certified.

## [HIGH] Known failed HTTP settlement prevents the iOS failure collection policy

File: `scripts/acceptance/ios/runner.py:234`

The settlement predicate requires every new terminal reservation to be successful:

```python
require(all(row["state"] == "completed" and "outbound_started_unix" in row["receipt"]
            and row["receipt"]["binding_id"] == after["binding_id"]
            for key,row in after["reservations"].items() if key not in before["reservations"]),
        "new forwarding accounting unknown")
```

Concrete trigger: an iOS seed forwards a model request whose upstream returns a fully observed HTTP 500. The proxy records this known terminal response as `failed`, with its HTTP status, response size, outbound finish, and settlement timestamps. At `scripts/acceptance/budget_proxy.py:581`:

```python
ledger.settle(request_id, "completed" if successful else "failed", {
    "http_status": status, "response_bytes": response_bytes, "response_sha256": "sha256:" + digest.hexdigest(),
    "outbound_finished_unix": time.time(),
    "downstream_disconnected": downstream_disconnected,
    "actual_usage": usage, "actual_charge": "unverified; full approved ceiling retained"})
```

The call to `remaining()` at runner line 411 invokes `settled_ledger(config,before)` through lines 325–329. The `failed` reservation therefore raises even though no reservation remains pending and accounting is known. Control exits `attempt` before its terminal receipt at line 418 and before the initial-failure/correction path at lines 431–440. The enclosing handler sets `safety_stop=True` at lines 485–487. The final settlement at line 488 encounters the same reservation again.

Existing guards do not resolve this: `meta["blocked"] is None` at line 183 distinguishes a known failed HTTP response from unknown accounting, but line 234 still rejects the known failed state. No useful-success claim is needed to settle a failure. The server-side `PacketProcesses._ledger_snapshot` already distinguishes `failed` terminal HTTP responses from `completed` success while retaining the full reservation.

Impact: an ordinary known provider failure cannot be collected as the failed item with its permitted targeted correction; later independent seed items cannot continue after confirmed cleanup and known settlement. This contradicts the explicit requirement that known failed HTTP may settle without useful success, while UNKNOWN blocks.

Required correction: admit known, internally consistent failed HTTP terminal reservations for settlement while preserving failure outcome, original reservations/limits, and strict UNKNOWN/pending blocking. Retain the failed item receipt before correction/continuation. This review made no repair and authorizes no retry.

## Inspected scope and limitations

Read the current waypoint for phase position. Selected source scope solely from `changed_regular_inputs_since_A15` in `four-platform-whole-phase-inventory-003.json`; other historical/narrative inventory fields were not used as review evidence.

Source inspected before the stop:
- `scripts/acceptance/prepare_platform_inputs.py`: controller and six-stage interface, source bindings, limits, command/environment validation, ownership, and output receipts.
- `scripts/acceptance/prepare_supporting_inputs.py`: preparation/interface flow, source projection, inert materialization and private input handling.
- `scripts/acceptance/packet_processes.py`: full source, including consistent ledger snapshot, failed HTTP settlement, bounded drain and packet ownership.
- `scripts/acceptance/platform_suite.py`: source through main, shared packet/counters, platform rebind and iOS integration.
- `scripts/acceptance/ios/runner.py`: full source, including preparation/build interface, collection and correction control, cleanup and settlement.
- `scripts/acceptance/failure_sweep.py`: contract, authority, correction, refresh and collection flow; trailing summary portion not fully inspected.
- Necessary referenced helper `scripts/acceptance/prepare_artifacts.py`: source capture and stage environment interface; read obtained, review partial.
- `scripts/acceptance/budget_proxy.py`: relevant reservation/outbound/settlement/HTTP terminal excerpts only.
- `scripts/acceptance/tests/test_ios_failure_policy.py`: authored coverage inspected only; never run.
- `scripts/acceptance/core.py`: fetched but not substantively reviewed before stop.

Assigned inert configs:
- `four-platform-preparation-config-004.draft.json`: command/tool/environment/interface fields inspected.
- `four-platform-artifact-preparation-004.draft.json`: stage/environment, tree, installed output, closure, limit and identity/interface fields inspected. The 29,045-entry source-file projection was retrieved but not independently certified or individually reviewed.
- `four-platform-supporting-config-004.draft.json`: current fields inspected against the supporting helper.

Remaining changed-source scope is **unreviewed**: both changed documentation files; cases_capabilities.py; compose_fixture.py; the failure-sweep README and trailing summary; all iOS Xcode/Swift/Cedar/schema/Rust/scenario/README files; linux_runtime.py and README; observers_runtime.py; platform/preparation/supporting READMEs; test_failure_sweep.py; tools_fixture.py; user_outcomes.py; Rust SDK Cargo.lock/Cargo.toml/runtime.rs; vendor/git/README.md; versions.toml; test_packet_settlement.py; and the portions of fetched sources explicitly marked partial above. No PASS_SOURCE_ONLY conclusion is made for any complete assigned scope. Rust and Swift specialist reviews remain later serial work, not performed here.

## Execution and independence

Only filesystem MCP reads and this single report write were used. No shell/terminal commands, tests, lint, format, type, compiler/import probes, builds, packaging, network/provider/service/simulator operations, source/config edits, or agent spawning occurred. No executable source was imported or run. Runtime correctness, persistence, actual platform artifacts, cleanup behavior and acceptance remain **UNVERIFIED**.

No production changes or guards were added. The report is the sole requested write; no unrequested implementation was added. Inspection stopped at the blocker and the lead was notified.

This is an isolated native same-family review with no producer history used as evidence. That isolation is weaker than cross-model review.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: **WARNING — BLOCKED_SOURCE_ONLY**. Resolve the known-failed-HTTP iOS settlement blocker before requesting another explicitly authorized source review. This is not runtime acceptance.
