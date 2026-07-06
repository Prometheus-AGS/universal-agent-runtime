-- Durable cost-budget spend history (CH-07).
--
-- `CostBudgetTracker` (src/uar/runtime/cost_budget.rs) is intentionally
-- in-memory-only for the hot path; this is the durable roll-up layer its own
-- doc comment anticipated subscribing to. Append-only -- one row per spend
-- event, not an aggregate (aggregation still happens in-memory at request
-- time via `CostBudgetTracker`).
CREATE TABLE IF NOT EXISTS cost_ledger (
    id          BIGSERIAL PRIMARY KEY,
    scope       TEXT NOT NULL,
    scope_id    TEXT NOT NULL,
    cost_usd    DOUBLE PRECISION NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_cost_ledger_scope ON cost_ledger (scope, scope_id);
CREATE INDEX IF NOT EXISTS idx_cost_ledger_recorded_at ON cost_ledger (recorded_at);
