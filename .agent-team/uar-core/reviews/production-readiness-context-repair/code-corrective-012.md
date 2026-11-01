# Gate012 A: bounded E8 corrective source review

Verdict: **APPROVE — no concrete blocking source defect found in the selected corrective scope.** This is a source assessment only, with zero runtime certification. It does not itself authorize another review or gate.

## Scope and isolation

Reviewed the fixed `target/production-readiness-476/repair-012-review/artifact.patch` and `requirements.txt`, necessary production/fixture source, the rc.2 manifest and vendored dependency selection, and applicable repository/tool rules. Focus remained E8 and preservation of E6/E7; the original whole-phase finding hunt was not reopened.

No producer rationale, handoffs, memories, inventories, or other reviewers' reports were read. This reviewer retains its own previous corrective findings; therefore this is a corrective assessment, not a wholly fresh isolated assessment. Native same-family/non-specialist review is weaker than independent-family specialist review.

## E8 disposition

**Corrected at source.** `src/llm/orchestrator.rs:1176` clones the source request and applies destination admission/output narrowing before reading the effective output cap at line 1184, calculating fixed reserves, or reducing history. `AdmittedDestination::restrict_output` in `src/llm/router.rs:95` preserves a smaller caller cap by taking the minimum; its absent-cap behavior supplies the destination maximum. With no admission restriction, the existing response-reserve fallback remains in force.

The concrete previous failure — a 16,384-token primary cap causing immediate refusal for an 8,192-token fallback whose maximum output is 2,048 — no longer follows from this ordering. The fitter uses the downward-restricted request on both its already-fitting and reduced-history returns. It does not silently restore the larger source cap.

Preservation observations from source:

- Exact destination preparations bypass estimated fitting at `src/llm/orchestrator.rs:1169`. Their own contract, canonical history, settings and schema are subsequently installed by `DestinationRequestPreparations::prepare`; a primary contract is not admitted against the fallback first on this branch.
- Unknown windows return the source unchanged without an estimated fit report at line 1166. Ordinary behavior still reaches final preparation and admission. This branch was inspected, not executed.
- Fresh admission remains after fallback preparation at line 2864 and before stream opening at line 2871. The earlier narrowing does not replace immutable scope/live capability checks or the finite-accounting unproven-image refusal.
- Early fitter/admission refusals are handled in the local-refusal branch and do not record a fallback provider health failure. Final destination-admission failures retain their separate health-neutral branch.
- Fit metadata still labels counting `estimated`; the attempt manifest continues to disclaim guaranteed fit. Under the stable registration exercised by the authored cases, the reserve and input allowance use the same narrowed cap carried to the captured stream.

## Six authored orchestration cases

The table at `src/llm/orchestrator.rs:5256` includes six cases:

| Case | Source cap | Expected fallback cap | Source-level assertion purpose |
|------|------------|-----------------------|--------------------------------|
| Large primary cap | 16,384 | 2,048 | Avoid false fixed-request overflow |
| Longer optional history | 4,096 | 2,048 | Avoid unnecessary history displacement |
| Lower caller cap | 1,024 | 1,024 | Preserve caller restriction |
| Absent caller cap | absent | 2,048 | Preserve destination-default compatibility |
| No live admission | 4,096 | 4,096 | Preserve existing uncontracted behavior |
| Exact preparation | 16,384 | 2,048 | Install destination contract/settings and bypass estimates |

The test enters production orchestration with a real provider registry/admission object and synthetic captured streams. It asserts one primary attempt, one fallback dispatch, the effective cap, complete optional and protected history, cache strategy, contract identity, exact settings/schema, and appropriate manifest fields. Its longer-history precondition distinguishes the old 4,096 reserve from the corrected 2,048 reserve. The exact case asserts no estimated `context_fit` report. This is authored regression coverage only: none of these assertions was executed, and the synthetic drivers establish no installed/provider qualification.

## Retained E6/E7 dispositions

- **E6 remains corrected:** `src/uar/runtime/cost_budget.rs:2032` extracts the error with a match, avoiding stream `Debug`. Admission origin, root cause and zero-stream assertions remain.
- **E7 remains corrected:** `src/llm/liter_driver.rs:539` skips final validation only for requests with neither contract nor profile. Actual profiled body/control comparison occurs before the no-contract return. Existing positive and negative real-preparation assertions remain authored.

The artifact also retains explicit `local_loopback_owner: false` fixture literals; this corrective diff does not change production identity admission. The manifest remains rc.2 with the vendored Liter dependency. Historical receipts were not used as current evidence.

## Execution and limitations

Only source reads/searches (`cat`, `sed`, `rg`, `nl`) were performed, with every shell call explicitly rooted at `/Users/gqadonis/.claude/worktrees/production-readiness-476`. Some exploratory source reads named nonexistent files and returned no content; subsequent reads used the discovered paths. These were inspection misses, not verification commands or acceptance evidence.

**Executed checks: zero.** No Cargo, tests, check/lint/fmt/typecheck, compiler/import probes, builds, API/container calls, machine screening, `git diff --check`, or delegation occurred. Product source and other checkout/services were untouched. The only write is this assigned report; no unsolicited implementation or new guards were added.

Compilation, six-case outcomes, real provider behavior, registry changes during an attempt, durable accounting/cancellation, and packaging remain unverified. Source inspection supports the corrective disposition above; it does not certify runtime acceptance.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: APPROVE for the bounded source review. No concrete selected-behavior defect remains to trigger the operator's blocking rule. Runtime acceptance is unverified.
