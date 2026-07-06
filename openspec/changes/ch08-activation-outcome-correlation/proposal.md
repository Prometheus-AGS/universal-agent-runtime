# ch08-activation-outcome-correlation

## Why

`src/uar/runtime/skills/service.rs:472` already documented the gap
precisely: `record_skill_activation` (the "recall" half — a skill was
matched and selected) is called per matched skill, but
`record_skill_activation_outcome` (the "outcome" half — did the model
actually use that skill's tools) was defined in
`src/uar/telemetry/metrics.rs:245` and never called anywhere. Without
it, operators can see activation counts but not whether activated
skills' tools were actually exercised — the precision half of the
recall/precision pair the comment already names as intentionally
scope-cut.

## What changed

- **Capture skill→server ownership before merge.** Each matched
  skill's `mcp_config` (when present) introduces one or more MCP
  server names via `McpRegistry::from_config`. `RunManager::start_run`
  now records `skill_servers: HashMap<skill_id, Vec<server_name>>`
  from each skill's own registry (via its `server_names()`) *before*
  merging into the per-run `final_mcp` registry — the merged registry
  no longer distinguishes which skill contributed which server, so
  this has to happen at the point of construction.
- **Resolve invoked tools back to their server** at run end via the
  already-existing `McpRegistry::resolve_mcp_tool(namespaced_name) ->
  Option<(server_name, raw_tool_name)>` — the same reverse-lookup the
  registry already uses to dispatch real tool calls, not a
  reimplementation of its namespacing/sanitization logic.
- **New pure function** `correlate_skill_activation_outcomes(
  skill_servers, invoked_tool_servers) -> Vec<(skill_id, bool)>` —
  a skill is "used" if any of its introduced servers appears in the
  set of servers the run's actually-invoked tools resolved to.
  Deliberately takes plain data (no `McpRegistry` dependency) so it's
  directly unit-testable without a live registry.
- **Wired at run end**, once per run, in the single place all
  terminal branches (cancelled / has-usage / no-usage) pass through —
  *not* nested inside the cost-tracking-enabled branch, since
  activation-outcome tracking has nothing to do with whether cost
  tracking happens to be on.
- **Explicit exclusion, not a proxy signal**: skills with no
  `mcp_config` (prompt-overlay-only skills) never appear in
  `skill_servers` at all, so `correlate_skill_activation_outcomes`
  never emits an entry for them — no `record_skill_activation_outcome`
  call happens, rather than recording a guessed `false`. This is a
  known, disclosed limitation: such skills have no distinguishable
  "used" signal at the tool-call layer. A future pass could look at
  their prompt-overlay content actually appearing to influence the
  model's response, but that's a materially harder and different
  signal, out of scope here.

## Capabilities

Closes the outcome half of CH-08's activation-recall/precision pair.
No new HTTP surface, no schema change — purely an additional
`uar_skill_activation_outcome_total{skill_id,success}` metric now
actually incrementing, alongside the existing
`uar_skill_activation_total` counter.

## Verification

- 4 new unit tests (`activation_outcome_tests` in `manager.rs`) cover:
  a skill whose only server was invoked (`used=true`); a skill whose
  server was never invoked (`used=false`); a skill with multiple
  servers where only one was invoked (`used=true` — any-match, not
  all-match); and the empty-`skill_servers` case (prompt-overlay-only
  skills) correctly yielding zero outcome entries, not a `false` one.
- `cargo test --lib`: full suite green (confirmed at the Round 2
  shared checkpoint alongside `ch06-wire-agent-cost-budget`).
- `cargo check --lib`: clean (same 2 pre-existing unrelated warnings).
