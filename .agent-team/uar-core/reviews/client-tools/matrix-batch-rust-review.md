# Gate003 repair Rust source review

Phase: `uar-post-lifecycle-verification-readiness`, Execute. Review scope is the Anthropic assistant conversion in `src/server.rs`, A2A durable observer in `src/uar/api/a2a/thread_service.rs`, `checked` in `src/uar/persistence/providers/accounting/surreal.rs`, and `storage_error_detail` in `src/uar/runtime/cost_budget.rs`. Corresponding three repair handoffs were read. This reviewer owns this report only; no production edits.

## Initial disposition: block on one HIGH finding (resolved by corrective source review below)

**HIGH — A2A observer cancellation discards the invocation completion receipt.** In `thread_service.rs:614–625`, the original `ActorTurn.completion` is moved out of Entry into the observer and Entry receives a proxy receiver. The `cancellation.cancelled()` branch returns without returning or forwarding the original receipt. Dropping that observer drops both the original receiver and the proxy sender. If cancellation wins before completion is forwarded (including when both are ready), Entry retains only a closed proxy. Its next `refresh` takes the `TryRecvError::Closed` path at lines 92–96 and marks `execution_unconfirmed` and `cleanup_unconfirmed`, even if the actor subsequently delivers a legitimate terminal result. Previously observer cancellation left the original receiver in Entry. This changes task completion semantics during the shutdown path being repaired. Preserve the original receipt in Entry, or restore/retain it on observer cancellation before returning. The same ownership loss occurs when `scope.spawn` rejects admission: `ResourceScope::spawn` drops the supplied future on `admission_closed` (`lifecycle.rs:772–777`). Do not repair this by labeling an unobserved result successful.

This is a source-derived scheduling scenario, not a reproduced runtime failure. No claim is made about which ordering the next integration gate will exercise.

## Other reviewed changes

- Anthropic: omitting absent `tool_calls` matches `validate_visible_history` (`src/uar/api/client_tools.rs:197–201`), which interprets a present null or empty field as invalid. Actual tool calls remain arrays. No blocking finding in the scoped conversion change.
- Accounting: `(is_not_executed, statement_index)` orders substantive failures before rollback placeholders and preserves deterministic statement ordering within each class. The vendored executor explicitly replaces earlier transaction results with `QueryError::NotExecuted` before appending its actual failure. This preserves error evidence; it does not fix or identify the actual admission query failure. No blocking finding in the selection change.
- Storage diagnostics: labels use static kind/subkind strings; variant payloads and error messages are not formatted. Local pinned `surrealdb-types-3.3.0` source contains the matched variants and methods. Unmatched non-exhaustive variants retain their outer kind. No blocking finding in this helper.
- A2A registration-before-lookup correctly avoids the demonstrated initial-subscription race. Forwarding actor completion through Entry retains existing run correlation checks. Immediate authoritative reload after a CAS conflict avoids waiting for a notification already consumed. These observations do not negate the receipt-loss finding above.

## Evidence and limits

Only source inspection commands (`cat`, `sed`, `rg`, and scoped `git diff`) and this documentation write ran. No tests, check, lint, format, compiler probe, build, query probe, network, startup, or runtime command ran. The waypoint inspected reports revision 183, Execute, 143/166 implementation items, verification CLOSED. The lead owns the specifically authorized local functional test/fix/retest exception; this review grants no execution or publication authorization and establishes no phase acceptance.

Compilation, functional continuation/replay, durable query execution, shutdown ordering, and CI/merge status remain unverified. No unsolicited production change or guard was added. This isolated reviewer did not receive generation transcripts, but is a same-family agent; this report is not cross-model adversarial evidence. The budget query root cause remains UNKNOWN.

## Corrective source review

Reviewed the current observer artifact in `src/uar/api/a2a/thread_service.rs` and its corrective ownership handoff alone, without implementation conversation. **The HIGH finding is resolved at source level; no remaining blocking finding in the corrective span.**

- Spawn rejection: completion extraction now occurs inside the admitted async future. Dropping an unpolled future after rejected `scope.spawn` cannot execute that extraction, so Entry keeps the original receiver.
- Observer cancellation and ordinary error returns: the loop's return exits an inner future. The outer future then restores an unconsumed original receiver while holding Entry's lock, requiring both Entry and ActorTurn to match the observed run. The proxy sender stays alive until after restoration, so concurrent Entry refresh cannot observe a false proxy closure during this transfer.
- Completion wins: the received result is forwarded synchronously into the proxy before further awaits. If the actor sender actually closed without a reply, dropping the proxy preserves Entry's existing unconfirmed behavior. No manufactured successful result is introduced.
- A changed run is not overwritten during restoration. This keeps a newer turn's receiver intact.

Only `sed`/`cat` source reads and this report update were performed for the corrective pass. No tests, builds, checks, imports, formatting, runtime or network commands ran. This conclusion covers source ownership on normal observer exits and admission rejection; compilation and runtime shutdown ordering remain unverified. It does not establish correctness under forced task abortion or panic, which this bounded repair does not claim to address. Verification remains CLOSED for this reviewer; the lead retains the scoped local test/fix loop. Same-family review limitations and the UNKNOWN accounting query root cause remain unchanged.

## Provider identity repair source review

Reviewed the current `src/llm/liter_driver.rs` diff, `openspec/changes/openai-client-tools/handoffs/provider-identity-repair.md`, and actual pinned Liter construction/serialization source. **No blocking source finding in this repair.**

- `uses_generic_chat_adapter` matches pinned `client::build_provider`'s explicit-base-URL branch: exact DashScope multimodal model exclusion and the Azure, Anthropic, Gemini/Google AI, Vertex, and Bedrock prefix exclusions. DashScope inherits the default prefix stripper, which strips only its `dashscope/` prefix, matching the new helper.
- `DefaultClient::new_inner` builds the provider before environment loading (`vendor/git/liter-llm/crates/liter-llm/src/client/mod.rs:912`). Its subsequent `load_env` branch fills only the missing API key; it does not override the URL or model hint. Capturing selection from the supplied config/model therefore agrees with this pinned constructor. Explicit base URLs also pin this provider in `resolve_provider_for_model`.
- `OpenAiCompatibleProvider` does not override the trait's no-op `transform_request`. `prepare_request` serializes typed request fields, adjusts model/stream, applies that transform, merges extras, and serializes the resulting body. The existing typed-control extra-parameter guard prevents tool fields from being overridden by extras. Thus the capability allowance rests on this actual adapter path rather than the registry provider name or bare wire model.
- Inherited drivers copy the captured flag while reusing the original client, matching the pinned provider binding. Prebuilt clients with unknown construction remain conservative. Existing validated endpoint profiles retain their own path. No budget bypass or arbitrary recognized provider identity was introduced.
- The authored regression covers typed serialization of explicit false parallel control, named choice, strict tools, native control refusal, and selection exclusions. It does not itself execute Liter's final HTTP preparation or prove a remote endpoint accepts these fields; the handoff correctly identifies these limits.

Only `rg`, `sed`, `cat`, scoped `git diff`, and this report write ran. No executable verification, imports, network calls, or runtime commands ran. Compilation, the authored regression, actual final HTTP wire behavior, and the real five-interface matrix remain unverified. This is same-family source review, not cross-model adversarial evidence. No production file was edited by this reviewer. The lead owns the scoped local retest; this report does not open the global verification gate or certify phase acceptance.
