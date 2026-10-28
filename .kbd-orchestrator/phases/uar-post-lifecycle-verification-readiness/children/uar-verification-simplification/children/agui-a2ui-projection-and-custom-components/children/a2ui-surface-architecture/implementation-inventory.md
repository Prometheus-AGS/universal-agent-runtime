# Implementation-complete inventory — A2UI slice (revision 8)

Recorded 2026-10-28 by the lead. Stage: Execute, implementation complete.
Verification gate: **CLOSED**. Nothing below was compiled, tested, type-checked,
linted or run. Every behavioural claim is unverified until the gate runs.

Source identity: HEAD `12b7a8e6` plus the uncommitted working-tree files below
(sha256 of each at the time of recording).

## Delivered, by plan task

| Plan task | Files | sha256 |
|---|---|---|
| 1. OpenSpec change | `openspec/changes/agui-a2ui-activity-slice/` (proposal, tasks, spec delta `ag-ui-a2ui-activities`) | — |
| 2. Projection, restatement, action check and builder, 24 unit tests | `src/uar/api/agui_a2ui.rs` (new, 559 lines) | below |
| 3. Wiring: action requests skip the last-user check, activity messages dropped, graph refusal before reservation | `src/uar/api/agui.rs`, `src/uar/api/mod.rs`, `src/uar/runtime/manager.rs` (`uses_agent_graph_for`) | below |
| 4. Console: click-started user turns render as `Action: <name>` (or `Action sent`), never raw JSON; thread preview too | `frontend/src/shared/content/a2ui-user-action.ts` (new), its `.test.ts` (new), `frontend/src/features/chat/use-chat-runtime.ts`, `frontend/src/components/layout/left-sidebar.tsx` | below |
| 5. I19 rewrite and isolated PEM 4.3.0 client | `scripts/behaviors/features/agui-route-a2ui.feature`, `scripts/behaviors/steps/agui-route-a2ui.steps.mjs`, `scripts/behaviors/clients/a2ui-pem/package.json` + lockfile | below |
| 6. Rebuild command confirmed | plan §4 step 3 | — |

```
ec7e713e6501914c286a68a74de846fe6132f97920d9b0d22dcc09093556c1f4  src/uar/api/agui_a2ui.rs
cb462594b3dfd793290405f0d59b299c057673fe6bc328002da2fd473f5903bc  src/uar/api/agui.rs
4f82bd1b6d727707d7291a169ad8e4b1cdc3daf3960a9550a085c0aad36b10d5  src/uar/api/mod.rs
9a4e73343c5cba7b8fa796cf8f2a9a197c9e4267f13b9fc5d035012e476a436b  src/uar/runtime/manager.rs
ded69259a7e6815771416b09558e6eb82edb49831ea932a4a0e037e61a613779  frontend/src/shared/content/a2ui-user-action.ts
63c9bcb28be7a8be046567afdb0e3c6dfa8f47f34fe2d08635c9a61b83cf0b8f  frontend/src/shared/content/a2ui-user-action.test.ts
4f004cfeae72d360e5f59f1a6c17c8c8a7d64e2f8ea8721d81d1413aeea6dacb  frontend/src/features/chat/use-chat-runtime.ts
155b918484f64b0c2b7d268c2eb6d94bc67cbe199b5b0eca13f2d7627bde65ce  frontend/src/components/layout/left-sidebar.tsx
e1f72b0bc08ca3b86dcd2e1210233013847d21a0be2749f66d41fb741e4bca15  scripts/behaviors/features/agui-route-a2ui.feature
0e4588a4e6e2bbe913ae3c277a26251272b27329afde9ba0caa9dcc79b296816  scripts/behaviors/steps/agui-route-a2ui.steps.mjs
03fcee62cf52b1d4b7e711815e2c6e6ac00b37929ceabe8a7794c9299a3ec5b3  scripts/behaviors/clients/a2ui-pem/package.json
751c2c1be5aeb42bebc3acf25e6b022ffa798affb300c2ba1260d8e6f2092688  scripts/behaviors/clients/a2ui-pem/package-lock.json
```

`left-sidebar.tsx` and `use-chat-runtime.ts` already carried earlier uncommitted
edits; this slice added one import and one call to each.

## Decisions taken in Execute (not in the plan)

- `createSurface` on an existing surface id restarts that surface's operation list.
- `sourceRunId` is the run that most recently published to the surface.
- The newest definition of a component id wins in the action check.
- Action plus `resume` or caller tool results: `400 a2ui_action_with_resume`.
- Action runs skip the "Client supplied context" suffix, so the input stays pure JSON.
- Check order: graph refusal (422), then declaration (422), then `a2ui_action_with_text` (400).
- Console shows a label, not nothing: a hidden turn leaves the reply with no visible cause.

## Lead source review (read only)

The `agui.rs` diff matches plan §2: `a2ui_action_input` runs before
`create_agui_run`; `create_request` takes the action input as the prompt and
treats every non-activity message as history; restated frames use
`(start_cursor, 1..=48)` and honour `after`; the projector is seeded from the
thread record so snapshots stay cumulative.

## Known limits added in Execute (noted, not fixed)

- Console Edit on an action-label turn would send `Action: <name>` as a prompt.
- The console's own click path inserts no user turn live; the label appears on reload.
- `session-backfill.ts:48` local turns omit action turns (pre-existing with the REST action path).
- A client-resent assistant message with empty content and no tool calls is rejected as `history_invalid`. I19 doesn't send one.

## Remaining

No implementation item remains. Pending: the gate in plan §4, on the operator's go.

## Unverified (all of it)

Compilation (named risks: `Cow<[Value]>` indexing, `reject` closure reuse, let-chain
in `fold_events`, trait method resolution for `list_agui_runs`), Clippy, all 24 Rust
unit tests, the console test, the type-check, PEM's acceptance of the snapshot
shape, restatement against real durable history, and I19 end to end.

## Gate result — 2026-10-28 (operator go; unit tests and typecheck removed by the operator)

| Step | Command | Result |
|---|---|---|
| Validate | `openspec validate agui-a2ui-activity-slice` | valid |
| Build | `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime` | exit 0, 19m20s, 3 lib warnings; binary sha256 `50ac01d68248aac51f285651c44b39989c8719cc025db8386cebbd7e5f4ac104` |
| Owned restart | `restartOwnedUar(record, 50ac01d6…)` | pid 64280 → 15316, ready, store preserved |
| I19 | `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agui-route-a2ui.cucumber.mjs` | attempt 3 PASS, 12/12 steps, 12.0 s |
| selection-stream-replay | same runner, its config | PASS 8/8, first run |
| live-roster-routing | same runner, its config | PASS 13/13, first run |
| native-tool-approval | same runner, its config | PASS 7/7, first run |
| agui | same runner, its config | PASS 4/4, first run |
| writing | same runner, its config | PASS 7/7, first run |

All scenarios ran with `UAR_BASE_URL=http://127.0.0.1:60675`,
`UAR_WORKSPACE_ID=uar-user-outcomes-007-supporting-workspace`, `UAR_MODEL=liter/MiniMax-M3`.

I19 attempts (all retained in `.prometheus/verification/`):
1. `17-51-36` — invocation lacked `UAR_WORKSPACE_ID`/`UAR_MODEL`; failed at the precondition, no product call.
2. `17-51-54` — test defect: `rendered()` spread web_core's `getAll()` Map as `[id, model]` entries. Raw evidence showed one `a2ui-surface` activity and 5 components. Fixed to `.values()`; assertions unchanged.
3. `17-52-41` — PASS.

I19 raw evidence (`agui-route-a2ui/disposable-agui-a2ui-f2778abd-….json`):
- Publish run: `TOOL_CALL_*` then `ACTIVITY_SNAPSHOT`; client rendered `Ticket 8fab2dc3`, a note TextField and a Confirm button.
- First click: new run, restated snapshot, answer `NOTE: note-d6f3028a` (the note existed only in the client).
- Second click: answer `NOTE: note-1d84f671`.
- Late attach: `ACTIVITY_SNAPSHOT` with no `TOOL_CALL_START`; surface re-rendered.
- Forged `doesNotExist` action: HTTP 422 (the response code body was not captured).
- No credentials supplied by the test; cleanup clean.

After the gate, the unrequested authored unit tests were removed (`#[cfg(test)] mod tests`
in `agui_a2ui.rs`; `a2ui-user-action.test.ts`). Production code is unchanged, so the
gated binary still matches the source; the file hash of `agui_a2ui.rs` differs from the
table above for that reason only.

Not verified by this gate: the console action label (no BDD scenario renders the console
for a click-started run).
