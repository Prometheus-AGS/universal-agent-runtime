# Mixed crash seed proof source review

2026-10-22. Execute phase `uar-post-lifecycle-verification-readiness`. **Warning: one bounded canonical-history proof gap; no demonstrated production failure.** Source inspection supports the repaired seed provenance. No runtime acceptance is certified.

## [WARNING / MEDIUM] Host call message role is not asserted

File: `scripts/release/client_tools_acceptance.py:1033–1043`, `mixed_host_exchange`.

Exact source: `for index, message in enumerate(messages):` followed by `for call in message.get("tool_calls", []):`. The later result match requires role=tool and matching tool_call_id, but the call's enclosing message is never required to be assistant. Thus an identical host call inside a non-assistant message followed by a matching tool result can satisfy this helper and subsequent exact call/result comparisons. The required proof is specifically canonical assistant host-call, then host result, then caller results; role matters to that claim, not merely style.

Unexecuted falsifying observation: provide a captured request whose host-call message has role=user but otherwise identical call/result and ordering; current helper still returns that exchange. No claim is made that the running candidate emits such a request. Fix: require the candidate host-call message role to be assistant before matching its tool_calls. This is a local proof assertion, not a request to change production behavior.

## Source-supported repair observations

- Read complete modified `restart_case`, `mixed_host_exchange`, peer provider-response handling, MCP effect handling and the final `mixed-crash-seed-proof-repair.md` handoff. Peer `tests/fixtures/client_tools_provider.py:281–290` obtains `calls, text = produce(...)`, passes those same calls into the selected unchanged wire builder, retains calls/content type/exact body in the associated trace, and sends that exact body. Disk real_provider_response receipts preserve the same association. There is no extra provider round trip or hand-authored expected host ID in the driver.
- Peer `:237–249` constructs the actual result content from sequence/arguments before adding `receipt["mcp_result"] = result`, then uses that same result in the JSON-RPC response. Because text serialization occurs before receipt metadata is attached, the response payload remains nonrecursive and preserves its prior text content. The trace metadata does not alter the MCP effect or fabricate a runtime ledger result.
- Driver `:1062–1075` requires one new settled effect and exactly one marked provider request/response; finds exactly one actual generated host call by configured real tool name; requires nonempty actual wire receipt; and compares parsed emitted arguments against the new effect arguments. It saves that exact emitted call, actual mcp_result, full effect receipt and provider response. Public caller handoff must still succeed before Driver.answer/state persistence and seed_ready. Seed no longer incorrectly demands a generated host exchange in the request preceding its same-batch response.
- Cold resume `:1114–1138` still requires exactly one new provider contact with selected conversation marker. It requires exact saved emitted call equality and JSON-decoded host-result content equal to the saved actual MCP result, not guessed formatting or expected result text. Ordered matching requires host call before its role=tool result and that result before each exact submitted caller result. Full effect-receipt equality across crash, after continuation and after completed retry remains; retry must add no provider contact. The warning above is the missing assistant-role part of that proof, not a weakening of these retained comparisons.
- Native `src/mcp/registry.rs:3015` serializes the SDK result via serde_json::to_value; orchestrator result-history serialization is visible at `src/llm/orchestrator.rs:1406`. This supports semantic JSON comparison as an authored check. Actual SDK projection and resumed values remain execution observations and can still make the case fail.
- Scoped host_effects references use counts, emptiness or equality to captured lists; no inspected consumer requires only sequence/arguments keys. Added mcp_result metadata is retained on both sides of list comparisons. Supervisor aggregates continue to use pending/effect totals. No fixture framework, production endpoint, authority bypass or new scenario cross-product was added by this repair.

## Limits and method

The uncomfortable limit is that generated response/effect metadata proves source provenance, not actual socket delivery or durable host settlement. Seed readiness still relies on actual native handoff plus real peer effect observations; cold canonical history can be proven only by the actual resumed provider request. This gate does not prove crash-mid-effect exactly-once execution.

Commands were source inspection only: cat, sed, bounded rg, scoped git diff/tail. No imports, execution, syntax checks, lint, tests, builds, formatting, probes or product network calls occurred. Only this assigned report was added; no production code, guard or configuration was edited. Applied `uar-project-review` and `review-verification-protocol`. No further cohort or runtime gate was dispatched.

Reviewer is isolated native GPT-6-family with final artifacts/constraints and no producer conversation. Producer identity was not supplied. Same-family isolation is weaker than cross-family judging; cross-family certification remains pending. Actual seed pass, canonical history/SDK content, crash recovery, receipt retention and replay effects remain runtime-unverified.

Editorial sycophancy detection returned strict score **0.0**, classifications **[]**, correction mandatory **false**, timestamp `2026-10-22T21:57:31.372003Z`; it screens report bias only. No automatic correction or product evidence is claimed.

## Bounded corrective disposition

**Warning resolved in current source; no remaining blocking or warning finding in this bounded pass.** Original finding above remains historical. Freshly reread complete `mixed_host_exchange`: immediately inside its message loop it now executes `if message.get("role") != "assistant": continue` before matching tool_calls. It therefore requires assistant host-call, followed by the matching tool result; the unchanged cold caller-result ordering/equality assertions complete the required sequence. No other assertion change was requested or reviewed in this corrective step.

Source read only; no execution or checks. Only this review report was appended. Runtime seed/recovery and exact semantic result values remain unverified under the limits above.

## Corrective 13.28 — observed host argument formatting

**No source blocker found in the narrow Gate016 comparison correction.** Prior 13.27 review history remains unchanged.

Read retained Gate016 evidence with bounded jq selections only: mixed-crash-seed.json reports seed_ready=true, pending_batches=5 and mixed_host_effects=5; mixed-crash-resume.json reports failure at the settled host call/result assertion. local-gate.json records expected_crash_for_recovery signal9/exit_code-9 with joined=true/expected_crash_confirmed=true, but overall failed_extended. These are inspected existing receipts, not rerun results.

Actual saved OpenAI call in mixed-crash-state.json has id fixture-host-call, type function, name acceptance_peer__effect and arguments `{"query": "safe-local-acceptance-effect"}`. The resumed provider-trace.jsonl has the same ID/type/name with compact arguments `{"query":"safe-local-acceptance-effect"}`. Its matching role=tool result content encodes the saved MCP result object, including the actual sequence1 and safe effect arguments. This supports the reported formatting-only mismatch in the inspected first failed continuation; it does not certify every protocol's later continuation.

Fresh source `scripts/release/client_tools_acceptance.py:1122–1130` deepcopies both entire actual/saved call objects, then executes `call["function"]["arguments"] = json.loads(call["function"]["arguments"])` on each. It compares the complete resulting objects and the unchanged decoded actual MCP result against its saved actual result. Only the argument JSON representation is normalized; ID, type, function name, all other keys and semantic argument values remain compared. Invalid argument JSON raises rather than passes, and raw evidence is not mutated. The preceding assistant-role helper and following host-before-caller ordering, full effect receipt identity and exact retry zero-work assertions remain.

Only the review report was appended. Commands were sed/tail/rg source reads and bounded jq receipt reads; no imports, execution of the driver, checks, tests, builds, formatting, probes or product network calls ran. This Python comparison correction changes no production state or fixture effects. Corrected all-five cold behavior remains unverified until the lead's authorized gate. Existing isolation limits apply.

Editorial detection returned strict score **0.0**, classifications **[]**, correction mandatory **false**, timestamp `2026-10-22T22:04:22.416687Z`; it supplies no product acceptance evidence.
