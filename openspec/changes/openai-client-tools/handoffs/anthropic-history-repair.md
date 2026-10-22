# Anthropic text-history repair

Phase: `uar-post-lifecycle-verification-readiness`, Execute. Verification remains
CLOSED for this writer; the lead owns any explicitly authorized execution.

## Observed failure and source cause

`target/client-tools-local-gate-003/matrix.json` reports Anthropic initial history
requests rejected with `invalid_client_tool_calls`. In
`convert_anthropic_messages_to_openai`, text-only assistant history acquired
`"tool_calls": null`. Shared `validate_visible_history` treats any present
`tool_calls` field as a declared exchange and requires a nonempty array, so the
adapter manufactured an invalid tool declaration before provider execution.

## Files changed

- `src/server.rs`: the assistant-history conversion now inserts `tool_calls`
  only when the converted message actually contains tool-use blocks. Text,
  ordering, nonempty tool-call arrays, argument serialization, user/result
  conversion and all existing ID/ownership validation remain unchanged.
- This handoff records the repair and its verification limits.

No shared service, authentication, persistence, configuration, fixture or other
adapter was modified. No new guards or requested-scope extensions were added.

## Unverified and remaining work

Only source inspection and authorized implementation/documentation writes ran.
No tests, lint, formatting, compiler probes, builds or runtime commands ran.
Compilation and end-to-end Anthropic continuation/replay remain unverified.
The lead must execute the authorized production gate against this source;
this source repair does not establish acceptance or authorize publication.

The existing converter still normalizes native IDs through `ensure_toolu_id`;
this repair does not change that behavior or shared strict caller-ID checks.
No unresolved design decision is required for the observed null-field defect.
