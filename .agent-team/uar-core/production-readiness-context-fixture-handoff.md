# E cohort context fixture handoff

Phase: `production-readiness-476-capability-dispatch`, Execute. Verification **CLOSED**.
Source-only implementation in the shared `/Users/gqadonis/.claude/worktrees/production-readiness-476` checkout. No test, check, format, compiler, build, API, or review gate was run by this writer. The lead owns renewed whole-phase source inventory and operator authorization.

## Observed failures and source causes

- Historical `evidence/context-history-attempt-001.txt`: 27 passed, four failed, two pre-existing frozen diagnostics ignored; exit 101. These counts do not describe the edited candidate.
- E3: the summary fixture imposed input allowance32 on the entire final serialized summary request, producing `ProtectedOverflow { required_tokens: 699, input_allowance: 32 }`. `src/uar/runtime/context/summarizer.rs::summarize_span` builds the fixed system prompt and JSON prose; `AuxiliaryRequestAllowance::prepare_request` preflights it. `src/llm/liter_driver.rs::enforce_request_budget` counts this full final body as protected. The expected `proven final-wire allowance` substring did not match this real diagnostic and the allowance did not establish that the fixed prompt could fit.
- E4: `jsonwebtoken::encode` ran before the UAR verifier. `src/uar/security/jwt.rs::ensure_rustcrypto_provider` installs the process provider exactly once and rejects prior initialization; `middleware.rs::resolve_authenticated_context` maps that conflict to500. Existing `tests/support/truthful_tool_outcome_peer.rs::user` enters the production verifier with a malformed token before fixture signing. Current session partitioning in `manager.rs::start_run_internal` uses `ActorOwner::session_owner_key`, not literal subject `alice`.
- E2 runtime-writer disposition: stale anonymous/non-durable fixture. `manager.rs` records interrupted pending assistant calls and invokes `save_session` only when persistence is present; the old fixture supplied no persistence and no verified user. The required owned durable path uses a production-verified principal, `RunExecutionRequest::with_user_context`, and a fresh local Surreal store. No runtime-source repair was indicated.
- E1 provider-writer disposition: UAR compared its qualified destination model body with Liter's actual prepared stream body, which resolves the provider routing qualifier before final serialization. The provider writer owns the production correction to validate/count actual prepared bytes. Fixture trusted destination remains `openai/uar-exact-cl100k-json-v1`; the captured adapter wire literal is `uar-exact-cl100k-json-v1`.

## File changes and retained assertions

`tests/context_history_integrity.rs`:

- E3 retains the synthetic exact contract and real Liter no-dispatch listener. Input allowance becomes256 with context1024/output8. A short host-marked prose control must summarize successfully through the mock under the identical exact allowance. Long host-marked prose must fail with the captured auxiliary allowance diagnostic and `ProtectedOverflow` at input256; source history equality and zero actual provider connections are asserted.
- E1 changes only the captured body model literal expectation to `uar-exact-cl100k-json-v1`, with a comment explaining trusted destination versus adapter wire model. Synthetic contract, exact output ceiling, stream flag, metadata, protected call ID, escaped arguments, result bytes and actual final body count assertions remain.
- E2 uses fresh temp Surreal persistence, a production-verified owner and `execute_request` with that owner for both turns. The pending session is read by verified storage key. First-run Error, empty-content pending assistant call, `MissingResult`, second-run Error and exactly one provider request remain asserted. A direct owner-scoped durable `load_session` must reproduce the complete retained pending history, so merely supplying persistence cannot satisfy the durability claim.
- E4 reuses its existing actual production verifier and fixture signer through the shared helper described below. Warm history lookup uses that principal's `session_owner_key`. Raw checkpoint metadata and revoked authorization still return404 `checkpoint_resume_evidence_unavailable`; legacy direct admission still errors `checkpoint_admission_required`; execution count, provider call count and unchanged warm history assertions remain.
- No test deleted, no skip added. The two existing frozen diagnostics remain unchanged.

`tests/support/harness_context.rs`: contains the existing E4 SecurityConfig/signing mechanism extracted as `verified_fixture_user(subject)` for the two owned Rust regressions. It calls the actual production verifier with a deliberately malformed fixture token before signing and requires401; the signed fixture token must then produce the actual verified UserContext. No synthetic context/ActorOwner, privileged role, changed authentication boundary or functional/manual Cucumber authentication step was added.

## Producer coordination and unresolved work

- E1 provider writer recommendation integrated exactly as described above; production serialization changes remain with that writer.
- E2 runtime writer recommendation fully integrated as described above; runtime source left with its owner and unchanged by this fixture writer.
- Both producer recommendations are incorporated. No fixture implementation item or unresolved design decision remains in this packet. This does not open the phase verification gate or claim phase completion.

## Verification and limits

All authored behavior, fixture construction, warning cleanliness, timing, store lifecycle and regression outcomes are **UNVERIFIED**. Source inspection is not acceptance. The short-prose positive control is authored but has not run. Its source anchor is `src/uar/runtime/context/summarizer.rs::summarize_span`: the same `bounded.prepare_request(model, &mut request)` runs before either driver's `stream`. The control can establish fixed-envelope admission through that production seam when executed; its mock transport cannot prove an actual short-prose HTTP send. The long-prose case retains the real Liter/TCP zero-arrival assertion, identical allowance, explicit input overflow and unchanged history. All added fixture assertions trace to the observed E2/E3/E4 failures or their existing real identity/history boundaries; no production guard or privilege was added. No unrequested implementation was added.

Only source reads (`pwd`, `cat`, `rg`, `sed`) and authorized patch writes were issued. No Cargo, test binary, API, Docker, formatting, diff check or independent review was started. Exact source identity remains the lead's whole-phase inventory responsibility because other writers share the checkout.
