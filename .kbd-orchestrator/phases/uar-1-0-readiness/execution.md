EXECUTION: uar-1-0-readiness
Project: universal-agent-runtime
Date: 2026-08-13
Selected backend: openspec
Dispatched to: Codex
Backend rationale: Six spec-backed changes require ordered task and evidence traceability; A0 is the only change in this execution slice.
Backend entrypoint: /opsx:apply fix-jwt-crypto-provider
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/uar-1-0-readiness/plan.md

EXECUTION SCOPE

- fix-jwt-crypto-provider: Pin jsonwebtoken 11.0.0 to RustCrypto workspace-wide and fail closed on process-provider conflict.
- gap-02-jwks-token-verifier: Next pending change; not implemented in this slice.
- gap-03-a2a-tenant-partitioning: Pending after gap-02.
- skill-builtins-on-embedded: Pending Track B start.
- skill-scoped-governance: Pending after built-ins.
- skill-config-reconciliation: Pending after scoped governance.

DISPATCH CONTRACTS

- Each change uses its OpenSpec tasks as the implementation ledger and this phase's `progress.json` plus canonical `prometheus kbd` state as the phase ledger.
- Every change commits separately. No push, PR, or archive occurs in this execution slice.

APPROVAL GATES

- The operator approved RustCrypto standardization and the expanded A0 surface on 2026-08-13.
- On 2026-08-14, the operator accepted UAR-owned first installation: any
  process provider initialized before UAR fails closed.
- Every fail-closed assertion requires an observed failing negative control.

FALLBACK CONDITIONS

- Any stop condition in `EXECUTION-CONTRACT.md` marks the active change BLOCKED and returns control to the operator.

VERIFICATION REQUIREMENTS

- A0 focused tests, workspace feature tree, Tier 0 server-full checks, proxy check, iOS and Android embedded-mobile checks, strict OpenSpec validation, and artifact-refiner validation.
- Tier 2 remains prohibited until all six changes complete.

PROGRESS LEDGER

- [COMPLETE] fix-jwt-crypto-provider — Codex
- [PENDING — NEXT] gap-02-jwks-token-verifier — Codex
- [PENDING] gap-03-a2a-tenant-partitioning — Codex
- [PENDING] skill-builtins-on-embedded — Codex
- [PENDING] skill-scoped-governance — Codex
- [PENDING] skill-config-reconciliation — Codex

OUTPUTS

- Per-change `verification.md`, OpenSpec tasks, canonical KBD transitions, and one commit per change.

BLOCKERS

- NONE

CANONICAL HANDOFF

- Revision 91, plan revision 6.
- Active phase: `uar-1-0-readiness`.
- A0 `fix-jwt-crypto-provider`: complete.
- A1 `gap-02-jwks-token-verifier`: pending and next.
- Exact next command: `/kbd-execute uar-1-0-readiness`.

REFLECTION HANDOFF

- Lead with deviations between the reviewed plan and observed delivery, including provider feature-unification and target-build evidence.

EXECUTION READY
