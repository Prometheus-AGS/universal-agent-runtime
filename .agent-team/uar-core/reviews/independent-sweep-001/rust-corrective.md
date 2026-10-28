# A15 Rust corrective review — attempt 2

Fresh source evidence, read during this review:

```rust
// src/uar/runtime/matching/intent/mod.rs:251–254
NormalizedEvent::Custom { source, event_name, .. }
    if source == "uar.request" && event_name == "budget_receipt" => {},
NormalizedEvent::Done => { complete = true; break; },
_ => anyhow::bail!("skill_classifier_stream_invalid"),
```

```rust
// src/uar/runtime/context/summarizer.rs:175–181
NormalizedEvent::Custom { source, event_name, .. }
    if source == "uar.request" && event_name == "budget_receipt" => {}
NormalizedEvent::Done => {
    completed = true;
    break;
}
event => {
```

Verdict: **PASS — SOURCE ONLY, narrowly scoped corrective finding**. The original producer/consumer mismatch is resolved in the inspected source. No new actionable defect was established in the correction or its directly surrounding behavior. This does not certify compilation, runtime behavior, release readiness or the whole phase.

## Scope and independence

Phase: `uar-post-lifecycle-verification-readiness`, current task 2.9. Waypoint remains implementation 144/168, stage IN_PROGRESS. The operator verification gate remains CLOSED. Supplied current source identity: `sha256:e79f323b0f9de281afc2e13dc78406fe729e96972419c4398ca08698b66fa2f2`; not recomputed by this reviewer. Root Cargo.toml declares edition 2024; no rust-version appears in its inspected package block.

Inputs: source-review-inputs-after-RUST-correction.json, its referenced current-source-after-RUST-correction.diff, original independent RUST/attempt-1/report.md, actual consumer/producer/accounting/binding source, and the product-validation-evidence specification. Full consumer functions classify_skills_with_driver and summarize_span, the full summarize_marked_prose_with_allowance function, and BudgetedModelDriver's complete stream implementation were read. Binding construction and auxiliary selection were inspected at turn/bindings.rs:276–287 and 404–409. The review is bounded to the previous finding and regression risks from its correction, not a second whole-phase audit.

Fresh-context artifact-only native fallback: no producer conversation, implementation handoff or generation history read. Producer and judge are the same model family; this is weaker than cross-model review. Exact native judge model identifier: unknown. No API adversarial provider was called under the A15 no-provider scope. Skills consulted: uar-project-review, prometheus-rust-workspace, rust-best-practices, rust-code-review and review-verification-protocol. Repository map provenance was treated as historical navigation context, not evidence of current execution.

## Finding disposition and regression inspection

Original finding severity: WARNING, with high functional impact. Disposition: **RESOLVED IN SOURCE**.

Both guarded arms require the exact source AND event name. A Custom event with a different source or name still reaches the existing refusal branch. The payload is destructured away and never appended to the classifier JSON or summary text. Only MessageDelta text contributes to either result (intent/mod.rs:248, summarizer.rs:172). The observed Liter producer emits this exact metadata pair at liter_driver.rs:751–754, before generated chunks; the new arms therefore address the concrete event that previously aborted successful auxiliary calls.

Error propagation remains `match event?` in both functions. A normalized Error, tool event or unknown Custom event still fails through the existing catch-all. StreamStart and Usage remain permitted. Done remains necessary: the classifier checks complete at intent/mod.rs:257, and the summarizer checks completed at summarizer.rs:190. Receipt metadata alone cannot complete either operation or produce an accepted result. Classifier JSON/selection validation at 258–270 is unchanged in the inspected source; the summarizer's nonempty and output-ceiling checks remain at 191–197.

Accounting stays with the host wrapper: auxiliary() returns budget.bind(...) at turn/bindings.rs:408; BudgetedModelDriver::stream admits the call, enforces cancellation/deadline, charges Usage and settles terminal accounting before yielding its corresponding event (cost_budget.rs:1454–1561). Its emitted uar.accounting/unresolved Custom event is not newly ignored. Receipt omission from generated text does not bypass these host operations.

Allowances still prepare both requests before dispatch (intent/mod.rs:242; summarizer.rs:138–164). Summary replacement remains atomic relative to the input slice: all summaries and aggregate budget checks finish before output construction at summarizer.rs:271–278; an earlier failure returns without mutating the borrowed messages. No new fallback, retry, guard, dependency or mutation authority is introduced by the correction.

Source-derived falsifier, not executed: a stream with the exact budget receipt, valid result text, Usage and Done should proceed past the old failure point. Change either metadata identifier and the consumer should refuse; remove Done and it should reject incomplete output. A runtime result contradicting these branches would invalidate this source conclusion. These are inspection cases, not passing test receipts.

Requirement connection: product-validation-evidence/spec.md:32–33 requires the selected model-dependent algorithm to use its admitted model and candidates or report an explicit error. This fix removes rejection of normal driver metadata without changing the admitted binding or substituting another algorithm.

## Coverage limits and receipts

The latest correction has not compiled. Earlier passing builds do not cover this source identity. The input packet records S6/S7 PASS on earlier source, S8 PASS_AFTER_ONE_CORRECTION before this correction, S9 FAIL_AFTER_ONE_CORRECTION attributed to Android OpenSSL, S5 BLOCKED and S10 BLOCKED_TIMEOUT. Those packet statements were not independently rerun or promoted to current-source acceptance.

All 97 runtime cases / 582 rows, the main journey and Issue 426 remain **BLOCKED / NOT EXERCISED** because the runtime fixture is incomplete. Provider behavior, restart/backend parity, actual summary replacement, supported device builds and independently usable release delivery remain unverified. No green CI or resolved merge-conflict claim is made; repository policy limits GitHub Actions to deployment execution/validation. No whole-phase PASS follows from this corrective review.

## Actions and report screening

Only target/uar-diagnostic-sweep-001/reviews/whole-phase-rust-review.attempt-2.md was written; original report preserved. No production source, test, configuration or dependency changed. Nothing outside the assigned artifact was added; no guards were added.

Commands were source/artifact inspection only: cat, rg, sed and nl, plus the authorized report write. Reads showed the exact guarded arms, unchanged refusal/completion/output handling, producer metadata, and wrapper accounting described above. Some broad combined skill reads were truncated; uninspected portions are not claimed as reviewed. No test, Cargo check, clippy, fmt, build, compiler/typecheck/import probe, source-identity helper, service/provider call, acceptance script or external gate was run. The general executable review checklist remains unexecuted under the assigned artifact-only scope and CLOSED lock.

Manual report screening performed: claims are scoped to freshly inspected source, receipt statements are attributed to the input packet, and unknown model identity and missing execution evidence are explicit. Machine sycophancy detection was not invoked under the no-provider/source-only task; no machine score or automatic correction is claimed. This source-only PASS resolves the named defect, while all execution and phase acceptance remain withheld.
