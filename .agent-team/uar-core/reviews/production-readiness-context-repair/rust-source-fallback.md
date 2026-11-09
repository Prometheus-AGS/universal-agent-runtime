# Rust source-only fallback assessment

Scope: one initial pass over `target/production-readiness-476/context-repair-review/artifact.patch`, its adjacent `requirements.txt`, the constrained-model-dispatch spec delta, and necessary baseline production/test/vendor source. Review phase: isolated source assessment. The verification gate remained **CLOSED**.

This is the operator-authorized weaker native fallback, not specialist-role approval. The reviewer received requirements and artifacts without producer rationale, handoffs, progress, KBD inventories, memory, or other review reports. The native producer/judge same-family limitation remains; fresh context does not establish independent-family judgment. No subagents were dispatched.

## Concrete source findings

### P1 — New negative-path assertion requires an unavailable `Debug` implementation

Anchor: `src/uar/runtime/cost_budget.rs:2032-2035`, in `durable_contract_without_final_wire_evidence_refuses_before_inner_stream`.

The new `.expect_err(...)` is applied to `Result<ExternalDriverStream, anyhow::Error>`. `ExternalDriverStream` is `Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>` (`src/llm/external_driver.rs:16`); the trait object does not include `Debug`. `Result::expect_err` requires the successful type to implement `Debug`, even when this fixture intends to return an error. Consequently, this test source cannot satisfy that method's bound, preventing the test target from compiling before its zero-dispatch assertion runs. This conclusion follows from the source types and Rust's method bound; no compiler was invoked and no compiler diagnostic is claimed.

Correction: extract the error with an explicit `match`, panicking with a fixed message on `Ok(_)`, or use an existing repository pattern that does not require formatting the successful stream. Do not add `Debug` to the public stream contract to accommodate this assertion.

Falsifying observation: the receiver would need to have a different successful type that implements `Debug`, or the stream alias would need to include such an implementation. The inspected alias and `LlmDriver::stream` signature (`src/llm/mod.rs:481-484`) show neither.

### P2 — Uncontracted profiled sends lost final profile equality validation

Anchor: `src/llm/liter_driver.rs:539-540`, with the replaced dispatch validation at `:910-914`.

`validate_dispatched_budget` now returns immediately when `budget_contract` is absent. However, `build_chat_request` still explicitly supports an endpoint profile with no budget contract when the profile has no output ceiling, and computes its expected profile body (`:749-758`). The patch removes the previous comparison between that validated body and Liter's actual bytes. The new preparation call therefore counts and receipts an uncontracted profiled body without enforcing its literal model or allowed-field contract against the actual preparation.

A concrete accepted configuration is `LiterLlmDriver::new` with an explicit endpoint and an `openai/...` model, followed by `with_endpoint_profile` whose `wire_model` includes the matching `openai/` prefix and whose `output_ceiling` is `None`. Profile validation permits that nonempty literal wire model (`:179-189` and `validate_endpoint_profile`); request construction copies it verbatim into the expected body. The captured generic Liter adapter strips its hinted provider prefix (`vendor/git/liter-llm/crates/liter-llm/src/provider/mod.rs:773-778`). Previously the body mismatch refused the send; now the early return accepts it and dispatch proceeds with a different literal model than the declared profile. This is a final profile preservation regression, independent of whether a finite reservation is needed.

Correction: validate actual prepared profile controls even without a budget contract, then make counting/budget arithmetic conditional on the contract. Preserve the existing no-profile/no-contract path without adding a preparation prerequisite. If prefix normalization is intended for a declared profile, normalize under an explicit profile contract instead of dropping validation for every uncontracted profile.

Falsifying observation: an invariant rejecting all no-contract profiles or preventing Liter transformations of their literal body would remove this path. The explicit `:749-758` branch, public binding method, and inspected vendor prefix transformation demonstrate the opposite in source. Runtime reproduction remains unexecuted.

## Other inspected properties and limits

- The new trait method uses `&self`, no generic method parameters, and an owned concrete result. Source inspection found no newly introduced object-safety or borrow-lifetime defect in that method. This is not compilation acceptance.
- Durable contracted preparation is requested from `BudgetedModelDriver.inner` after host narrowing and only when both a durable budget and request contract are present (`cost_budget.rs:1729-1746`). `begin_call` adds uncertainty and reasoning with checked arithmetic and retains owner-scoped `admit_accounting_call` (`:2250` onward). The patch does not alter the existing cancellation select or usage/uncertain-settlement loop. Their concurrency, replay, and persistence behavior remains unexecuted.
- The Liter implementation obtains its count from `chat_stream_request_body`, and the vendor direct send calls the same `prepare_request` helper (`vendor/git/liter-llm/crates/liter-llm/src/client/mod.rs:1066`, `:1347`). The reservation and send are separate preparations rather than one retained byte object. The inspected ordinary constructor has `managed: None`; equivalence for arbitrary externally supplied managed clients is not established by this pass.
- The added durable preparation method rejects image parts before using serialized JSON as finite image accounting. The existing synthetic serialized-image lane remains distinct from that reservation evidence. Real image capacity is not established.
- The held-thread fixture remains a pending synthetic stream while forwarding preparation to a captured Liter binding. It does not prove a real provider send or final provider usage.
- The pending-tool fixture now supplies verified owner context and persistence and asserts the stored pending assistant call matches the retained history. The fixture helper enters existing production bearer verification before signing its synthetic token; it does not modify production admission. This is security-component fixture setup, not a functional/manual credential procedure. Process-wide JWT initialization ordering, database timing, and fixture execution remain unverified.
- Existing oversized-summary source retains zero-provider-send and original-history assertions and adds a short-envelope control. No execution, fit measurement, or timing reliability is claimed.

## Execution and ownership record

All shell operations were classified as source inspection: bounded `cat`, `sed`, `nl`, `rg`, and `head` reads/searches of the allowed artifacts, instructions/skills, and relevant source. Rust workspace/best-practices guidance and relevant ownership/error/dispatch/pointer references were consulted. No Cargo, test, lint, formatting, typecheck, compiler/import probe, build, API/container call, `git diff --check`, external gate, or other executable verification ran. No test result, warning result, or runtime result is available.

Only this report was written, through `apply_patch`. Product source, fixtures, configuration, services, and other reports were not changed. No unrequested product guards or other additions were made. The findings concern an introduced Rust assertion bound and loss of an existing captured-profile boundary check; proposed corrections were not implemented.

Source identity is the supplied patch artifact, whose affected Rust before/after blob labels include `cost_budget.rs` `a6ae1cf8..9baf15f1` and `liter_driver.rs` `5e4943fc..ffc754e2`. No new identity/check command was run. Compilation, warnings, runtime dispatch, durable atomicity/replay, cancellation, security fixture behavior, and test acceptance all remain unverified under the closed gate. This assessment supplies findings, not phase or specialist approval.
