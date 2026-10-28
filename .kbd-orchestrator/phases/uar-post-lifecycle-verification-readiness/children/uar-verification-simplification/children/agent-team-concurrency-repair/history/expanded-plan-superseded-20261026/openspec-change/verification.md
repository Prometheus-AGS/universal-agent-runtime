# Proposed production integration acceptance

This is a Spec-stage requirement, not a run receipt or verification approval. Three stable acceptance rows cover one focused repair; they need not become three separate projects or a new harness. The existing portable team entry remains the command. Its required controls are authored only after production completion (task3.1).

## Shared identity and boundary

- Canonical phase: `uar-verification-simplification::agent-team-concurrency-repair`; change: `repair-agent-team-concurrent-operation`.
- Source: this repo's future repaired candidate tree, NOT_EXERCISED until task2.3/3.1 inventory; old binary `25151ae66312e41c99060971645b2414b28817669a164f064cd59d3b0576828c` is historical and cannot certify new code.
- Real collaborators: native UAR, its actual durable Surreal store, configured filesystem sources, verified owner/workspace, Liter/MiniMax-M3 and the two saved specialist definitions. No synthetic positive inference.
- Production surfaces: configured startup; `/api/uar/skills`; existing collaboration package/binding/team/task/admission/execution/artifact/recovery endpoints; native run inspection/events/accounting. No test-only public API is authorized.
- Source, operational fixture, configuration and documentation must be complete across the applicable full-phase boundary. If the parent boundary still contains incomplete implementation, its disposition/exception must be explicit; child completion does not silently override AGENTS.md.
- The current live instance/store, math/writer agents, KBs and failed receipts are preserved. Fault/deletion controls must not corrupt the live store. No separate generic admission run or earlier prompt/KB test is added.

## ADMIT-01 — configured-source admission

Owner: tasks2.1+3.1. Functional scope with fail-closed control.

Given genuine configured filesystem sources and persisted selection settings, startup reconciles actual source identities; the same team project then admits both saved specialists without the source-mismatch storage error. Read-only owner diagnostics identify a missing durable record versus unavailable/undecodable storage if either fails; no skill bodies/secrets/foreign state appear. A known API override and scoped disable survive re-registration.

Negative control: submit a separate owned preflight/binding request containing an absent/mismatched exact required skill reference; it must refuse before provider dispatch. This proves resource refusal, not simulated database-failure recovery. Store-read failure category coverage remains NOT_EXERCISED unless an actual categorized failure or an explicitly approved genuine boundary fault is captured; it is not inferred from this negative control.

Isolation: owned temporary team/package IDs; fixture skill source/selection records reserved by task1.1. Fixture provenance/content digest, exact source roots, test-owned override and protected credential bindings are named prerequisite `FIXTURE-ADMIT-BOUND`, currently NOT_EXERCISED. No delete or mutation of unrelated skills.

Evidence: before/after source and stored-selection metadata, source-aware startup receipt, refused control with zero new provider dispatch, team member run IDs, safe cause if a failure occurs. The counted positive inference is TEAM-01, shared rather than repeated.

## SETTLE-01 — original-attempt reconciliation

Owner: tasks2.2+3.1. Required resilience scope, separate from positive concurrency.

Use the retained exact team `72860111-231c-49ec-a973-e0a4e53f2a63`, math attempt `285108f8-4cfe-4ff6-8b3b-3a8d9fba052a` / run `414193e0-50e2-4ec0-a7e2-ac1bd436b6cc` as input evidence. Through the authorized existing recovery boundary, bind current verified owner/workspace/fence and read authoritative original execution/settlement evidence before selecting an outcome. If terminal failure is established, retain that failure; if it is not, expose explicit uncertainty. In both cases unknown usage remains charged and the finished worker is not silently described as active. No inference/tool retry occurs.

Historical uncertainty handling alone MUST NOT pass SETTLE-01. After TEAM-01 produces a conclusively terminal owned attempt, exercise its production terminal reconciliation path twice and require identical committed output/receipt, unchanged usage revision/reservation/artifact set and no new attempt/provider call. This known-terminal control MUST also exercise an actual confirmed terminal outcome with unknown usage (`usage=None`), using the retained original attempt only if its authoritative terminal evidence supports that conclusion. If no such genuine unknown-usage terminal fixture can be established, SETTLE-01 is BLOCKED; an uncertain receipt, synthetic record or a known-usage result alone cannot substitute.

Repeat exact reconciliation commands once as the named idempotency controls, then reuse a command ID with a conflicting payload as negative control; refusal must preserve prior state. These controls MUST reach terminal-settlement reconciliation, not merely replay a cached recovery-command response: use two separately identified authorized reconciliation requests for the same exact terminal outcome (fresh command IDs/current expected revision), then replay one identical command as the cache/idempotency control. Changing request identity does not permit a new execution attempt or alter the requested terminal outcome. Neither control re-executes agent work. Record before/after outcome, receipt, usage revision, reservation, artifacts and provider/attempt identities for each control.

Isolation: only the retained original attempt and the exact owned known-usage/unknown-usage terminal fixture attempts frozen in Plan (including the selected fresh TEAM-01 attempt). Bind every allowed attempt to its owner/workspace/team/run/fence; no unrelated/foreign attempt mutation or accounting reset. Current ownership, conclusively terminal known-usage and genuine unknown-usage fixture evidence, availability of original terminal/effect records, privileged recovery authority and safely fenced store access are prerequisite `FIXTURE-SETTLE-BOUND`, currently NOT_EXERCISED. Missing evidence is recorded explicitly; unreadable storage or missing required authority is BLOCKED, not a passing uncertainty assertion.

Limitations: this scenario reconciles the retained observed failure and tests stable replay. It does not reproduce every possible lost-acknowledgement, transport failure or backend conflict. The repaired code must not claim those branches passed without their own actual evidence; supplemental source/integration coverage does not substitute for this real path.

## TEAM-01 — overlapping useful work and article integration

Owner: tasks2.3+3.1+3.2. Functional scope.

Create a durable two-member team from immutable captures of existing `math-specialist-db64c2a8` and `writing-specialist` IDs/instructions. Submit fresh garden arithmetic and an independently useful editorial outline, followed by an article that depends on both. Host-authored task decomposition is explicit; automatic natural-language planning is not claimed.

Pass: distinct member/run/root/session identities; a proven interval of both actual runs active (not two queued tasks or client promises); both successful genuine configured-model results; independently calculated numeric answer; attributed output artifacts; dependent writer receives both exact artifact IDs and returns a useful article with the math result, outline headline and saved byline. All corresponding calls settle with real usage evidence; final article success is required.

Negative control: attempt article admission before prerequisites complete; it must not dispatch a provider call or produce an article. Observe the documented refusal/non-executing state. Do not weaken overlap to compensate for an early terminal error or very short run.

Isolation: fresh task/team/package/command IDs in the existing owned behavior fixture. Preconditions `FIXTURE-TEAM-BOUND`: repaired binary digest/source inventory, endpoint/config/store identity, valid credential environment references, exact model profile/settings revision, active capacity2, saved-agent snapshots and owned output root. Present state NOT_EXERCISED; no assumption that earlier revision1 or expired JWTs remain usable.

## Exact proposed final local commands

Cwd for both: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`.

```sh
CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/team.cucumber.mjs
```

Task1.1/Plan must bind the concrete repaired-binary activation/startup command, process/log ownership, readiness operations, environment references (`UAR_BASE_URL`, `UAR_MODEL`, `UAR_AUTH_TOKEN`, `UAR_WORKSPACE_ID`, `UAR_CUCUMBER_REPORT` and any required recovery-admin reference), and the three fixture prerequisites before this can be an executable approval packet. Activation may require an owned restart; this Spec does not authorize one. Do not silently add probes, commands or verification to the list. The required Rust artifact review is a separately named bounded delegation in that final packet.

Inside the existing project gate, order named oracles ADMIT-01 then TEAM-01 then SETTLE-01 so a fresh conclusively terminal result exists before terminal replay controls. Run the approved build and project gate serially once; stop at first failure and discuss correction versus backlog before another attempt. Missing fixture/authority/remaining allowance is BLOCKED. Exit2 is BLOCKED, never success. A zero exit passes only the actually observed row oracles. No CI product testing or publication.

## Evidence, retention and limits

Append under `.prometheus/verification/teams/` and the child receipt directory: exact argv/cwd/redacted environment names, source/artifact/config/fixture digests, command outcomes and timestamps, owner-scoped run/task/member/call/receipt IDs, overlap observations, selected artifact IDs, article and accounting disposition. Preserve failed receipts. Invalidate only affected evidence after material edits; prior Mac behavior passes remain historical and are not automatically repeated.

Inherited cumulative ceilings remain4800 host calls/3000 active minutes/28 repairs, no-progress3, and independent USD10 Cohere/USD10 OpenAI embedding caps. Prior consumption must be reconciled; unknown remaining allowance prevents execution. Token budget remains UNKNOWN with parent fallback; no reset, signing, adoption or cap increase occurs here. The parent fourteen obligations remain in force until explicitly reconciled.

## Plan binding

See the canonical nested child plan.md and exit-contract.md for file owners, child-only parent-boundary decision, resource floors, operational lifecycle and clean closure. The new targeted no-dispatch recover variant is task2.2 production work; today's legacy endpoint cannot supply SETTLE-01. Only the retained original with authoritative evidence is proposed for unknown-usage fixture; absence blocks acceptance. A failed/blocked bound child cannot use normal child-exit or archive as accepted. No commands are approved or run by these documents.
