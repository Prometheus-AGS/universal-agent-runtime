# Presentation negotiation implementation contract

Phase: Execute. Change: select-and-observe-presentations. Implements the approved phase plan; no new release or UI redesign.

## Request and resolution

The optional HTTP extension fields are `presentation_mode` (`auto`, `text`, `a2ui`, `hybrid`) and `client_rendering` (`a2ui_profiles`: string array). The first-party profile identifier is the existing `uar.a2ui/1`, which fixes the supported wire version and component catalog. Unknown profiles do not imply compatibility. The same typed negotiation travels with the host-owned RunExecutionRequest; these fields contain no principal, template authority or target run identity.

| Input | Admission result |
| --- | --- |
| Both fields absent/null | Legacy behavior, under existing governance |
| Explicit text | Text; no surface publication |
| Support only | Requested auto |
| Non-text mode without declared support | Text; client-rendering-not-declared reason |
| Non-text mode with no compatible profile | Text; incompatible-profile reason |
| Compatible non-text mode with no eligible templates | Text; no-eligible-templates reason |
| Compatible support and eligible templates | Requested auto/A2UI/hybrid intent |

The resolved mode is an output ceiling, not a publication receipt. A2UI asks for surface-first output plus a brief accessible summary. Hybrid asks for substantive text and surface output. Auto permits host-governed selection without promising any surface. Later publication/terminal events distinguish no-surface generation and fallback from admission intent. Legacy mode is represented explicitly, not mislabeled as a negotiated Auto request.

## Ordered implementation

1. `src/uar/a2ui/presentation_selection.rs` and module export: typed serializable negotiation, requested/effective modes, fallback reasons and pure deterministic resolution. Extend `src/uar/runtime/turn/request.rs`, primary HTTP mapping in `src/server.rs`, OpenAI adapter types/routes, native run/resume DTOs and ACP DTO/parameter parsing. Retain negotiation plus admission intent in run context; A2UI action continuation carries that negotiation and rejects corrupt saved values instead of upgrading to Legacy. The request contract alone does not establish output enforcement.
2. Freeze validated eligible content/id/revision in a trusted-host run snapshot, retaining owner/run binding outside model arguments. Attach it to live/inherited run bindings. Govern template rendering, legacy tool output, policy artifacts, direct surface submissions and delegated publication with one captured output ceiling. Inherited and remote output cannot widen a parent's ceiling. The host chooses actual run/surface IDs. Templates remain immutable for an admitted run.
3. Persist/project selection and actual publication through existing normalized run events and typed entity hooks. Event authority stays in the host, with replay/correlation intact. UI remains a projection and never claims client display from publication alone. Follow the mandatory UI design/critique routing before adding run-detail controls.
4. Only after all phase code: compatibility, boundary, persistence, host and browser tests; Impeccable polish/finish and independent acceptance.

AG-UI event-contract guidance applies idempotent correlated events, explicit failures and truthful unknown-event handling. A2UI surface-contract guidance treats validated messages as projections, never permissions. Existing run/event envelopes and safe template validation are reused. User-requested end-of-phase testing overrides earlier per-skill test timing; the tests themselves remain required at that boundary.

The uncomfortable limit: a compatible profile declaration cannot prove a client actually displayed a surface. No display receipt is invented. Supplying a profile never expands resource eligibility. Code-only checkpoints must identify any ingress/enforcement or replay path still unwired.

## UAR-to-UAR output restriction transport

Carry an optional `presentation_negotiation` in the existing authenticated, digest-acknowledged UAR delegation contract. Omit it for Legacy; for a negotiated parent, retain its negotiation except that a resolved Text ceiling is transported as explicit Text. Compute this from the frozen snapshot narrowed by the actual remote-child policy, not from spawn arguments. The peer threads this immutable contract field through its governed actor session into every admitted run. It resolves against its own authenticated owner catalog and local policy; source template contents are not copied into another runtime's catalog. Each peer run freezes its own eligible revisions. The existing resource ID ceiling still intersects local eligibility.

The optional field participates in the exact contract digest and equality checks. A peer that cannot acknowledge it cannot silently fall back to an unconstrained run. Omitted negotiation retains existing legacy wire behavior; no contract-version or dependency pin change is required for that omission path. A2A terminal task text remains text data; a remote acknowledgement is neither attestation nor proof of client display. Surface generation on a remote run does not, by itself, establish publication on the source run.

Review the concrete Presentation resource selection in the delegation contract alongside this transport: an Inherit/Auto/All value is not a concrete authority ceiling. Do not silently widen an older or malformed contract while retaining its acknowledgement.

Compatibility review: preserve wire presence before defaults erase it. Only a contract omitting both Presentation selection and negotiation qualifies for the older-peer compatibility path. Keep that received contract immutable for equality/acknowledgement, and apply an additional target-local Presentation=None restriction to a separate execution-policy copy on every turn. This preserves legacy manual A2UI without granting the new template capability. Explicit non-concrete selections, or negotiation without a concrete resource ceiling, must be rejected. Historical digests used typed serialization: a presence-preserving legacy wire representation must reproduce that serialization, not hash raw incoming bytes or silently normalize the contract. Verify against an old-peer fixture at phase end. This is reviewed design, not implemented compatibility or interoperability evidence.

Implementation checkpoint2026-09-05: this transport and presence-preserving typed policy are now implemented and compile. Outgoing historical wire is selected only when negotiation is absent and the concrete template grant is None with no IDs/denials. Its internal omitted field is canonicalized to the deserialization default for equality, while target execution still overlays None. A real template grant or any negotiated restriction retains the new wire; older peers may reject it, with no unconstrained retry. Compiling this code does not establish historical digest or live interoperability acceptance; those fixtures remain phase-end work.
