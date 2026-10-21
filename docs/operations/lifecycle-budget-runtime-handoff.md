# Lifecycle budgeted-shutdown runtime handoff

## Changed path

- `src/uar/runtime/lifecycle.rs`

## Public interface

`RuntimeSupervisor` and `RuntimeHandle` now expose:

```rust
async fn shutdown_with_budget(&self, budget: Duration) -> ShutdownObservation
```

`ShutdownObservation` is additive:

- `Completed { shutdown_id, report }` means the retained supervisor owner has completed. `report.released` can still be `false`; completion does not claim successful resource release.
- `Pending { shutdown_id, outstanding_resources }` means only the caller's observation budget elapsed. It never aborts, retries, or releases cleanup work.

`ShutdownOutstandingResources::SnapshotPending` means the retained shutdown worker has not yet acquired the async registry lock, so inventory is unavailable. It does not mean that the supervisor has no outstanding resources. Once inventory is available, `Resources(Vec<ResourceSnapshot>)` reads the current snapshots from retained scope capabilities rather than returning frozen admission counts.

Every supervisor receives a UUID-backed `shutdown_id` at construction, and every call for that supervisor uses the same value. Existing unbounded `shutdown()` delegates to the same retained initiation path.

## Ownership and cancellation

Shutdown initiation synchronously publishes the supervisor owner task before any caller awaits a registry lock or a budgeted observation. It then begins retained closure of the supervisor scope. Dropping or timing out any waiter therefore cannot abandon accepted cleanup. The worker takes the registry lock later, closes registry admission and configured external intake, retains scope capabilities for the dependency-ordered algorithm, and removes a capability from pending inventory only after that resource reports `released = true`.

Failed close reports, unresolved effects, and scopes retained for missing or cyclic dependencies remain inspectable in pending inventory. The supervisor terminal report continues to preserve child reports and failure data.

## Source-only status

No test, check, build, lint, formatter, compiler probe, or review command was run. The current phase verification gate remains CLOSED. This source change is unverified and needs the lead's renewed whole-phase inventory and explicit operator-approved gate before execution.

## Limit

Budget expiry may return `SnapshotPending` while the shutdown worker waits for registry-lock contention. This is intentional: the observer does not wait unbounded, while the already-retained worker continues toward the same lifecycle finalizer. Pending observation clones retained scope capabilities under its synchronous mutex and reads their snapshots after releasing that mutex; a retained completion report is checked again after that snapshot work.
