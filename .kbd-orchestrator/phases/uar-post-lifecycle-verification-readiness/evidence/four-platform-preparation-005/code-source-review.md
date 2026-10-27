# A20 / P16.005 code source review

Status: BLOCKED_SOURCE_ONLY

Phase: `uar-post-lifecycle-verification-readiness`, task `2.9`.
Supplied source identity: `sha256:fe0d649a16a604bacb57054a5470d05815be56a8b8d0cf87d3b46960f1ddfa94`.
The supplied identity was read, not independently certified. Runtime: UNVERIFIED.

This review stopped at its first required unresolved blocker. It does not approve the uninspected remainder, preparation, specialist review progression, or product execution. The operator verification gate remains CLOSED except for this named filesystem-only review and this report write.

## Finding

[HIGH] Malformed terminal forwarding evidence permits continued dispatch

File: `scripts/acceptance/packet_processes.py:159` (terminal admission at lines 157–162).

```python
                require(type(status) is int and 100 <= status <= 599
                        and (200 <= status < 300) == (state == "completed")
                        and "outbound_started_unix" in receipt and "outbound_finished_unix" in receipt
                        and "settled_at_unix" in receipt
                        and type(size) is int and 0 <= size <= reserved["response_bytes"],
                        "shared forwarding terminal accounting is inconsistent")
```

Failure scenario: a shared-ledger reservation has state `failed`, HTTP status 401 or 500, correct epoch/input/binding/pool, valid reserved counters and matching cumulative totals, and `blocked=None`, but its terminal receipt contains `outbound_started_unix=None`, `outbound_finished_unix=None`, and `settled_at_unix=None`. The presence checks accept these values. Reversed or nonfinite timestamps are likewise unchecked. The receipt can also omit `response_sha256` entirely. With no reserved rows, `_ledger_snapshot` returns an empty pending list and `settle` records `forwarding-settled` and returns. `failure_sweep.execute_collection` uses that successful return to permit correction or the next independent item after cleanup.

This violates the assigned requirement that only known, internally consistent failed HTTP forwarding may settle, while malformed or unknown forwarding evidence stops continuation. The consequence is additional dispatch admitted without valid terminal evidence for the preceding shared forwarding work.

Existing guards do not catch this input: source/config/owner/epoch checks establish ledger ownership, and reservation/totals checks establish accounting arithmetic, but neither validates terminal timestamps or response identity. The successful-usage check runs only for completed rows and does not address the failed-row scenario. `budget_proxy.handler_class` normally authors real timestamps and a response digest, but this settlement reader is the admission boundary for durable evidence and currently accepts malformed records instead of enforcing the explicit stop requirement. The authored `test_packet_settlement.py` failed-401 fixture itself contains no response digest; its cases do not reject malformed timestamp values. No test was executed.

Required correction: require terminal timestamps to be finite positive numbers in valid chronological order, including the reservation timestamp, and require a correctly formed response SHA-256 digest before admitting either completed or failed terminal evidence. Preserve failed status and its full reservation. Author coverage for malformed failed terminal evidence, leaving execution behind the operator gate.

## Actual inspected scope

Selection of changed source paths used only `changed_regular_inputs_since_A15` from `four-platform-whole-phase-inventory-004.json`. Its other narrative/history fields were not used as review evidence. The current waypoint was read solely to establish the active phase/task and CLOSED boundary.

Fully inspected implementation bodies: `scripts/acceptance/prepare_platform_inputs.py`, `scripts/acceptance/failure_sweep.py`, `scripts/acceptance/packet_processes.py`, `scripts/acceptance/platform_suite.py`, and `scripts/acceptance/ios/runner.py`. The supporting preparation helper was inspected from its returned source text. Fully inspected authored source tests: `scripts/acceptance/tests/test_packet_settlement.py`, `scripts/acceptance/tests/test_ios_failure_policy.py`; also the current `scripts/acceptance/ios/Tests/EmbeddedAcceptance.swift` source.

Partially inspected: `scripts/acceptance/core.py` (environment, allowances, process ownership, observer invocation, restart and cleanup); `scripts/acceptance/observers_runtime.py` (inference/forwarding evidence and cleanup); `scripts/acceptance/user_outcomes.py` (fixture admission); `scripts/acceptance/linux_runtime.py` and `scripts/acceptance/cases_capabilities.py` (returned text had truncation and therefore is not claimed fully reviewed).

Necessary referenced source inspected: `budget_proxy.py` reservation, outbound, settlement, snapshot, owned-hop and forwarding-handler implementations. Its complete source was retrieved, but only those portions were assessed.

All three assigned inert configuration artifacts were read: `four-platform-preparation-config-005.draft.json`, `four-platform-artifact-preparation-005.draft.json`, and `four-platform-supporting-config-005.draft.json`. Relevant preparation CLI arguments, tools, stages, output paths, limits, environment mappings and supporting-helper inputs were inspected. The artifact configuration's large `source_files` array was not individually assessed or independently compared with the source manifest. Configuration review is incomplete because this first blocker stopped the assigned review.

Not completed: the rest of the selected changed-source set, all required referenced-source tracing beyond the finding, full source-manifest assessment, and full helper/config interface consistency assessment. Rust and Swift specialist reviews were not started or delegated.

## Verification and changes

Only filesystem MCP reads and this single report write were performed. No shell/terminal, code import, syntax probe, check, test, lint, formatting, compilation, build, packaging, provider/network/service/simulator operation, source/config edit, or agent spawn occurred. The inspection evaluated source text; it does not establish runtime behavior or preparation success. No unrelated changes or guards were added. The proposed guard correction traces to the explicit malformed-evidence stop requirement at a real evidence admission boundary.

Native same-family isolation is weaker than a cross-model review. No producer history, writer handoffs, old reports, amendment narratives, execution history, or `.prometheus` history was used to reach the finding.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: WARNING — resolve the HIGH issue before continuing the serial review/preparation sequence. Review result remains BLOCKED_SOURCE_ONLY with incomplete scope; no runtime acceptance is claimed.
