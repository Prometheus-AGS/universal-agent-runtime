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

**2. Nothing runs the matrix in CI.**

```
$ grep -rln 'capability_cases\|live::capability' .github/workflows/
(no matches)
```

`live-integration.yml:75` reads *"No continue-on-error. This is the gate."* — but
that is a **compile** gate. Two later steps in the same file are still
`continue-on-error: true`, and no job executes the matrix. The tier compiles in
CI and never runs.

That is the 25-day harness failure one level up. That failure was "the tier does
not compile and CI shows green." This one is "the tier compiles and CI never runs
it." Both produce a green checkmark carrying no information about whether the
runtime works.

## What Changes

- Correct the two miscalibrated assertions so each asserts the contract the
  runtime actually implements, with the contract named in the assertion message.
- Add a dedicated GitHub Actions job that executes the capability matrix with
  `continue-on-error: false`.
- Prove the gate fails: break one named case, observe that job go red **for that
  case**, revert.

## Impact

- Affected specs: `spec-conformance-measurement` (new capability)
- Affected code: `tests/integration/live/capability_cases.rs`,
  `.github/workflows/` (new job or new file)
- Risk: low. No runtime code changes. The CI job costs ~195s per run plus build.
- Blocks: `conformance-close-capability-gaps` — do not extend a miscalibrated
  instrument.
