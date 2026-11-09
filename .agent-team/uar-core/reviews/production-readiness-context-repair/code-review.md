# E repair independent source review

Scope: initial isolated review of `target/production-readiness-476/context-repair-review/artifact.patch` against its `requirements.txt` and the constrained-model-dispatch OpenSpec delta. Phase: authorized source-only repair review; the verification gate remains CLOSED.

Isolation: reviewed the supplied resulting artifact, repository rules, tool policy/map, relevant production and fixture source, and the pinned vendored Liter preparation code. Did not read producer handoffs, generation rationale/history, progress/inventory artifacts, memories, or other reviewers' reports. This is a native same-family assessment, weaker than an independent-family review. It does not reopen the original phase review cap.

## Findings

### [HIGH] New negative-path unit test cannot compile

File: `src/uar/runtime/cost_budget.rs:2035`

Exact new expression:

```rust
let error = driver
    .stream(contract_request)
    .await
    .expect_err("durable contract without final-wire evidence must refuse");
```

Failure scenario: compiling the library with its unit tests includes this function. `stream` returns `Result<ExternalDriverStream>`, and `Result::expect_err` requires its success type to implement `Debug`. `ExternalDriverStream` is explicitly `Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>` (`src/llm/external_driver.rs:16`); its erased stream trait object has no `Debug` bound. Thus the test target fails type checking before this new refusal assertion or the existing library tests can run. The assertion's intended error path does not avoid that compile-time bound, and the concrete fake stream is erased by the method's declared return type.

Correction: extract the error with an explicit `match`, or use `.await.err().expect(...)`, retaining the existing admission-origin, cause, and zero-stream assertions. Do not add a `Debug` requirement to the public stream contract merely for this assertion.

Classification: definite source-level type defect; no compiler was executed and no diagnostic is claimed as observed.

### [MEDIUM] Profiles without budget contracts lose final wire-control validation

File: `src/llm/liter_driver.rs:539` (early return), with the removed equality guard replaced at `src/llm/liter_driver.rs:912`.

`validate_dispatched_budget` immediately returns `Ok(None)` when `source.budget_contract` is absent. The old stream path compared the profiled serialization with Liter's prepared bytes regardless of the presence of a budget contract. The replacement calls this function but no longer performs that comparison, so such a profile now validates only the preflight representation.

Concrete failure scenario: an otherwise valid settings-only `OpenAiCompatibleChatV1` profile has `output_ceiling: None`, qualified identity `gateway/custom/model-a`, and exact `wire_model: "custom/model-a"`. `from_endpoint_profile` constructs the generic adapter; `build_chat_request` copies that wire-model alias into the typed request and admits the no-contract profile. Pinned Liter constructs the adapter with name `custom` (`vendor/git/liter-llm/crates/liter-llm/src/client/mod.rs:1254`) and strips that prefix from the request model (`vendor/git/liter-llm/crates/liter-llm/src/provider/mod.rs:773`). The resulting body therefore sends `model-a`. The new early return accepts this mismatch and `profiled_wire_receipt` also certifies the changed body. Before this diff, the equality guard rejected it. A gateway may select a different model or reject the request; either outcome violates the captured exact alias contract (`src/llm/mod.rs:339`). Profile construction permits this nonempty alias and does not independently compare the prepared provider body.

Correction: validate the actual prepared body against the captured endpoint profile even without a budget contract. Keep contract-dependent output/count arithmetic conditional, but preserve profile model, allowed fields, and typed controls. A profile whose final alias is not preserved should refuse before dispatch. This does not require adding a final-preparation prerequisite to ordinary unprofiled, uncontracted calls.

Classification: definite source-level loss of an existing guard, with a concrete transformation path; no gateway behavior was executed or certified.

## Evidence and limitations

- Source inspection only: `cat`, `sed`, `rg`, `nl`, and `pwd`, always with shell working directory `/Users/gqadonis/.claude/worktrees/production-readiness-476`. The supplied patch was the review diff, rather than unrelated checkout changes.
- Executed checks: **zero**. No tests, Cargo commands, compiler/import probes, lint, formatting checks, builds, API/container calls, external review agents, or `git diff --check` were run or delegated.
- No runtime certification. Actual captured sends, durable admission/settlement, cancellation, and fixture completion remain unverified under the CLOSED gate.
- Changed file: this owned report only. Product source was read-only. No production guards or unsolicited implementation were added.
- The profile correction above restores a real captured destination boundary; the test correction addresses a concrete Rust type constraint. No speculative hardening is requested.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 1 | info |
| LOW | 0 | pass |

Verdict: WARNING — resolve the new test compilation defect before merge; also restore final profile validation for no-contract settings-only sends. This verdict is a source assessment, not acceptance evidence.
