# TEAM-01 runtime result repair handoff

## Result

No Rust production source changes are required for this repair.

The retained Mac fixture configured `liter/MiniMax-M3` with
`max_output_tokens: 512` in
`target/prompt-macos-hbitxj2c/config.json`. That setting is the observed
later output ceiling that overrode the requested 64,000-token response cap.

## Source trace

1. `src/uar/runtime/team_execution/execution.rs` derives the team output cap
   from the 128,000-token attempt reservation through
   `team_token_allocation`, yielding the requested 64,000 cap.
2. `src/uar/runtime/manager.rs` resolves the exact `liter/MiniMax-M3`
   configured model and reduces the supplied cap to
   `configured_model.max_output_tokens` when present.
3. The retained model configuration sets that field to 512, so the effective
   cap was 512, not 64,000.
4. `src/llm/orchestrator.rs` forwards the resolved cap to the driver.
5. `src/llm/liter_driver.rs` writes it to `max_completion_tokens` for the
   admitted OpenAI-compatible route.

The stored writer output contains only provider content framed as `<think>`
and no final outline. The retained records do not include the provider
`finish_reason` or final serialized request, so they cannot prove the exact
provider termination mechanism. They do prove the host-side 512-token clamp.
The observed 6,582 total-usage figure is not an output-cap receipt and cannot
override this trace.

## Related result handling

The math result was also rejected because
`src/uar/runtime/team_execution/execution.rs` accepts a contract string only
when its entire contents parse as JSON. The retained math content has reasoning
blocks, one fenced JSON object, and trailing prose. The lead owns the bounded
BDD parser repair for that existing scenario; this runtime handoff deliberately
does not broaden its text-contract behavior.

`src/llm/liter_driver.rs` maps a populated upstream `reasoning_content` field
to `ReasoningDelta`, and `src/uar/runtime/manager.rs` does not append such
events to terminal member content. Therefore the retained `<think>` strings
were received in provider `content`, not introduced by that host-side reasoning
collection path.

## Baseline and patch

Complete pre-edit copies are in `handoffs/result-repair-baseline/` for:

- `src/uar/runtime/manager.rs`
- `src/uar/runtime/team_execution/execution.rs`
- `src/llm/liter_driver.rs`
- `src/llm/orchestrator.rs`
- `src/llm/team_profile.rs`

`handoffs/result-runtime-repair.patch` records the intentional empty Rust
production diff. The lead owns the fixture correction and BDD parsing patch.

## Unverified

The verification gate is closed. No build, test, formatter, linter, typecheck,
review, service action, model call, or retry was run. A future approved run must
capture the final request cap and provider finish reason if it needs to prove
the upstream reasoning-only termination behavior beyond the confirmed fixture
clamp.
