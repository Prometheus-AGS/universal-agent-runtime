# CH-08 skill-activation-metrics

## Why

`record_skill_activation`/`record_skill_activation_outcome`
(`src/uar/telemetry/metrics.rs:234-251`) already existed but had zero
callers — the per-skill/per-backend precision-recall counters the fable
called for were dead code.

## What changed

- `SkillService::match_skills` (`src/uar/runtime/skills/service.rs`) now
  calls `record_skill_activation(skill_id, backend, accepted=true)` for
  every skill in the final matched set, labeled by which matching backend
  actually produced the result (`keyword`, `embedding`, `local_embedding`,
  `llm`, `hybrid`).

## Scope notes (deliberate cut)

- This wires the **activation-recall** half of the precision/recall pair:
  which skills get selected, how often, by which backend. It does **not**
  wire `record_skill_activation_outcome` (whether an activated skill's
  tools were actually *used* by the model afterward) — that requires
  correlating a match-time decision against the run's later tool-call
  stream, and `match_skills` has no visibility into candidates that were
  *considered but rejected* either (the underlying algorithms filter
  internally). Both are harder, separate problems left as a follow-up,
  consistent with this phase's other documented scope cuts
  (plan.md D-A..D-D).
- Console/dashboard surfacing beyond the raw Prometheus counters (already
  visible via the existing `/metrics` endpoint) was not built this pass —
  no existing skill-stats API/UI surface exists to extend.
