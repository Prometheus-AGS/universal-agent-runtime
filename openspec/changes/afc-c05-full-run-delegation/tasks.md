## 1. Shared native execution seam

- [x] 1.1 Extract reusable run admission assembly from the native route and add reserved-run-ID entry points; verify by inspection that both routes call the same assembly and `RunManager` loop.
- [x] 1.2 Add canonical request digest support without retaining or exposing credential material; verify receipts and diagnostics contain no request secret values.

## 2. Process-ephemeral task authority

- [x] 2.1 Implement owner-and-workspace-scoped admission reservation, runtime epoch, native IDs, task state, and exact terminal TTL/cap pruning; verify each declared retention field derives from the active configuration.
- [x] 2.2 Implement exact retry receipts, digest conflict, admission reconciliation, and typed retention-expired/unresolved outcomes; verify all outcomes use the frozen snake_case contract.
- [x] 2.3 Implement lifecycle monitoring and distinct cancellation requested/acknowledged/terminal/cleanup-uncertain states; keep terminal tied only to native cancellation and cleanup uncertainty tied only to explicit cleanup-unconfirmed evidence.

## 3. Versioned control surface

- [x] 3.1 Mount authenticated `/api/uar/full-harness/v1` admission, reconciliation, and status routes; verify all handlers derive owner identity from verified host context.
- [x] 3.2 Implement non-cancelling replay/observation, revision-checked approval forwarding/cancel/detach, observer-scoped detach, and typed unsupported steer; keep stale revisions mutation-free, detach and disconnect non-cancelling, and steer run-free.
- [x] 3.3 Document the public schemas and routes in OpenAPI; verify the versioned operations and stable receipts are represented.

## 4. A2A authority projection

- [x] 4.1 Route full-harness task IDs in A2A `tasks/get` and `tasks/cancel` to the same authority while preserving legacy A2A handling; verify the projection exposes the reserved native run ID and never executes a second loop.

## 5. Phase completion

- [x] 5.1 Reconcile production files against the OpenSpec tasks and mark completed implementation work; final integration verification remains owned by the parent C05 gate.
