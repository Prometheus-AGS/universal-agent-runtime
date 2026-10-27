# Complete r7 validation-result regression correction

KBD remains Execute in the isolated repair child, supporting Kubo Execute/FIR-9. R6's exact first command stopped on failure. All verification is CLOSED; no further command or diagnostic is authorized.

## Assessment and analysis

R6 compiled in4m31s, passed826 library tests with1 existing ignored provisioning case in55.15s, and passed all6 admission integration tests. The protocol target completed in0.40s with9 passed and2 failed. The previously overflowing readonly Ask-approval case passed and no stack overflow occurred. Both failures read `result["type"]`, expected `invalid_arguments`, and observed Null. Source drift was empty; customer build and client gate never started. Preserve the complete receipt/raw logs and all prior source/failure records unchanged.

The retained logs do not print the complete payload and do not establish that the later zero-effect/retry/history assertions ran. Source resolves the contract: `src/uar/tools/validate.rs:24` produces the structured validation payload; orchestrator.rs serializes it in both sequential and parallel validation paths, then `preserve_terminal_tool_failure` wraps it in the canonical trusted-host envelope. The outer object has status `error`, the original `tool_call_id`, tool `path_tool`, native/failed/trusted_host provenance, and a string `message` containing the validation JSON. The inner payload retains `type: invalid_arguments` and its parse/schema diagnostic.

The same envelope constructor, both validation wrapping routes and both top-level test assertions exist in baseline774469746fee4de99be41dc1cd231857067e4487. No native identity repair or production behavior change caused this shape mismatch. `tests/context_history_integrity.rs:696` already asserts the analogous outer identity/status/provenance contract. Related-family source inspection across src/tests found only these two top-level invalid_arguments expectations. The other nine protocol cases and the admission target assert different scheduling/governance/typed authority contracts. No production correction is supported.

## Entire implementation batch

The existing specialist owns exactly `tests/tool_call_protocol.rs`. Root owns this plan, proposal/design/tasks, task packet, append-only history and the final inventory. Preserve all earlier integrated source and every fixture input.

1. Add one local assertion helper, or equivalent assertions, for the native validation failure envelope. Require outer status `error`, exact original call ID, tool `path_tool`, and exact provenance object `{source: native, terminal_state: failed, observed_by: trusted_host}`. Require message to be a string and parse it as structured JSON. Require the nested payload's type to equal `invalid_arguments`.
2. Use that assertion for both malformed-JSON and schema-invalid-JSON cases. Apply their original EOF/end-of-input and string diagnostic predicates to the nested validation payload. Do not loosen those predicates, silently accept either envelope shape or substitute fabricated output.
3. Keep the model-visible and history helpers returning the full outer envelope. Preserve failed success flag, zero native executions, exactly two provider requests, authentic IDs and equality of the entire outer envelope with the next provider request's tool history. Preserve all11 protocol tests, fixtures, validators, approval and scheduling assertions.
4. No production, API/schema, catalog, dependency/vendor, provider route, runtime/thread/stack or budget change is included. Do not clean up warnings or add unrelated tests.
5. Complete all source/docs, stop the writer, integrate as source, and capture the expanded immutable manifest/patch/preimages plus typed canonical completion before requesting renewed exact verification instruction. No test/build/check/format/lint/compiler/generator/validator/probe/debugger/review/publication or Kubo cumulative gate may run beforehand.

## Verification boundary and limits

Only protocol regression source changes in this batch. After complete source integration and renewed explicit approval, propose rerunning only the failed protocol target, followed serially by the previously unstarted customer build and extended client gate. Do not repeat the passing library/admission gates merely to expand a passing gate: their exact r6 receipts remain bound to unchanged production/library/admission source through reconciliation. They are retained scoped evidence, not full r7 or whole-UAR certification. All11 protocol cases must complete under the corrected assertions.

Runtime success is unverified until the renewed command completes. The r6 absence of overflow establishes only that exact run's outcome; it does not measure stack sizes or prove universal stack safety. Cold bootstrap still lacks a finite cancellation/join guarantee, and one existing ignored provisioning case remains unverified. Durable recovery infrastructure/target, formatting/lint, independent reviews, corrected immutable artifact and Kubo pin/runtime/refreeze/cumulative acceptance remain pending. No PR13 readiness or publication is claimed.

## Complete source handoff

The specialist implemented the entire assigned test-only batch and stopped writing. One local helper requires the exact outer status/ID/tool/provenance and nested invalid_arguments payload. Both original parse/schema diagnostic predicates and all failure/zero-effect/retry/full-envelope history assertions remain. No fixture input, production source or other test changed. Root inspected the source diff as integration only. No known scoped implementation item or extra path need remains. Complete source/documentation totals31 paths; derived inventory, patch and typed canonical completion follow these final docs. No tests/builds/checks/formatting/probes or diagnostics ran after edits. All verification and broader obligations above remain held pending renewed exact instruction.
