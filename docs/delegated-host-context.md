# Scoped full-harness private host context

Original AFC C14.4 uses the existing selected UAR full-run authority. This additive contract lets its paired host supply private MCP/admission resources without sharing the sidecar launch credential with BossFang. Source implementation alone does not certify a real approved/denied effect or pending cancellation; the root owns that completed delivery operation.

## Registration and safe identity

GET `/api/uar/full-harness/v1/capabilities` includes `delegated_host_context_v1: true`.

The paired host issues the existing scoped delegation grant, then uses its own host authentication and the same verified principal to POST `/api/uar/full-harness/v1/delegated-host-contexts` with:

```text
{grant_id,workspace_id,runtime_epoch,deployment_binding_id,
 definition:{id,version,digest},working_directory,mcp_servers,tool_admission}
```

The host supplies actual existing `RunMcpServerInput[]` and `RunToolAdmissionInput`, including private bridge authentication. Existing validation requires nonempty supported MCP inputs, an absolute existing non-root directory and the authenticated private loopback admission protocol. No caller principal field exists.

The response is HTTP201 with `Cache-Control: no-store` and only:

```text
context_id, grant_id, workspace_id, runtime_epoch,
definition:{id,version,digest}, binding:{id,revision,digest},
agent_id, expires_at
```

`definition` is exactly the native private binding's `package` immutable reference, also recorded as durable instance `definition`. It is distinct from `agent_id`, the actual ordinary-agent artifact resolved from that package entrypoint. The existing resolver requires exactly one AgentDefinition entrypoint and an admitted current binding. Binding inventory alone does not classify the entrypoint; registration returns `delegated_host_binding_not_agent` for those existing unsupported-agent errors and `delegated_host_binding_unavailable` for other binding admission failures.

Expiry is copied from the exact issued grant. DELETE `/api/uar/full-harness/v1/delegated-host-contexts/{context_id}` is host-only, idempotent and returns204. Deletion, grant revocation/expiry and process restart invalidate the context. There is no renewal, mutable registration or original-run migration.

## Admission and invocation verification

Only the opaque `delegated_host_context_id` is added to the existing scoped POST `/api/uar/full-harness/v1/tasks`. It participates in the original admission digest. The authenticated grant must be the context's original grant, with the exact logical workspace and target binding. Caller artifact/agent selectors, MCP/admission credentials, working directory, provider credentials and history overrides are refused for context-bound tasks.

The original reservation allocates run/task/native-task IDs. Its TaskReceipt gets optional `delegated_host_context` containing the identical safe registration receipt before native execution. The existing catalog resolver runs again; its exact package, binding revision/digest, agent and verified owner must match before private resource injection. Exact admission retries retain existing reservation/response semantics, including a truthful rejection.

Host-only GET `/api/uar/full-harness/v1/delegated-host-contexts/{context_id}/runs/{run_id}`, with `x-uar-workspace-id`, returns the original owning TaskReceipt. This lookup is registered before execution so the host can validate prepare/claim even while BossFang's admission response remains pending. Scoped grants cannot call registration, deletion or this host lookup.

The host verifies actual `PreparedToolInvocation` against the original owning run/context, canonical workspace path, resolved agent, bridge generation and exact admitted binding. The UAR port independently checks original root run, subject owner, canonical path and live grant/context at prepare/resolve/claim; existing collaboration claim revalidation still checks current binding authority. Cancellation and terminal finish can settle the original admission after expiry, without granting a new effect.

The full-harness/context `runtime_epoch` and `PreparedToolInvocation.runtimeEpoch` are different authorities. Do not compare them: the latter is RunManager's tool runtime epoch. The private bridge captures and checks its actual tool epoch independently.

## Host-first approval

The existing task `tool-approval` body remains `{expected_revision,approval_id,approved}`. For a context-bound task, UAR reads its existing live PendingApprovalView and POSTs privately to the captured admission base's `/uar/admission/v1/delegated-approval`:

```text
{version:1,contextId,grantId,runtimeEpoch,workspaceId,workingDirectory,
 definition,binding,taskId,nativeTaskId,runId,
 approval:{version,eventId,cursor,rootRunId,approvalId,issuerId,challengeId,
           admissionId,admissionOwner,callIndex,toolCallId,name,argumentsJson,riskReason},
 approved}
```

The existing admission owner wire spelling is `paired-host`. The host validates against its original owning UAR task/pending challenge and private prepared bridge admission, records the canonical human decision, then acknowledges:

```text
{version:1,contextId,runId,issuerId,challengeId,admissionId,approved,recorded:true}
```

All echoed identities and the decision must match. UAR rechecks the live original grant/context and unchanged pending challenge, then resolves its existing waiter. It does not create a pending approval store. Cancellation or expiry can win after host recording; no unrelated waiter is resolved. Original UAR admission, claim, execution and terminal settlement remain authoritative.

HTTP failure diagnostics expose POST operation/status or controlled transport/protocol reasons only. Private URLs, headers, callback credentials and MCP grant bodies never enter new persisted records, Debug output or diagnostics. Existing names-only host resource markers and canonical working-directory metadata retain their behavior.

## Honest refusal and delivery boundary

Stable context errors are `host_authentication_required`, `delegated_host_binding_not_agent` (422), `delegated_host_binding_unavailable` (409), `delegated_host_context_invalid` (422), `delegated_host_context_mismatch` (409), and `delegated_host_context_unavailable` (410). Approval coordination returns `delegated_host_approval_failed` or `delegated_host_approval_mismatch` without resolving the waiter. Port failures expose `delegated_host_admission_failed` with the operation and safe HTTP status when available.

An expired original context cannot authorize a later protected effect. A replacement registration is for a new explicitly admitted task, not a repair of the original task. No native tools are enabled, no host token is forwarded, no scheduler/budget/effect owner is replaced, and no remote federation is claimed.

Implementation policy forbids intermediate tests, Cargo checks, lint, builds, review and workflow dispatch. Completed UAR/Boss/BossFang builds and real approved/denied/pending-cancel operations remain root-owned and pending. The documentation planning commit's repository hook unexpectedly ran its GitHub Actions policy validator and commit-message lint; that unrequested check is not feature completion evidence and subsequent commits disable those hooks under the explicit operator policy.
