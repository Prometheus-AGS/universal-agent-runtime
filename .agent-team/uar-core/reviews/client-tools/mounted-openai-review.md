# Mounted OpenAI bridge source review

Phase: Execute. Reviewed current mounted OpenAI spans in `src/server.rs`, extracted `execute_prepared_chat_completion` in `src/uar/api/openai/routes.rs`, and `openspec/changes/openai-client-tools/handoffs/openai.md`. Related shared prepare, runtime admission/settlement, request decoding and policy source were inspected. This report is the reviewer's only edited file.

## Initial disposition: blocked

**HIGH — screened input can differ from executed caller input.** `extract_input_message` prioritizes the convenience `req.message`. The mounted handler screens that text with configured blocking guardrails and uses it for memory context. The caller bridge independently converts only `req.messages`, then the extracted helper clears `run_request.input` and executes that complete history. A request containing benign `message` plus a caller history whose user text should trigger configured blocking policy screens the benign text but executes the other text. The old branch executed the selected convenience message, so the bridge introduces this mismatch. Reject contradictory convenience input or derive screening from the same executed caller history before admission. This is a demonstrated source trust-boundary mismatch, not a reproduced runtime result.

**HIGH — the mounted JSON response deadline excludes settlement and persistence waits.** The new absolute deadline wraps `rx.recv` only. After RunDone, `await_settled_run_status` runs outside it; later `confirm_handoff_for_run` and continuation completion persistence do too. A terminal event emitted before indefinitely pending outer cleanup can therefore leave the response pending beyond the configured mounted timeout. The repair must bound these response waits while retaining host ownership and failing closed on interruption; a deadline does not prove cleanup failed or permit replaying uncertain effects. Lead reports the writer is correcting this span.

## Other source observations

- Ordered text history, assistant nullable content with calls, tool result IDs, strict schemas, typed choice, and explicit parallel controls reach the existing OpenAI projection and shared durable preparation. Unsupported attachments/content arrays/modes/cursors reject rather than silently losing history.
- Current full principal/tenant/agent scopes bind continuation lookup. Saved session resolution occurs before conversation-policy selection; explicit conflicting sessions reject. Shared prepare retains exact history/schema/result validation and completed response replay, including caps narrowed against stored caps.
- The admitted RunExecutionRequest retains verified model proof, agent artifact/target receipt, collaboration binding, verified user context, credential capture, session, policy, memory hits and presentation settings. The helper revalidates collaboration binding before prepare/replay; this matches the binding revalidation used by `execute_bound_request` before ordinary execution. There is one existing runtime executor through atomic observed startup.
- Caller requests remain private until real producer settlement and durable handoff confirmation; predecessor completion receipts are persisted before terminal success. Existing errors/cancellation do not become successful completions. These observations do not negate the blockers.

## Evidence and limits

Source inspection only: `cat`, `sed`, `rg`, scoped `git diff`, followed by this report write. No tests, builds, checks, compiler probes, imports, formatting, runtime, or network commands ran. No production changes were made by this reviewer. Global verification remains CLOSED for the reviewer; the lead owns its scoped local functional test/fix loop. No compile, runtime, five-interface acceptance, CI/merge, or publication claim is made. This is same-family isolated source review rather than cross-model adversarial evidence.

## Corrective artifact review: no remaining source blocker

Re-read the current caller conversion, complete JSON projection span, and corrective handoff. Both initial HIGH findings are resolved at source level.

- `caller_openai_request` rejects any supplied legacy `message` alias before returning a caller request. Therefore the mounted caller path cannot screen that convenience field while executing a different messages history. The rejection is scoped to caller-tool requests and precedes admission/execution.
- One absolute `timeout_at` now wraps the JSON projection future containing event collection, producer/graph settlement, durable handoff confirmation, and continuation completion persistence. No reviewed response-stage await in those operations sits outside that deadline. The timeout returns 504 without manufacturing successful settlement. The existing manager join paths retain ownership of pending producer handles across cancellation; durable writes that already committed are not falsely described as rolled back, and exact durable replay remains available through shared preparation.

No further blocking finding in these corrective spans. This is source clearance for the lead's authorized local build/retest, not acceptance evidence. Only source reads and this report update ran; compilation, timeout scheduling, configured guardrail behavior, persistence interruption/replay, and full HTTP functional behavior remain unverified. No production edits or executable verification commands were performed by this reviewer. The global reviewer gate remains CLOSED.
