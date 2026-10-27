# TEAM-01 token-budget repair handoff

## Baseline and scope

The complete current source of these intended edit targets was read from the
shared checkout before the first edit:

- `src/uar/runtime/cost_budget.rs`
- `src/uar/runtime/team_execution/execution.rs`
- `src/uar/runtime/manager.rs`

No pre-edit source snapshot was persisted at that time. That was a handoff
defect: the earlier statement recorded a read, not a saved snapshot. The
baseline is reconstructed from the archived
`history/before-token-reservation-repair/source-manifest.json` identities and
the exact incremental patch in `handoffs/token-budget-repair.patch`; it must
not be represented as a saved pre-edit source copy. The existing running UAR
process was not touched.

Archived baseline identities and reconstructed current-source identities:

| Path | Archived SHA-256 | Current SHA-256 |
| --- | --- | --- |
| `src/uar/runtime/cost_budget.rs` | `45248cab0de43b8b449f9e6a0401a1a693534921329dcfc5c7025526541bbc39` | `6a184b89e9135aaad3f7b790a43f2fadffad292cfd096b3e4e969b0e8da2ee83` |
| `src/uar/runtime/manager.rs` | `1c1604260db32cbe42eab56f2c1767837a6c682a86c0e79a1314ff5cc2c09d6a` | `c6379dd5ec970e349cbad27e0b0c5e1b966b35d0d0be2bd5e013a3512c5addb6` |
| `src/uar/runtime/team_execution/execution.rs` | `9d6e6cff98a151a3b3f5bb0b7dab8f9608424d11579895645c4a151f734a7549` | `35cd5956f785558c7ca4fb905db58f5f89831465e1b5d2d6034ec317a894ffc2` |

The current SHA-256 values are metadata source reconstruction only, not
product verification.

## Implemented behavior

`TEAM_MAX_OUTPUT_TOKENS` is `64_000`. The shared allocation helper assigns
`output = min(64_000, total / 2)` and `input = total - output`, rejecting only
zero-sided allocations. A trusted team attempt passes the output side through
the existing `RunExecutionRequest.output_token_cap`. For the operator-selected
TEAM-01 allocation of 128,000 tokens, this is a 64,000-token maximum response
and a separate 64,000-token host input allowance. Smaller valid team grants
remain supported.

For the trusted `team-budget:<attempt>` remote grant only and only when no
exact request budget contract exists, durable model-call admission reserves the
host input allowance plus the actual enforced request output cap. It calculates
reservation cost from those same finite values. The exact-contract branch still
wins before the team allocation is inspected. The reservation source is
`team_host_input_allowance_and_output_cap`; it is explicitly not a measured
provider input count.

The manager uses the same host input allowance for its existing estimated
reduction path and request-context limit, clamped to the model's estimated
capacity minus the actual output cap. It does not turn a settings-only profile
into a final-wire fit contract. The Liter typed output-cap path was already
present: it sends `max_completion_tokens` for the OpenAI-compatible team
profile and continues to label uncontracted final-wire counts as approximate.

## Changed source

- `src/uar/runtime/cost_budget.rs`
- `src/uar/runtime/team_execution/execution.rs`
- `src/uar/runtime/manager.rs`

`src/llm/liter_driver.rs`, `src/llm/mod.rs`, and `src/llm/team_profile.rs`
were inspected and intentionally left unchanged.

## Invariants retained

- Exact request-budget contracts still use `request_budget_bound`; the team
  allowance branch only applies where no exact contract is present.
- The settings-only `TeamFitDisposition` remains unqualified and no local
  approximate tokenizer result is claimed as exact.
- The remote grant, per-turn/session limits, atomic durable admission, actual
  usage settlement, and unknown-usage reservation behavior remain in place.
- A zero-sided or over-cap team allocation is rejected before provider dispatch
  rather than falling back to the catalog's context window.

## Verification status

The repository verification gate is closed. No test, Cargo command, formatter,
lint, build, review, service/process action, or provider call was run.

Lead integration note: complete reconstructed pre-repair files are now retained under handoffs/baseline-reconstructed/. Their SHA256 identities match every corresponding file in the previously frozen source manifest; see baseline-reconstruction.json. token-budget-repair.patch is now a proper difflib-generated unified patch with concrete hunk ranges. This is historical source reconstruction, not a product test or a claim that the writer saved pre-edit snapshots.
