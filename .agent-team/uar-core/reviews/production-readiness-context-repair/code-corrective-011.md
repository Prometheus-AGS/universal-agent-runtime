# Gate A: corrective E6/E7 and bounded rc.2 integration review

Reviewed `target/production-readiness-476/repair-011-review/artifact.patch` and its `requirements.txt`, plus necessary baseline source, repository/tool rules, and the rc.2 manifest/path dependency. This is the authorized first corrective/integration source pass. It does not reopen the old whole-phase review. Product source remained read-only; this report is the only file written.

Isolation: no producer rationale, handoffs, memories, progress inventories, or other reviewers' reports were read. This reviewer retains its own earlier finding context, so this is a corrective assessment rather than a wholly fresh review. Native same-family/non-specialist assessment is weaker than independent-family specialist review.

## Corrective dispositions

- **E6: corrected at source.** `src/uar/runtime/cost_budget.rs:2032` uses an explicit result match rather than `expect_err` on the stream result. It imposes no new stream `Debug` requirement. The unexpected-success panic, admission-origin assertion, root-cause assertion, and zero inner-stream count remain at lines 2033–2049. No compiler or test was run.
- **E7: corrected at source.** `src/llm/liter_driver.rs:539` bypasses final validation only when both the budget contract and endpoint profile are absent. Profiled requests now compare the actual prepared JSON body to the captured profile and typed controls before the no-budget return at line 578. Contract-dependent output arithmetic/counting stays conditional. The positive fixture at line 1265 preserves the literal alias, tools, named tool choice, parallelism and response schema; the negative fixture at line 1232 exercises Liter's actual alias rewrite. Altered control/schema/unapproved-field assertions remain in the positive fixture. These are authored assertions, not observed passing results.

## New bounded integration finding

### [MEDIUM] Fallback fitting runs before destination output narrowing

File: `src/llm/orchestrator.rs:2827` (fitting call); related new arithmetic at `src/llm/orchestrator.rs:1178`.

The new fallback fit takes `attempt_source.output_token_cap` as its reserved output and refuses immediately when that cap plus tool definitions reaches the fallback context window. The fallback's live output restriction is applied later, at `src/llm/orchestrator.rs:2859`, only after fitting has succeeded.

Concrete failure scenario: a healthy, captured fallback has an 8,192-token context window and a registered 2,048-token maximum output. The primary request carries a 16,384-token output cap and a short ordinary text prompt, with no exact budget contract. After a permitted primary failure, the new fitter reserves 16,384 tokens and raises `FixedRequestOverflow`, so the fallback is skipped before admission. The existing destination admission would accept that plain-text destination and narrow the cap to 2,048, leaving enough input capacity. `AdmittedDestination::restrict_output` explicitly takes the minimum of the existing cap and the destination maximum (`src/llm/router.rs:95`), and `evaluate_destination` returns the registered `max_output_tokens` (`src/llm/router.rs:492`). Those guards cannot fix the request because the new refusal occurs before they run. Smaller cap differences can also displace optional history unnecessarily and leave the manifest's reserved output inconsistent with the eventual request cap.

Correction: obtain and apply the destination-specific downward output restriction before computing the estimated fallback allowance, or use the same captured destination restriction to compute the effective reserve. Preserve fresh admission of the final prepared request immediately before stream opening; do not infer permission, capability, or exact fit from the estimate. The fit report must reflect the cap used by the dispatched attempt.

Classification: definite source-level ordering defect with a concrete configuration/input trigger. Provider behavior and the scenario above were not executed.

## Other bounded source observations

- The rc.2 manifest identifies `1.4.0-rc.2` and retains the vendored Liter path dependency. The inherited-input merge changes an ownership comment and introduces a local binding without changing the registration operation.
- The fallback path still invokes final preparation, immutable-scope/live-capability admission, team handoff validation and the captured driver. The new fit manifests label counting `estimated`; attempt manifests continue to set `fit_guarantee: false`. Exact destination preparations bypass this estimated reducer and remain governed by their existing leaf contract.
- Finite accounting's unproven-image refusal remains in `admit_prepared_user_attempt`; the diff introduces no credential lookup or destination authority expansion.
- Both merged fixture literals explicitly set `local_loopback_owner: false`. The production flag remains serde-defaulted off and constrained by its existing local/credential checks. The shared Rust fixture still obtains a verified user through the production verifier; context-history persistence continues to use the derived session owner. Historical macOS receipts were not treated as current validation evidence.

## Execution limits

Commands performed were source reads/searches (`cat`, `sed`, `rg`, `nl`) with every shell call rooted at `/Users/gqadonis/.claude/worktrees/production-readiness-476`, followed by the authorized report write. **Zero executed checks.** No Cargo, tests, lint, format/type checks, compiler/import probes, builds, API/container calls, machine screening, or `git diff --check` were run or delegated. No product edits or new guards were added. The finding concerns the actual destination-output restriction boundary; no speculative hardening is requested.

Compilation, test outcomes, actual prepared sends, fallback reduction and admission interaction, durability, cancellation, and packaging remain unverified. This report supplies no runtime certification and grants no authority for another gate.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 1 | info |
| LOW | 0 | pass |

Verdict: APPROVE under the review severity criteria, with one concrete MEDIUM integration defect recorded above. E6/E7 are corrected at source; acceptance remains unverified.
