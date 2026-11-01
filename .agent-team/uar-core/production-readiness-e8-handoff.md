# E8 fallback output-order repair handoff

Phase: `production-readiness-476-capability-dispatch`, Execute. Verification gate **CLOSED**. This source handoff grants no permission to run any check or review. All behavior, compiler, warning, formatting, test, production-path and packaging outcomes remain **UNVERIFIED**.

## Owned source and delivered implementation

- `src/llm/orchestrator.rs`: `Orchestrator::fit_fallback_request` now clones the source request and calls the existing `admit_prepared_user_attempt` in the known-window, estimated fitting lane, before computing the output reserve. The existing admission supplies `AdmittedDestination::restrict_output`; no catalog cap arithmetic, routing, credential resolution, or public policy was introduced. Both fit-success returns carry the narrowed cloned request. The existing fresh `admit_prepared_user_attempt` call after `prepare_user_attempt` and immediately before fallback stream opening remains intact.
- The existing exact-destination preparation bypass and unknown-window return precede the new admission. A fallback with its own exact contract must install that contract before final admission; its primary's larger context contract must not become an early capacity requirement for the fallback. Canonical history, exact settings/schema and driver-owned final-wire checks remain on their existing paths.
- `src/llm/orchestrator.rs`: authored `llm::orchestrator::tests::fallback_stream_reserves_destination_output_before_fitting_history`. This is supplemental synthetic-stream evidence, **not** installed-provider or customer acceptance evidence. It drives actual `chat_with_history` fallback orchestration with the real `ProviderRegistry`, `DestinationAdmission`, captured failing primary, captured fallback driver, final preparation and emitted attempt manifest.
- `.agent-team/uar-core/production-readiness-e8-handoff.md`: this handoff only. No other files were changed by this teammate.

## Authored regression cases (not executed)

The single module test uses six cases: primary cap16,384 to fallback max2,048/window8,192 with short history; cap4,096 with history between the old4,096 and corrected6,144 input allowances; caller cap1,024 below the fallback maximum; no caller cap; legacy no-admission cap4,096; and exact primary32,768/fallback8,192 contracts with caller cap16,384.

Assertions cover one captured primary failure and one fallback dispatch, final output cap, no optional-history displacement, intact pinned policy and prior assistant/tool-result group, cache preservation, estimated manifest reserve and allowance matching dispatch, no fit guarantee, and no manufactured exact budget contract. The exact case carries the primary contract in the attempt source, installs each destination's own preparation, retains canonical history and destination temperature/JSON schema, compares the fallback contract, and requires absence of estimated `context_fit`.

A `#[expect(clippy::too_many_lines, reason = "...")]` annotation names the intentional cohesive table-driven fixture. No warning has been observed or suppressed by a gate; compiler/lint status remains UNVERIFIED.

## Guards and scope

The only added production admission call traces to corrective finding E8 in `.agent-team/uar-core/reviews/production-readiness-context-repair/code-corrective-011.md`: the old16,384 reserve refused a viable2,048-output fallback before narrowing, while smaller differences needlessly displaced history. Existing immutable destination scope, live capability admission, finite-image refusal and local-refusal provider-health semantics remain in their existing implementations. The new call runs inside the existing local-fit-refusal branch and therefore does not record a provider failure.

No unrelated production behavior, configuration, service, dependency pin, credential, fixture file or public policy was changed. No unresolved implementation decisions or cross-file needs remain in this bounded assignment. The estimate is still an estimate; a synthetic driver cannot establish installed-provider final-wire behavior or whole-phase acceptance.

## Source state and command classification

Every shell call used `/Users/gqadonis/.claude/worktrees/production-readiness-476` explicitly. Source identity known from the authoritative waypoint is `sourceBase: 629f5702`; this handoff describes uncommitted owned edits in the shared checkout after the resolved rc.2 integration. The lead owns renewed exact-source capture and whole-phase inventory; this teammate ran no source-identity command.

Commands were only source inspection (`pwd`, `cat`, `sed`, `rg`, `tail`, `head`) or authorized implementation/document writing through Python standard-library text replacement. Some inspection searches returned missing-path/no-match statuses; no executable verification was invoked. No Cargo, tests, fmt/lint/type checks, compiler/import probes, builds, API/container calls, independent/source review gate, `git diff --check`, commit, hook or external mutation was run or delegated. Zero acceptance credit is claimed.

The lead must reconcile the entire phase ledger and writer handoffs, present a new inventory and exact proposed commands, and wait for operator instruction. This regression name may be included in an approved gate's coverage inventory; it is not authorization for a standalone filtered module test.
