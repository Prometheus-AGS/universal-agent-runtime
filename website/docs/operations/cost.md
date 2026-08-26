---
sidebar_position: 5
title: Interpret Cost and Budgets
description: Understand UAR cost estimates, budget events, and dashboard limits.
source_records:
  - docs/product-surface-inventory.md
  - openspec/specs/runtime-event-replay-entity-sync/spec.md
current_authority: /docs/operations/cost
---

# Interpret Cost and Budgets

UAR can estimate model cost from observed token usage and catalog pricing, aggregate that estimate against runtime budgets, and project the result in the packaged Cost page.

:::warning Boundary statement
Every amount is an estimated cost. Provider billing is the authority for charged usage, credits, batch discounts, cache treatment, taxes, and pricing changes.
:::

## Enable estimates

Cost tracking is opt-in through `llm.cost_tracking`. When enabled and a completed run reports input or output usage for a priced `provider/model`, UAR calculates a USD estimate from the local model catalog. It emits that value on the terminal run event and records a provider/model Prometheus histogram.

No estimate is produced when tracking is disabled, usage is absent, the model route is unrecognized, or catalog pricing is missing. Zero visible spend can therefore mean “not measured,” not “free.”

## Budget scopes

`CostBudgetTracker` defines Run, Task, Session, Agent, and Global scopes. The current orchestrator records completed-run estimates into Run, Session, Agent, and Global aggregates. Task is intentionally not recorded because this runtime has no separate task entity at that boundary.

The global ceiling comes from `llm.budget.global_limit`. Agent artifacts can contribute a per-session maximum that is applied to the agent scope. A configured warning threshold produces a budget alert before the hard limit; reaching the limit marks it exceeded.

Budget aggregation is process-local. Restarting the process resets the tracker's accumulated values and limits reconstructed only in memory. Budget events inform operators and clients; they should not be treated as a provider-side spending cap.

## Session-scoped dashboard

Open `/admin/cost`. The session-scoped dashboard totals priced run entities currently in the shared browser graph, charts spend over time, groups it by model, and lists budget warnings and exceeded events. It explicitly does not query a persisted all-time run-history endpoint.

The backend can write cost ledger entries through configured persistence on a fire-and-forget path. The current Cost page does not hydrate from that ledger, and a failed write logs a warning without blocking the run. Do not reconcile long-term spend from the page.

## State ownership and durability

The provider owns actual billing. The pricing catalog owns the rates UAR uses for calculation. The run manager owns process-local aggregation. Configured persistence may own individual ledger entries. The browser entity graph owns the displayed session projection. These stores can legitimately differ because they answer different questions.

## Operational use

- Alert on missing usage or pricing separately from low estimated spend.
- Label charts by provider/model and source revision so catalog changes remain interpretable.
- Treat an exceeded event as a runtime signal, then enforce provider-side limits independently.
- Compare estimates with the provider statement before making accounting or release claims.

## Profile limits

The estimate and budget logic applies to `server-full` and relevant `minimal` server execution. The branded Cost page and populated telemetry are `server-full` capabilities. `embedded-mobile` depends on the host and provider adapter for usage, display, and persistence. No amount on this page is an invoice or a durable all-time total.

See [Prompt Caching](/docs/providers/prompt-caching) for provider cache-write
and cache-read pricing boundaries, [Observability](/docs/operations/observability)
for metric ownership, and [Runs](/docs/operations/runs) for terminal usage
events.
