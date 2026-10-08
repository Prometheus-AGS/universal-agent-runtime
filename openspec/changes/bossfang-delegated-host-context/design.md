# Design

## Context

See proposal.md. The existing full-harness authority reserves admission/task/run IDs before execution. `admit_run` resolves an owner/workspace private binding before attaching request-owned MCP resources. Scoped grants capture the verified user but currently expose no grant identity to handlers. The existing approval broker exposes a live issuer/challenge/admission snapshot.

## Goals / Non-Goals

**Goals:** Extend those trusted boundaries for original C14.4 real approved/denied effects and pending cancellation. The Boss remains human decision/admission authority; UAR remains the only executor and budget/effect settlement authority.

**Non-Goals:** New scheduler, persisted private resources, host-token forwarding, native file tools, remote federation, grant renewal or replacement of an original run's context.

## Decisions

Host-only POST `/api/uar/full-harness/v1/delegated-host-contexts` accepts snake_case `{grant_id,workspace_id,runtime_epoch,deployment_binding_id,definition:{id,version,digest},working_directory,mcp_servers,tool_admission}`. Identity comes from the live grant, not the request. Existing catalog resolution captures `binding.package` as definition (the same native immutable identity reported by durable instances), the private binding_ref and actual agent artifact ID. Existing host attachment validates resources and canonicalizes the directory. GET capabilities advertises `delegated_host_context_v1:true`. DELETE collection `/{context_id}` is host-only; no mutable registration or renewal exists.

The safe receipt is `{context_id,grant_id,workspace_id,runtime_epoch,definition,binding:{id,revision,digest},agent_id,expires_at}`. An optional task request `delegated_host_context_id` participates in the original canonical admission digest; its exact grant/workspace/context identity is checked. The existing reservation remains the retry authority. The safe receipt is attached to the reserved task before execution. Host-only GET `/delegated-host-contexts/{context_id}/runs/{run_id}` returns its original task receipt for prepare/claim verification before a delegated admission response arrives. Existing binding resolution is checked again before injection, and the host admission port checks live grant/context authority at prepare, resolve and effect claim. Caller private resource/cwd/selector overrides are refused. Credentials, MCP and callback material remain private process memory and never serialize into persisted records. Existing canonical working-directory metadata is preserved. Full-harness epoch and native tool epoch remain distinct.

The task's original context coordinates tool approval using POST on its private admission base `/uar/admission/v1/delegated-approval`. CamelCase request is `{version:1,contextId,grantId,runtimeEpoch,workspaceId,workingDirectory,definition,binding,taskId,nativeTaskId,runId,approval:<existing PendingApprovalView>,approved}`. Host response must exactly echo `{version:1,contextId,runId,issuerId,challengeId,admissionId,approved,recorded:true}`. The host validates the original pending UAR challenge and owning task, then records its canonical human decision. UAR checks the echo, lease and unchanged pending challenge before resolving its existing waiter. Callback failures leave the waiter pending and return stable codes, never transport secrets.

Alternatives rejected: caller-supplied host token/resources would elevate scoped authority; host recording after waiter resolution races actual effect claim; custom pending records would create a competing approval authority.

## Risks / Trade-offs

- Original grant/context expiration → protected effects fail closed with an actionable refusal; an original run cannot silently adopt a new grant.
- Host decision recorded while UAR cancellation wins → no waiter resolution/effect; host keeps its honest canonical decision and existing cancellation lifecycle.
- Process restart → private contexts disappear with the existing process-ephemeral full-harness authority.
- Bound identity ambiguity → compare the native package entrypoint immutable reference, exact private binding_ref and resolved artifact identity; do not equate package ID with agent artifact ID.

## Migration Plan

Additive capability and optional opaque request field. Existing callers without a context retain behavior. Deploy UAR, The Boss private bridge integration and BossFang safe-ref plumbing together at root's serialized completed boundary; rollback removes capability and refuses new context use. No dependency or persistence migration.
