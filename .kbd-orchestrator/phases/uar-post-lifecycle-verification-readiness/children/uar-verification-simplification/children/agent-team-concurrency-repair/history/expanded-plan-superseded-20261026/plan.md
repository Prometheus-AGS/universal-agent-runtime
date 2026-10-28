# Plan — agent-team-concurrency-repair

Project: Universal Agent Runtime. Date: 2026-10-26. Stage: Plan, prepared for operator agreement; implementation and verification remain closed. One existing OpenSpec change: `repair-agent-team-concurrent-operation`. Do not create a duplicate change.

## Outcome and finite exit

Repair the failed saved-math/saved-writer project on Mac. Independent calculation and outline must genuinely overlap, then the writer must integrate both outputs. Preserve durable skills/settings and settle the original attempt without replaying work. The three required acceptance rows are ADMIT-01 → TEAM-01 → SETTLE-01, using one existing portable Node/Cucumber entry. `exit-contract.md` states the complete exit decision; `adopted-contract.json` remains an unadopted proposal until explicit agreement.

The uncomfortable fact: neither the exact old skill error nor the old database commit outcome is established. The current source has concrete defects worth fixing, but old logs cannot certify this checkout. Also, the existing recover endpoint cannot perform the specified terminal replay: it skips terminal attempts and can dispatch queued work. Task2.2 must supply the missing production behavior before the BDD can legitimately assert it.

## Ordered change

1. `repair-agent-team-concurrent-operation`: restore durable admission, safe settlement and useful concurrent team completion.
   - Scope: native startup, runtime skills, host collaboration persistence/recovery, existing Node BDD, operational documentation. No UI, dependency upgrade, new executor, release or platform matrix.
   - Depends on: operator agreement on child scope/exit contract and inherited allowance reconciliation before Execute. No dependency on completing unrelated Boss/UI tasks is silently introduced or discharged.
   - Libraries/patterns: adapt cand-001 and cand-002; retain cand-003. See library-candidates.json and analysis.md for evidence, including pinned-SDK limits.
   - Agent: Codex lead plus uar-runtime specialist; one source writer serially handles2.1 then2.2. Independent artifact-only Rust critic after the full boundary. At most three active agents.
   - Complexity: L (4–8 active hours rough planning range, not a delivery promise); settlement evidence/fencing is the dominant uncertainty. Customer value high.

## Task order and file ownership

All seven backend IDs remain stable. The full key is this canonical phase ID + change ID + backend ID. Canonical registration does not complete a task.

| Task | Work / dependency | Sole writer / files | Completion evidence |
|---|---|---|---|
| 1.1 | Agree scope, exit, ownership, resource and fixture rules before source work | Lead: child documents, OpenSpec change, append-only .prometheus history | Agreed contract/binding, canonical task map and prerequisites; no product PASS |
| 2.1 | Align every actual configured filesystem provider with durable reconciliation; preserve API overrides/scoped selection; keep public error code with redacted private cause | Runtime specialist: src/server.rs; src/uar/runtime/skills/service.rs; src/uar/runtime/skills/registry.rs; src/uar/runtime/skills/service/selection.rs; src/uar/runtime/manager.rs | Source handoff, behavior documentation to lead; ADMIT-01 deferred |
| 2.2 | After2.1 handoff, typed catalog failure handling and no-dispatch terminal reconciliation; exact replay including usage=None; predecessor fence/evidence handling | Same runtime writer: src/uar/compiler/collaboration/storage.rs, service.rs; team_execution/{settlement,recovery,ownership,mod}.rs; src/uar/runtime/team_execution/{controller,execution}.rs; src/uar/runtime/manager.rs; src/uar/domain/team_execution.rs; src/uar/api/collaboration/team_execution.rs | Source handoff, caller/DTO/persistence compatibility inventory; SETTLE-01 deferred |
| 2.3 | Integrate both repairs; freeze operational fixture and documentation; inventory all production work | Lead: child operational documents; OpenSpec design/verification; specialist retains production files until handoff | All2.x production implemented; no unresolved production item; explicit parent-boundary disposition |
| 3.1 | Only after2.x, update existing team BDD, controls and receipts; freeze actual source/config/fixture/commands | Lead: scripts/behaviors/team.cucumber.mjs; features/math-writing-team.feature; steps/math-writing-team.steps.mjs; child final packet/inventory | Authored assets and exact operator packet; still no execution |
| 3.2 | Only after full2.x+3.1 inventory and named operator approval: independent review → customer build → owned activation → one BDD invocation | Independent critic owns findings only; lead sole build/process/test owner | Actual review/build and all three acceptance rows; stop first failure |
| 3.3 | Record actual Mac result, consumption, limitations and parent handoff | Lead: child receipts/reflection/handoff and append-only .prometheus verification history | Evidence-linked status, no broader certification; closure only under exit-contract.md |

Paths in a brace group are literal files under the displayed directory, not permission to edit neighboring modules. If a necessary additional caller/file is found, name it and update ownership before editing; a changed product requirement requires operator agreement. No concurrent writer touches manager.rs or any shared composition root. No tests/checks/builds/review-driven product verification before the complete boundary. Writers preserve others' existing edits and leave the live UAR untouched during implementation.

## Production decisions

**Admission:** enumerate actual configured filesystem providers through existing provider metadata; do not rename only the first provider or globally bypass selection. Preserve source precedence, API ownership and enabled/scope settings. Persist/load consistently before readiness. StoreUnavailable and StorageMissing remain distinct private categories; neither becomes empty successful selection. Follow existing error boundaries without logging bodies/secrets.

**Settlement:** introduce a targeted terminal-reconciliation request variant on the existing POST `/api/v1/collaboration/team-instances/{id}/recover`. This is planned new behavior, not a payload the current route accepts. Keep the current legacy TeamControlRequest behavior compatible. The new branch selects one exact attempt, authenticated owner/workspace, run/root, original fence, current expected team revision and terminal evidence identity; it never falls through to queued dispatch. The trusted host derives outcome/output/usage from authoritative records, never caller-asserted success. Reuse one settlement reducer with immutable result/evidence identity. Two fresh commands may create distinct audit receipts but must preserve the same terminal receipt, artifacts, usage revision and reservation. Identical command replay returns its recorded receipt; changed payload refuses before mutation.

Existing unknown-usage terminal repeats increment revisions: return a stable same-outcome result for usage=None as well as known usage. Do not use cold RunManager usage zeros as proof of known-zero accounting. Keep absent accounting unknown and charged. For a predecessor fence, allow settlement only under current host authority plus trusted proof of predecessor exclusion and exact original lineage; never regrant dispatch authority or weaken the global fence. Missing/contradictory proof yields uncertainty or BLOCKED, never guessed failure/success. No new generic retry loop; existing bounded CAS-miss handling remains bounded, and ambiguous writes require exact-state reconciliation, never model/tool replay.

**Concurrency:** retain existing Tokio member lanes and artifact DAG. Do not rewrite the scheduler absent new evidence. Keep independent calculation/outline and dependent article. Their task decomposition is host-authored. Automatic planning and team KB parity remain outside this claim.

## Fixture decisions and command boundary

The retained endpoint is http://127.0.0.1:53587, service identity uar-macos-behaviors, config/store recorded in target/prompt-macos-launch.json. Existing process64463/session66217 is historical ownership evidence, not current liveness proof. The old binary is not the repaired candidate. Keep saved agents math-specialist-db64c2a8/writing-specialist, all KBs, original failures and pending reservations.

ADMIT uses a uniquely owned temporary source directory/skill under the existing fixture root. First repaired startup persists it; the same journey sets an API override and scoped disable, then the one explicitly named re-registration/restart verifies preservation before team inference. No unrelated skills are edited. Use a separate absent required-skill binding as the pre-provider refusal control. This does not certify a storage outage branch.

TEAM makes exactly three primary member executions (calculation, outline, article), within the existing two-active-member limit, 32768-token/120-second per-attempt reservation and 98304-token/360-second team reservation. All actual auxiliary calls remain charged; no invisible free classifier work. Existing Liter/MiniMax-M3 binding must use captured current profile/settings revision, not assumed revision1. No embedding work is planned. Run one project, not the previously accepted prompt/stream/KB journeys.

SETTLE first uses the retained original team72860111-231c-49ec-a973-e0a4e53f2a63 / attempt285108f8-4cfe-4ff6-8b3b-3a8d9fba052a / run414193e0-50e2-4ec0-a7e2-ac1bd436b6cc. Authoritative terminal root, children, effect state and exclusion lineage must qualify it; old error logs alone cannot. Fresh TEAM output supplies the known-usage control. The original is the planned genuine terminal/unknown-usage control. If it cannot qualify, SETTLE is BLOCKED; no synthetic row or added fault campaign is authorized. Read-only evidence discovery may happen inside the approved gate, with no model replay. Both terminal controls use fresh command A, fresh command B/current revision, exact replay B, conflicting reuse B. Capture immutable terminal identity separately from changing command audit receipts.

Proposed fixed build and BDD entry, repository cwd:

```sh
CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/team.cucumber.mjs
```

No command ran. Task3.1 must deliver the literal activation/environment/cleanup packet after implementation: owned graceful stop and confirmed writer release, offline recoverable store snapshot, repaired binary `target/uar-team-concurrency-repair/debug/universal-agent-runtime --config <existing absolute config path>` with the original fixture cwd, bounded readiness, exact skill re-registration/restart, source/binary/config identity binding and retained process/log receipt. Paths/argv must be fully expanded in that actual packet; these command forms are not executable authority. No simultaneous store writers. On success keep the repaired UAR running; on failure retain data/evidence and only stop a process that is owned and requires shutdown. No automatic old-binary restart against possibly changed data and no restore-overwrite of durable mutations.

Protected inputs remain environment references: UAR_BASE_URL, UAR_MODEL, UAR_AUTH_TOKEN, UAR_WORKSPACE_ID, UAR_CUCUMBER_REPORT and the exact recovery authority selected from source. Existing owner token expiry2026-10-26T14:26:32Z must be accounted for before dispatch. No implicit renewal or secret printing. Missing/expired authority blocks the gate. Single BDD invocation, retry0; bounded20-minute gate including approved fixture lifecycle, each existing HTTP request150s; no second attempt without discussion. Native build proposed limit60 active minutes. All of these fit inside remaining cumulative allowance or block.

## Resource accounting and parent scope

Inherited ceilings:4800 host calls,3000 active minutes,28 repairs,no-progress3; separate USD10 Cohere/USD10 OpenAI. No reset and no unused repair allocation is assumed. Last retained accounting034 cutoff2026-10-26T06:14:27.911773Z had3940 calls,2254.5123721 conservative active minutes and repair floor20. New plan-consumption.json extends the same receipt lineage to2026-10-26T10:17:37.855993Z:4360 unique host calls across104 sessions,2497.6781091 conservative active minutes, no parse errors or new idle exclusions. At that cutoff440 calls and502.3218909 minutes remain; later activity accrues. Repair20 remains a floor, not a confirmed total. Task1.1 must reconcile append-only host receipts and repairs before Execute; unknown/exhausted remaining allowance blocks dispatch. No new idle exclusions. The child is one bounded repair against the original failure, charged once on implementation entry under explicit adoption; any subsequent repair/retry is discussed first. Exact remaining repair capacity must be known before that charge.

Proposed operator decision: use this finite child implementation boundary for its three acceptance rows, even while unrelated parent work is pending. Preserve the parent's fourteen obligations and all unchecked parent tasks as unfinished; this is a child-only exception to the parent-wide verification boundary, not parent acceptance or a new release contract. Parent signed authority is not overwritten. Before execution, reconcile whether an amendment is required for that exception; if so present its concrete body, never claim that this draft supplies the signature. No child test approval is inferred from Plan approval.

## Archive and clean return

Adopt/close through the installed supported KBD commands with one contract and one evidence set. The flat-path helper compatibility decision and exact closure sequence are in exit-contract.md. Child delta adds distinct requirement names; archive this accepted child first, creating/augmenting canonical durable-team-execution. Later afc-c09-team-admission-runtime archive must ADD its distinct requirements onto that capability without replacing the child requirements. Do not archive or complete that broader change now. Full OpenSpec validation and archive are post-acceptance named operations, never a covert check during Plan.

## Task model assignments

Phase key P = uar-post-lifecycle-verification-readiness / uar-verification-simplification / agent-team-concurrency-repair; canonical ID above. Change key C = repair-agent-team-concurrent-operation. Each row expands P/C/ID, with no alternate task identity.

| Phase | Change | ID | Requirements / selected native route | Evidence, handoff and prerequisites |
|---|---|---|---|---|
| P | C | 1.1 | Lead current Codex host; exact model identity unreported | Scope/authority accounting; lead owns plan; operator agreement required |
| P | C | 2.1 | uar-runtime / OpenAI gpt-5.6-terra, high, Codex role | Existing .agent-team/uar-core/team.json and exposed role; fresh bounded spawn, source tools only; handoffs/runtime-admission.md |
| P | C | 2.2 | Same uar-runtime / OpenAI gpt-5.6-terra, high, native | Serial ownership and shared manager file; same role handoff/runtime-settlement.md; availability exposed, new repair performance unverified |
| P | C | 2.3 | Lead current native host | Integrate handoffs and operational documentation; no product checks |
| P | C | 3.1 | Lead current native host | Portable BDD authoring after2.x; final inventory/packet; no runner invocation |
| P | C | 3.2 | Lead build/run; independent uar-verifier / OpenAI gpt-6-astra high plus Rust expertise | Exposed native role and existing policy; fresh artifact-only critic, no producer history; findings/review.md; dormant until exact gate approval |
| P | C | 3.3 | Lead current native host | Evidence/closure and parent handoff; no model-produced PASS without receipts |

Native tools expose the named roles, not verified external Liter worker access. Use collaboration.spawn_agent with fresh context and explicit bounded ownership; no REST worker or model catalog probe is needed. Current role configuration is policy evidence, not proof of model quality. Native alternative is the existing host only after an explicit recorded routing revision; no silent substitution. Real product inference remains Liter/MiniMax-M3, distinct from workspace workers.

## Learning, review and completion status

plan-prior-context.md recalls that old artifact evidence differs from current source and scope changes must be explicit. Apply those lessons here; unrelated UI lessons are not extra tasks. gotchas.md records the operator's correction to use a small direct behavior script and the failure of cached-receipt-only settlement assertions. Reuse the existing BDD and require actual terminal persistence.

No evolver bridge is associated with this child. No production changes or verification ran during Plan. Plan remains pending explicit contract agreement/adoption; final execution permission follows implementation and its exact packet. A document review is not a product acceptance result.
