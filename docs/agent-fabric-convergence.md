# Agent Fabric Convergence — UAR Planning Note

Status: planning handoff; no product behavior or public contract is changed by this document.

Initiative: `agent-fabric-convergence`

Source baseline inspected: `c29af47be3439c69e1a3c124fdcf09ce4cbb5cba`

Date: 2026-09-25

## Finding

UAR does not start from zero on multi-agent execution. It already has a persisted
agent-thread kernel, parent/child lineage, typed inter-agent messages, policy
intersection, shared tree limits, root approval routing, graph execution, A2A
adaptation, lifecycle events, and a Markdown-to-descriptor compiler. The missing
story is the product contract above those primitives: reusable team and workflow
definitions, durable team instances and task boards, service placement, connector
effects, administrative surfaces, and a lossless document profile shared with
other runtimes.

The uncomfortable fact is that the current Draft RFC, compiler IR, runtime
`AgentArtifact`, OpenSpec child-thread contract, and partner manifests are not one
lossless definition model. Treating any one of them as already canonical would
hide conversion loss and create authority bugs. Convergence must preserve those
differences explicitly until the versioned schemas and adapters prove round trips.

## Current responsibility

UAR is the execution and policy authority for an admitted agent run:

- run creation, streaming, cancellation, recovery, checkpoints, and owner-scoped
  session/thread state;
- model/provider routing, request-scoped credentials, skills, knowledge, memory,
  MCP/native/WASM tool execution, and normalized results;
- compiled agent artifacts, graph/actor/child-thread execution, local and remote
  A2A adaptation, and lifecycle events;
- runtime governance, including Cedar-backed policy where the selected build
  profile supports it, plus non-overridable deny and host approval integration;
- persistence of UAR-owned run, thread, lineage, approval, artifact, and future
  team/task records.

The existing boundary remains valid: BossFang supervises UAR out of process, and
UAR executes agents without depending on BossFang. Application workflow ownership
stays with the application that created the workflow unless a later change assigns
that exact workflow object to UAR. A channel transport, event bus, memory service,
or UI must not become a second run executor or workflow owner.

Evidence:

- [`ARCHITECTURE.md`](ARCHITECTURE.md) assigns authentication, policy,
  orchestration, persistence, and transport to Rust/Axum and recommends an
  out-of-process BossFang-supervised service.
- [`SPECIFICATION.md`](SPECIFICATION.md) names run lifecycle, compilation,
  sessions, the descriptor schema, and governance as UAR capabilities, while
  assigning application-level workflow orchestration to the consuming
  application.
- [`agent-thread-kernel`](../openspec/specs/agent-thread-kernel/spec.md) requires
  one persisted thread kernel, narrowing child policy, typed messages, tree-wide
  limits, root approvals, observable lifecycle, and A2A mapping.
- [`runtime/thread`](../src/uar/runtime/thread/mod.rs) implements owner/root/parent
  lineage and canonical task paths; [`spawn.rs`](../src/uar/runtime/thread/spawn.rs)
  implements explicit history forks; and
  [`policy_intersection.rs`](../src/uar/runtime/thread/policy_intersection.rs)
  narrows resources, credentials, sandbox permissions, budgets, and approval.

## Future UAR multi-agent and subagent story

UAR should expose one kernel through several orchestration views rather than add
another agent loop:

1. **Agent definition.** An immutable, versioned description of behavior,
   required skills/tools/policies, model requirements, context strategy,
   input/output schemas, and deployment constraints. A definition requests
   capabilities; it grants none.
2. **Deployment binding.** A host-owned binding from a definition revision to a
   runtime instance, provider/model, credentials, workspaces, MCP servers,
   installed skills, and policy/grant revisions. Secrets and grants never enter
   a portable package.
3. **Agent instance.** An owner-scoped, addressable activation of a definition.
   It has a durable identity and bounded inbox but receives fresh admitted
   context for each turn. Resident, on-demand, and request-only are lifecycle
   policies, not different execution engines.
4. **Team definition and instance.** A versioned set of member roles,
   eligibility constraints, review separation, communication rules, aggregate
   budgets, and task contracts. A team instance records exact member and
   definition revisions. Member permissions are never unioned.
5. **Workflow definition and run.** A typed graph of tasks, dependencies,
   waits, joins, retries, compensation, approvals, and effect boundaries. Each
   workflow object has one durable owner. UAR may own UAR workflows; BossFang
   continues to own BossFang application/channel workflows that delegate runs
   to UAR.
6. **Representation grant.** A private, revocable consent record for a human
   digital twin. It binds represented subject, acting principal, audience,
   purpose, allowed actions/resources, policy revision, expiry, and escalation.
   It is separate from the public agent definition and cannot be inferred from
   a role such as CEO or CFO.

All paths—linear, graph, actor, team task, A2A, or delegated BossFang run—must
enter the existing trusted thread/run kernel and emit one correlated lifecycle.
Parent and child identity, task/run/attempt/effect identity, authority lineage,
budgets, cancellation, and effect receipts must remain inspectable across
adapters.

## Governance contract

Cedar adds a material advantage only if it evaluates real execution facts. The
future policy request should bind at least the authenticated subject, initiating
human, executing agent instance, represented subject when present, root and
current thread/run, team/task, action, resource, audience, workspace, definition
and deployment revisions, policy/grant revisions, effect identity, and current
budget/lease state.

The composition rule is restrictive:

- definition declarations constrain eligibility;
- deployment and team policy can narrow;
- parent policy and delegation can only narrow;
- current Cedar policy can deny or require approval;
- an authenticated, exact host decision may satisfy a required approval;
- no prompt, role name, signature, message, transport, or child artifact grants
  authority by itself.

Policy is re-evaluated at the protected effect boundary. A completed external
effect is tied to its durable intent and receipt; an uncertain result is exposed
as uncertain and is not automatically replayed.

## Dependency on the accepted UAR P1 contracts

Future work consumes checkpoint `D-UAR-P1`; it does not amend the active P1
integration phase. The following accepted contracts are prerequisites:

- The host owns profile/workspace/conversation identity, canonical history,
  provider secrets, catalog restrictions, and the human approval decision. UAR
  owns the execution loop.
- Launch-token authentication and the host-asserted conversation principal are
  distinct. One supervised sidecar may multiplex sessions only while every run,
  stream, approval, cancellation, resume, and child operation remains bound to
  the verified principal.
- The admitted run envelope freezes agent/provider/model/credential, workspace
  and allowed roots, catalog, policy/reasoning, and sanitized history. Child and
  resume paths inherit or explicitly reacquire matching bindings.
- A prepared tool invocation has a unique invocation identity and freezes root
  and executing run, owner/workspace, runtime and host epochs, catalog/server/tool
  identity, policy revisions, and validated arguments. A model call ID or digest
  is not authority.
- Host and UAR policy compose restrictively: deny is final, ask requires one
  identified human decision, and auto may skip UI but still creates an exact
  admitted record before execution.
- Approval state progresses through prepared, waiting/acknowledgment,
  authorized, claimed, and terminal states. Persist claim intent before the
  effect. A lost response after claim becomes outcome-unknown, never a new
  automatic claim.
- Renderer detach, SSE reconnect, process restart, and cancellation are distinct
  events. Reconnect reattaches to an existing pending identity; restart
  invalidates pending authority; root cancellation covers descendants.
- Only a safe display projection reaches UI or logs. Complete arguments,
  credentials, authority records, and argument digests remain in the trusted
  host.

Until the P1 checkpoint records the actual versioned fields and accepted source
commits, convergence work that touches those contracts remains blocked.

## Codex CLI Rust patterns worth adapting

The reference implementation under `/Users/gqadonis/Projects/references/codex`
offers implementation patterns, not an authority model to copy verbatim:

| Codex pattern | UAR adaptation |
| --- | --- |
| Root-scoped `AgentControl` shared by descendants, separate from the global thread manager | Keep one root-scoped control plane over UAR's existing thread service; do not add per-feature registries or loops. |
| Canonical `AgentPath`, persisted spawn edges, and restored cold metadata | Reuse UAR's canonical path and richer owner/root edge record; restore identity without silently starting work. |
| Explicit `spawn`, `list`, `wait`, `send`, `followup`, and `interrupt` tools | Preserve UAR's descriptor-registered operations; distinguish mailbox delivery from triggering a turn and keep cancellation outside a blocked turn. |
| `fork_turns = none | all | N` with filtering of reasoning, tool traffic, usage state, and stale instructions | Extend UAR's existing `HistoryForkMode` only through a typed context-selection contract; never clone authority or tool traces as dialogue. |
| Shared rollout budget, execution limiter, and reservation guards that release on drop | Use UAR's existing tree limits and budget reservations; make admission/dispatch/settlement explicit and release only proven unused reservations. |
| Bounded root authorization snapshot | Supply Cedar and host approval with bounded, versioned evidence. Do not turn selected conversation prose into a grant. |
| LRU residency for completed/idle workers and cold rehydration | Apply only to future addressable instances after durable inbox, ownership epoch, and activation contracts exist. |
| Structured lifecycle activity and typed context fragments | Emit redacted, bounded, versioned records; prompts and hidden reasoning stay out of lifecycle events. |

Two Codex choices need deliberate strengthening in UAR: its small persisted spawn
edge is insufficient for UAR's owner/root/artifact/policy needs, and its root
conversation snapshot is evidence for review rather than a substitute for Cedar
facts or P1 exact approval identity.

## Document standard direction

Do not expand the existing agent RFC into one document that also represents a
team, workflow, running instance, deployment, and private human consent. C03
should define five public, versioned document kinds—`AgentDefinition`,
`TeamDefinition`, `WorkflowDefinition`, `DeploymentBinding`, and a package
manifest—plus a separately stored private `RepresentationGrant`.

Each public document needs a stable kind/schema/version, immutable content
digest, owner and provenance, compatibility constraints, typed references,
required-versus-optional semantics, explicit unsupported-field behavior, and a
canonical machine representation. Markdown may be the authoring envelope, but
the machine schema and deterministic descriptor are the execution contract.

The adapter acceptance rule is lossless for required semantics: compile, store,
export, import, and bind the artifact; prove the same required skills, policy,
schemas, budgets, and execution constraints are effective. Unknown mandatory
fields or unsupported required semantics block execution. Conversion warnings
are field-addressed and cannot silently widen authority.

## Non-goals

- Reopening, expanding, or delaying the active P1 approval/session/release work.
- Adding another agent loop, thread store, provider registry, approval system,
  workflow owner, or process supervisor.
- Making UAR depend on BossFang, Flint services, Notion, Slack, Jira, or another
  connector for standalone operation.
- Treating Cedar policy text embedded in a definition as an installed grant.
- Treating a team as a shared transcript, shared credential pool, or union of
  member permissions.
- Claiming exactly-once external effects, live-session migration, automatic
  cross-host takeover, or unlimited autonomous/background execution.
- Shipping a role-labelled human digital twin without explicit consent,
  provenance, revocation, audience, and escalation.
- Moving application/channel workflow objects into UAR merely because they
  invoke a UAR agent.

## Next repository-scoped KBD child

Create `uar-governed-action-boundary` only after `D-UAR-P1` is accepted. This is
the UAR portion of initiative C02 and should have exclusive ownership of the
governance composition and real direct/managed/embedded effect admission paths.

Its exit criteria are:

1. inventory every real protected effect entry point and its current identity,
   policy, approval, intent, claim, and receipt path;
2. define the versioned Cedar action/resource/context vocabulary using the P1
   principal, run, invocation, epoch, catalog, policy, and payload bindings;
3. make governed-profile missing/invalid policy fail visibly before an effect;
4. recheck revocation and payload/binding identity after an approval wait;
5. prove forged identity, revoked approval, changed arguments, and stale epochs
   do not produce effects across the complete direct/managed/embedded boundary;
6. preserve the explicit constrained local posture and standalone UAR behavior.

After that child passes, create a separate `uar-collaboration-document-profile`
child for C03. Keep schema/compiler/adapter work out of the effect-boundary child
so the accepted P1 contract remains reviewable and source ownership stays clear.
