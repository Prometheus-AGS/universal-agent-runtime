# E7 source repair handoff

Phase: `production-readiness-476-capability-dispatch`, Execute. Worktree: `/Users/gqadonis/.claude/worktrees/production-readiness-476`. Verification gate: **CLOSED_OPERATOR_LOCK**. This handoff records implementation, never acceptance.

## Delivery and source ownership

- `src/llm/liter_driver.rs`: `validate_dispatched_budget` now retains the early return only when both the budget contract and endpoint profile are absent. An optional `(contract, effective_output_tokens)` pair preserves existing output-cap narrowing. Liter-prepared bytes are parsed and compared against the captured profile's exact model, allowed fields, typed controls, and schema regardless of budget-contract presence. Only output-ceiling arithmetic, final counting, and budget-fit validation remain conditional on a contract.
- The ordinary unprofiled, uncontracted path retains its immediate `Ok(None)` and acquires no new parsing, profile, counting, or preparation prerequisite. The existing contracted synthetic path continues to use the captured adapter's literal model and the existing remaining controls/output/count guards; no model-specific production exceptions were added.
- `.agent-team/uar-core/production-readiness-e7-handoff.md`: this handoff only.

Other writers share the checkout. No other source, authoritative waypoint, plan, ledger, services, original chat checkout, or dependency files were edited. No commit was made. The lead owns final source-identity inventory and whole-phase reconciliation.

## Authored regressions (not executed)

- `llm::liter_driver::prompt_caching_tests::settings_only_profile_refuses_liter_rewritten_wire_alias`: a settings-only profile with no contract passes preflight for `custom/model-a`; the real pinned `DefaultClient::chat_stream_request_body` prepares `model-a`. The production final-wire helper and public profiled receipt must refuse that changed alias.
- `llm::liter_driver::prompt_caching_tests::settings_only_profile_preserves_prepared_alias_controls_and_schema`: the real pinned preparation retains `vendor/model-a`, named tool choice, disabled parallel calls, strict tools/input schema, and structured response schema. The final-wire helper and receipt must accept with no counted-token claim. Bounded mutations to the actual prepared body (parallel setting, response schema, and an unapproved field) must refuse through the same validator.
- Shared setup is local to the existing test module. No network calls, listener, credentials flow, new test harness, dependency, or schema fixture file was added.

The restored equality guard traces directly to E7's observed source finding: Liter's provider-prefix transformation can change a captured exact alias after the no-contract preflight. It is an existing destination/profile integrity boundary, not speculative hardening. The additions are limited to the requested repair, its regressions, and this handoff.

## Commands and limits

All commands were classified as source inspection or authorized implementation/documentation writing. Source inspection used `pwd`, `cat`, `sed`, and `rg` (with `head` for bounded displayed output), with explicit workdir on every invocation. Authorized implementation used Python text edits restricted to `src/llm/liter_driver.rs`; documentation used a shell heredoc restricted to this file.

Executed verification commands: **zero**. No tests, check, lint, formatting command, typecheck, compiler/import probe, build, packaging check, API/container call, `git diff --check`, source-identity check, review delegation, or review-driven gate was executed. No gate violation or incomplete accidental gate occurred. Inspection output is not verification evidence.

Uncomfortable limit: the authored regressions and production repair have not been compiled or exercised. Their runtime behavior and warning-free compilation remain unverified under the operator lock. Existing #388 contracted behavior and ordinary dispatch compatibility have not been rerun. No test or build artifact was produced. There are no unresolved design decisions in E7; the lead must include these source edits and exact regression names in its renewed whole-phase inventory and request exact operator approval before any verification.
