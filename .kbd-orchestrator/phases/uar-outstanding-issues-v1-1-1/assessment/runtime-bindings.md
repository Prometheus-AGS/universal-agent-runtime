# Runtime model-binding assessment — issue #385

Date: 2026-10-15. Work: assessment for `uar-outstanding-issues-v1-1-1`, before implementation planning or verification. Scope: model binding, provider verification, runtime errors, upgrade behavior, and preservation of #384.

This assessment contains source inspection and deployment observations relayed by the phase lead. It is not a reproduction or acceptance receipt. The lead created and activated this assessment phase through canonical KBD commands (activation revision 2923; assessment entry revision 2924). The lead owns canonical artifacts and projections.

## Assessment

The strongest current explanation is that the upgraded runtime now requires durable accounting for an explicitly token-bounded agent, while the reported remote Surreal deployment has not supplied the trusted durability attestation. That combination rejects preparation before provider construction, and the manager replaces the cause with `Run model binding failed`.

The uncomfortable limit is that the persisted live agent was not read and the underlying error was erased. The repository agent definition and deployment evidence strongly support this explanation but do not independently establish the exact configuration used by the failing run. A runtime fix must not silently disable accounting or assert durability to make the symptom disappear.

## Evidence and provenance

| Evidence | Observation | Limit |
| --- | --- | --- |
| Operator report supplied to investigation | Every chat run fails on main `76f2ad30` and v1.1.0 `d815ef72`; baseline `3e6af7c4` works. JWKS JWT authentication, per-agent `openai` / `qwen3.8-max`, explicit endpoint, remote Surreal. | No successful reproduction performed here. |
| Read-only deployed logs relayed by phase lead | know-me v1.1.0 emits the model-binding failure after reporting the per-agent `openai/qwen3.8-max` assignment. | Generic error hides the failing inner operation. |
| Repository configuration inspected by phase lead | `charcoal-agent/uar/agents/knowme-site.json` has `extensions.budgets.max_tokens_per_turn = 16000`. | Persisted live artifact may differ from the repository file. |
| Deployment, ConfigMap and Secret key-name inspection relayed by phase lead | No `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED` key was found; startup reports dispatcher disabled. | Key-name inspection is not a dump of the process environment or proof of durable storage infrastructure. No secret values are retained here. |
| Source history inspected locally | `22088cee` added durable accounting preparation between baseline and main. Model-verification and target-admission source did not change across those revisions. | A commit delta narrows causality; it does not prove a deployed branch was taken. |

## Source chain

1. `src/uar/runtime/thread/policy_intersection.rs:127–155`: `ThreadBudgets` derives `Default`; absent/null budgets produce all-`None` limits. An explicit `max_tokens_per_turn` remains a real limit. `src/uar/compiler/ir.rs:684` likewise declares optional budget fields without a default financial ceiling.
2. `src/uar/runtime/cost_budget.rs:1423`, `ModelCallBudget::has_configured_limits`: a token ceiling activates required accounting independently of `cost_tracking`. It also recognizes session token/cost ceilings, remote grants, and finite tracker cost limits. Thus the reported 16000-token ceiling matters even though `cost_tracking` defaults false (`src/config.rs:2137`).
3. `src/uar/runtime/manager.rs:5317`: model-budget preparation calls `with_durable_accounting` if cost tracking is enabled, configured limits exist, or the backend reports durable accounting and the owner has a verified principal. `src/uar/runtime/manager.rs:1719`, `with_global_cost_budget`, leaves the global tracker unlimited unless explicitly configured.
4. `src/uar/persistence/providers/surreal.rs:153–159,623`: remote Surreal is eligible for durable accounting only when `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED` equals `1`. Local `surrealkv://` independently qualifies. A remote endpoint alone is deliberately insufficient evidence of durability.
5. `src/uar/runtime/cost_budget.rs:1432`, `with_durable_accounting`: an unsupported backend returns `durable_accounting_unsupported` before scope initialization or client construction. A missing verified principal is a separate failure, `accounting_verified_principal_required`.
6. `src/uar/runtime/manager.rs:5329–5354`: the accounting preparation result is combined with team profiles; an error skips `RunModelBindings::capture` entirely.
7. `src/uar/runtime/manager.rs:5357–5403`: the failure handler replaces all ordinary causes with `Run model binding failed`, logs that replacement, and emits `orchestrator_start_failed` / `Failed to create the run orchestrator`. Consequently the observed message cannot distinguish accounting eligibility from provider construction.
8. `src/llm/orchestrator.rs:60`, `build_driver_captured`, independently erases constructor errors to `Provider client construction failed`. Raw error-chain logging is not an acceptable repair: constructor-owned credentials may be outside the captured secret corpus.

## Upgrade and migration behavior

`22088cee` adds the accounting requirement; `RunModelBindings::capture` in `src/uar/runtime/turn/bindings.rs` itself is unchanged between the reported working baseline and main. The same accounting preparation and generic failure handler are present in release `d815ef72`.

`src/uar/persistence/providers/surreal_host.rs:22,91` runs accounting schema initialization during connection. `migrations/surrealdb/durable_accounting.surql` creates accounting tables with `IF NOT EXISTS`; schema presence and the runtime durability eligibility flag are distinct conditions. A migration cannot manufacture trusted durability evidence.

When accounting becomes eligible, `src/uar/persistence/providers/accounting/surreal.rs:24`, `ensure`, checks for an existing scope and otherwise probes legacy `cost_ledger` rows. `AccountingScope::new` (`src/uar/persistence/accounting.rs:108`) marks a newly initialized scope unreconciled if legacy history exists. `reserve` (`accounting.rs:192`) refuses unreconciled scopes during subsequent model-call admission. This later refusal is distinct from the present capture failure.

The legacy probe currently decodes `SELECT * FROM cost_ledger` directly to `Vec<serde_json::Value>` solely to determine non-emptiness. Established `load_cost_history` (`providers/surreal.rs:2927`) instead reads native Surreal values and applies `surreal_to_json`. This is a secondary source inconsistency to qualify against an existing ledger after the main eligibility problem is resolved; it is not established as the deployed root cause. A scalar/typed existence projection or established native-value conversion is a candidate correction only if the reproduction or diagnostic evidence confirms this seam.

Existing reconciliation APIs are in `src/uar/api/accounting.rs`: scope inspection and `PUT /scopes/{scope}/{scope_id}`, plus call reconciliation. Scope changes require trusted-host authority, expected generation, paired opening cost/token balances when supplied, and evidence. Preserve old history and require explicit reconciliation; do not zero balances, delete records, infer opening balances, or create an automatic migration that grants economic authority.

## Model verification and provider configuration

`src/uar/api/model_verification.rs` and `src/uar/api/targets.rs` have no changes from `3e6af7c4` to `76f2ad30`. Manager capture does not consult verification receipts. `ModelSources::verified_model` (`targets.rs:78`) gates explicit target admission earlier and returns an explicit disabled/unverified/configuration-changed message when no current proof exists.

`POST /api/uar/providers/{id}/test` (`src/uar/api/providers.rs:397`) invalidates the old proof, performs actual inference, and persists a new configuration fingerprint only after success. Provider configuration or protected credential revision changes invalidate the proof. Successful provider verification therefore would not prove that the bounded chat run can initialize accounting. Reverification must not be prescribed as the diagnosed repair for the present error.

`ProviderRegistry::resolve_to_llm_config` (`src/llm/registry.rs:578`) preserves explicit endpoint behavior: with a configured URL the driver receives the unqualified model ID; `resolved_provider_id` retains provider identity. Do not change `openai/qwen3.8-max` routing merely because the model is served by a compatible endpoint. The inspected Liter submodule constructor delta delegates through an equivalent `new_inner(..., None)` path; its material behavior changes concern stream completion after dispatch, later than the reported failure seam.

## Candidate work for the phase plan

### Required diagnostic correction in UAR

- Classify failures at their source seams with typed, bounded metadata: invalid budget configuration, accounting unsupported, verified principal missing, accounting storage operation, team profile, inherited binding, budget admission, and client construction.
- Preserve existing typed `AccountingError` (`persistence/accounting.rs:61`) and `BudgetFailureOrigin` (`runtime/cost_budget.rs:24`) categories; classify constructor/egress error variants without formatting their arbitrary payloads.
- Emit an actionable allowlisted public code/message and a diagnostic reference. Operator logs may include stage, reference, and booleans such as accounting-required, backend-supported, and verified-principal-present. Do not log raw credentials, endpoint strings, provider responses, full configuration, or opaque error chains.
- Keep actual accounting preparation in the trusted host and preserve fail-closed behavior for bounded execution. Preserve team error contracts and error terminalization.

### Configuration resolution, subject to actual infrastructure evidence

If the live artifact retains the 16000-token ceiling and remote storage meets the supervisor's durability contract, deployment configuration must explicitly attest that fact. Document the operator prerequisites and resulting eligibility. Merely adding the flag without checking the underlying persistence guarantee would weaken the trust boundary. Removing budgets or enabling implicit untracked execution would change intended behavior and is not the proposed repair.

If the live effective configuration differs, use the new typed error category to choose the narrowly applicable correction. Do not add speculative retries, fallback credentials, replacement models, or broader storage migration changes.

### Preserve #384

Keep `InheritedLlmSettings`, `set_llm_defaults`, and the inherited timeout/retry fields in `src/llm/registry.rs:231,266,635`. The repair must retain operator timeout and retry settings through per-agent resolution and downstream driver configuration. Do not revert the inherited behavior to `LlmConfig::default()` while adjusting diagnostics or accounting eligibility.

## Unresolved inputs and acceptance gaps

1. Read the live persisted agent's budget and effective model assignment through an authorized read-only surface; retain only the necessary sanitized fields. The repository JSON is not live-state proof.
2. Establish the actual failing error category and stage. No original inner error has been captured.
3. Confirm effective durability attestation and infrastructure guarantees before changing deployment configuration. Startup dispatcher-disabled evidence supports the reported absence but does not certify storage durability.
4. Inventory whether existing accounting scopes or legacy cost rows require reconciliation; no production ledger was read or changed by this investigation.
5. After the entire active phase's production implementation is complete and the lead records its explicit inventory, qualify the real authenticated chat path with the reported per-agent provider, explicit endpoint, and token ceiling. Include an unattested remote configuration that returns the actionable refusal and an actually attested durable configuration that completes a run, preserving budget enforcement and #384 timeout/retry inheritance.
6. At that same authorized boundary, qualify upgrade behavior with retained legacy history, model-verification distinction, and diagnostic secret exclusion. Provider-test success alone is insufficient evidence for ordinary run success.
7. UAR v1.1.1 build, production integration, installed delivery, and release evidence remain pending. This assessment claims neither a repaired deployment nor a passing acceptance gate.

## Handoff

Changed only this assessment document. Source findings came from `rg`, `sed`, `cat`, and read-only Git history/diff inspection. No production code, canonical phase file, deployment, credential, database record, or model-verification receipt was changed. No verification command was run. The lead owns the complete issue inventory, plan, source ownership, integration order, and full-phase verification boundary.
