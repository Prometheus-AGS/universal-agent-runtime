## 1. Binding delta classifier (owner uar-runtime)

- [ ] 1.1 Add `BindingCompat { Same, ModelOnly, Incompatible }` and its classifier in `src/uar/compiler/collaboration/bindings.rs`, reading `binding_history` by `binding_revision_key`. Equal revisions are `Same`; a missing revision is `Incompatible`; remove `revision`, `contentDigest`, `effectiveBindingReceiptRef` and the whole `modelBindings` array from both documents and compare the rest (design D1). Confirm no consumer of `modelBindings[].credentialRef`, `version` or `provenance` exists in the team path before finalising the compared set.
- [ ] 1.2 Apply the classifier at every gate that compares a binding revision on the team path and that must accept a model-only delta: G1 `resolution.rs:35-48`, G2 `team_execution/mod.rs:36`, G3 `admission.rs:117-128`, G4 `continuations.rs:44-52`, G5 `team_mailbox.rs:321`, G6 `runtime/team_execution/host.rs:137-138,192-195,227-229` (host context keeps working when only models changed; a context registered for a binding that differs by more than models keeps failing). `Incompatible` keeps the existing text and code at each site. G8 (`context.rs:194`) compares two attempt-captured values and is not changed.

## 2. Boundary capture and re-pin (owner uar-runtime)

- [ ] 2.1 Extend `capture_team_models` (`team_execution/model_capture.rs`) to return the captured models together with a `TeamModelProvenance` (design D8), comparing against the latest earlier attempt of the same member. Entries without a pin use the registry's current request; entries with a pin keep today's behavior, so a stale pin fails with `TEAM_REVISION_CONFLICT` (D5).
- [ ] 2.2 Admission (`admission.rs:117-128`, `:251`): on `ModelOnly`, re-pin `team.binding.revision` to the current revision and bump `team.revision` and `updated_at` in the same catalog compare-and-swap as the attempt; store the current revision as `attempt.binding_revision` and the provenance. On `Incompatible` return the existing refusal. A capture error returns the existing conflict code and creates no attempt (D6).
- [ ] 2.3 Wait continuation (`continuations.rs:44-52,148-156,175`): accept `ModelOnly` in the wait-validity gate, re-pin as in 2.2, store provenance with boundary `continuation`, and on an unavailable route keep the blocked wait with `TEAM_ROUTE_PROFILE_MISMATCH` plus the diagnostic fields (D6, D7).
- [ ] 2.4 Delegation (`peer_commands.rs`, task and attempt creation near `:246-282`): re-pin and capture with the same helper, store provenance with boundary `delegation`; the `expected_team_revision` check at `:185` is unchanged.
- [ ] 2.5 Dispatch claim (`settlement.rs:13-118`, `controller.rs:236,287-293`): while the attempt is `queued`, run the pre-dispatch resolution in `Boundary` mode; inside the claim's compare-and-swap re-capture models, accept `ModelOnly`, re-pin, update `attempt.binding_revision`, `effective_models` and provenance (boundary `dispatch`, replaced only when the re-capture differs). When the claim fails on an unavailable route the controller settles the queued attempt through `settle_team_attempt` with state reason `model_route_unavailable_at_boundary` and a `TeamExecutionDiagnostic` carrying the existing code, so it is never left queued. `TEAM_PENDING_LIMIT` handling is unchanged.
- [ ] 2.6 Workflow runs: read how a run records its binding revision (`workflow_execution/start.rs:71,85,200`, `mod.rs:59-60`). Apply the classifier where a re-pin by another task would otherwise invalidate an active run, and leave a workflow's own revision pin semantics unchanged.

## 3. In-flight pinning (owner uar-runtime)

- [ ] 3.1 Add `TeamRouteMode { Boundary, Pinned }` to `resolve_team_member_run` (`resolution.rs:14`). `Pinned` loads the document from `binding_history[attempt.binding_revision]`, requires the current binding to be `Same` or `ModelOnly`, takes route facts, price identity and the `modelRequirements` outcome from `attempt.effective_models` and the attempt's admitted receipt, and does not call `resolve_team_model_settings` or the live registry for model facts (R1, R2). Skill resolution stays live.
- [ ] 3.2 Use `Pinned` for `revalidate_member_binding` (`runtime/team_execution/epoch.rs:7-25`), for `execution.rs:263`, and for every fence of a `running` or later attempt (`team_execution/mod.rs:60-108`, settlement, scope, context). `capture_team_endpoint_profile` (`llm/team_profile.rs:114-140`) stays as is and keeps running once at run start.
- [ ] 3.3 Make sure an attempt that is running cannot be failed, re-routed or settled differently by a later `ModelOnly` revision, a route edit, or a route being disabled; a later `Incompatible` revision still invalidates it as today.

## 4. Provenance and failure surface (owner uar-runtime)

- [ ] 4.1 Add `TeamModelProvenance` and `TeamExecutionAttempt.model_provenance: Option<...>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` in `src/uar/domain/team_execution.rs`. Do not add fields to `EffectiveTeamModelReceipt` (`deny_unknown_fields`).
- [ ] 4.2 Define the state reason `model_route_unavailable_at_boundary` and the diagnostic fields for the boundary failure; no new public error code and no change to the allowlists at `llm/team_failure.rs:33-36` or `runtime/manager.rs:7118-7119`.
- [ ] 4.3 Confirm the existing execution summary route returns `attempts[].modelProvenance` unchanged (`TeamExecutionSummary.attempts`, `domain/team_execution.rs:102-117`; route `team-instances.execution`, `api/administration_capabilities.rs:874`). No new endpoint.

## 5. Console contract (owner uar-console; needs a single-writer assignment from the lead)

- [ ] 5.1 Add optional `modelProvenance` to `TeamExecutionAttempt` in `frontend/src/platform/entities/teams/contracts.ts` and carry it through `ingest.ts`, with no second copy of the record outside the entity graph.
- [ ] 5.2 Show per task the model that ran and, when `status` is `changed`, a "changed from X, binding revision N" note in `features/teams/ui/run/attempt-parts.tsx` (`ModelLine`); show `stateReason` and `diagnostic.code` for a boundary failure. Follow the project's UI routing and dual-critic rules in `CLAUDE.md`.
- [ ] 5.3 Keep polling through `watchTeamRun`. Hand the public team and binding topic to `uiux-realtime-topics-and-replica-refresh`; do not define a topic here.

## 6. Unchanged by design (verify by reading, no edit)

- [ ] 6.1 Leave these sites untouched and list them in the completion summary: `dispatch.rs:32,43`, `waits.rs:139,180,203`, `peer_commands.rs:47,128,185,337`, `context_evidence.rs:29,63,145`, `model_capture.rs:27`, `llm/registry.rs:177,183,190,197`, `llm/team_profile.rs:37,74`, `api/providers.rs:285`, `native_skills/team_tools.rs:231,266`.

## 7. Test fixture prerequisites (shared roots; need a single-writer assignment from the lead)

- [ ] 7.1 Extend the `stub-llm` CLI fixture schema in `src/bin/stub-llm.rs` (and `tests/integration/live/stub_llm.rs` where the matcher lives) with `model: "*"`, `last_user_message_prefix` matched together with `has_tools` and `has_tool_result`, and an optional `delay_ms` on `tool_call`, so `fixtures/stub-a.fixtures.json`, `fixtures/stub-a.hold.fixtures.json` and `fixtures/stub-b.fixtures.json` load. Remove the `#[expect(dead_code)]` at `src/bin/stub-llm.rs:19-22` if it becomes unfulfilled (clean-compile rule). Not owned by `uar-runtime`.
- [ ] 7.2 Author the team package and bindings that realise `bindingRevisions` in `fixtures/team-adoption-oracle.json` (revisions 1, 2, 2-invalid, 2-absent; alias `team-main`; two tasks for one member). The expected values stay in the oracle file; the runner must not derive them from the authored package.
- [ ] 7.3 Optional, for SCN-TEAM-006 only: a stub fixture template that fills `teamRevision` and member ids into `team_delegate` and `team_wait` arguments. If not built, SCN-TEAM-006 is NOT_EXERCISED.

## 8. Boundary acceptance (runs only at the phase boundary)

These tasks describe what the final integration gate exercises. They must not be executed early. Each uses the real server, real catalog store, real provider registry and two `stub-llm` processes (no paid model calls), reads expected values only from `fixtures/team-adoption-oracle.json`, and uses no credentials. A mandatory scenario whose negative control did not run is BLOCKED.

- [ ] 8.1 SCN-TEAM-001 (5 samples, all must pass): install model-only revision 2 while a task is held in flight, admit and dispatch the next task, plus the route-edit and queued cases; assert revision, route, pin, provenance, output and per-stub request counts. Negative control: `2-invalid` is refused and nothing changes.
- [ ] 8.2 SCN-TEAM-002 (5 samples): apply the route edit and install revision 2 while a tool-call response is held; assert the attempt settles on A. Negative control: `2-invalid` installed while held.
- [ ] 8.3 SCN-TEAM-003 (5 samples): disable B; assert admission refusal and, for a queued attempt, the failed settlement with `model_route_unavailable_at_boundary`, with both stub logs empty and A enabled as bait. Negative control: `2-absent`.
- [ ] 8.4 SCN-TEAM-004 (3 samples): read the execution summary before and after a server restart and open the console run view; assert provenance and rendered text. Negative control: another workspace id returns 404 with no provenance.
- [ ] 8.5 SCN-TEAM-005: send a stale dispatch revision, a stale roster cursor and a stale provider settings edit after an adoption; assert `TEAM_REVISION_CONFLICT` for each and unchanged state. Negative control: an unknown attempt id returns 404.
- [ ] 8.6 SCN-TEAM-006 (5 samples; mandatory only after D7 is confirmed and task 7.3 exists): delegate, yield, rebind, wake; assert one continuation on B. Negative control: `2-invalid` invalidates the wait.
- [ ] 8.7 SCN-TEAM-007 (mandatory only after D5 is confirmed): pin a settings revision, edit the route, admit; assert `TEAM_REVISION_CONFLICT` and no attempt; install a model-only revision with the current revision and assert the attempt. Negative control: bogus profile pin.
- [ ] 8.8 SCN-TEAM-008 (5 samples; mandatory only after D10 is confirmed): disable A during the hold and assert the attempt succeeds on A. Negative control: cancel during the hold stops it.
