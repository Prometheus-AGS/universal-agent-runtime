# Independent Rust artifact review — A15

Verdict: **FAIL — SOURCE ONLY**.

Phase: `uar-post-lifecycle-verification-readiness`. Scope: the supplied `source-review-inputs.json`, its changed-source diff, actual Rust source and applicable OpenSpec requirements. Supplied source identity: `sha256:c6963d648f9961315d62c8a975da6501f64c2dd5f60f69b21aac3612a701c500`. This reviewer did not recompute that identity or execute a source-identity helper. The inspected root Cargo package uses Rust edition `2024`; no root package `rust-version` is declared in the inspected package block.

This is a fresh-context, artifact-only native fallback with no producer conversation, implementation handoffs, generation history or self-assessment supplied or read. Producer/judge are the same model family; that is weaker than cross-model review. The precise configured native model identifier is unknown. The full API adversarial-review gate was unavailable under the A15 no-provider scope and is not claimed here. The old Compass provenance in the repository map was treated as historical navigation context, not proof of current behavior; no graph rebuild occurred.

## Finding

### [src/uar/runtime/matching/intent/mod.rs:251] Auxiliary consumers reject the driver's normal budget receipt

Severity: **WARNING** — high functional impact on supported LLM skill matching and model-based context reduction; no memory-safety exploit is asserted.

Freshly read artifact:

```rust
// src/uar/runtime/matching/intent/mod.rs:247–251
match event? {
    NormalizedEvent::MessageDelta { text } => response.push_str(&text),
    NormalizedEvent::StreamStart { .. } | NormalizedEvent::Usage { .. } => {},
    NormalizedEvent::Done => { complete = true; break; },
    _ => anyhow::bail!("skill_classifier_stream_invalid"),
```

The changed Liter producer deliberately emits a `Custom` receipt before model output:

```rust
// src/llm/liter_driver.rs:751–753
let out = async_stream::stream! {
    if let Some(payload) = wire_receipt {
        yield Ok(NormalizedEvent::Custom { source:"uar.request".into(), event_name:"budget_receipt".into(), payload });
```

`wire_receipt` is present for a validated-wire request, a generic chat adapter, or the detected OpenAI/Azure paths (`liter_driver.rs:708–709`). The full `classify_skills_with_driver` function was read. Its nonempty candidate path dispatches successfully, then immediately fails on this ordinary first event before reading the ranking. This is a producer/consumer contract incompatibility, not a style preference or missing runtime receipt.

The same incompatibility affects the fully read `summarize_span` function: `src/uar/runtime/context/summarizer.rs:170–183` permits only text, start, usage and Done, and returns `summarization emitted unsupported ... event` for the budget receipt. Thus a successful genuine provider call cannot produce an accepted summary through this combination. Both consumers should be corrected together as one event-contract defect.

Surrounding-path inspection: `RunModelBindings::auxiliary` (`turn/bindings.rs:404–409`) returns the captured driver under the shared budget wrapper. Capture explicitly constructs `LiterLlmDriver` for endpoint profiles (`bindings.rs:276–287`). The full `BudgetedModelDriver::stream` implementation was read; it passes the inner event through at `cost_budget.rs:1561` rather than removing the receipt. The service LLM branch calls this classifier at `skills/service.rs:835–843`; the intent-classifier LLM branch also calls it at `runtime/manager.rs:5373–5379`. The manager turns the classification failure into `skill_matching_unavailable` instead of useful skill-assisted execution (`manager.rs:5404` onward).

Source-derived reproducer, **not executed**: configure service LLM matching or intent-classifier LLM matching with at least one eligible skill and an admitted OpenAI-compatible Liter binding with a known auxiliary allowance. The driver's initial `uar.request/budget_receipt` is sufficient to trigger `skill_classifier_stream_invalid`; no malformed ranking is needed. Separately, activate a model-based summary over eligible prose with the same binding and a valid allowance; the receipt triggers the unsupported-event branch before summary text. A stream consisting of that exact receipt, valid result text, usage, and Done isolates the fault without relying on provider quality.

Falsifier: establish that the actual captured auxiliary path filters this exact receipt before either consumer. The inspected binding and budget wrapper do not. Native drivers that never emit the receipt do not falsify the affected Liter combination.

Minimal correction: explicitly accept the known body-free host budget receipt in both auxiliary consumers while preserving refusal of errors, tool calls, unsupported events and incomplete streams. Keep accounting ownership in the budget wrapper. An eventual authorized regression should exercise the production Liter/wrapper event sequence for both consumers, rather than a text-only fake stream.

Requirement: `reengineer-user-outcome-verification/specs/product-validation-evidence/spec.md`, “Strategy evidence records the actual governed implementation” and “A selected strategy has no effective implementation.” Successful build receipts cannot establish either behavior.

## Inspection coverage and limits

Read source and changed hunks for configured output contracts/provider preparation, built-in seeding and protected agent store deletion, current-roster routing and child selection, skill service/classifier dispatch, context allowances/reduction/summarization, retained-source regeneration/progress and both durable adapter implementations, embedding mutation/startup settings, team-session persistence, shared execution capabilities, optional memory feature gates, and AArch64 FP16 helper annotations. Depth was concentrated on the finding and its callers; this bounded review is not exhaustive over the large whole-phase diff.

The roster path refreshes candidates and rechecks its revision before selected child admission; the public store deletion function checks protected IDs. These inspected mechanisms are not proof that all transports, restart paths, policies or concurrent executions satisfy their contracts. Team threading/history reassembly and all public protocol paths were not exhaustively traced. No complete line-by-line vendor comparison against a pristine same-version GEMM source was performed. The inspected GEMM helpers now declare FP16 preconditions and nearby runtime dispatch tests FP16 support; device execution and full patch equivalence remain unverified. Frontend rendering and SDK behavior are outside this Rust review.

## Existing receipts and unverified outcomes

Existing diagnostic receipts were read, not rerun. S6 and S7 report PASS against preceding source identity `28137dc46cbb05aa9169f93b46232ea6aafc8d3f04e7c841274e4bbf4d109018`; they do not certify subsequent edits. S8 attempt 2 reports exit 0 / PASS_AFTER_ONE_CORRECTION for the iOS library at the supplied c696 identity. S9 attempt 2 reports exit 101 / FAIL_AFTER_ONE_CORRECTION for Android. The supplied source-review input attributes that failure to missing Android OpenSSL; this reviewer did not independently diagnose its stderr. S5 final reports BLOCKED_AFTER_ONE_CORRECTION with no product compiler run. S10 final reports BLOCKED_TIMEOUT, no image, and acknowledged build-step cancellation. S14–S21 receipt files report PASS for their respective local specification items at c696. None of these establishes genuine runtime acceptance.

The runtime 97 cases / 582 rows, main journey and Issue 426 remain **BLOCKED / NOT EXERCISED** because the fixture is incomplete, as recorded in the supplied input. This review cannot certify runtime behavior, restart/backend parity, device behavior, usable release delivery, green CI or resolved merge conflicts. Repository policy reserves GitHub Actions for deployment work; no general CI acceptance is inferred.

## Actions and execution boundary

Only `target/uar-diagnostic-sweep-001/reviews/whole-phase-rust-review.md` was written. No production source, test, configuration, guard or dependency was changed; nothing unrequested was added. Commands were classified as source/artifact inspection (`cat`, `sed`, `rg`, `nl`, bounded output selection) or this authorized report write. Observed source reads exposed the concrete event mismatch quoted above. Large combined reads were truncated, so uninspected portions are not claimed as reviewed.

No tests, Cargo commands, builds, lint, formatting, typecheck, compiler/import probes, helper execution, service/provider calls, git diff checks or external gates were run. The general Cargo review checklist remains unexecuted under the explicit A15 source-only scope and operator verification lock. Existing receipts are the only command outcomes discussed here. The report is a source finding, not phase acceptance. No guard was introduced; the proposed correction traces directly to the observed normal event sequence.
