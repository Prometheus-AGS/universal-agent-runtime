## Why

Initiative C05 requires The Boss to delegate one complete agent run to UAR without becoming a second executor. UAR currently owns the native run loop, but it does not expose a retry-safe task admission and control authority that can recover a lost response, distinguish detach from cancel, or truthfully report process-ephemeral retention.

## What Changes

- Add an authenticated, versioned `/api/uar/full-harness/v1` task surface that reuses native `/api/uar/runs` assembly and the existing `RunManager` executor.
- Reserve owner-and-workspace-scoped admission, task, and run identities before execution using an idempotency key and canonical request digest.
- Expose reconciliation, task status, non-cancelling event replay, tool approval, cancellation, detach, and typed unsupported steer operations.
- Report runtime epoch, process-ephemeral terminal retention limits, native IDs, effective binding, diagnostics, and distinct cancellation lifecycle states.
- Project full-harness tasks through the existing A2A task lookup and cancel methods while retaining the full-harness authority as the single record source.
- Keep native run routes and model-provider routes backward compatible; add no persistence migration.

## Capabilities

### New Capabilities

- `full-harness-run-delegation`: Owner-scoped, retry-safe full-run delegation and lifecycle control over UAR's existing native executor.

### Modified Capabilities

None.

## Impact

This repository-scoped change implements the UAR provider half of initiative change `afc-c05-bossfang-full-run-delegation` and tasks C05.1-C05.3. It affects UAR run API routing, A2A task lookup, OpenAPI documentation, and server wiring. It adds no dependency, database schema, model-provider behavior, frontend surface, or KBD workflow-state mutation. Runtime UX gains explicit diagnostics and task lifecycle receipts; existing providers and native `/api/uar/runs` remain compatible. Realtime observation uses the existing run event stream without subscriber-disconnect cancellation.
