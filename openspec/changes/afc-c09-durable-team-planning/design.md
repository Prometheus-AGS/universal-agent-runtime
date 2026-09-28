# Design

## Authority and flow

The existing authenticated principal and `x-uar-workspace-id` select the private scope. The exact immutable TeamDefinition must be an entrypoint in the package named by the current private binding. Create validates the team input schema and materializes bounded slots, including a coordinator, in one catalog compare-and-swap. The resulting instance is `inactive`; members are `inactive`; tasks are `queued`.

Task addition requires the current team revision. A task selects a role that has a materialized slot, supplies input and a valid JSON Schema output contract, and may depend only on distinct tasks already on the same board. This insertion order forms an acyclic graph without a separate execution scheduler. Idempotent command receipts and the team revision are committed with the board. Existing SurrealDB 3.3.0 catalog storage provides the durable CAS; other configured catalog backends use the same interface.

## REST contract

- `GET /api/v1/collaboration/team-definitions`: immutable summaries and member role bounds.
- `GET/POST /api/v1/collaboration/team-instances`: scoped list and create. Create body: `{commandId,deploymentBindingId,teamDefinition,input,memberSlots?}`.
- `GET /api/v1/collaboration/team-instances/{id}`: scoped full instance.
- `GET/POST /api/v1/collaboration/team-instances/{id}/tasks`: queued task list and create. Create body: `{commandId,taskId,expectedTeamRevision,title,role,input,outputContract,dependsOn}`; response is the updated full instance.
- `GET /api/v1/collaboration/team-instances/{id}/tasks/{taskId}`: scoped task.

Every write is a private, authenticated, workspace-scoped transaction. Capabilities report `collaboration.activation.teamPlanning=true` and retain `teamInstance=false` because no team turn can run. The administration descriptor lists each method as owner-scoped.

## Verification boundary

The complete C09 production slice will exercise a real installed package and binding through UAR, create two workspace-isolated teams, create a dependency board, restart the storage-backed runtime, reject stale revisions and cross-workspace reads, and prove no team execution claim. C09.1 source completion alone does not certify these outcomes. No Cargo verification runs before the parent completes the functional delivery slice.
