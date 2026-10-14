# Proposal

## Why

Observer polling does not define durable user scheduling. This change supplies the D1 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

First run the exact-version timing fit gate for tokio-cron-scheduler 0.15.1, without making its storage authoritative. Implement owner-scoped one-shot UTC and recurring calendar schedules, IANA timezone, revision/cancel/pause and atomic cursor-plus-occurrence admission. Default no overlap; coalesce backlog to the latest due occurrence with recorded skipped range, skip nonexistent DST local times and select the earlier repeated instant. Current authority, budget and approval apply on every admission.

## Capabilities

### New Capabilities

- `dot-durable-schedules`: Schedules persist intent and unique governed occurrences.

### Modified Capabilities

None.

## Impact

The operator selected latest-pending coalescing while the previous run remains active, awaits approval or has an uncertain outcome. The durable overlap guard spans revisions; edits cannot bypass it. The detailed busy-time contract and acceptance scenarios are in design.md and the spec delta.

Code ownership: `src/uar/runtime/observer`, `src/uar/runtime`, `src/uar/api`, `src/uar/persistence`. Dependencies: dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-10-durable-budget-admission, dot-11-supervisor-fencing. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

