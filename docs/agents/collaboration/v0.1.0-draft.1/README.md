# UAR Collaboration — Official Draft Specification 0.1.0-draft.1

Status: **official draft; design approved by the operator on 2026-09-26; runtime conformance not demonstrated.** Normative owner: Universal Agent Runtime (UAR). Profile identifier: urn:prometheus:uar:collaboration:0.1.0-draft.1. This is a project-owned identifier, not a claim of external standards registration.

This additive companion to [RFC-0001](../../AGENTS_SPEC_RFC.md) specifies reusable agents, subagents, teams and nested teams. Existing agent formats remain supported according to their own version. A team is a durable UAR instance whose bounded turns run through the existing kernel; The Boss and protocol adapters expose the same authoritative state.

## Read the contract

- [Definitions and binding](definitions.md)
- [Lifecycle, scheduling and recovery](runtime.md)
- [Governance and effect boundaries](governance.md)
- [Administration and API behavior](administration.md)
- [AG-UI](protocols/ag-ui.md), [A2UI](protocols/a2ui.md), [A2A](protocols/a2a.md)
- [Compatibility and migrations](compatibility.md)
- [Implementation phases and acceptance](implementation.md)
- [Schemas](schemas/README.md), [examples](examples/README.md), [protocol traces](traces/README.md)
- [Evidence and provenance](sources.md)

MUST/MUST NOT express requirements of a future conforming implementation. They do not describe all currently shipped behavior. SHOULD permits a documented deviation with its operational consequence; MAY denotes an optional, advertised capability. Portable definition, installed binding, durable instance, task and execution attempt are separate identity classes.

## Conformance classes

| Class | Claim permitted | Evidence required | Current status |
|---|---|---|---|
| Document profile | Valid shape, resolved immutable dependencies and declared semantics | Schema agreement plus semantic checks and conversion report | Illustrative artifact validation only |
| Durable local runtime | Admitted teams/tasks recover through one UAR kernel with current authority | I2 complete production integration gate | Pending |
| Team interfaces | Negotiated AG-UI/A2UI/A2A projection of same state | I3 interoperability and isolation gate | Pending |
| Customer workflow release | Both workflows and supported installed platforms work | I4 Windows x64 and Mac ARM64 receipts | Pending |
| Federated runtime | Remote members, cross-host ownership and external harness effects | Separately approved I5 profile evidence | Out of first release |

An implementation MUST publish capabilities per version and MUST NOT infer runtime conformance from this document, a schema-valid object, source presence or a successful build. Unsupported required semantics prevent activation. Enhanced protocol features require negotiation; ordinary supported clients retain their base behavior.

First release requires development AND feedback-to-approved-issue workflows in one UAR instance. Marketing/design examples illustrate composition. Executive roles and human representation are future extensions with separate grants. AG-UI endpoints for teams do not imply distributed execution.

## Publication and change policy

Normative documents and proposed schemas live exclusively under this documentation directory; no runtime generation path imports them in this phase. Changes to a published draft receive a new draft revision and conversion notes. Definition semantic version, profile version, wire-envelope version and mutable instance revision MUST NOT be conflated. Publication establishes the design baseline; operator installed acceptance remains a separate record.
