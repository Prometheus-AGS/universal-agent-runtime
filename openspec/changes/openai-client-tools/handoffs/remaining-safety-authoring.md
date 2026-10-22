# Remaining safety acceptance authoring

Execute phase. Source inspection and authorized authoring only; no execution,
imports, tests, checks, builds, formatting or network calls ran. The lead owns
the scoped functional loop and any execution approval. Shared edits retained.

## Owned files and inputs

- `tests/fixtures/client_tools_provider.py`: new `host_after_results` scenario
  emits an ordinary caller request first, then deliberately emits the configured
  real host-tool call after caller results even if its schema is absent. This
  models a malicious provider; only an actual host result permits final text.
  `mixed_text` emits text alongside the caller call. OpenAI and Anthropic JSON
  and SSE helpers preserve both text and tool content.
- `scripts/release/client_tools_acceptance.py`: separate `--mode safety` with
  required `--safety-agent-id` and `--host-tool-json`. The former names a dedicated
  mutable registered agent initially allowing the actual MCP name in the latter.
  All existing URL/model/auth/protocol inputs remain required. Added an optional
  method argument to the HTTP helper for actual authenticated PATCH/PUT. Native
  Anthropic and AG-UI declarations now forward each tool's parameters rather than
  a fixed schema; ordinary valid payload values are unchanged.
- This handoff records the bounded result. No production runtime or supervisor
  file was changed by this authoring task.

## Separate 50-case safety mode

Each of the five interfaces receives ten recorded cases:

1. Duplicate caller names rejected before provider contact.
2. Caller/actual-host name collision rejected before provider contact.
3. Invalid JSON Schema type rejected before provider contact.
4. Full two-call parallel batch handed to the caller and completed together.
5. Mixed text/caller tool response and continuation in JSON mode.
6. Mixed text/caller tool response and continuation in SSE mode.
7. Actual provider HTTP503 surfaced as failure.
8. Malformed provider function arguments surfaced as failure.
9. Forced parallel batch under parallel=false surfaced as failure.
10. Current selected-agent host policy denial between handoff and results.

The final case retains the actual GET artifact, establishes initial live host
advertisement, PATCHes policy.tools.allow=[]/deny=[actual host name], submits exact
caller results, and inspects subsequent actual provider requests for absent host
schema. The malicious peer still returns that host call. Acceptance requires
public failure and zero actual MCP effects. PUT restores the retained artifact
in finally and checks restored policy equality. The dedicated agent must not be
shared with concurrent writers; restore failure is a case failure.

Positive caller cases require zero real MCP effects before manual result
submission and after continuation. This is evidence about the configured real
host effect and caller handoff boundary; it is not a claim that every possible
host executable was instrumented. All 95 original matrix cases remain intact.

## Remaining authority gaps and verification limits

The safety report explicitly retains same-subject/different-tenant continuation
and removed/deactivated destination or changed executable-binding acceptance as
unverified gaps. Cross-owner results do not prove cross-tenant isolation. With
these gaps, the existing report exit convention remains2 even if all50 scoped
cases pass; supervisors must retain the report without claiming complete safety
acceptance. Failed scenarios retain exit1.

All authored cases remain unexecuted. Compilation, Python execution, native wire
behavior, policy restoration and real provider/MCP counts are unverified here.
The A2A current-policy and ACP selected-agent production repairs belong to other
writers; this driver deliberately exercises them through real HTTP paths and
does not substitute fake managers, snapshots, ledgers or admission results.

## Scoped extension: tenant and executable-binding coverage

The lead subsequently required authoring both gaps above. Safety now has60
recorded cases, twelve per interface; the prior50-case/exit2 description is
superseded for the authored mode. The original95-case matrix remains intact.
Safety adds no known coverage-gap entries and returns0 only if all scoped cases
pass. None have executed here, so the security properties remain unverified.

Additional required input: `--other-tenant-bearer-token-env`. The supervisor must
mint/attest a JWT with the same subject as the owner and distinct tenant claims;
the driver requires different credential bytes, submits the exact pending batch
with that credential, and requires failure with no provider/MCP effects. The
driver does not decode unverified JWT claims and pretend that proves provenance.

The real peer exposes POST `/_host_binding` with exactly `{"enabled": false}` or
`{"enabled": true}`. This changes actual MCP `tools/list` to omit/restore the safe
effect tool; it does not reset provider requests/effect receipts or touch runtime
state. `/_trace` adds `host_binding` and `binding_requests`, with actual tools/list
and tools/call method receipts and enabled state. The on-disk trace retains the
same secret-free receipts. A disabled tools/call returns a protocol error and
records its attempted invocation, enabling the driver to detect stale authority
even when the peer independently refuses execution.

The new withdrawal case establishes initial host advertisement/caller handoff,
withdraws the real MCP discovery binding, submits exact results, and requires an
actual disabled tools/list receipt, zero tools/call attempts, zero real effects,
public failure and absent host schema on any subsequent provider requests. It
restores the binding in finally. The peer's host_after_results scenario still
attempts the known host call regardless of current advertisement.

Source inspection: manager.rs discovers run-scoped names before final policy;
McpRunResources::discover_tool_ids invokes runtime preflight. runtime.rs prepare
uses cached catalogs only for non-global sources, while Global requests connect
through get_or_connect_pinned. Whether retirement produces the required fresh
wire tools/list remains a real-path observation, not a claim inferred from the
Global comment. The case will fail rather than substitute an internal cache write
if fresh discovery does not occur.

Mixed text continuation now passes the observed fixture text into Driver.answer;
its default remains empty for existing matrix cases. This preserves the exact
assistant history instead of creating a transcript conflict on result submission.

All source changes above remain unexecuted. No test/import/check/build/network or
formatting command ran. Current-policy, tenant and revoked-binding acceptance
must be established by the lead's concrete authorized real HTTP gate.

Source follow-up from the runtime owner: manager.rs creates a new
McpInteractionChannel for each nonchild verified durable root; its UUID enters
McpBindingRequest and the binding-cache key through with_interactions. Projected
activation invokes preflight; the new key reaches ConfiguredMcpConnector and
registry connect_http_binding_with_headers, whose list_all_tools performs the
actual peer discovery. This supports the withdrawn-binding assertion for the
isolated durable Surreal gate. It does not establish unconditional freshness for
generic configurations without an interaction channel, nor replace real gate
receipts.
