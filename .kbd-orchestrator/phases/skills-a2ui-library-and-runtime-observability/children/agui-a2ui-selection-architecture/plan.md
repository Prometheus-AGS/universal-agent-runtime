# PLAN: agui-a2ui-selection-architecture

Project: Universal Agent Runtime. Date: 2026-09-04. OpenSpec: YES.
Three existing changes; corrective tasks stay under their original identities.
The user approved reusable UI templates and autonomous implementation. Tests run only after all phase code is in place; Tier 0 compilation/static checks still follow cohesive edits.

## Ordered change list

1. **establish-presentation-catalog-workspace** — durable templates and production management.
   - Scope: domain, persistence, API, normalized entity graph, production UI.
   - Depends on: none. Agent: Codex/Astra. Complexity: L. Customer value: high.
   - Define a Presentation with stable ID, owner, revision, title, description, enabled state and a declarative A2UI template. Validate at the trusted host boundary with the existing protocol parser, including one surface and an unambiguous root graph. Assign run-specific surface IDs during instantiation without string interpolation or execution.
   - Add owner-scoped CRUD to the persistence contract and actual memory, PostgreSQL and Surreal implementations. Host routes derive ownership from authenticated context; updates use an expected revision to reject lost edits. No silent default/no-op persistence implementation.
   - Add typed Presentation records and editor drafts in the normalized entity graph; domain actions sequence mutations and transports perform I/O. Components use public domain hooks. Add a production registry/editor/preview using the incumbent admin design system; leave the A2UI tester development-only. Preview does not dispatch actions.
2. **scope-presentation-capabilities** — assign templates safely.
   - Scope: policy resolver, agent extensions, conversation/global settings, domain hooks and controls.
   - Depends on: catalog. Agent: Codex/Astra. Complexity: M. Customer value: high.
   - Add Presentation ResourceSelection to existing global/agent/conversation/turn scopes and the immutable effective policy. Resolve against enabled, owner-accessible templates; reuse deny-safe set intersection. Preserve legacy deserialization.
   - Add graph-backed assignment controls beside existing resource settings. Show inherited versus explicit choices and unavailable IDs; do not silently convert inheritance to an explicit list. Carry resource ceilings into delegated thread contracts.
3. **select-and-observe-presentations** — negotiated rendering and truthful lifecycle.
   - Scope: request contracts, host turn assembly, governed tool projection, AG-UI events and chat UI.
   - Depends on: scopes. Agent: Codex/Astra. Complexity: L. Customer value: high.
   - Add optional client rendering support and requested mode. An omitted negotiation preserves legacy behavior; explicit text disables surface tools/output. Explicit unsupported A2UI/hybrid falls back to text with a reason. Auto makes eligible templates available to the model without promising a surface.
   - Freeze validated template content together with identity/revision and eligibility at run admission. Edits, disablement and deletion affect subsequent admissions, not already-admitted snapshots; cancellation remains the mechanism for stopping an admitted run. A host-owned render tool accepts an eligible template ID plus data and returns validated declarative messages from that snapshot. The host publishes under the actual run ID; neither kernels nor tool arguments select another owner/run. Apply the negotiated output ceiling to legacy a2ui_render, host policy artifacts, direct surface-message submission and delegated publication, not just tool discovery.
   - A2UI mode requests surface-first output with a brief accessible textual summary; hybrid requests both substantive assistant text and a surface. Text fallback remains allowed on failed/no-surface generation. Support-only negotiation defaults to auto; mode-only negotiation treats rendering support as absent and falls back to text when necessary. Only absence of both fields selects legacy compatibility.
   - Project and persist selection provenance through existing run events. Only actual output publication establishes a rendered template; no event claims the client displayed it without a receipt. Show requested/effective mode and fallback in chat run details, preserving plain-text readability.

## Execution order and task boundaries

Serial vertical slices, one build writer. Each change has four tasks: host contract/storage or policy; host API/integration; typed domain/UI integration; phase acceptance evidence. Write all implementation before the final consolidated test sequence. Do not check acceptance tasks complete from compilation alone. Run kbd-status after every completed task/change/phase.

Initial task: catalog contract and persistence. First cohesive edit touches `src/uar/a2ui/presentations.rs` and `src/uar/a2ui/mod.rs`; add storage/route files only after inspecting their real implementations. The subsequent UI file list will be named after the source/design audit, before UI edits.

## Verification and review

- Tier 0 after cohesive Rust edits: `cargo check --locked --no-default-features --features server-full`. TypeScript Tier 0 follows the project rule. No dependency additions are planned.
- At phase completion: Rust formatting and server-full tests; supported persistence behavior tests; frontend type/build and targeted policy/entity/component tests; browser creation/edit/preview/assignment/negotiated-run workflows, with desktop and narrow-screen captures. Verify persisted records after reload, cross-owner denial and stale-revision conflict.
- Before UI code: memory recall (completed with unavailable-memory stub), Impeccable 4.2.0 audit and two isolated critiques, frontend-design, UI/UX Pro Max, Vercel and entity-management guidance; persist a task-specific distillation and obtain a fresh-context adversarial design review. Preserve incumbent design tokens and avoid a whole-console redesign.
- Finish UI with Impeccable polish, bounded capture/fix rounds and an independent finish review. Phase reflection compares plan with delivery and retains all unverified claims.
- Independent plan review identified ungoverned host/direct publication and ambiguous snapshot content. The plan now explicitly covers every publication ingress and immutable content snapshots, including edit/disable/delete after admission; phase tests must exercise these boundaries.

## Risks and explicit cuts

The first template editor may require editing declarative JSON with validation and safe preview; this is not a no-code builder. No arbitrary code, remote catalog loading, dependency upgrades, public marketplace, release publication or restored soak gates. Owner checks and template validation are named trust-boundary protections. Existing child archive warnings, including the deferred real-provider 429 test and coverage limits, remain unchanged. The overall 120 ledger needs separate reconciliation with the cancelled release-tail decision; do not promise that implementing this feature alone makes those cancelled rows pass.

PLAN COMPLETE
