# Governance, authority and privacy

## Existing boundaries remain authoritative

The existing Cedar/runtime/host approval composition is reused. Authentication identifies the actor; stored team/task IDs select resources only after owner/workspace checks. Policy evaluation uses current subject, action, resource, arguments and context. A portable definition, role, signature, model message, endpoint or UI control grants no authority.

| Logical action family | Resource/facts | Enforcement point |
|---|---|---|
| Definition register/replace/install | Catalog ID/digest, owner, expected revision | Catalog mutation and binding |
| Team activate/member change | Instance, workspace, resolved membership/limits | Collaboration admission |
| Task submit/claim/delegate | Parent task, role, child bounds, ownership epoch | Transaction and turn admission |
| Message/broadcast/context read | Sender/recipient, task, selected content labels | Inbox enqueue and delivery/replay projection |
| Model/tool/connector effect | Exact destination/arguments, credential audience, current grant | Existing effect boundary immediately before effect |
| Surface action/recovery override | Server-resolved surface/task/member, current revision | Trusted host/API action handler |

These are logical policy mappings, not declarations that new Cedar action names already exist. I2 maps them onto the actual entity schema and rejects missing required policy facts. New roles cannot bypass resource-side enforcement or grant their own approval.

## Restrictive delegation

Effective child authority is the intersection of host limits, current principal grants, parent constraints, installed binding and child requests. Context sharing is independently authorized; permission to run a task does not automatically grant another member's history. Root aggregate budgets, filesystem/workspace scopes and network restrictions cannot reset at subteam boundaries.

The Boss remains the trusted client for supported human approvals. The approval identity binds issuer, subject/actor, audience, run/task/effect identity, destination and canonical payload. Re-evaluate after waiting and on restart; deny changed/revoked scope. A2A input-required or an AG-UI message is not equivalent to an approval. A model-generated text saying approved cannot satisfy the boundary.

## Surface and protocol inputs

A2UI actions, A2A tasks, imported definitions and tool output are untrusted inputs at real trust boundaries. Server-side surface ownership resolves intended team/task/member; client fields cannot redirect privileged actions. Cross-owner lookup is denied before returning content. Events, snapshots, artifacts and logs are permission-filtered again on reconnect. Historical authorization does not guarantee present disclosure rights.

Only approved declarative catalogs render. Do not treat generated markup as privileged code or provide an arbitrary main-process bridge. Credential values, executable grants and private reasoning MUST NOT enter portable files, model-visible event streams or ordinary preferences responses. Logs redact secret-bearing fields before durable append.

## Uncertain effects and human override

Operator controls include inspect, cancel, drain, suspend, reassign eligible work and reconcile an uncertain effect. Overrides are auditable commands with expected revision and current authorization. Reconciliation must state what evidence exists and whether a duplicate risk remains; it cannot fabricate a successful connector response.

Feedback publication specifically requires approval of the destination and exact issue payload in the first release's acceptance story. A product or critic agent cannot self-approve. Future executive representation requires private, revocable grants from the human and relevant organization; personal consent alone cannot authorize organization spending or impersonate an approval officer.
