# ACP selected-agent and persistence source review

Phase: Execute, `uar-post-lifecycle-verification-readiness`.
Verification: CLOSED. Source inspection only; no tests, builds, compiler, formatter, SQL probes, scripts, network or runtime requests executed. This document is not acceptance evidence.

## Blocking finding

**P1 — Restored session retains request history and can replay an old response for a new prompt.**

`src/uar/api/acp/handler.rs:363` assigns `Some(supplied_tools)` to the restored session and publishes it to the shared session map. This value contains the submitted continuation history. A later `runs/create` without an explicit `client_tools` field inherits that history through `tools.or(session.client_tools)` (line 374). Shared `prepare` sees the old completed result batch and returns Replay; the handler immediately returns the persisted response (line 424), ignoring the new request input. If the previous continuation produced another pending batch, the inherited history still replays its predecessor rather than executing the new prompt.

The same publication precedes current-agent resolution and full predecessor/schema/result validation. An invalid submission with one genuine pending call ID therefore leaves a restored session carrying its invalid history even though the request fails admission.

Required repair: restore session identity and declarations without retaining per-request history. Validate the supplied continuation through the existing exact-scope/shared admission path before publishing recoverable session defaults. Preserve the explicit submitted history for this request and exact retry replay. Do not remove retry idempotency to repair fresh-turn behavior.

Suggested later authorized functional scenario: selected nondefault agent issues a batch; restart; submit complete results; send a distinct fresh prompt using only the restored session ID and input. The fresh prompt must execute under the current selected artifact and must not return the earlier run ID/response. Also submit invalid predecessor/schema history with a genuine locator and inspect whether session state was published.

## Scope and safety assessment

- Persistence trait, scoped facade forwarding/read classification, and both provider wrappers include the new method. The facade retains lifecycle admission semantics.
- PostgreSQL binds all values as parameters. Its query filters owner, session, batch and null-safe tenant equality; the deserialized row is validated and owner checked. A second exact match fails closed instead of selecting an arbitrary agent.
- Surreal binds the owner parameter, then compares tenant/session/batch in Rust, validates the selected row, checks owner, and rejects ambiguity. No caller-selected agent controls recovery lookup.
- ACP additionally checks protocol `acp`, authenticated owner/tenant, exact session and call membership. The later exact-agent lookup and shared prepare preserve full batch, visible predecessor, schema, protocol and CAS validation. The locator alone grants no execution authority.
- Each request resolves the session's selected artifact using `resolve_registered_agent`; only the public `default` alias maps to `default-agent`. The resolver loads the current persistent artifact before built-in fallback. Current UserContext and credential capture enter the new execution request.
- Replay requires matching full result content and a completed row, after shared visible-history and schema validation. No duplicate host execution was found in this inspected path.

## Nonblocking limitation

Session recovery reads all matching owner candidates: PostgreSQL has owner-leading indexes but no session/batch JSON expression index; Surreal scans and deserializes the owner's serialized rows. This grows with retained owner history. No query execution or cardinality evidence was gathered, so no performance claim or migration recommendation is made. Malformed serialized rows in an owner's candidate set can fail recovery before a matching row is reached; this is fail-closed availability behavior.

## Handoff

Only this review artifact changed. No production edits, guards or unrelated additions. Blocking finding sent to lead. Provider syntax/driver compatibility, compilation, restart behavior, current policy enforcement and functional replay remain unverified because the gate is closed. Reviewed source is the shared working-tree content read during this task; concurrent edits after inspection require renewed source review.

## Corrective source disposition

Re-read the frozen `handler.rs` cold-recovery/admission path and the handoff's deferred-session-publication section. The P1 above is resolved at source level: recovered identity remains in `session_to_restore` until exact-agent lookup and shared prepare succeed; admission errors return before insertion. Publication occurs before matching the successful preparation, covering Continue and Replay. Cached restored declarations explicitly clear history, and a later request inheriting these defaults builds a new user message from fresh input. Explicit continuation history remains request-local, so completed result retry comparisons are preserved.

No additional blocker found in this corrective span. Persistence queries were unchanged and were not re-executed. This disposition supersedes the blocking status of the initial finding; it does not certify compilation or functional behavior. The fresh-after-cold, invalid-history nonpublication, selected-current-artifact and exact-retry scenarios remain unverified for the lead's explicitly authorized real-path gate. Only this report was appended; all commands remained source inspection or authorized documentation writing, with verification CLOSED.
