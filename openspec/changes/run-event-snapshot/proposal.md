# Proposal

## Why

The actual packaged C14 run b485b0cd-5651-4a6c-89f1-c26d1b13f49a reported team_roster TEAM_REVISION_CONFLICT, but application/sidecar shutdown discarded the process-local raw tool arguments. The supplied roster cursor is unobserved; no cursor normalization or roster/write compare-and-swap relaxation is justified. Existing SSE contains exact public tool arguments but opening/dropping a subscription participates in disconnect cancellation.

## What Changes

Add an owner-scoped read-only GET /api/uar/runs/{id}/events snapshot over existing RunManager history, with versioned typed envelope, public SSE projections, ordered event IDs, exclusive after cursor and explicit bounded-retention gaps. Reuse get_run_for_context ownership. Add capability and OpenAPI metadata and document retention. No subscription, RunDisconnectGuard, new store, scheduler, auth bypass, or roster changes.

## Capabilities

### New Capabilities

- run-event-snapshot: Read-only owner-scoped bounded public run trace.

### Modified Capabilities

None.

## Impact

src/uar/api/{mod.rs,routes.rs,run_events.rs,administration_capabilities.rs,openapi.rs}, docs/run-event-snapshot.md and this OpenSpec only. Root owns matching Boss adapter/UI/capture. No provider or runtime authority change, no KBD transition. Parent owns later actual native packaging and normal Work operation. Worker runs no tests/compiler/build/review.
