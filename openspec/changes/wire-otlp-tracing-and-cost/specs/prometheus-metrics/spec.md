## ADDED Requirements

### Requirement: LLM call latency metric
The server SHALL record the wall-clock duration of each LLM driver call as a histogram labeled by provider and model.

#### Scenario: Latency recorded per call
- **WHEN** an LLM completion call to `anthropic/claude-sonnet-4-20250514` returns after streaming
- **THEN** `uar_llm_call_duration_seconds{provider="anthropic",model="claude-sonnet-4-20250514"}` histogram is updated with the elapsed time of that call

### Requirement: Per-request LLM cost metric
When cost tracking is enabled, the server SHALL record an estimated per-request cost in USD derived from the model pricing catalog and the request's token usage.

#### Scenario: Cost recorded when tracking enabled
- **WHEN** `cost_tracking` is enabled and a run completes using a model present in the pricing catalog
- **THEN** `uar_llm_cost_usd` is incremented by the computed cost (input/output, plus cache tokens when priced) for that run

#### Scenario: No cost recorded when disabled or unpriced
- **WHEN** `cost_tracking` is disabled, OR the run's model is absent from the pricing catalog
- **THEN** no cost is recorded and no error is raised

### Requirement: Cache token metrics
The server SHALL record provider cache-read (cached prompt) token usage when the provider reports it. Cache-write/creation tokens are not separately reported by the provider abstraction and are out of scope.

#### Scenario: Provider cache tokens tracked
- **WHEN** an LLM completion reports cached (read) prompt tokens
- **THEN** `uar_cache_read_tokens_total{provider=...,model=...}` is incremented by the cached-token count
