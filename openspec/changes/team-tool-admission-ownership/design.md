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

## Observed team effect owner mapping repair — 2026-10-05

The actual packaged operation c14-95842c8e-1564-421f-9490-716bdee42ce5 reached worker run 10526982-fb09-4221-8b81-10c97aa1d488 and filesystem__read, whose retained native result is the fixed error Paired host rejected tool admission (HTTP 409). The host preparation response body is not retained, so this receipt alone does not classify its guard. The source establishes a mandatory mismatch at that guard: collaboration owner_key and ActorOwner.presentation_owner_key use the same length-prefixed subject/tenant partition, and dispatch requires the attempt owner to match that verified partition. RunExecutionRequest.with_verified_owner intentionally sets user_id to the raw verified subject; RunManager currently reuses that raw session owner for ToolAdmissionContext. The Boss team binding requires the invocation owner to equal the partitioned team owner before any authority-envelope or duplicate check. Agent projection retains the exact member definition ID, and this team attempt retains its own run as the invocation root.

The bounded correction changes only tool-admission owner construction for a collaboration-bound team attempt. Derive the admission owner from the verified ActorOwner.presentation_owner_key, require equality with both the admitted collaboration binding owner and captured attempt owner, and construct the existing ToolAdmissionContext using that owner. Ordinary admission retains its existing raw owner. Session, thread and policy identities remain the raw verified subject; with_verified_owner is unchanged. The existing admission-context error path handles absent or mismatched verified team ownership. This check is at the actual authenticated identity-to-effect boundary and cannot accept model-supplied identity.

Lease and authority digests retain their existing schemas and bind the corrected captured owner consistently; root/attempt/member/workspace/runtime/host fences, installed claim revalidators, governance, approvals, budgets, cancellation and cleanup remain unchanged. No new route, persisted identity format, permission or fallback is introduced. Native build, packaged host admission and repository-effect acceptance remain unverified until the parent performs the actual affected delivery boundary. No worker tests, compiler, build or operation are authorized; commit hooks that invoke those gates are suppressed per invocation under that boundary instruction.

Implementation is confined to RunManager tool-admission context construction: only a captured collaboration team attempt selects the verified partition owner after exact binding/attempt equality. Context construction and identity validation share the existing tool_admission_context_failed terminal path. All other callers still pass their original owner_id; no authority hash key, host guard, session or thread identity was changed. This is source implementation only, with no worker validation gates run.
