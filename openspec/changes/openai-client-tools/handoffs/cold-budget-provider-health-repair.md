# Cold budget refusal must not poison provider health

CT5, Execute. Verification CLOSED. This teammate performed source inspection
and authorized implementation/documentation only. No checks, tests, formatting,
compiler probes, builds, imports, network calls or runtime commands ran.

## Observed cause

Gate014 passes ordinary core/safety/cancellation/cold and warm/cold graph groups,
and all five warm budget cases. In cold budget resume, `candidate-4.log` records
three actual `accounting_budget_exhausted` admissions at lines 222, 239 and 255.
Each is followed by orchestrator stream-creation failure. Subsequent requests
report provider/model unavailable before budget admission, including ACP.

This is a production health-classification defect. The primary driver is a
BudgetedModelDriver whose refusal occurs before the real provider is contacted.
Orchestrator's primary stream-error match previously routed every Err through
`health.record_failure`, both when failover is enabled and when disabled.
Default failover health threshold is three with a 60-second cooldown
(`config.rs:2753–2757`). `ProviderHealthMonitor::record_failure` increments the
streak and enters cooldown at that threshold. Registry resolution checks this
health state before resolving the model (`registry.rs:588`), so three expected
local budget refusals make later policy selection return None.

The fixture's `openai.models=[]` is not the cause. Registry enriches an empty
model list from the known catalog; its resolver also does not reject a model
merely because no explicit list entry exists. The first three successful cold
provider resolutions followed by this exact cooldown path distinguish the
observed production defect from a missing persisted catalog model.

## Change

Only `src/llm/orchestrator.rs` changed: add an early primary Err branch using
the existing typed `cost_budget::client_failure(&e)` classifier. It projects
the same budget-owned error and exits the model attempt before provider-health
failure recording or fallback dispatch. The classifier already excludes errors
carrying a typed ProviderError, and the existing rendering path uses it.

Real provider failures retain their prior health/failover branches. Health is
not disabled, budget limits are unchanged, and no assertion is relaxed. No
fallback refactor or new classification scheme was added; the lead explicitly
scoped this repair to the observed primary path. This required handoff is the
only additional file. The guard traces directly to the observed pre-provider
budget refusals being miscounted as provider failures.

## Remaining evidence

The edited source is unverified. Compilation, all-five cold budget outcomes,
provider-contact counts, and unchanged real-provider cooldown/failover behavior
require the lead's next authorized whole-phase gate. Existing fallback-driver
error classification was outside this bounded repair. No passing earlier gate
certifies this later edit.
