# Cold budget/provider-health source review

Task 13.26, Execute. Bounded artifact review of the primary stream Err guard in `src/llm/orchestrator.rs`, `cold-budget-provider-health-repair.md`, actual typed classifier, BudgetedModelDriver admission order and existing retry/health/failover branches. Own report only.

## Disposition

No blocking source finding in this scoped primary-path correction.

The new branch precedes both generic Err branches, projects the existing budget failure and breaks the attempt loop. It records neither provider health success nor failure and never enters fallback dispatch for that refusal. The existing typed classifier requires BudgetFailureOrigin and explicitly excludes an error containing typed ProviderError; it does not infer budget ownership from error text. BudgetedModelDriver's admission/tracker/durable call refusal occurs before `inner.stream`, matching the named boundary.

The existing retry predicate requires typed retryable ProviderError, so the classified budget-owned refusal does not enter the provider retry path. Real provider errors are excluded from this guard and retain their existing retry, health failure and enabled/disabled failover handling. The success branch still records actual successful stream creation. Error projection retains the existing closed client budget code/message; no successful completion, reset budget, ignored refusal or alternative provider attempt was added.

## Evidence and limits

Only source reads (`cat`, `sed`, `rg`, scoped `git diff`) and this report write ran. No tests, checks, builds, imports, formatting, compiler probes, runtime or network commands ran. No production file was edited. Global reviewer verification stays CLOSED; the lead owns its scoped local functional loop.

Compilation, native cold-budget outcomes, real provider-contact counts and actual cooldown/failover behavior remain unverified on this edited source. Existing fallback-driver error classification and post-stream usage failure handling are outside this bounded change. Same-family isolated source review is not cross-model adversarial evidence; no acceptance or publication claim is made.
