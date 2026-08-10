# conformance-baseline-gate

Phase: `uar-spec-conformance-2026-08` (change C-01b + C-02)

## Why

The capability matrix in `tests/integration/live/capability_cases.rs` measures
the runtime against `docs/SPECIFICATION.md`. Two things are wrong with it today,
both measured on 2026-08-09.

**1. Two assertions are miscalibrated.** The baseline run reported 2 failures out
of 20. Neither is a runtime defect:

| Case | Asserts | Runtime actually returns |
|---|---|---|
| `l3_c04_credentials_listing` (`:356`) | `status == 200` | `401 {"error":"Authentication required"}` |
| `shape_only_c13_sessions` (`:511`) | `status == 200` | `404 {"code":"legacy_route_disabled"}` |

C-04's credentials endpoint guards unconditionally — correct for a credentials
surface — and the test never sends a token. C-13's route was deliberately
retired and the runtime says so precisely, naming its replacement
(`Reuse X-UAR-Session-ID with POST /api/chat/completion`).

This is the second consecutive run where the instrument's error rate exceeded
the runtime's: the previous run reported 5 failures of which 3 were real. An
instrument that manufactures false defects cannot be extended — every capability
added on top inherits the miscalibration.

**2. Completion needs local execution evidence.**

Repository policy reserves GitHub Actions for deployment and deployment
validation. Unit and integration testing belongs on the developer machine, so
this change cannot use a workflow as its gate. The exact matrix command is
instead a mandatory local completion condition for every change in this set.

A command recorded only when green is still weak evidence: it does not prove
the selected test module ran or that a named failure produces a non-zero exit.
The local gate therefore needs a deliberate red run followed by a green run
after the one-line break is reverted.

## What Changes

- Correct the two miscalibrated assertions so each asserts the contract the
  runtime actually implements, with the contract named in the assertion message.
- Make the pinned capability-matrix command a mandatory local completion gate
  for all three changes.
- Prove the local gate detects failures: break one named case, observe a
  non-zero result that names the case, revert, and observe green.

## Impact

- Affected specs: `spec-conformance-measurement` (new capability)
- Affected code: `tests/integration/live/capability_cases.rs`; conformance
  planning and verification records
- Risk: low. No runtime code changes. Each local matrix run costs ~195s after
  build on the measured machine.
- Blocks: `conformance-close-capability-gaps` — do not extend a miscalibrated
  instrument.
