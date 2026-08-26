---
sidebar_position: 3
title: Observe the Runtime
description: Use probes, Prometheus metrics, structured logs, and optional OTLP traces.
source_records:
  - openspec/specs/prometheus-metrics/spec.md
current_authority: /docs/operations/observability
---

# Observe the Runtime

UAR exposes process health, application metrics, structured logs, and optional distributed traces. Each signal answers a different question and has a different owner.

:::warning Boundary statement
Operational signals show what the instrumented process observed. They are not an immutable audit ledger, a provider invoice, or proof that uninstrumented work succeeded.
:::

## Liveness and readiness

- `GET /health` and `GET /healthz` are lightweight liveness probes. HTTP 200 with `status: ok` means the process can serve that handler; no dependency is checked.
- `GET /readyz` checks configured persistence access, initialized memory service state, and the MCP registry. It returns 503 when a required persistence check fails.
- All three probe routes bypass JWT authentication.

Readiness reflects the coded checks, not every external model, tool, queue, or downstream service. Provider status has its own signal.

## Prometheus metrics

In `server-full`, `GET /metrics` returns Prometheus text exposition. The runtime records:

- HTTP request totals and duration by method, path, and status;
- LLM call duration, estimated cost, and input/output token counts by provider/model;
- provider health and prompt-cache read/write tokens;
- tool results, active sessions, sandbox activity and errors, and MCP server status;
- guardrail and sycophancy observations, evaluation scores/regressions, and skill activation/outcomes.

Provider/model signals describe external-call latency, health, usage, and estimates. UAR-owned signals describe its HTTP handling, routing, tools, sessions, sandboxes, policies, and other instrumented runtime work. Keep these classes separate when assigning an incident or SLO.

The metrics recorder is process-global. If another component installs that global recorder first, UAR logs a warning and `/metrics` returns an empty scrape rather than panicking. HTTP 200 alone therefore does not prove metric series are present.

## Structured logs

`server.log_format` or `UAR_SERVER__LOG_FORMAT` selects `json`, `compact`, or `pretty`; JSON is the default. `RUST_LOG` controls filtering, with a UAR debug-oriented default when the variable is absent. Secret-bearing configuration types redact their values in debug output, but operators must still control log access and review any application payload logging they add.

Logs remain attached to the process output unless the deployment collects them. Rotation, retention, indexing, and access control belong to that collector.

## OTLP trace export

OTLP is opt-in. UAR creates an HTTP/protobuf exporter only when `OTEL_EXPORTER_OTLP_ENDPOINT` is set and `UAR_LLM__TRACING` is not false, zero, off, or no. Exporter construction failure prints a diagnostic and continues without trace export. On enabled shutdown, the process retains the provider so it can flush buffered spans.

An absent collector or failed export does not stop ordinary runtime work. Monitor the collector and the runtime's enablement log separately.

## State ownership and durability

Prometheus owns scraped samples after collection. The log platform owns collected records. The OTLP backend owns exported spans. UAR itself exposes or emits these signals but does not provide their durable retention. In-process metric counters reset on restart, and an empty scrape can result from recorder ownership conflict.

## Audit limits

Metrics aggregate labels and values; logs and traces may omit paths that lack instrumentation. None is signed, append-only evidence. For security or release claims, retain the exact command, response, source revision, profile, limit, and any negative control required by the claim.

## Profile limits

The `telemetry` feature—and therefore populated Prometheus export—is included in `server-full`. In `minimal`, `/metrics` returns 404 because telemetry is not compiled, though liveness, readiness, and structured logging remain server concerns. `embedded-mobile` delegates collection and export to the host. No signal here transfers a readiness claim between profiles.

See [prompt-caching configuration and metric interpretation](/docs/providers/prompt-caching),
[cost interpretation](/docs/operations/cost), [shutdown outcomes](/docs/operations/recovery-and-shutdown),
and the [Runtime Console](/docs/operations/runtime-console).
