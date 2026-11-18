# Observed team API payloads

Recorded on 2026-11-18 from a UAR 1.4.0-rc.5 dev instance (embedded SurrealKV, instance id `uar-dev-local`, owning team execution) with a two-member sample team (researcher, writer; writer coordinates) bound to `alibaba/qwen4-max`. Each file is `{method, path, status, request, response}` as the service answered it. Nothing here is hand-written.

| File | What it shows |
|---|---|
| `team-definitions.json` | `GET /team-definitions` with one installed team |
| `package-export.json` | `POST /packages:export`: `export.files` maps path to a JSON string |
| `binding-preflight-activation-unsupported.json` | Preflight answers 200 with `activationSupported: false` and `required-unsupported` diagnostics when the provider is not configured. The service still installs such a binding if asked |
| `bindings-list.json` | `GET /deployment-bindings` |
| `team-create.json` | `POST /team-instances`; team and member `status` are `inactive` |
| `task-claim.json` | Claim sets `assigneeMemberId` and `assignmentAuthority`; task stays `ready` |
| `admit-409-capability-unsupported.json` | Admission refused: HTTP 409, `code: TEAM_CAPABILITY_UNSUPPORTED`, `diagnostic: {code, retryable, action}` |
| `task-admit-200.json` | A successful admit returns the attempt |
| `execution-two-running.json` | Two attempts `running` |
| `execution-two-failed-output-budget.json` | Both `failed`, `stateReason: output_budget_unavailable`, no diagnostic object |
| `execution-two-failed-budget-admission.json` | Both `failed` with `diagnostic: {code, retryable, action, protectedDiagnosticRef, sourceStage, category}` |
| `team-after-two-failed.json` | Tasks become `failed` with `stateReason` |
| `artifacts-empty.json`, `peer-messages-empty.json` | Empty collections |
| `execution-one-succeeded.json` | One attempt `succeeded`: `output` is a string, `usage` has real tokens, cost and seconds, `effectiveModels[0].pricingIdentity` is present |
| `team-after-one-succeeded.json` | The task after success, with its `output` |
| `artifacts-after-success.json` | `GET /artifacts` after a successful attempt |

A model call only succeeds when the bound model has a catalog price. `alibaba/qwen4-max` has none, so the dev instance's model entry carries `pricing_identity: alibaba/qwen3.7-max`; without it every attempt failed with `TEAM_EXECUTION_BUDGET_FAILED`.

Not observed yet: `yielded` with waits, continuations, delegation, peer messages, cancellation, `uncertain`.

## Delegation was not available to the sample coordinator

A coordinator task that asked for `team_roster`, `team_delegate` and `team_wait` completed as a single `succeeded` attempt whose output said the delegation tools were not callable. Its turn manifest named no team tools, there were no waits, command receipts or continuations, and the researcher never ran, although the service advertises `team_execution_peer_tools_v1`. Team tools are offered only when the member's effective tool policy admits them (`src/uar/runtime/manager.rs` near 6375-6395); the sample agent definitions declare no tools. The open change `team-attempt-native-tool-catalog` covers peer tools missing from a coordinator and has its acceptance task unchecked.

Consequence for the console: an operator can route a task to any role directly (create, claim, admit), and that is observed working. Coordinator-driven delegation to another member is not observed, so the waiting, handed-off and delegated-task states of the run view are built from source reading only.
