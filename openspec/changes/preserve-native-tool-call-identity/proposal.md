# Preserve native tool-call identity

## Why

Kubo's r11 real-UAR verification failed after an approved `activate_skill` executed with an empty tool-call ID and canonical receipt acquisition rejected that identity. The selected UAR 1.3.0 source also overwrites an accumulated ID with empty continuation IDs. Original provider frame order is unknown; the observed post-execution rejection and destructive merge are separate established facts.

## What changes

Preserve authentic nonblank IDs across stream fragments, reject conflicting identities, and validate the complete native batch before approval or execution. Require a nonblank model call identity in the existing admission authority envelope. Preserve client-owned host IDs, canonical receipts, checkpoint binding, policy, denial and replay protections.

## Capabilities

- `tool-call-protocol`: immutable stream identity and complete native-batch admission.
- `tool-approval-workflow`: exact identity through approval, recovery and canonical persistence.

## Impact

Private Rust implementation and regression source only. The approved scoped attempts also exposed baseline fixture compilation drift, four test-contract contradictions and a stack-prologue fault through the hosted-root request boundary. Their complete corrective batch is recorded in `runtime-corrections-r4-plan.md`; it preserves host authority, finite accounting and all acceptance oracles. The subsequent approved library run passed the four contract fixes but timed out during fixture cold-store construction. `bootstrap-corrections-r5-plan.md` records the complete fixture bootstrap, failure-ownership and identity-namespace correction, with migration timing and runtime acceptance explicitly unverified. No dependency, schema, public API, provider/model route, UI, or vendored source changes. Source starts at `774469746fee4de99be41dc1cd231857067e4487`. An eventual corrected immutable artifact and explicit Kubo pin integration are required; this branch is not the published 1.3.0 image.

R5 subsequently passed826 library and6 admission integration tests, including the lifecycle repairs, before a protocol stack overflow stopped the first command. The consumed bounded diagnostic identified the composed ordinary admission path at an already-boxed kernel entry without measuring individual frame sizes. `protocol-stack-corrections-r6-plan.md` records the private boxed-admission boundary correction with unchanged compatibility signatures, request identities and protocol acceptance assertions. Customer build, extended client acceptance and release/consumer integration remain pending.

R6 completed the protocol target without overflow and passed the readonly approval case, alongside826 library and6 admission passes. Two baseline assertions expected raw validation data where the existing trusted-host contract nests it. `protocol-contract-corrections-r7-plan.md` records the complete test-only correction, preserving original identity/provenance, zero effects, retry and whole-envelope history equality. No production change is needed; the failed protocol target and unstarted customer/client gates remain pending renewed instruction.

R7's later protocol/build/local-client receipts and independent source reviews are retained separately. Product review exposed incomplete outward normalized identity publication despite repaired execution identity. `identity-emission-corrections-r8-plan.md` records the complete correction for public deltas/completions, original indexes and real manager dialogue/event regressions, preserving caller-owned durable authority. Runtime verification remains held through full source integration, corrective source review and renewed inventory/instruction. Local client exit2 continues to mean broader release/backend acceptance is pending.
