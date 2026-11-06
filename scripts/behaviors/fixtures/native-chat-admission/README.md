# Mounted native chat admission fixture — source only

Gate CLOSED. No fixture, peer, service, browser, API call, import, or check has
been executed during authoring. This is an acceptance input, not a pass receipt.

After the complete #462–475 implementation inventory and exact operator gate
approval, the external fixture owner provisions the frozen Linux amd64 artifact
behind its existing local identity gateway. Test code uses only UAR_BASE_URL,
UAR_WORKSPACE_ID and UAR_NATIVE_CHAT_FIXTURE (an absolute manifest path). It does
not obtain, carry, renew, or inspect credentials.

External provisioning owns the following prerequisites:

- A dedicated namespace matching `native-chat-[A-Za-z0-9_-]{1,64}`. The manifest
  records exact source commit, image and configuration identities. The selected
  provider must point at this peer with stream/tools/vision enabled, retries and
  failover disabled. The journal assertion checks the actual wire model.
- Launch the authored peer with UAR_NATIVE_CHAT_NAMESPACE,
  UAR_NATIVE_CHAT_PEER_PORT, UAR_NATIVE_CHAT_PEER_JOURNAL (absolute fresh path),
  and UAR_NATIVE_CHAT_EFFECT_TOOL_NAME (the exact advertised governed write name).
  The peer creates its provider journal exclusively and records every provider
  arrival before choosing any scenario response. No prompt or header is stored;
  only synthetic case markers, request digests and image-part counts are retained.
- Use the existing `scripts/behaviors/support/record-mcp-peer.mjs` under external
  ownership with dedicated RECORD_FILE and RECORD_LOG. Its actual `record_write`
  commits are the effect oracle; model text and UAR counters never substitute for
  this journal. Provision the MCP connection and a tools-capable agent that can
  execute precisely that governed tool. This fixture never uses caller-owned
  function continuations to stand in for host execution.
- Provision three existing agents: ordinary (no tools, memory/retrieval disabled),
  effect (governed write eligible), and approval (same write requires a current
  operator decision). Preserve normal Cedar/owner/budget/approval admission. The
  fixture submits the originating challenge ID through the current decision API.
- A genuine externally interrupted original turn is needed for the restart
  scenario. Before restarting the same frozen runtime against the same durable
  store, the external owner sends the `hold` marker with recorded stable turn,
  message, session and run IDs, captures positive partial output and provider
  journal correlation, then completes the actual runtime restart. Put those
  original IDs and expected partial in `interrupted`. Keep the external before /
  after restart receipts with the whole-phase evidence. The manifest's
  `restartCompleted` flag is a prerequisite declaration, not independent proof of
  a restart. This Cucumber scenario proves read-only interrupted reconciliation
  and zero new journal activity; it does not certify the external restart itself.

The config is `scripts/behaviors/native-chat-admission.cucumber.mjs`. It uses the
existing Cucumber framework, serial scenarios, no scenario retries, and fresh
per-scenario sessions / logical IDs. Nothing in its imports starts a peer or UAR.
The manifest template is deliberately non-runnable until external provisioning
fills its identities and paths. A failed scenario poisons further cases until
external reconciliation. It preserves disposable durable sessions and external
journals for diagnosis; it does not erase the evidence during cleanup.

Coverage is the mounted `/api/chat/completion` path, same original IDs across
concurrent independent HTTP transports and a later request, changed original
message/policy/attachment conflict, discard-and-read reconciliation, rich typed
canonical history, partial provider EOF and explicit cancel, historical approval
decision projection, read-only interrupted recovery, native-ID refusal for
caller-owned tool batches, and unchanged legacy no-ID streaming. Normal governed
write traffic is exactly two provider loop calls (`tool_request`, `after_tool`)
and one actual recorded write; ordinary chat is one provider arrival and zero
writes. Unknown/unattributed/foreign provider arrivals fail the dedicated oracle.

The API discard case consumes then intentionally discards the admission body; it
does not claim to cause network loss. The existing authored browser history and
A2UI cases separately admit through the actual server then destroy delivery via
browser routing. Their C35/C54 original assertions remain in place. The frontend
fixture source additionally covers storage refusal/no POST, durable rich drafts,
read-only unknown receipt recovery, abrupt partial reload, and historical card
rendering. UI critique, migration/build, all tests and real browser execution
remain part of the single later approved combined validation.

An additional mutable-catalog case creates its own fresh agent definition by
reading the dedicated ordinary template, cloning the descriptor under a
namespace/case-specific ID, and requiring a successful create plus persisted
read. It completes one original turn, deletes only that confirmed-created ID,
observes the persisted read return 404, then repeats the byte-equivalent original
request. That request must still return the original JSON receipt/run without
resolving the now-absent agent. Before/after complete provider and effect journals
must remain identical. The normal scenario deletes its own agent as the actual
product trigger; it never deletes or edits any manifest agent. On a scenario
failure, a partially created disposable agent is retained for external diagnosis
rather than guessing at cleanup ownership. No API was called during authoring.
