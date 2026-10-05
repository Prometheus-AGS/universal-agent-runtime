# Design

## Context

At 0402514a, peer native descriptors are advertised. execute_direct_tool chooses the native handler correctly, but prepare_and_admit_tool first sends every call through the run's HTTP host admission. Boss prepare returns422 because its parser requires governancePolicyRevision while PreparedToolInvocation explicitly renames that field to expectedGovernancePolicyRevision. Native server=None already produces the established builtin source marker; absence of mountedServerId is not the observed defect.

## Goals / Non-Goals

Repair exact v1 wire compatibility and separate UAR-owned coordination admission from paired-host effects. Keep all current installed claim revalidators, authority digests, persistence, cancellation, UAR/Cedar governance, budgets and Required descriptor approvals. Do not infer ownership from names, prefixes or BuiltIn/ModelOnly alone.

## Decisions

A typed AdmissionOwner marker defaults to paired-host on native handlers. Only the registered TeamTool implementation overrides uar-runtime. The orchestrator captures this property from its actual native registry handler after descriptor validation. Prepared invocation retains it privately with serde skip; deserialization cannot select runtime ownership. A composite admission port handles local controls using the captured run context and delegates other calls unchanged to the existing port. Both branches keep the original run/runtime/host binding. Local prepare/resolve/claim/cancel/finish remain inside the same durable ToolAdmissionRuntime. No standalone_ephemeral or root-only StandaloneToolAdmissionPort is substituted.

Claim retains original context matching, authority envelope, collaboration fences, governance engine/gate and durable claim intent. Cancellation after a local claim remains outcome-unknown rather than asserting effect cancellation; unclaimed controls can cancel normally. Native handlers still revalidate exact attempt owner/session and kernel team authority.

Remove only the wire rename attribute. Intentional expectedGovernancePolicyRevision keys inside canonical authority hashes remain exactly as before.

PendingApprovalSnapshot exposes admissionOwner as uar-runtime or paired-host. The normal request wrapper defaults paired-host; the run gate publishes captured invocation ownership. Root adjusts Boss to acknowledge bridge decisions only for paired-host while retaining run/attempt/approval/event/cursor checks. No ApprovalClass is relaxed.

## Risks / Trade-offs

Local receipt ownership must remain host-created and non-deserializable. A claimed interrupted native mutation retains uncertainty; restart never replays it. Source correctness does not prove native dispatch, approval or filesystem effects until the parent rebuilds and runs the packaged Work operation.

## Migration Plan

No data migration. Older pending records without admissionOwner default paired-host. Check in this plan, finish UAR and matching Boss source, then parent pins/builds/packages and runs the affected actual delivery gate. No worker compiler/tests/review/build.

### Durable provenance clarification

Existing ToolAdmissionEvidence records additionally carry admission_owner, default paired-host on historical records. Every prepared/claim/terminal/cancellation/restart projection preserves that typed ownership. It is explanatory metadata, never permission to replay. Public invocation deserialization still cannot create local ownership. This additive typed field requires no storage migration or new service.

## Source implementation boundary

The trusted TeamTool handler is the sole runtime-owner override. The orchestrator captures it from the actual registered native handler and prepares private invocation provenance. The original admission runtime wraps its existing paired-host port with the local control port; other tools delegate unchanged. Existing governance and installed claim revalidators still run before durable claim intent. Local prepare/resolve/claim validate the captured identities and authority envelope. Cancel/finish retain identity checks without renewing expired authority. Claimed local cancellation records outcome-unknown; durable evidence and restart projections retain admission_owner. Historical evidence defaults paired-host and never authorizes replay. Pending approvals expose admissionOwner; ordinary request callers retain paired-host defaults.

PreparedToolInvocation is not persisted by the production admission evidence path; the evidence store contains only identity/state records. The exact wire rename was removed without adding speculative read aliases or changing canonical authority hash keys. No compiler, tests, review or build ran. Scoped rustfmt was applied to the new module and changed responsibilities; unrelated formatting was retained. Plan and production commits suppress repository hooks invocation-locally under the explicit completed-delivery boundary instruction. Parent owns native/Boss packaging and the real Work gate, which remain unverified here.
