# Assessment: agui-a2ui-selection-architecture

Project: Universal Agent Runtime
Date: 2026-09-04 (America/Los_Angeles)
Stage: Assess
Baseline: runtime-comparison child complete and all ten changes archived; parent implementation ledger records two of three changes complete, but that count does not establish delivery of the named prerequisites.

## Implementation status

- **PARTIAL — Presentation catalog/workspace.** Existing `src/uar/a2ui/registry.rs` stores five built-in artifact schemas in an in-memory registry and accepts runtime registration. `src/uar/a2ui/design_systems/types.rs` defines persisted design systems, components and overrides. These are reusable infrastructure, not evidence of the recorded Presentation domain. `frontend/src/app/shell/nav-destinations.ts` exposes an A2UI testing destination only in development, separately from Skills. The reviewed source does not establish a production Presentation workspace.
- **MISSING in the inspected policy contract — scoped Presentation capability.** `src/uar/domain/policy.rs` defines global, agent, conversation and turn resolution for skills, tools, MCP servers, knowledge bases and scalar settings. Neither RunPolicy, EffectiveRunPolicy nor PolicyResolutionInput carries Presentation eligibility or client rendering capabilities. Existing non-widening resource selection is reusable, but cannot be described as a completed Presentation policy.
- **PARTIAL — text/A2UI/hybrid selection and AG-UI lifecycle.** `src/uar/runtime/native_skills/a2ui_render.rs` validates canonical messages, and `src/uar/runtime/a2ui_output.rs` publishes state patches and artifact-display events after successful rendering-tool output. `src/uar/api/adapters.rs` maps display artifacts to the AG-UI custom artifact event. This proves existing implementation structure, not a tested selection decision. No requested/selected Presentation mode or client capability field appears in `src/uar/runtime/turn/request.rs`. Selection provenance and fallback semantics remain unspecified.
- **DONE with retained limitations — runtime comparison child.** All ten changes are archived. The corrected phase suite and independent review are recorded in the child reflection and archived phase-close evidence. The deferred model-path 429 test remains unchecked.

## Cross-tool progress and ledger limits

The parent progress projection lists:
1. establish-presentation-catalog-workspace — DONE, zero recorded tasks.
2. scope-presentation-capabilities — DONE, zero recorded tasks.
3. select-and-observe-presentations — IN_PROGRESS, zero recorded tasks.

There are no parent assessment, goals or plan artifacts preceding this assessment, nor OpenSpec changes matching those three identifiers in the inspected repository. The parent stage was Execute without a written plan. Activation returned to the parent at revision 2305; revision 2306 revised the plan to 7 and revision 2307 entered Assess.

Do not rewrite historical completed rows merely from a filename search. The discrepancy is grounded additionally in the actual policy types, registry, turn request and navigation inventory. Corrective implementation tasks must be registered against the original changes once their acceptance criteria are specified.

The overall implementation projection remains 111/120. It is not a phase count or a test-coverage percentage. Inherited evidence/certification/publication fields refer to older PR 274 work and do not certify this parent feature.

## Spec alignment and required decisions

The canonical AG-UI chat conformance and A2UI React conformance specs cover normalized events, replay, safe rendering and action correlation. They do not define a Presentation catalog, assignable Presentation identity, client capability negotiation, or the meaning and precedence of text/A2UI/hybrid selection.

A relevant compatibility constraint already exists: the 2026-08-08 C-12 session receipt intentionally retired the A2UI round-trip tester from production routes and navigation while retaining live chat rendering. Exposing that tester in production is not an acceptable shortcut to a new Presentation workspace.

The uncomfortable gap is that two recorded completed changes are not sufficiently specified or evidenced to build the third on top of them. A production reusable-template catalog and a renderer/design-system catalog would create different domain records, APIs and UI; the three ledger titles do not choose between them. The next Spec step must establish that domain meaning before implementation. Recommended boundary: a dedicated production workspace for reusable Presentation definitions, with the existing A2UI tester remaining development-only. This is a recommendation, not an approved architecture decision or a claim of delivery.

Preserve existing client behavior until an explicit negotiation contract and migration path are written. Do not silently turn existing A2UI-capable clients into text-only clients, or promise a surface was delivered merely because a mode was selected.

## Build health and coverage

- Build baseline: PASS from the corrected child phase receipt: `cargo check --locked --no-default-features --features server-full`, `cargo fmt --all -- --check`, and `cargo test --locked --no-default-features --features server-full`, exit 0.
- Recorded results: library 713 passed / 1 ignored; BDD 9 scenarios / 49 steps; broad integration 94 passed / 1 ignored; doctests 26 passed / 17 ignored.
- No runtime tests or builds were rerun for this source inventory. Tests remain at phase end after the full planned phase implementation.
- Presentation selection coverage: UNKNOWN, not inferred from the broad suite. No coverage percentage was measured.
- Retained child limits: deferred 429 recovery test; three-corpus parity scope and two live scenarios; loopback governance-inactive scope; real-peer and live-billing certification not established; SurrealKV close warnings. See the archived phase-close report for exact limits.

## Constraint check

No production code, dependencies, pins, UI, workflow or deployment configuration changed during assessment. No new guards were introduced. The observed phase-order discrepancy was corrected by returning to Assess, not by starting unplanned implementation. Existing dirty worktree changes are preserved.

Before UI design or implementation, perform the project's required memory, Impeccable dual-agent critique, frontend-design, UI/UX Pro Max, Vercel and entity-management routing. Before host policy/event changes, apply agent-runtime-security and agui-event-contract. Assessment here inventories source; it is not a UI design approval.

## Goal progress

- Catalog/workspace: PARTIAL — artifact and design-system infrastructure exists; the recorded Presentation domain has no verified acceptance contract.
- Scoped capabilities: NOT MET in the inspected policy contract.
- Selection and observable lifecycle: PARTIAL — output projection exists; explicit selection, negotiation and its evidence are absent.
- Runtime child closure: MET with recorded warnings.

ASSESSMENT COMPLETE
