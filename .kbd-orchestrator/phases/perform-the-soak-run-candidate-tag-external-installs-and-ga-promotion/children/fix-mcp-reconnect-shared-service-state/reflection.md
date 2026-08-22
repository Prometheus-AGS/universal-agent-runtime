# Phase Reflection: fix-mcp-reconnect-shared-service-state

**Project:** universal-agent-runtime
**Date:** 2026-08-21
**Phase completion:** reported per goal; no aggregate percentage
**Changes completed:** 1 / 1

## Delta Between Plan and Delivery

The first implementation shared the replacement service pointer but left the
reconnect configuration on each registry view. A pre-existing filtered view
could therefore use configuration B after an A-to-B upsert, fail, and reconnect
configuration A back into the shared slot. The first artifact judge passed that
candidate; the history-free critic supplied the reachable counterexample and
blocked it. KBD plan revision 10 records the correction: the shared slot now
owns the authoritative reconnect entry and generation, and a reconnect swap is
accepted only while its captured generation remains current.

The initial artifact also retained only summaries for five installed-result
references and stored constraint IDs rather than the complete load-bearing
objects in its checkpoints. Delivery consequently includes all raw referenced
files, six chronological checkpoints with exact constraint objects, and a
byte-identical finalized history snapshot. These are evidence corrections, not
additional runtime scope.

## Goals

| Goal | Status | Observed evidence | Limit |
|---|---|---|---|
| Share replacement transport state across authorized filtered and merged views. | MET | Focused registry test passed 1/0; independent critic and judge traced the shared slot through both projection paths. | Private `McpRegistry` behavior under `server-full`. |
| Keep an upserted configuration authoritative during reconnect. | MET | A-to-B old-view crash/reconnect regression passed 1/0; no post-upsert operation reached A. | Sequential observed defect; reconnect-storm serialization remains out of scope. |
| Fail crash and timeout calls once without replay. | MET | Retained SSE and the exact five-row process trace show one crash in PID 58390, one timeout in PID 58463, and later success in replacement PIDs 58463 and 58743; six negative controls were rejected. | Local installed preflight only. |
| Preserve server, MCP-tool, and native-tool authorization. | MET | Focused exclusion assertions passed and both reviewers confirmed that only service slots are shared. | Registry policy maps only; unrelated policy systems are not covered. |
| Return a reproducible candidate to parent certification. | MET | Immutable source `f0298d76ea3c39853020c8a33e13f136c07a1806`, local macOS arm64 release, Linux arm64 container, focused operational suite 5/0, and 60-second preflight passed. | The parent three-hour soak, deployment, external installs, tag, and GA remain unverified. |

## Delivered Change

- `fix-mcp-reconnect-shared-service-state` — generation-guarded shared MCP
  service/configuration slots, focused authorization and A-to-B regression
  coverage, retained process-boundary evidence, and local installed preflight.
  (by: Codex)

## Technical Debt and Risk

- Concurrent reconnect storms remain deliberately unserialized; every failed
  call remains non-replayed, and only a generation-current replacement may win.
- Existing views may retain a slot after server removal. This is the pre-existing
  snapshot-view behavior and was not changed by this child.
- Three pre-existing package warnings remain outside the child edit.
- The parent three-hour operational certification must restart from `f0298d76`;
  the earlier candidate is invalid evidence.

## Architecture Integrity

- AGENTS.md violations introduced: NONE observed.
- Capability inversion: unaffected; the change is trusted-host transport state.
- Dependencies, public APIs, protocols, UI, submodules, and GitHub Actions:
  unchanged.
- Routine and installed-artifact verification ran locally; no GitHub Actions
  product test was run.

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact QA | 1 / 1 |
| Initial independent review | BLOCK |
| Corrected independent review | critic PASS; judge PASS |
| Refinement iterations | 2 |
| Final blocking constraints | 5 satisfied |

The active and persisted history artifacts contain six chronological
checkpoints, 16 live manifest references, 17 matching hash receipts, and 14
byte-identical artifact files.

## Cross-Tool Coordination Notes

- Progress tracking: reliable after canonical KBD revision 10; OpenSpec owns the
  13-item task ledger while KBD owns the single change and completion dimensions.
- Handoff quality: the raw process trace and explicit source SHA were decisive.
  Summary-only recovery claims were not.
- Recommendation: make an old authorized view plus configuration replacement a
  standard review adversary whenever mutable transport state is shared.

## Lessons Learned

- Sharing a transport pointer without sharing its authoritative reconnect
  configuration creates a rollback channel after upsert.
- A generation check is required when reconnect construction occurs outside the
  slot lock; otherwise a stale asynchronous result can overwrite newer state.
- Event-level failure evidence and cross-request ownership are independent. A
  correct failed event does not prove the next request sees the repaired transport.
- Constraint IDs and schema validity are insufficient when the descriptions and
  validation methods carry the safety contract.

## Next Phase Focus

Return to the parent Execute phase at `/opsx:apply
certify-operational-resilience`. Freeze a new parent candidate from the child
closure commit, run the local three-hour operational certification, and do not
reuse any pre-`f0298d76` candidate or evidence.

## Sycophancy Self-Check

- S-08: the reflection leads with the stale-configuration and evidence-integrity
  failures, their root causes, and the corrections.
- S-03: concurrency, removal semantics, warning debt, platform limits, and the
  unrun parent certification remain explicit.
- S-02: every MET result cites an observed test, raw artifact, or independent
  review. The optional sycophancy analyzer was not available in this harness.
