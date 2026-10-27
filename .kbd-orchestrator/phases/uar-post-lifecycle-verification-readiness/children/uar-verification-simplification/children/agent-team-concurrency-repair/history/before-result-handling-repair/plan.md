# Plan — agent-team-concurrency-repair

Date: 2026-10-26. Project: Universal Agent Runtime. OpenSpec: YES.
One change: **repair-agent-team-concurrent-operation**. Customer value: high. Complexity: M, with actual repair time dependent on the observed catalog error. Agent: Codex lead + one uar-runtime source writer, working serially; independent artifact-only review at the completed implementation boundary.

## Agreed scope and finish line

The operator explicitly narrowed this child to fixing the bug blocking the existing math/writing team scenario. Success means that the **existing BDD proves real concurrent math/writing work and a correctly merged article**. Record that Mac result, reflect and return to `uar-verification-simplification` for operator-directed manual verification. Stop after this Plan handoff now.

No new acceptance scenarios, recovery API, unknown-usage fixture campaign, verification framework, dependency upgrade, platform matrix or rerun of earlier accepted behaviors. The expanded unadopted drafts are preserved under history/expanded-plan-superseded-20261026, not active instructions. Prior failures, parent obligations, reservations and consumption remain historical; this child cannot certify the parent.

## Ordered implementation tasks

All tasks belong to canonical phase `uar-verification-simplification::agent-team-concurrency-repair`, change `repair-agent-team-concurrent-operation`.

| ID | Work | Owner and files | Dependency / completion |
|---|---|---|---|
| 1.1 | Fix skill admission blocking the two team members, preserving existing API overrides/scoped settings and safe failure reporting | uar-runtime: src/server.rs; src/uar/runtime/skills/{service.rs,registry.rs,service/selection.rs}; src/uar/runtime/manager.rs | First. Source implementation and behavior notes complete; no per-task tests |
| 1.2 | Fix the observed catalog terminal-settlement failure as needed for this same team workflow; retain honest outcomes/accounting and prevent duplicate execution | Same uar-runtime writer: src/uar/compiler/collaboration/{storage.rs,service.rs,team_execution/settlement.rs}; src/uar/runtime/team_execution/{execution.rs,controller.rs} | After1.1. Use typed error/result evidence; no speculative backend replacement or new recovery endpoint |
| 1.3 | Integrate the repair and prepare the same BDD against the repaired native binary | Lead: existing scripts/behaviors/team.cucumber.mjs, features/math-writing-team.feature, steps/math-writing-team.steps.mjs only if needed; child documentation/inventory | After production1.1–1.2. Preserve overlap and merged-result assertions, saved agents/store and old failures. Freeze source and exact build/restart/run details; no additional scenarios |
| 1.4 | Obtain the named final verification authority, build and run the existing scenario; record its actual Mac result and parent handoff | Lead sole build/process/run owner; independent Rust/code reviewer owns findings only | After all implementation1.1–1.3. PASS requires the scenario's real outcomes; failure stops for discussion |

One writer per production file and build target. Runtime writer uses the existing native uar-runtime role (gpt-5.6-terra/high); lead uses current native host. Independent reviewer uses fresh artifact-only context with Rust expertise, no producer history. Others share the checkout; preserve existing edits. If a required file outside this list is discovered, name it and assign ownership before editing; do not broaden product scope. No production worker is started by this Plan.

## Repair direction grounded in evidence

Both retained member runs failed with skill_selection_unavailable. Source registers filesystem-skills-* but reconciliation searches older provider names; adapt existing reconciliation (cand-001), without globally disabling skills. The precise old binary failure category is not proven.

The catalog returned an error reading its CAS result. Inspect/preserve the actual statement/result error and correct this terminal path (cand-002). An error does not prove rollback; never replay agent work or erase unknown liability. The old ambiguous attempt remains historical evidence, not a new required recovery scenario. Keep existing Tokio member lanes and artifact dependency graph (cand-003); a scheduler redesign is not justified by the observed admission failure.

The uncomfortable limit: repairing the identified paths may expose another failure. Report that specific failure and discuss fixing it versus backlog. Do not silently add a new acceptance program or claim the next run must pass.

## Verification and exit

Acceptance is **TEAM-01 only**, the existing `Concurrent calculation and editorial work merge into one article` scenario. It must demonstrate distinct actual member runs overlapping, successful correct math and editorial outputs, then a final writer response using both output artifacts with the expected result/headline/byline. Existing honest settlement assertions remain; they are not expanded into separate resilience tests. Host-authored task decomposition is explicit.

After all child implementation is complete, record its inventory and present the exact final review/build/owned-restart/BDD commands under the repository verification lock. Proposed build and existing BDD entry, repository cwd:

```sh
CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/team.cucumber.mjs
```

No checks, builds, tests, service changes or verification run during Plan. Do not restart the retained UAR until applying the authorized repair. Preserve existing math/writing agents, KBs and durable store; use target/prompt-macos-launch.json for retained process/config identity, not a guessed PID. Keep the repaired UAR running afterward. The BDD uses the same real Liter/MiniMax-M3 configuration and valid owner/workspace authority; capture actual model settings revision rather than assume revision1. No new embeddings.

Inherited limits remain cumulative:4800 calls/3000 active minutes/28 repairs/no-progress3; USD10 per embedding provider. Reconcile actual remaining allowance before Execute; no reset or new idle exclusion. Last plan accounting is a dated receipt, not a current balance.

On PASS: record evidence tied to repaired source/binary, reconcile tasks, archive only this accepted OpenSpec change, reflect, obtain canonical child closure and hand off to the existing parent. Preserve unrelated parent tasks. Child durable-team-execution requirement names are distinct from pending afc-c09 requirements; sync by addition, never replace that sibling's work. On FAIL/BLOCKED: keep the result and child incomplete; discuss the next action. No automatic retry or successful exit claim.

The scope/exit criterion above is agreed by the operator's current instruction. Contract adoption is local KBD metadata only; it does not open the verification gate or authorize implementation during this turn. Full parent verification remains outside this child; the later command approval must explicitly cover this child boundary rather than silently complete unrelated parent work.

## Handoff

Next command: `/kbd-execute agent-team-concurrency-repair`, starting task1.1. It means this bounded repair, not any superseded expanded draft. After successful execution: `/kbd-reflect`, then `/kbd-child-exit`, and return to manual verification. No evolver bridge applies. Stop after Plan.

## Authorized follow-up — token reservation mismatch, 2026-10-26

The real TEAM-01 run failed before inference: catalog_context_and_output reserved1,512,000tokens against32,768turn/session limits. Operator instructed fixing this now, then clarified responses must be capped at64,000tokens, not a million-token reservation. This supersedes the old fixture token figures only; financial/time limits, prior consumption and TEAM-01 oracle remain. No retry is authorized until the refreshed implementation inventory and exact gate packet are presented.

Task1.5 (source implementation plus integration/docs; ordered before pending1.4): uar-runtime owns cost_budget.rs, llm/liter_driver.rs, llm/mod.rs, llm/team_profile.rs and runtime/team_execution/execution.rs only as required. Lead owns existing BDD steps and child/OpenSpec metadata. Enforce the response cap at actual outgoing request, keep input allowance distinct, and correct reservation construction without substituting model capacity for request size. Do not relabel approximate counts as exact, bypass accounting, release unknown old liabilities, change price/time ceilings, add dependencies or scenarios. Capture baselines before editing. Exact input allowance/bound mechanism is resolved by source analysis and recorded in the implementation handoff before the gate.

Cumulative source repair22 is charged once at entry; previous21 remain. Ceilings4800calls/3000active minutes/28repairs and separate USD10 embedding caps persist. Gate CLOSED; existing UAR PID10715 stays running. Task1.4 remains blocked pending this repair/inventory and renewed final instruction. Historical tasks1.1–1.3 and failed run receipts are retained; the canonical runtime does not reopen completed tasks, so1.5 records this specific additional repair without adding a product scenario.

Resolved allocation for task1.5: response cap64,000; independent input allowance64,000; per-attempt128,000total; team384,000total for three attempts. Existing cost1USD/attempt and3USD/team and120s/attempt360s/team limits remain. Reservation is an explicit host allocation, not a claim of exact provider input counting. Existing estimated context reduction must respect the allocation; exact-contract path stays strict and unchanged. Runtime writer may also edit manager.rs to carry the bounded context.
