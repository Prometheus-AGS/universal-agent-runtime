## ADDED Requirements

### Requirement: OTLP trace export is opt-in and gated

The server SHALL export traces via OTLP only when tracing is enabled (`llm.tracing == true`) AND an OTLP endpoint is configured (`OTEL_EXPORTER_OTLP_ENDPOINT`). When either is absent, the server SHALL run with local `fmt`/`json` logging only and SHALL NOT require an OTLP collector.

#### Scenario: Export enabled
- **WHEN** `llm.tracing` is true and `OTEL_EXPORTER_OTLP_ENDPOINT` is set
- **THEN** an OTLP tracer is installed as a `tracing` layer and spans are exported to the configured endpoint

#### Scenario: Export disabled by default
- **WHEN** no OTLP endpoint is configured (or `llm.tracing` is false)
- **THEN** the server starts normally with no OTLP exporter and no error, using local logging only

#### Scenario: Clean shutdown
- **WHEN** the server shuts down with the OTLP exporter active
- **THEN** the tracer provider is flushed/shut down so buffered spans are not lost

### Requirement: Run path emits a hierarchical span tree

A run SHALL emit a hierarchical trace: a run-level span with child spans for each LLM driver call and each tool dispatch. Spans SHALL carry attributes identifying at least the run id, and (where applicable) provider, model, and tool name.

#### Scenario: Span hierarchy for a tool-using run
- **WHEN** a run performs one LLM call that triggers one tool call followed by a second LLM call, with tracing enabled
- **THEN** the exported trace contains a run span with child LLM-call spans and a tool-call span, each annotated with the relevant model/provider/tool/run-id attributes

### Requirement: Per-request cost is computed from the pricing catalog

When `cost_tracking` is enabled, the server SHALL compute an estimated USD cost for a completed run from the model pricing catalog and the run's accumulated token usage, and SHALL include it as `cost_usd_estimate` on the run-completion event. When disabled or when the model is absent from the catalog, `cost_usd_estimate` SHALL be `null` and no error SHALL occur.

#### Scenario: Cost on completion event
- **WHEN** `cost_tracking` is enabled and a run completes using a catalog-priced model
- **THEN** the `RunDoneWithUsage` event carries a non-null `cost_usd_estimate` computed from input/output (and priced cache) tokens

#### Scenario: Cost omitted gracefully
- **WHEN** `cost_tracking` is disabled OR the model has no catalog pricing
- **THEN** `cost_usd_estimate` is `null` and the run completes normally

### Requirement: Telemetry excludes sensitive data

Spans and metrics SHALL NOT include secrets, API keys, full prompts/messages, or user PII. Only non-sensitive operational fields (model id, provider, token counts, durations, cost, tool name, run id) may be attached.

#### Scenario: No secrets in spans
- **WHEN** a span is emitted for an LLM call that used an API key and a user prompt
- **THEN** the span attributes contain the model/provider/token/duration fields but neither the API key nor the prompt text
