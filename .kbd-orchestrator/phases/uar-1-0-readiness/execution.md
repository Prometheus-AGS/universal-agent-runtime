EXECUTION: uar-1-0-readiness
Project: universal-agent-runtime
Date: 2026-08-13
Selected backend: openspec
Dispatched to: Codex
Backend rationale: Six spec-backed changes require ordered task and evidence traceability; A2 is the active delivery slice after authenticated principal verification.
Backend entrypoint: /opsx:apply gap-03-a2a-tenant-partitioning
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/uar-1-0-readiness/plan.md

EXECUTION SCOPE

- fix-jwt-crypto-provider: Complete, with a focused follow-up preserving UAR first ownership because `jsonwebtoken` cannot expose an earlier provider's identity.
- gap-02-jwks-token-verifier: Complete — JWKS verification, caching, claim validation, and effective `jwt_required` enforcement passed its gates.
- gap-03-a2a-tenant-partitioning: Implementation and focused controls complete;
  canonical completion remains deferred until the live C-21 Tier 2 assertion
  and control run at phase completion.
- skill-builtins-on-embedded: Pending next implementation slice.
- skill-scoped-governance: Pending after built-ins.
- skill-config-reconciliation: Pending after scoped governance.

DISPATCH CONTRACTS

- Each change uses its OpenSpec tasks as the implementation ledger and this phase's `progress.json` plus canonical `prometheus kbd` state as the phase ledger.
- Every change commits separately. No push, PR, or archive occurs in this execution slice.

APPROVAL GATES

- The operator approved RustCrypto standardization and the expanded A0 surface on 2026-08-13.
- On 2026-08-14, observed dual-provider execution proved pointer comparison
  cannot identify the installed provider. UAR retains first ownership; any
  earlier provider fails closed.
- Every fail-closed assertion requires an observed failing negative control.
- A cohesive implementation unit is completed before broad Tier 0/Tier 1 runs;
  focused failures are debugged without restarting broad groups.

FALLBACK CONDITIONS

- Any stop condition in `EXECUTION-CONTRACT.md` marks the active change BLOCKED and returns control to the operator.

VERIFICATION REQUIREMENTS

- A0's provider-ownership follow-up runs as the first exact test in the warmed A1
  verification sequence and commits separately.
- A1 runs one consolidated Tier 0 sequence, its focused security tests, the
  `uar-sidecar` tests, strict OpenSpec validation, negative-control restoration,
  artifact-refiner validation, and history-free critic/judge review. All passed.
- A2 begins only after A1 is committed separately.
- A2 focused tenant tests, fail-closed controls, Tier 0, integration-target
  compile, strict OpenSpec validation, and independent critic/judge review pass.
  Its live C-21 row remains explicitly unobserved until phase Tier 2.
- Tier 2 remains prohibited until all six changes complete.

PROGRESS LEDGER

- [COMPLETE] fix-jwt-crypto-provider — Codex
- [COMPLETE] gap-02-jwks-token-verifier — Codex
- [IN PROGRESS — IMPLEMENTED, LIVE GATE DEFERRED] gap-03-a2a-tenant-partitioning — Codex
- [PENDING — NEXT IMPLEMENTATION] skill-builtins-on-embedded — Codex
- [PENDING] skill-scoped-governance — Codex
- [PENDING] skill-config-reconciliation — Codex

OUTPUTS

- Per-change `verification.md`, OpenSpec tasks, canonical KBD transitions, and one commit per change.

BLOCKERS

- NONE

CANONICAL HANDOFF

- Canonical KBD revision 94, plan revision 6.
- Active phase: `uar-1-0-readiness`.
- A0 `fix-jwt-crypto-provider`: complete.
- A1 `gap-02-jwks-token-verifier`: complete.
- A2 `gap-03-a2a-tenant-partitioning`: in progress; implementation complete,
  live C-21 evidence deferred to phase Tier 2.
- B3 `skill-builtins-on-embedded`: next implementation slice.
- Exact next command: `/kbd-execute uar-1-0-readiness`.

REFLECTION HANDOFF

- Lead with deviations between the reviewed plan and observed delivery, including provider feature-unification and target-build evidence.

EXECUTION READY
