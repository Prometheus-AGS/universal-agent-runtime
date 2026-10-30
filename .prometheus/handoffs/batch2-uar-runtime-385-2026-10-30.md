# Batch 2 handoff: uar-runtime, issue #385 (2026-10-30)

Source edits only. No cargo, test, lint, format or diff-check command was run (verification lock CLOSED).

## Files changed

### 1. src/uar/runtime/manager.rs
- `run_binding_failure` doc comment (~line 959-963): no longer says trusted logs never get constructor details. States that only the catch-all `client_capture_failed` cause is logged, and only after the run secret scrubber has projected it.
- Binding-failure handler (~line 6845-6858, the `tracing::error!("Run model capture refused")` site): when `category == "client_capture_failed"`, the ERROR log gains `cause = %cause`, where `cause = emitter.secret_scrubber.scrub(&format!("{error:#}"))` (alternate format, so the anyhow context chain is included). All other categories log exactly as before (separate `else` branch, same fields). The run-stream `NormalizedEvent::Error` message and code are unchanged and generic.
- `error` is only borrowed (`run_binding_failure(&error)`, `error.to_string()` earlier), so no move issue. `emitter.secret_scrubber` is not mutably borrowed at this point (the `&mut` use ended with the `model_bindings_result` await).

### 2. src/uar/runtime/cost_budget.rs (~lines 2212-2232, `begin_call`, non-team `else` branch)
- Previously the reservation output was the catalog `max_output` alone. Now, when `request.output_token_cap` is `Some(cap)` with `cap > 0` and the catalog has a positive `max_output`, output reservation = `min(cap, catalog max_output)`, and the reservation_source label is `catalog_context_and_request_output_cap`. Otherwise behavior is unchanged (`catalog_context_and_output`).
- Input side is untouched: still the full catalog `context_window` (the existing counting-uncertainty reserve). Cost estimate is derived from the same (input, output) pair, so it shrinks accordingly.
- Team branch (`team-budget:` remote) and the `budget_contract` (final-wire) branch are untouched.

## max_tokens call path traced (no plumbing needed)
`server.rs` (5039/5079/5419/6920: `req.max_tokens` / `max_completion_tokens` -> run request `output_token_cap`) -> `manager.rs` ~4270 (merged with `client_tools.output_token_cap`, min) -> `manager.rs` ~7189-7240 `bounded_output` (caller cap, clamped to contract limits and configured/catalog model max output) -> resolved turn `output_token_cap()` -> `llm/orchestrator.rs` ~2333 `base_req.output_token_cap` (and `DestinationBudgetDriver` in manager.rs ~66-92 `min`s it for graph nodes) -> budgeted driver `stream()` (`cost_budget.rs` ~1704-1728) -> `begin_call(&self.model, &request)`. `request` already carries `output_token_cap` at the call site; only the sizing code was changed.

## Unresolved decisions / limits
- Whether an agent-config `max_tokens_per_turn` (a budget limit, not a request cap) is ever copied into `output_token_cap` was not traced. If the failing agent sets only `max_tokens_per_turn` and the caller sends no `max_tokens`, `output_token_cap` is `None` on the request in the legacy path and this change will not reduce the reservation. Manager.rs `bounded_output` (~7204) falls back to `RESPONSE_RESERVE_TOKENS` when no cap is supplied, and sets `output_token_cap = Some(cap)` on the turn, so on that path the cap IS present; confirm in BDD with the real agent from #385.
- The reservation uses catalog `max_output` only; a per-model `configured_model.max_output_tokens` override is not consulted in `begin_call` (not available there). The cap already includes that clamp upstream (`bounded_output`).
- Existing tests may assert the `catalog_context_and_output` label or exact reserved tokens for capped requests; tests were not inspected or edited. Grep found no other reference to the label in src/ or tests/ (only an openspec design note).
- Zero-warning: no new imports or bindings left unused by inspection; unverified by compiler.

## Unverified
Everything: nothing was compiled, tested, linted or formatted. Types assumed: `request.output_token_cap: Option<u32>`, `limits.context_window/max_output: u64`, matching the team branch tuple `(Option<u64>, Option<u64>, &str)`. rustfmt line wrapping of the new lines is unchecked (the `request_cap` line is ~95 columns).
