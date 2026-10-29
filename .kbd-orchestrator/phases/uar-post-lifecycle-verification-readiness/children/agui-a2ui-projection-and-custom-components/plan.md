# Plan — agui-a2ui-projection-and-custom-components (revision 4, for operator review)

Phase: Plan. Plan only; no implementation, builds or tests are authorized.
OpenSpec change: `openspec/changes/agui-a2ui-projection-and-custom-components/`
(proposal, design, three spec deltas, tasks). Assessment: `assessment.md`.

## Operator decisions

Accepted 2026-10-27 ("Accept D-1 to D-5 as recommended"):

| # | Question | Decision |
|---|---|---|
| D-1 | Component kinds | Server-expanded templates + host components in extension catalogs |
| D-2 | Who may register | Templates via REST under settings-mutation authority; host components by host seed files only |
| D-3 | Catalog naming | `urn:uar:a2ui:catalog:1+<name>` |
| D-4 | Catalog scope | New catalog IDs beside I19 |
| D-5 | Authority refusal in tests | Verified by source review only (BDD may not send API keys) |

New in revision 4, **awaiting operator decision** (recommendation first):

| # | Question | Recommendation | If different |
|---|---|---|---|
| D-6 | The catalog requires one ID per independently run scenario; revision 3 bundled ~11 rejection cases into I23 | Split by user goal into I23 (invalid definitions refused), I24 (disallowed components in surfaces refused), I25 (isolation); denominator 102 → 107 instead of 105 | Keep one I23 and accept that it bundles independent cases |
| D-7 | Other-owner action refusal needs a second owner identity, which would put token minting into a test | Verify by source review only, like D-5 | Provide an externally provisioned second-owner path |

## Review history

- **Revision 1 → 2:** 3 critical, 9 warnings; adopted server-side template expansion.
- **Revision 2 → 3:** 3 critical, 9 warnings; generic render-tool schema plus `a2ui_catalog`, wider props subset, per-session catalog index.
- **Revision 3 → 4:** 1 critical, 8 warnings, suggestions (full text below). Revision 4 addresses each:

| Finding | Revision 4 |
|---|---|
| 1 Deltas cannot apply (pre-existing `replace` on a missing key) | `realtime.rs` emits `add` for components and dataModel (D1); `run-trace-projection.ts` accepts both; dedicated snapshot built from the thread's replayed runs (D2); new spec requirement for ordered patch application on every route; I19 checks create-then-update and repeated updates |
| 2 Action echo contradicts REST and races the terminal | Echo synthesized right after `RUN_STARTED` from the binding (task 2.5); REST keeps its source-run patch, only the action-name check changes (task 2.2) |
| 3 Host checks run after reservation; sealing rationale | `continue_run` split into prepare and execute; prepare runs before reservation; execution via `execute_request_with_run_id` cannot fail; existing unknown-outcome reservation semantics kept, no new sealing (D3) |
| 4 Saved surface templates have no owner or session | Validated against their own batch's `createSurface` catalog, else baseline (D7, task 3.10) |
| 5 `surface_test_trigger` publishes unvalidated payloads | Routed through `submit_messages` validation (task 3.11); covered by I25 |
| 6 `a2ui_output.rs` also serves `presentation_render` | Mode per tool: `presentation_render` stays `Baseline` (task 3.9); covered by I25 |
| 7 Parity scenario impossible | Compares event field shape, not ids or content |
| 8 Fixtures: false C52 claim, token minting, permanent `+bdd` | Other-owner refusal by source review (D-7); seed files authoritative so `+bdd` is removed by deleting its file and restarting (D6, task 6.3) |
| 9 KBD state split across two directories | Task 0.4 reconciles through the KBD runtime before Execute |
| Suggestions | Citations corrected; governance permit task dropped (`tool-approval.cedar:32-37` permits all); chat block and preview files named; `require_admin_key_for` extraction with security values wired into `A2uiApiState`; seed-time collisions skipped and logged; extension catalogs refused without owner or session; I23 split (D-6) and coverage added for host upgrade, entities listing and existing-template collision |

Revision 4 has not yet been reviewed.

## Outcome

1. A2UI state patches apply in order on the server and in the browser, and
   every route's snapshot reflects the surfaces its deltas build on.
2. AG-UI clients on `/api/uar/ag-ui/agents/{id}` receive A2UI surfaces,
   input requests (display-only) and state from a thread snapshot, and send
   actions as same-thread runs that execute exactly one continuation (I19).
3. Operators define template components that expand to built-ins on the
   server; hosts declare versioned host components in authoritative seed
   files; the server validates every admission path; clients render or refuse.

## Work packages and file ownership (one writer per file)

| WP | Tasks | Owner role | Files (sole writer) |
|---|---|---|---|
| A — AG-UI route | 1.3-1.7, 2.3-2.7 | uar-runtime | `src/uar/api/adapters.rs`, `src/uar/api/agui.rs`, `src/uar/api/sse.rs`, `src/uar/api/routes.rs`, `tests/bauar_full_harness_cursor.rs`, `tests/bauar_receipt_projection.rs`, `tests/context_history_integrity.rs`, `tests/truthful_tool_outcomes.rs`, `tests/stream_terminal_contract.rs`, `docs/protocols/ag-ui-profile.md` |
| C — A2UI server | 1.1, 2.1, 2.2, 3.1-3.12 | uar-state; uar-trust-tools reviews 3.2, 3.4, 3.5, 3.7, 3.11 | `src/uar/a2ui/realtime.rs`, `src/uar/a2ui/routes.rs`, `src/uar/a2ui/mod.rs`, new `src/uar/a2ui/actions.rs`, new `src/uar/a2ui/component_registry.rs`, `src/uar/a2ui/protocol.rs`, `src/uar/a2ui/presentations.rs`, `src/uar/a2ui/design_systems/*`, `src/uar/api/settings.rs`, `src/uar/runtime/native_skills/a2ui_render.rs`, `src/uar/runtime/native_skills/mod.rs`, new `src/uar/runtime/native_skills/a2ui_catalog.rs`, `src/uar/runtime/a2ui_output.rs`, `docs/protocols/a2ui-profile.md` |
| L — Wiring and seed | 0.4, 4.1-4.2 | lead | `src/server.rs`, `src/config.rs` (new `a2ui.host_component_seed_dir` key), the `+entities` seed file, KBD child state |
| D — Clients | 1.2, 5.1-5.5 | uar-console | `frontend/src/features/chat/model/run-trace-projection.ts`, `frontend/src/platform/a2ui/protocol.ts`, `frontend/packages/a2ui-uar/**`, `frontend/packages/a2ui-lit/**`, `frontend/packages/a2ui-svelte/**`, `frontend/src/features/chat/components/a2ui-artifact-block.tsx`, `frontend/src/features/presentations/ui/presentation-preview.tsx` |
| E — Scenarios | 0.1b, 0.2, 6.1-6.7 | lead | `scripts/behaviors/**`, `scripts/manual-host/**` (`+bdd` seed fixture), catalog rows |

Not changed: `runtime/manager.rs`, `runtime/native_skill.rs`, `graph/turn.rs`,
`publish_tool_output`'s signature, governance policies.

Order: 0.4 KBD reconciliation first → C's `realtime.rs` fix and `actions.rs`
API and A's `RunApiState` field (interfaces) → A and C in parallel on disjoint
files → L wiring once both interfaces exist → D once C's catalog endpoints are
fixed (1.2 can start immediately) → E alongside D.

UI rules for WP D: the repo's UI routing, then the dual Impeccable critique and
a fresh-context adversarial review before accepting the renderer direction.

## Verification boundary

Nothing runs until every implementation task is complete and the operator
approves the exact commands. Proposed gate, serial:

1. `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`
2. `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo test --locked --no-default-features --features server-full --no-run` (compiles test targets only; runs nothing)
3. Owned restart on the new binary (`restartOwnedUar` with its sha256), with `a2ui.host_component_seed_dir` set in the owned config
4. `pnpm build`; the Lit package `build` (`tsc`); the Svelte package `build` (`svelte-check`); copy `static/` into the owned UI directory
5. `openspec validate agui-a2ui-projection-and-custom-components`
6. I19, I21, I22, I23, I24, I25 once each (browser scenarios with `tier3.allow` opened per run)
7. Regression: C14, C15, C53

No unit-test run, per the operator's BDD-only rule.

## Risks the operator should weigh

- **Uncomfortable:** host components still need code in every client.
  Third-party AG-UI clients and Lit/Svelte refuse surfaces with host
  components they don't implement.
- The patch fix changes a wire op (`replace` → `add`) on every A2UI route.
  External clients that key on `replace` miss updates until they accept both.
- Seed files are authoritative: removing a seed entry removes the definition
  even if a saved surface template uses it; those surfaces then fail closed.
- With the auth flag off (the owned test instance), any loopback caller,
  including an agent with `terminal_exec`, can write templates.
- The render tool's compiled schema is more permissive; strict checks live in
  `execute_with_context`.
- Surface replay, catalog index and thread snapshots are in memory.
- The stricter action check can refuse actions today's string match accepts
  by accident; C14, C15 and C53 reruns cover the shipped flows.

## Estimate (active agent time, excluding operator waits)

A ~2 days, C ~3.5 days, L ~0.5 day, D ~1.5 days, E ~1.5 days, gate and fixes
~1 day: roughly 10 agent-days, 6-7 elapsed with parallel packages. An
estimate, not a measurement.

## Revision 3 independent review — 2026-10-27 (fresh-context critic, source only)

Verdict: defects found. Revision 3 is **not ready for Execute**. Findings to fold into revision 4:

**Critical**
- 1 State deltas cannot apply to the baseline, on either route. `createSurface` becomes `add /a2ui/surfaces/<id>` with `{surfaceId,catalogId}` and no `components` key (`realtime.rs:76-80`, payload from `protocol.rs:314-323`); `updateComponents` becomes `replace …/components` (`realtime.rs:81-85`); the applier refuses `replace` on a missing key (`sse.rs:79`), and the chat replay already stops synchronizing at that point (`sse.rs:142-145`). Verified by the lead against source. A fixed per-run snapshot would also drop a source run's surface on a continuation. Fix: give `realtime.rs` an owner and emit `add` for components and dataModel; build the dedicated snapshot from the thread's replayed state; rerun C14/C15. **This is a pre-existing product defect in the chat replay path, independent of this plan.**

**Warnings**
- 2 Action echo contradicts and races: tasks 2.1 vs 2.3 (REST publishes to the source run before continuing, `routes.rs:867-884`; no wire run id there); the continuation may terminate before the echo (`manager.rs:3308-3341`, `agui.rs:689`). Fix: synthesize the echo frame after `RUN_STARTED` from the binding; leave REST unchanged.
- 3 Host-field checks live inside `continue_run` (`routes.rs:225-249`), after reservation, so "validate host fields before reserving" is not achievable as tasked; `agui.rs:369-373` keeps reservations on unknown outcomes; sealing is safe only because `execute_request_with_run_id` returns `String`. Fix: split `continue_run` into prepare/execute and state why sealing is safe.
- 4 Saved surface template handlers have no `UserContext`, run or session (`routes.rs:1026-1109,1171-1241`). Fix: catalog from the batch's own `createSurface`, or keep the library Baseline.
- 5 `surface_test_trigger` (`routes.rs:1296-1349`, mounted at `server.rs:2435`) publishes unvalidated payloads into the replay the action check reads. Fix: validate or remove.
- 6 `a2ui_output.rs:101,137` also serves `presentation_render`, so `AllowExtensions` there breaks presentation Baseline. Fix: mode per tool.
- 7 Chat/dedicated parity scenario cannot pass (fresh UUID ids `a2ui_output.rs:192`, `routes.rs:758`; model content differs). Fix: compare field shape.
- 8 Fixtures: no script uses `PROXY_JWT_SUB` (the C52 claim is false); a second token-minting proxy with a chosen subject conflicts with the 2026-10-26 no-auth-in-tests rule; the `+bdd` seed can never be removed and would appear to every agent.
- 9 KBD state: child `progress.json` shows `implementation_total: 0`, `scope.json` allows writes only to the phase directory, and plan and progress live in two directories; must be reconciled before any gate.

**Suggestions:** citation drift (`adapters.rs:217,228`; `routes.rs:812-823`, `:867-882`; `entity-card-api.ts:36`; `agui.rs:250-252`); task 4.3 permit is unnecessary (`policies/tool-approval.cedar:32-37` already permits all tools); unnamed chat A2UI block and preview files; `require_admin_key` takes `SettingsApiState`; seed-time name collision with templates; refuse extension catalogs when owner or session is absent; I23 bundles ~11 cases into one ID (catalog line 16 requires splitting).

**Confirmed sound:** MODIFIED header; props subset fits all 7 entity schemas; `a2ui_render` reached via `execute_with_context` (`orchestrator.rs:741-744`, `native_skill.rs:320`); `execute_request_with_run_id` exists; `RunApiState` literals exactly the 5 test files plus `server.rs`; task/owner consistency; catalog ids. Revision-2 fixes: C1, C2, W2, W7 confirmed; C3, W1, W3, W4, W5, W6, W8, W9 partial.

Status: stopped before implementation at the operator's instruction.

## Revision 4 independent review — 2026-10-27 (fresh-context critic, source only; traced real patch sequences)

Verdict: defects found. Revision 4 is **not ready for Execute**.

**Critical** (both source-verified by the lead)
- C1 Replay is evicted 600 s after a run ends, not only on restart: `self.a2ui_backbone.remove(run_id)` (`runtime/manager.rs:1610`, retention sweep `:1548-1610`), default `retention_after_terminal_secs: 600` (`config.rs:344`), covered by `tests/run_retention/runs.rs:31-46`. The thread snapshot, action declaration and the continuation's view of source surfaces all vanish ~10 minutes after the source run; a later run's `STATE_SNAPSHOT` would wipe surfaces the client shows. Every BDD scenario finishes inside that window, so the gate would pass.
- C2 Projecting every `StatePatch` leaks native identity: `replace /run` carries the native `run_id` and `conversation_id` (`manager.rs:7858-7868`), and `add /approval` carries an `ApprovalRecord` with `owner_key` and `root_run_id`. Fix: project only `/a2ui/` operations and the echo.

**Warnings**
- W1 `replace`→`add` breaks C14's own steps (`scripts/behaviors/steps/a2ui-response.steps.mjs:159`, verified) and contradicts unarchived change `a2ui-realtime-backbone-from-flint-realtime-fabric` (`specs/a2ui-live-update/spec.md:16-19`, MUST be `replace`).
- W2 The snapshot stores the last message payload, not reduced state (`protocol.rs:318-321`); the browser merges components by id and data by path (`a2ui-surface-processor.ts:84-100`), so two data-model updates on different paths lose the first.
- W3 "Every route's deltas apply to its snapshot" conflicts with the chat non-goal: a chat continuation has no `createSurface` in its history (`sse.rs:68-70`).
- W4 Thread snapshot gaps: warm resume reuses the native id (`agui.rs:325`) so a replay applies twice; `stream()` also serves reconnects (`agui.rs:186-189,443-449`), only `RUN_STARTED` is gated on `after` (`:674`); the echo path is not JSON-pointer escaped (`valid_id` allows `/` and `~`, `:145`); the echo is in no replay, so the next snapshot drops it.
- W5 Executing via `prepare_interaction_request` + `execute_request_with_run_id` skips `admit_run` service placement and deployment-binding checks (`routes.rs:538-560`; source artifact snapshot reused, `manager.rs:3148-3152`).
- W6 The surface test trigger takes a bare payload (`routes.rs:177-183,1325`); validating it changes its contract and breaks `tests/sidecar_session_principal/isolation.rs:363-367` and `tests/run_retention/runs.rs:36-40`, neither owned; the gate only compiles tests.
- W7 `AppConfig` has no `a2ui` section and no `deny_unknown_fields` (`config.rs:243-244`); an unset or mistyped seed dir either seeds nothing or, under seed authority, removes every host definition.
- W8 The action-name check must recurse into nested action props (`entity-extensions.tsx:11-17`, `entity-card-api.ts:64-72`) and treat `deleteSurface` (`routes.rs:839-841`).
- W9 I19 still bundles ~12 independent cases while D-6 splits I23 under the same catalog rule.
- W10 `proposal.md` still lists a governance permit that D8 removed.

**Suggestions:** design-system import (`design_systems/import.rs:75-90`) and the embedder can reach definitions by slug; a file-write tool could plant a seed file effective at next restart; citation fixes (`a2ui_output.rs:16` is a provenance guard; `realtime.rs` assertions at `:181-201`; "task 2.x" → 2.1).

**Confirmed sound:** MODIFIED header; `RunApiState` literal sites; `execute_request_with_run_id`; owner/session from the resolved turn (`orchestrator.rs:1493-1515`); denominator arithmetic (107); within one run, create → update → update → data → delete now applies on both appliers. Revision-3 fixes: 4, 6, 7 confirmed; 1, 2, 3, 5, 8, 9 partial.

**Lead assessment after four reviews:** each revision closes the prior findings but the next review finds new critical defects, and C1 shows why. Parts (a) and (b) rest on per-run, in-memory A2UI replay that is evicted after 10 minutes. Revision 5 should not patch around that. Recommended re-scope, for the operator to decide:
1. A small standalone change: correct A2UI patch semantics (reduced state per surface, `add` ops, C14 step and the conflicting unarchived spec updated), with the chat replay fixed.
2. A durable, thread-scoped A2UI surface state store (persisted per owner/session, not tied to run retention) as the foundation for the dedicated route's snapshot, action declaration and catalog index.
3. Then this change, rebuilt on 1 and 2, with I19 split per the catalog rule.

Status: stopped before implementation at the operator's instruction.

## Re-scope accepted — 2026-10-27

Operator decision: "Yes: re-scope into (1) A2UI patch-semantics fix, (2)
durable thread-scoped surface state store, (3) this change rebuilt on them.
Plan change 1 first, have a fresh critic review it, and stop before
implementation."

- Change 1: OpenSpec `a2ui-surface-state-patches` (proposal, design rev 1,
  spec `a2ui-surface-state`, tasks). Server-side surface reduction mirroring
  the client reducer; `add` of the reduced subtree; replay log, run trace and
  test trigger unchanged; `actions:{}` snapshot baseline; C14 extended.
- Change 2: durable thread-scoped surface store. Not yet planned.
- Change 3: this child phase's change (`agui-a2ui-projection-and-custom-components`),
  to be rebuilt on changes 1 and 2. Revision 4 above is superseded as an
  implementation basis; it stays as review history.
- KBD registration of changes 1 and 2 is pending, together with task 0.4's
  reconciliation of the two child directories. Nothing is implemented.

## Change 1 revision 1 review — 2026-10-27

Fresh critic: not ready. 1 critical and 10 warnings, recorded in
`openspec/changes/a2ui-surface-state-patches/design.md`, section "Revision 1
independent review". The main problems:

- the C14 extension violates the `root` rule;
- the reducer mirrors the run-trace console reducer instead of the renderer
  (`@a2ui/web_core` `MessageProcessor`);
- the stream reads hang, and the cursor check can pass vacuously;
- re-sending the whole subtree grows without a cap;
- the snapshot change belongs as a MODIFIED delta of `ag-ui-chat-conformance`.

Stopped before implementation.

## Change 1 revision 2 review — 2026-10-27

Fresh critic: not ready, with 1 critical and 12 warnings. The findings are
recorded in `openspec/changes/a2ui-surface-state-patches/design.md`, section
"Revision 2 independent review". The critic confirmed the D2 reducer rules
against `web_core` and the C14 route mechanics (steps 1-6). The open problems
are:

- the trigger body field name;
- the spec over-claims equality with a full `MessageProcessor`;
- a requirement collision with the unstarted `agui-runs-stream-fidelity`;
- the superseded change has not been withdrawn;
- array patch growth;
- continuations are refused under per-run scope.

Stopped before implementation.

## Change 1 revision 3 written — 2026-10-27

Operator instruction: write revision 3 to fix the trigger body, scope the spec
to web_core's state operations only, coordinate the shared requirement with
agui-runs-stream-fidelity, withdraw the overlapping requirement from the
superseded change, use per-index array patches, name the continuation limit,
and add a Node step comparing the reducer with web_core DataModel. The
operator's message was cut off; its ending was taken from the offered button.

Revision 3 includes:

- a per-message mirror of the four state operations only;
- per-index array patches;
- a JSON-equality no-op rule;
- `ForbiddenId` hardening;
- `pub` types;
- an ADDED conformance requirement instead of a MODIFIED one;
- a MODIFIED `runtime-console` replay requirement;
- limit L8 for continuations;
- a C14 extension with a 13-case data-model sequence compared with web_core.

It also edited two other changes: the superseded change's patch requirement
and tasks are withdrawn, and a coordination note was added to
agui-runs-stream-fidelity. A fresh critic review is pending. Nothing is
implemented.

## Change 1 revision 3 review — 2026-10-27

Fresh critic: not ready. 1 critical and 6 warnings, recorded in the design
under "Revision 3 independent review".

- **Critical:** D2 still differs from `DataModel.set`. It misses the
  null-through-hole and non-numeric-null no-ops, vivification past 10000, and
  the reference's partial mutation before a throw.
- **Main warning:** the hand-picked `web_core` comparison cannot find
  deviations it does not anticipate.

Stopped before implementation.

## Change 1 revision 4 written — 2026-10-27

Operator instruction: write revision 4, which must:

- fix D2 per the review (an absent test via `get`, the vivification limit,
  partial mutation);
- replace the hand-picked check with a seeded generated-input comparison
  against web_core `DataModel`;
- fix W1-W6;
- go to a fresh critic, then stop.

Revision 4 contains:

- **D2.** It is now a statement-by-statement port of `set` (S1-S9) and `get`.
  Mutation before a throw is kept, and the result type is now `StateOutcome`.
- **D4.** It adds whole-array patches for gap writes.
- **D9.** It collects all frames first, pairs bodies by `artifactId`, and uses
  a seeded 180-message corpus with 18 fixed edge cases compared per body with
  web_core. It adds an explicit continuation check, a forbidden-id message,
  and a version read from the resolved `package.json`.
- **D11.** The console skips action log entries, which resolves Q1.
- **The superseded change** gets a banner and its task cleanup.

A fresh critic review is pending. Nothing is implemented.

## Change 1 revision 4 review — 2026-10-27

Fresh critic: not ready. Two critical findings and seven warnings.

- **The port is now faithful to `web_core`.** The critic confirmed it
  statement by statement, and all 18 fixed cases trace correctly.
- **Faithfulness creates trust-boundary defects.** Unbounded path depth can
  abort the process with a stack overflow under the shared backbone mutex.
  Gap fill and whole-array resends amplify size without bound.
- **Both critical findings need named departures from `web_core`.** These
  are server-side depth and size caps.

Recorded in the design under "Revision 4 independent review". Stopped before
implementation.
