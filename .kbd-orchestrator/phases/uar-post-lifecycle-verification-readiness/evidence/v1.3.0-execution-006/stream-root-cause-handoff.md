# Packet006 stream diagnosis handoff

Phase: uar-post-lifecycle-verification-readiness, Execute. Operator verification gate CLOSED. Commands in this diagnosis were source/log inspection and this documentation write only. No test, lint, format, build, compiler probe, service, external mutation, or verification command was run. No production source was changed. Others share the checkout; their edits were preserved.

## Delta and concrete evidence

ERROR-01 and ERROR-02 failed sse_object_count; HISTORY-01 failed the same assertion class. Packet006 cases.json and body.json retain FAIL and cargo exit101. The lead's response receipts report HTTP200 bodies685/679/685bytes respectively and zero matching expected terminal objects. The raw response bodies were not retained, so this diagnosis does not claim to reconstruct them.

Source trace: tests/post_lifecycle_main.rs request() collects the response text, and observe_response() parses data: lines as JSON before comparing the asserted JSON pointers. src/uar/api/sse.rs build_sse_response_inner emits compact JSON as a data field. Its to_agui_event maps NormalizedEvent::Error to /kind=error and /code=the original code; RunDone and RunDoneWithUsage both map /kind=done. src/uar/api/routes.rs stream_run selects that projection by default; only explicit stream_mode=agui_spec chooses the spec projection. It subscribes before collecting replay and yields a terminal event before breaking. These inspected seams do not establish a decoder, projection, or premature drop defect.

The root_closed_class recipe requests /api/uar/runs/{root-denied_run}/stream without agui_spec. It requires exactly one budget_admission_failed error and zero done. An earlier Error carrying another code terminates this route and would produce the same observed failure. The historical bytes/hash receipts alone cannot distinguish that from the other unobserved payload possibilities. Do not change the oracle or add an error-category fallback based on byte lengths.

The retained raw/peer-events.jsonl contains one model_request with case=default, mode=success (sequence390), followed by model_replied stop (sequence391). This supports bounded observation that the expected tagged provider scenarios were not captured there; it does not establish why the three run streams ended or bind that request to a particular failed run.

## Assessment

A broad architectural cause is unproved. The observed problem crosses run admission, stream error identity, and fixture observation; the currently proved defect is inadequate failure evidence retention. A localized earlier admission/ownership error remains possible, but should not receive speculative source repairs. Existing source emits several deliberate terminal refusal categories before model admission, so preserving exact code is necessary.

## Minimal next diagnostic, not executed

On an operator-authorized whole real-path rerun, retain each stream response's exact bytes in the private evidence directory before assertions, bound to method/path/control, run ID, HTTP status, content-type, byte count, hash, and parsed event name plus code. Retain corresponding run snapshot/status and the complete source identity. Keep the original oracles unchanged. This identifies the actual first terminal error versus missing/malformed frames; it does not by itself certify budget admission, history deletion, provider noeffect, or full phase acceptance. Lead owns fixture source and the renewed packet/approval; no fixture edit is made here.

## Files and open decisions

Only this handoff file was authored. No guards, fallbacks, or other unrequested production behavior were added. All runtime behavior remains unverified under the CLOSED gate. The lead must reconcile this evidence with other writers' findings before choosing a repair or requesting exact diagnostic gate approval. A6 cap/repair allowance was not extended by this diagnosis.
