# CT5 observed budget error projection repair

Phase Execute. This writer used source inspection and authorized implementation
and documentation only. No tests, imports, checks, builds, formatting or network
requests ran. Unrelated verification remains CLOSED; the lead owns gate execution.

## Observed defect

Gate013 budget.json records passing OpenAI, AG-UI and A2A cumulative-budget cases,
but failing Anthropic/ACP assertions that the resumed refusal identify the budget
boundary. Their adapters replaced the captured normalized error with generic
Messages/Caller-tool run did not settle successfully: Error. The actual runtime
already refused before the second provider contact. This repair changes public
error projection, not admission, reservation, settled usage or budget limits.

## Exact owned changes

- `src/server.rs`, bound Anthropic Messages projection: retain normalized Error
  code together with its already projected message. After actual host settlement,
  Error status uses that captured failure before the generic status-only message.
  Other non-success statuses still use the actual status; settlement failure and
  lost observation retain their existing failure paths. A captured error with
  Done status still follows the existing explicit failure path. No successful
  stop marker or completion is introduced.
- `src/uar/api/acp/handler.rs`, caller-tool collected projection: after actual
  settlement returns Error, find the existing collected normalized Error's
  code/message and return that text through the existing RPC_INTERNAL_ERROR
  response. Missing Error evidence retains the generic actual status; cancellation
  and settlement exceptions retain their existing failure behavior. No pending
  handoff or success response is synthesized on this path.
- This handoff records the observed cause, repair and verification limits.

Both adapters consume public normalized events. The existing host event
projection scrubs Error.message before publication; no private error chain,
credential, provider payload or ledger content is newly read. RPC numeric error
semantics and Anthropic error envelope/status remain unchanged. The public text
can now retain budget_admission_failed: accounting_budget_exhausted rather than
erasing its cause with the settled Error status alone.

## Limits and remaining work

No new guard, fallback success, policy bypass or budget weakening was added.
All other writers' source edits were preserved. Compilation and native JSON/SSE
behavior remain unverified here. The lead must obtain the required source review
and run the concrete authorized build/production gate against these Rust edits;
earlier gate013 results do not certify this new source.
