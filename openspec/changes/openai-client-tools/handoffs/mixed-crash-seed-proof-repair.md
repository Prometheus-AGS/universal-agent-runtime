# 13.27 mixed crash seed evidence repair

Execute phase; source inspection and authorized driver/peer authoring only.
No execution, imports, tests, checks, builds, formatting or network calls ran.
Production source and supervisor were not changed. Shared checkout edits retained.

## Observed cause

Gate015 mixed-crash-seed.json fails before seed readiness with actual provider
history lacks the ordered real host call/result exchange. Its retained trace
contains one mixed provider request and one settled MCP effect. The peer's one
response emits the host and caller calls in the same batch. UAR correctly settles
the host work and hands off caller work without another provider contact. The
pre-response provider request cannot contain that newly generated host exchange.
The seed assertion was invalid; this is not evidence of a production history bug.

## Exact owned changes

- `tests/fixtures/client_tools_provider.py`: the existing request trace entry now
  retains its generated response tool_calls, content_type and exact wire_body
  bytes decoded as text. The same calls feed the unchanged wire builder, whose
  resulting body is the one passed to reply. A real_provider_response disk receipt
  retains the same actual request/response association. This adds no model call.
  Each actual MCP effect receipt also retains mcp_result, the exact result object
  used in the JSON-RPC response. Its text payload is serialized before metadata
  is attached to the receipt, so actual MCP response content stays unchanged and
  contains no recursive receipt. Existing trace reset/lifecycle behavior remains.
- `scripts/release/client_tools_acceptance.py`: mixed seed requires exactly one
  actual marked provider request/response, exactly one emitted call named by the
  configured real host tool, and exactly one newly settled effect. It retains the
  emitted call's actual ID/name/arguments from that response receipt, requires its
  decoded arguments equal the new real effect's arguments, and stores that exact
  MCP result plus provider wire/effect receipts. No driver-generated host ID or
  synthetic assistant/result history is created.
- Cold resume still requires one actual provider contact, the selected marker,
  and ordered assistant host call/result before every submitted caller result.
  It compares the exact saved emitted host call and the JSON-decoded canonical
  host-result content against the exact saved MCP JSON-RPC result object. This
  handles the actual string-encoded result shape without guessing formatting or
  projecting a new result. Full host-effect receipt equality and completed retry
  zero-provider/zero-host-effect assertions remain unchanged.
- This handoff records the causal repair and its verification limits.

## Provenance and limits

The peer records only its controlled synthetic wire values. Generated response
calls are the same values consumed by OpenAI/Anthropic response builders; exact
wire bytes are retained alongside them, and a successful real native handoff
still must occur before seed readiness. The new MCP result receipt is the same
object returned by the existing real tools/call route. registry.rs serializes
the SDK CallToolResult to JSON and the orchestrator serializes that value into
provider history, supporting exact semantic JSON comparison on cold resume.

The intentional crash/join/restart remains supervisor-owned. A seed cannot claim
the post-crash canonical history exists before a real resumed provider request
is observed. The repaired gate therefore keeps seed evidence and cold history
proof separate while binding them to the same actual host call/effect/result.
It does not add a production endpoint, extra provider round trip, guessed ID,
new test cross-product or weaker cold-order assertion.

All new Python and peer trace behavior remain unexecuted and unverified. The lead
must run the concrete authorized gate after isolated source review. Prior gate015
results do not establish this authored proof passes.

Source-only compatibility inspection: existing host-effect assertions use length,
emptiness or equality against a previously captured peer receipt list. No
consumer requires an exact two-key effect-receipt shape. The new mcp_result field
is retained unchanged in both sides of cold/graph equality checks. The supervisor
checks aggregate mixed_host_effects/pending_batches, not effect object keys.
No acceptance assertion was relaxed or count expectation changed. No additional
Cargo build is required by these Python-only source edits; build/gate scheduling
remains the lead's decision.

## 13.28 Gate016 correction: host argument JSON formatting

Gate016 confirmed all five mixed crash seed handoffs, then cold OpenAI comparison
failed because saved emitted arguments contained JSON spaces while the actual
canonical host call used compact JSON. Actual host ID/type/name and parsed MCP
result were retained. Byte identity of host JSON argument formatting is not the
contract; the argument values are.

The cold assertion now copies both full actual/saved call objects and JSON-decodes
only function.arguments on each before comparing the complete objects. All other
keys, ID, type and function name remain exact; argument values must still match.
Actual raw call/wire receipts remain unchanged. Result semantic comparison,
assistant-role requirement, host-before-caller ordering, full effect receipt
identity and exact retry zero-effects assertions are unchanged. No fixture,
supervisor or production change was needed.

Only the acceptance driver and this handoff changed for this correction. No
execution, imports, tests, checks, builds, formatting or network requests ran.
The corrected comparison remains unverified until the lead's authorized gate.
