# Candidate documentation handoff

Phase: Execute, `uar-post-lifecycle-verification-readiness`, change `openai-client-tools`. Current waypoint read reports source/derived revision 114, planRevision 91, implementation 143/166. Verification remains CLOSED. This handoff does not mark canonical tasks complete or establish phase acceptance.

## Files changed

- `docs/releases/v1.2.1.md`: new UNVERIFIED source-candidate release note. Explicitly disclaims image, digest, tag, publication, compilation and acceptance; names the active phase and pending whole-phase reconciliation/inventory.
- `docs/releases/v1.2.1-kubo.md`: new five-interface contract. Covers standard OpenAI tool calls/results and controls; native AG-UI tools, private forwardedProps controls, pendingToolCallIds and tool messages; A2A 1.0 metadata URI, task/context and InputRequired; bound/unbound Anthropic native tools/choices/results and positive max_tokens ceiling; ACP session/run client_tools and runs/submit_tool_results.
- `CHANGELOG.md`: added 1.2.1 UNRELEASED/UNVERIFIED candidate section. Historical 1.2.0 entry and both frozen 1.2.0 release files were not edited.
- This documentation handoff.

## Contract decisions recorded

Caller descriptions never register executable host tools. Complete owner/tenant/agent/protocol/session/source-run and schema/history bindings protect durable continuation; IDs are public locators, not authority. Shared Claimed recovery is fail-closed. Completed means a protocol receipt/projection and is not universal model success: AG-UI stores admitted run binding; A2A task receipt can still be in progress; native handoff terminals can contain pending work.

Provider limits reflect actual authored driver predicates. Liter explicit parallel controls without an exact profile are limited to OpenAI/Azure/Azure OpenAI, choice additionally allows Anthropic, strict remains limited to the OpenAI-family paths, and Anthropic Liter requires object-root schemas. Native Anthropic constraints and budget refusal remain explicit. These statements are source descriptions without runtime evidence.

Acceptance documentation names real source paths and current parser flags, including graph and budget/graph restart modes. It contains clearly marked proposed/unexecuted entry-point examples only. It does not provision services or invoke any command. --model has different route semantics; route-specific invocations may be necessary. Positive max_tokens support was added by another writer while these docs were authored and is reflected in the document.

Bound Anthropic examples specify a stable session_id: the current source compares it to the durable batch, while the message response does not expose a generated UAR session ID. Parent was notified to reconcile the acceptance driver. Runtime ClientToolInput gained output_token_cap while this documentation task owned docs only; parent was notified to update every source initializer, including prior AG-UI literals.

## Source activity and limits

Source inspection used `cat`, `sed`, `rg`, and `ls` on affected runtime/API/provider sources, candidate acceptance programs, existing frozen release documents, implementation handoffs and the canonical waypoint. Writes were confined to the three assigned documentation files and this handoff. No tests, lint, formatter, compiler probe, build, packaging check, diff check, review gate, publication, commit or push ran. No checks are reported as passing.

The source is an evolving shared checkout, not a frozen release identity. The lead must record the complete exact source inventory at the proper boundary, reconcile canonical tasks, and request the exact gate commands. The docs add no production guards or unrequested behavior. Actual wire/schema behavior, provider semantics, governance, race/restart recovery, accounting and customer artifacts remain unverified because the operator lock is CLOSED.
