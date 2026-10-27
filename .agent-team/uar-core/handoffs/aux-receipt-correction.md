# A15 repair14 — auxiliary budget receipt correction

Phase: `uar-post-lifecycle-verification-readiness`; Execute. Verification gate: **CLOSED**.

Owned production changes only:
- `src/uar/runtime/matching/intent/mod.rs`, `classify_skills_with_driver`, collection match immediately after the `StreamStart | Usage` arm: added one guarded `Custom` no-op arm requiring both `source == "uar.request"` and `event_name == "budget_receipt"`, plus a driver-metadata comment.
- `src/uar/runtime/context/summarizer.rs`, `summarize_span`, collection match immediately after the `StreamStart | Usage` arm: added the same exact guarded no-op and comment.

The receipt payload is never bound or appended. Other custom events still reach the existing unsupported-event branch. Existing text collection, start/usage handling, Done requirements, stream-error propagation, summary output ceilings, aggregate framing budget and atomic history replacement were left unchanged. No accounting code changed.

Source basis: `target/uar-diagnostic-sweep-001/reviews/whole-phase-rust-review.md` identified the producer/consumer incompatibility. Source inspection read the actual collector functions, `src/llm/liter_driver.rs` request receipt construction and initial `Custom` emission, `BudgetedModelDriver` usage accounting and event pass-through in `src/uar/runtime/cost_budget.rs`, captured auxiliary binding in `src/uar/runtime/turn/bindings.rs`, and classifier callers in skill service/runtime manager. The normal Liter receipt formerly reached each collector's unsupported-event branch. The new guard is limited to that observed contract defect; no speculative guards, fallbacks, retries, helper, refactor or version change were added.

Commands performed were source/artifact inspection (`cat`, `sed`, `rg`, workspace manifest read) and authorized implementation/documentation writing (`apply_patch`). No verification command ran: no tests, lint, format check, typecheck, Cargo, build, helper, source inventory, review, service, provider, external gate or git diff check. Some broad guidance reads were truncated; only inspected source spans are claimed. Rust workspace, best-practices and async guidance were loaded; their execution suggestions remain deferred under CLOSED.

Current source state: both collection arms are authored in the shared checkout. Other writers' edits were preserved. This handoff is the only additional file authored. Nothing outside the assigned outcome was added. No unresolved design decision remains for this bounded correction.

Compile and runtime behavior remain unverified. No prior passing build certifies this edit. Parent owns the permitted source inventory and one fresh Rust review; no additional compilation or second correction cohort is requested or implied here. The whole-phase implementation ledger and operator lock remain authoritative.
