# Feedback workflow execution 1.0.0

The required portable capability is `prometheus.workflow-execution/1.0.0`.
`extension.schema.json` defines the closed draft.2 extension envelope. Existing
draft.2 schemas/examples are unchanged. `example/package.json` and its four files
form an immutable installable package using the existing package API; no automatic
install or new service is introduced.

This source adds a bounded interpreter, not an operational qualification receipt.
`UAR_WORKFLOW_EXECUTION_PROFILE_STAGE=operation` enables the completed-path gate.
Without qualification or that explicit operation stage, activation is unavailable.
Capability availability also requires durable team execution ownership.

The authenticated `/api/v1/collaboration` API exposes:

- `GET /workflow-definitions`: definition summaries and field diagnostics.
- `POST /workflow-runs`: `StartWorkflowRequest`, returning `WorkflowRun`.
- `GET /workflow-runs` and `GET /workflow-runs/{id}`: scoped run inspection.
- `POST /workflow-runs/{id}/decide`: exact revision/wait/artifact decision.
- `POST /workflow-runs/{id}/cancel` and `/recover`: revisioned control commands.

Requests use the same workspace header and verified owner as team administration.
The authoritative camelCase DTO is `src/uar/domain/workflow_execution.rs`. Public
optional fields serialize as `null`. Runs expose the complete pinned definition,
compiled-plan digest, original task/attempt identities, exact artifacts, wait,
decision and an accounting projection of the C09 ledger. Identical controls return
their stored result; a fresh inspect returns current accounting.

Both tasks run through C09's ordinary actor host, with tools/skills/MCP/memory
disabled and exact selected feedback/classification context. The final decision
produces no model turn and no external effect. Unknown usage remains reserved;
unknown execution blocks progression. Cancellation records intent first and only
becomes terminal after original execution cleanup is known. Recover requires the
existing team's live-producer check and never creates a replacement attempt.

The uncomfortable boundary: source and compiler success do not prove pinned
restart, actual provider execution, overhead, mobile operation or offline storage.
Those measurements belong to the complete UAR + packaged Boss operation. Timers,
joins, loops, automatic retry, compensation and connector writes are unsupported.
