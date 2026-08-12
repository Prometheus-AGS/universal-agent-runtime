# Goals — uar-1-0-readiness

Opened 2026-08-11. Assess stage authored in Claude Code; execute/reflect handed
to Codex per `.kbd-orchestrator/HARNESS-HANDOFF.md`.

## The goal condition

Close the three UAR-local gaps that `docs/SPECIFICATION.md` records as blocking
adoption, and widen `TokenVerifier` to the PAGS-SPEC-PID-001 FR-5.1 shape so the
C-25/26/27 exclusions stop depending on a crate UAR does not consume.

Done means: **each gap has a test that fails on `main` today and passes after the
change** — not "code exists", not "review says so". The prior phase's evidence
ladder applies unchanged (L0 asserted → L4 round-tripped).

## In scope

| Item | Gap | Grounded at |
|---|---|---|
| Real JWKS/RS256 verifier behind a `TokenVerifier` trait | GAP-02 | `src/uar/security/middleware.rs:45-46` |
| Tenant-partition the A2A task store | GAP-03 | `src/uar/api/a2a/task_store.rs:17-21` |
| Register builtin skills on the embedded path | GAP-05 | `src/server.rs:448`, `src/server.rs:511` |
| Widen `TokenVerifier` to the PID FR-5.1 `Presented` shape | — | new trait; no existing verifier abstraction |

## Explicitly out of scope

- **Any `frf-*` dependency.** PID §2.2 supersedes `frf-wallet` issuance. Wiring
  UAR to a layer already scheduled for replacement would create work that PID P4
  then deletes. This is the correction PID forced on the original scope.
- **PID P0 decisions (D-1/D-2/D-3).** PID §8 sequences UAR's GAP-02 behind its own
  P4, which sits behind three blocking decisions — one of them an external business
  negotiation. **PID §6.1 preserves the RS256/JWKS lane unchanged**, so UAR closes
  GAP-02 on its own timeline and the two designs coexist.
- **CI/CD gating.** Per the standing operator decision in `.prometheus/decisions.md`:
  GitHub Actions validate deployments at deployment time; they do not run
  development tests. Verification here is local and command-shaped.
- **Docs and GitHub Pages.** A parallel phase, deliberately independent of this one.
- **P2P peer transport (C-23/C-24).** Needs two devices; unchanged from the
  conformance phase's published exclusion.

## Honesty constraints (carried from uar-spec-conformance-2026-08)

1. No aggregate percentage, no runtime-level verdict.
2. Absence of failure is never success — every test asserts something positive.
3. Exclusions are emitted by the run, not written afterward.
4. **Every change is grounded in a file and line before its exit criteria are
   written.** This is the corrective action from the prior phase's reflection,
   where three spec errors all took the shape of assuming infrastructure that was
   present-but-absent or absent-but-present.
