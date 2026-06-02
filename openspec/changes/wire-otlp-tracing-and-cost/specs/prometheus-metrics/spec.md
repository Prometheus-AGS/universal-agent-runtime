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
The server SHALL record provider cache token usage (cache-write/creation and cache-read) when the provider reports it.

#### Scenario: Anthropic cache tokens tracked
- **WHEN** an Anthropic completion reports cache-creation and cache-read input tokens
- **THEN** `uar_llm_cache_tokens_total{provider="anthropic",model=...,kind="write"}` and `kind="read"` are incremented by the respective counts

### Requirement: Sandbox execution metrics
The server SHALL record sandbox lifecycle and execution metrics when code-execution tools run in a sandbox.

#### Scenario: Sandbox execution recorded
- **WHEN** a sandboxed code-execution tool call completes
- **THEN** `uar_sandbox_executions_total` (labeled by language and exit-code class) is incremented and its duration recorded

#### Scenario: Active sandbox gauge reflects reality
- **WHEN** sandboxes are created and torn down
- **THEN** `uar_active_sandboxes` reflects the current count
