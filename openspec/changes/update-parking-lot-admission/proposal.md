# Proposal

## Why

Issue #426 requests an exact Parking Lot source upgrade and bounded local ingestion admission. The existing constructor ignores its queue bound, while durable acceptance and actual worker ownership already belong to UAR and must survive the upgrade.

## What Changes

- Vendor exact upstream `4b747350c6883497813bed33623a9937ad7a5894`, manifest version 0.3.0, including required workspace/path crates and licenses.
- Migrate breaking executor, error, resource-cost and metadata APIs.
- Use dispatcher grants for one-slot ingestion admission, with CPU-count active capacity, 100 locally waiting descriptors, Normal FIFO ordering, and no new default execution deadline.
- Retain durable overflow, generation-fenced publication and scope-owned workers until actual completion and shutdown settlement.
- Author finite integration coverage and delivery notes; bump UAR 1.2.3 to 1.3.0 before final acceptance.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dot-durable-ingestion`: bounded local admission independent of durable acceptance, and capacity retained through actual work settlement.

## Impact

Changes the vendored Cargo workspace, dependency lockfile and ingestion adapter. HTTP upload acceptance, provider selection, realtime projections and SSE/owner cancellation contracts remain compatible. Runtime UX gains bounded local scheduling without losing accepted work. KBD registration and source-bound handoffs are required. Independently versioned frontend/Tauri packages retain their versions. Verification remains CLOSED under AGENTS.md; implementation approval grants no test, build, review, publication or release permission.

The uncomfortable failure case is a dropped executor future freeing a scheduler permit while its underlying extraction still runs. The adapter must retain actual work ownership rather than treat timeout or cancellation as settlement.
