# Batch 4 handoff: issue #388, fallback destination budget (uar-runtime)

Source state: working tree on `feat/rc2-batch4-release`, nothing committed. Verification lock was CLOSED. Nothing was compiled, tested, linted or run. Every claim below is from source inspection.

## Decision that departs from the agreed plan (needs lead/operator sign-off)

The agreed plan was: build exact `RequestBudgetContract`s from the catalog at server construction and call `with_destination_preparations`. I did not do that, because source inspection shows it would break production runs:

1. `DestinationRequestPreparations::prepare` (`src/llm/orchestrator.rs`) returns `No exact request preparation for <model>` for any model not in the map, and `prepare_attempt` calls it on every attempt once preparations exist. Installing a partial map at construction makes every run on an unlisted model fail closed. Primary models, agent models and fallbacks are chosen per run (global `failover.fallback_models`, or `artifact.policy.provider.fallbacks` plus run credentials, `manager.rs` ~6600), so no construction-time list can be complete.
2. An exact contract needs more than window and output numbers: a `TemplateDestination`, `PromptTemplateProfile`, `EndpointRequestProfile`, the `SyntheticOpenAiCompatibleChatV1` wire contract and `ExactCl100kJsonV1` counting. Only tests build these. The leaf driver also refuses any contract unless the client has an explicitly bound base URL (`enforce_request_budget`: "request budget requires an explicitly bound endpoint"). A catalog-built contract on a native-endpoint provider would fail every request.
3. The in-code invariant (`DestinationRequestPreparations::budget_contract`) says catalog metadata and tokenizer lookup never manufacture an exact contract.

Instead I compile each fallback at fallback time, per run, using the existing reducer. `src/server.rs` is untouched. The exact-contract path is left as is; a fallback with an installed exact contract is skipped by the new code and still enforced by the leaf driver.

## Files changed

- `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/llm/orchestrator.rs`: new private `FallbackBudgetRefusal` error; new field `fallback_context_windows`; new `with_fallback_context_windows`; new `fit_fallback_request`; fallback loop calls it before `prepare_user_attempt`; final error code when every fallback was refused locally.
- `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/runtime/manager.rs`: after `model_context_window`, resolves each captured fallback's window (run credential, then provider-registry model, then compiled catalog, same cascade as the primary) and passes the map to the orchestrator.
- `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/runtime/turn/bindings.rs`: `RunModelBindings::fallback_models()` accessor (the fallbacks actually captured, after cooldown skips).
- `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/scripts/acceptance/fixtures/dedicated-runtime.mjs`: fixture change. New `stubFixtures` option; `config` and `env` may be (async) functions of `{ stubBaseUrl }`; `record.stub_llm.base_url` recorded. Object-valued callers are unaffected.
- `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/scripts/behaviors/support/provider-fault-proxy.mjs`: fixture change. `POST /__fault/arm?mode=unavailable` answers 503 before contacting upstream. Plain `arm` is unchanged (still `disconnect_mid_stream`), so `provider-failure` is unaffected.
- New: `scripts/behaviors/features/fallback-destination-budget.feature`, `scripts/behaviors/steps/fallback-destination-budget.steps.mjs`, `scripts/behaviors/fallback-destination-budget.cucumber.mjs` (all under `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/`).
- This file.

## How it works

- Where windows are built: `manager.rs`, per run, from `model_bindings.fallback_models()`. Fallbacks come from `FailoverConfig.fallback_models` (global) or the agent's `policy.provider.fallbacks` (run-credential/collaboration runs); both end up in `RunModelBindings`, so both are covered.
- Where each fallback is compiled: `Orchestrator::fit_fallback_request`, called in the fallback loop right before `prepare_user_attempt`, on `attempt_source`. Input allowance = window - output reserve (the request's `output_token_cap`, else `RESPONSE_RESERVE_TOKENS`) - tool-definition tokens. It then runs the existing `reduce_history_with_input_allowance` with the fallback model's tokenizer, the agent's declared strategy (`skill_activation.strategy`), no driver (no model call during failover) and no new budget code. System pinning and tool-group integrity come from that reducer.
- Typed refusal: `context_overflow` (reducer code; also used for `FixedRequestOverflow`). `protected_history_overflow` and `invalid_history` are possible reducer codes too and propagate as the final code only when every candidate was refused that way. Shape: the run's final `NormalizedEvent::Error { code: "context_overflow", message: "primary: <e>; fallbacks: <model>: context_overflow: ..." }`, the same error event and stream/HTTP surface as the existing primary overflow refusal. It is emitted before any fallback dispatch. A local refusal does not call `health.record_failure`, and the loop continues to the next fallback; if all were refused locally the code above is used, otherwise the existing error path is unchanged.
- Displacement reporting: the existing `uar.request` / `attempt_manifest` custom event (prepended to the fallback stream, appended to the run's `attempt_manifests`, surfaced as an artifact). I added a `context_fit` object to that manifest for fallback attempts: `context_window`, `input_allowance`, `reserved_output_tokens`, `tool_definition_tokens`, `required_input_tokens`, `counting: "estimated"`, `displaced`, `messages_removed`, `tokens_saved`. No new event type. It is an additive field on an existing payload.

## Behavior scenario

`fallback-destination-budget.feature`: loopback owner only, no credentials. Primary is the stub behind the fault proxy (verified, then armed `unavailable`), fallback is a configured `budgetsmall/budget-small-model` provider with a 4096 window pointing at the same stub. 60 seeded messages exceed that window.
- Scenario 1 (keep-first-and-latest): run completes with the fallback's answer; the stub's recorded fallback request keeps record 1, the latest record and the question, drops a middle record, is smaller than the seeded history; `context_fit.displaced`, `required_input_tokens <= input_allowance`.
- Scenario 2 (strategy `none`): run ends `error` with `context_overflow`; the stub records zero fallback requests.
- Evidence: `.prometheus/verification/fallback-destination-budget/<uuid>.json` (non-secret).

## Known limits

- Unknown window: a fallback with no window from credentials, registry or catalog gets no check (current behavior). Not fixed.
- Counts are tokenizer estimates (`TokenService`), not a proven final-wire count. `counting: "estimated"` is reported honestly. The exact leaf-driver guarantee only exists when exact preparations are installed, which production still does not do. Issue #388's "hard total capacity enforced before send" is therefore met as an estimated pre-send fit plus typed refusal for failover, not as an exact guarantee.
- Output reserve for the fallback reuses the primary's cap; the fallback's own `max_output_tokens` is not applied. A cap larger than the fallback window is refused (`FixedRequestOverflow`), not clamped.
- Only the failover path is covered. Primary per-step recompilation, tokenizer change and summary-failure acceptance items from #388 are not addressed here.
- Fallback reduction never summarizes (no driver); `Summarize` strategies degrade to the non-model stages and may refuse.
- Exact preparations (`with_destination_preparations`) remain unwired in production. If the operator still wants that, it needs a design for per-run model sets and for catalog-derived exact wire/counting contracts; I could not do it safely.

## Unresolved decisions

1. Accept the per-run estimated fallback fit instead of construction-time exact contracts? (lead/operator)
2. Whether `context_fit` on the attempt manifest is an acceptable report, or the operator wants a `ContextAction` event (that type exists only on the runtime-event side, not in this provider stream).
3. Whether the shared `provider-fault-proxy.mjs` and `dedicated-runtime.mjs` edits are acceptable outside my owned paths.

## Unverified (everything)

- Nothing compiled: possible type errors in `fit_fallback_request` (e.g. `ReduceReport.context_action` field use), the `stream!` edits in the fallback loop, and the manager loop. Possible warnings not checked.
- Not confirmed: whether the stub receives the fallback model bare or provider-qualified (both fixtures supplied); where `attempt_manifests` / `context_fit` appears in `GET /api/uar/runs/{id}` or events (the step searches both recursively); that run-policy `context_strategy: {type: "none"}` is accepted; that `POST /api/uar/providers` with the body used is valid; retry/backoff time on the 503 primary within the step's 200 s poll; that global `failover` config with a stub `base_url` and no key builds a driver (key defaults to empty).
- No cucumber run, no existing test re-run. Existing fallback tests should be unaffected (empty window map means unchanged behavior) but this is unconfirmed.
