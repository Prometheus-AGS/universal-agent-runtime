# Phase specification: UAR outstanding issues v1.1.1

Phase: `uar-outstanding-issues-v1-1-1`. Stage: Spec.
Authoring started 2026-10-15 and resumed 2026-10-16. Baseline:
`91a091d647fe2ce63651c07140172aba2c6c6302`.

## Scope and authoritative artifacts

One coordinated OpenSpec change owns this phase:
[uar-outstanding-issues-v1-1-1](../../../openspec/changes/uar-outstanding-issues-v1-1-1/proposal.md).
It contains proposal, design, tasks, verification contract and eight capability
deltas. KBD owns lifecycle position and the complete-phase gate. Plan will order
and register the implementation ledger using the stable IDs below; Spec does not
mark any implementation item complete.

| Capability delta | Stable tasks | Issue/observable |
| --- | --- | --- |
| provider-diagnostic-status | R1,R5,S1 | #385 safe binding stage/category/reference; no secrets or provider/tool effects after admission refusal. |
| model-path-resiliency | R2,R3,R5 | #380 truthful manager completion and native Anthropic byte framing, terminal/error behavior and preserved #384 configuration. |
| tool-call-protocol | R4,R5 | #380 MCP typed application/transport/uncertain outcome, no replay and immutable earlier receipts. |
| tenant-isolation | T1,T2,T3,T4,S3 | #380 credential-only installation binding, explicit shared policy, exact fallback and approved conditional migration/preview. |
| dot-durable-budgets | S1,R1,S4 | #385 actionable eligibility without dropped budgets or liabilities. |
| dot-durable-ingestion | S1,S2,S4 | #385 same safe eligibility and durable bytes/job before acceptance. |
| knowledge-rag-product-certification | S2,S4 | #385 authenticated reupload, retained legacy data and explicit source/identity limits. |
| kubo-runtime-conformance | L2,L3,L4,V1,V2,P1,P2,P3 | #379/#383 preservation/reconciliation, v1.1.1 immutable release and complete acceptance for both published architectures. |

#382 remains an external dependency tracker. Its UAR status is part of P3;
external implementation and cancelled Dot qualification are outside this phase.
Existing supervisor recovery requirements remain binding and are exercised only
through the bounded S2/S4 recovery controls, not expanded into a new program.

## Compatibility and selected constraints

- Public binding error codes and AG-UI RUN_ERROR code/message fields remain.
  Safe category/reference is added to the message and structured log fields.
- Explicit budgets and durable admission remain; no blind remote attestation.
- Credential mode omitted means existing single-owner compatibility, even when
  remotely hosted. Shared mode is explicit and requires complete verified
  ownership plus exact issuer/tenant/provider/source fallback grants.
- Local/issuerless credential operations require an operator-provisioned stable
  installation UUID and approved migration of old fixed-local v2 credentials.
  This is an explicit local-credential compatibility break. External owners and
  general session/KB/schedule identities do not change.
- Legacy credential request bodies stay compatible; conditional preview-guided
  apply and fixed-local migration are additive. Source/journal retention and
  exact replay prohibit overwriting rotations or resurrecting deletions.
- KB recovery is reupload of retained originals under the correct verified owner.
  Old IDs/chunks are not imported. Team takeover still requires valid exclusion.
- Liter and Surreal pins remain unchanged. SurrealDB 3.3.0 in versions.toml takes
  precedence over stale role text. No UI, global identity or vendor edits.

## Complete-phase boundary

All planned production source, additive schemas, fixture code, runner, operational
settings and required documentation must be implemented before L4 can record an
implementation-complete inventory. A finished task does not authorize tests.
Scope revisions join the same boundary. V1 isolated review and V2 local acceptance
follow L4; no validator, model probe, test, compiler, lint, formatter, build or
packaging check runs before it.

The skill's earlier artifact review and template fixture-authoring sequence are
superseded by the operator's explicit full-phase rule. Fixtures may be authored
before the boundary, but no verification runs. GitHub Actions remains limited to
artifact deployment builds/publication and narrow deployment validation.

## Acceptance and unresolved evidence

The [verification contract](../../../openspec/changes/uar-outstanding-issues-v1-1-1/verification.md)
selects exact future local/arm64/amd64 runner invocations. Every behavior names a
production entry point, real collaborators, negative control, isolation,
prerequisites, evidence and limitations. The runner is an L3 deliverable; no
command or receipt is claimed to exist as a result of this specification.

The complete group set is binding, streams, mcp-outcomes, credentials, recovery,
combined and kubo. Source integration and image acceptance are separate;
published mode runs the immutable image without rebuild or source patch.
Missing backends/groups/observations are blocked, not silently skipped. Native
and emulated architecture execution must be reported accurately.

Actual customer recovery still needs the live persisted agent, safe failure
category, durable backing, ownership/source inventory and any qualified team
exclusion evidence. Earlier permission was specified read-only get/describe/logs,
not rollout or data mutation. `site-chat-kill-switch` stays untouched. The bundled
skill warning remains triaged but unproven; source evidence of a new defect would
revise the phase before its full implementation boundary.

## Process and next stage

Runtime, trust and state roles authored separate deltas and acceptance fragments;
the lead integrates proposal/design/tasks/verification and release scope. No
isolated artifact approval is claimed. Memory recall was unavailable; local
append-only lessons on durability, private owner namespaces and whole-phase
testing are cited in design.md.

Next: `/kbd-plan uar-outstanding-issues-v1-1-1`. Plan must preserve the selected
behavior, register all implementation and later evidence tasks, assign exclusive
files and serialize shared-source/schema handoffs. Do not skip directly to Apply
because OpenSpec artifacts exist.


## Execute scope correction — 2026-10-16

The operator explicitly limited validation to real integration of the known
scenarios supported by this phase. This supersedes earlier full-Kubo, whole-target
and routine schema-validation instructions in this document. Production scope,
implementation-first ordering, isolated artifact review and release provenance
remain. Required acceptance groups are binding, streams, mcp-outcomes, credentials,
recovery and combined on the candidate and both published child images. No
unrelated tests, unit-only substitutes or broad validation sweep is selected.
verification.md is updated accordingly; each actual integration case must be
registered from implemented fixtures before L4.
