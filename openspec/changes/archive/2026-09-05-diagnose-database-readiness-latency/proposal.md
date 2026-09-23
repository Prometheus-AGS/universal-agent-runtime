## Why

The installed UAR still has intermittent database-backed readiness timeouts after
an approved shared-database restart. Existing receipts and source inspection do
not distinguish connection/authentication overhead, query work, full-row transfer
or application/host contention, so another recovery or code change is unjustified.

## What Changes

- Deliver a bounded, credential-safe diagnostic collector and evidence records
  inside the active KBD child, using existing tools without new dependencies.
- Separate installed identity, setup timings, same-session query timings and
  readiness behavior; report a supported or explicitly inconclusive diagnosis.
- Produce a minimal remediation recommendation and independent evidence review,
  without implementing fixes, restarting services or running product tests.
- Register one KBD diagnostic change with five ordered tasks. This is planning
  only; observations begin only after a subsequent Execute request.

## Capabilities

### New Capabilities

- `readiness-latency-diagnostics`: The operational diagnostic workflow's identity,
  bounded observations, privacy and evidence-based acceptance contract.

### Modified Capabilities

None. Existing deep-health-probes requirements and readiness responses remain
unchanged; their observed discrepancies are diagnostic evidence, not fixes.

## Impact

Writes are confined to this OpenSpec change, the diagnostic KBD child and
append-only memory, with canonical KBD commands updating workflow projections.
Runtime UX, provider compatibility and realtime entity state are unchanged.
There are no product APIs, dependencies, service configuration or database-record
changes. Execution's read-only requests still consume shared resources, so the
plan imposes one active request, deadlines and early stops.

The uncomfortable limit: a safe sample may remain inconclusive. A reviewed
diagnostic deliverable does not establish recovered readiness or GA certification.
Cancelled gates, existing warnings and separate Presentation archive approval
remain unchanged.
