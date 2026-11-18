# Verification log

Runs under the operator's grant recorded in `implementation-inventory.md`. Commands were run directly with `node`, because `pnpm run` refuses to start in this checkout (`ERR_PNPM_VERIFY_DEPS_BEFORE_RUN`); each is the exact command the package script wraps.

## Round 1 (2026-11-18)

| Step | Command (from `frontend/`) | Result |
|---|---|---|
| Type-check | `node node_modules/typescript/bin/tsc -b` | FAIL: 5 errors in 4 files. Fixed; rerun exit 0. `tsc -p packages/uar-ui/tsconfig.json` exit 0 |
| Lint | `node node_modules/eslint/bin/eslint.js .` | FAIL: 1 error (`commands.ts`, `no-useless-assignment`). Fixed; rerun over the touched directories exit 0 |
| Tests | `node node_modules/vitest/vitest.mjs run src/platform/entities/teams src/features/teams src/app/shell/nav-destinations.test.ts` | FAIL: 4 of 59 (fixture path in `digest.test.ts`). Fixed; `digest.test.ts` rerun 6 of 6. The digest tests recompute the runtime's own fixture digests |
| Build | `node node_modules/vite/bin/vite.js build --outDir ~/.uar-dev/static --emptyOutDir`, with the PEM PR 81 build of `entity-graph-core` swapped in | exit 0. Same pre-existing warnings as the previous build (4 direct-`eval`, 1 chunk size). The 5.0.0 `dist` was restored and its checksums verified |
| Browser | Dev instance on 1907 through the identity proxy on 8090, `/admin/teams` | see below |

Type errors fixed: wrong `ModelSelector` import path; duplicate keys in a spread in `binding-sheet.tsx`; `Uint8Array<ArrayBufferLike>` passed to Web Crypto in `digest.ts`; an unsound cast in `loads.ts`.

### Browser pass, round 1

| Flow | Result |
|---|---|
| Page load, readiness strip, lists | PASS. 4 of 4 ready, 6 teams, 1 definition, no console errors, every request 2xx |
| Team detail and run view | PASS with a defect: artifacts and peer messages were not read when a team was opened from the list |
| Routing test to the researcher role | PASS. create 201, claim 200, admit 200; lane went Running then Succeeded (1,177 tokens, 7 s) with the member's output; polling stopped when nothing was live |
| Launch | PASS with defects: the sheet preselected a binding the runtime cannot activate; focus fell to `<body>` after the launch |
| Binding create | PASS. preflight 200 "can activate", install 201, listed, focus on the new entry. Each diagnostic was listed once per agent with no way to tell them apart |
| Package install | FAIL. The sheet refused to install because package preflight reports `activationSupported: false` |

### Defects found in round 1 and their causes

- **Install refused for every team package.** `validation.rs:83-92` adds a `required-unsupported` diagnostic for every non-agent definition at package preflight, and `service.rs:295` installs regardless. Activation is decided at binding preflight. The spec required a preflight "whose activation is supported"; that requirement was wrong and is corrected, with the sheet.
- **Artifacts and peer messages missing.** `runLoad` shares one in-flight request per key. The page header started a run refresh without side records; the watcher's first tick joined it and its request for side records was dropped. `refreshTeamRun` now reads them when it joined a refresh that had none.
- **Launch default.** The sheet took the first binding. It now prefers one the runtime can activate and marks the others.

The independent review reported 16 findings (2 high). Round 2 below records their repair.

## Round 2 (2026-11-18)

Three writers repaired the review's 16 findings and the browser defects, one per ownership area (domain; page and sheets; run view). The lead then fixed what the second browser pass found.

| Step | Command (from `frontend/`) | Result |
|---|---|---|
| Type-check | `node node_modules/typescript/bin/tsc -b`; `tsc -p packages/uar-ui/tsconfig.json` | exit 0, exit 0 |
| Lint | `node node_modules/eslint/bin/eslint.js .` | exit 0 |
| Tests | `node node_modules/vitest/vitest.mjs run src/platform/entities/teams src/features/teams src/app/shell/nav-destinations.test.ts` | 7 files, 67 of 67 pass |
| Build | as round 1 | exit 0, same 5 pre-existing warnings; 5.0.0 `dist` restored, checksums verified |

The final edits (launch focus, cancel-retry check, usage wording) were re-checked with type-check, lint over the touched directories and the same 67 tests, then rebuilt.

### Browser pass, round 2

| Flow | Result |
|---|---|
| Readiness strip | PASS. One line while all four facts are ready; details open on request |
| Team opened from the list | PASS. Artifacts (1) per lane; one persistent live region per lane |
| Package install | PASS. Local digest check, preflight 200, install enabled, confirmation names permanence and takes focus ("Keep checking"), Escape returns focus to the opener, install 201, new definition listed and opened without reload |
| Launch | PASS. Preselects a binding the runtime can activate and marks the one that cannot; focus lands on the new team's heading |
| Cancel a running attempt | PASS. First observation of this path. Cancel 200, lane "Cancelling" with the operator's reason, focus moved to the attempt card, then settled "Cancelled" |

### Learned in round 2

- **A mid-call cancel charges the whole reservation.** The settled attempt had `usage: null` and `stateReason: team_usage_or_effect_outcome_uncertain`, and the budget counted 128,000 tokens and $1.00. `runtime/team_execution/execution.rs:211`: a model request without a provider usage report keeps its whole reservation. The card now reads "Charged in full, no usage report" and explains the reason code. Payload: `observations/execution-cancelled-mid-call.json`.
- **Settlement overwrites the cancel reason.** The committed-cancel check for a retry had matched on the operator's reason; it now accepts any attempt already `cancellation_requested` or `cancelled`, and keeps the reason match only for a waiting attempt.
- **A background tab hides focus results.** The browser pauses `requestAnimationFrame` for a tab that is not in front. Two focus "defects" from round 1 (launch, install confirmation) were this artifact; re-run with the tab in front, one was real (launch, see below) and one was not.
- **Launch focus.** The definition heading shares `teams-detail-heading` with the team heading and is still in the page while the sheet closes, so the sheet's own focus return must not look it up. After a launch the sheet now moves no focus and the new heading focuses itself.

## Not exercised

Waiting, handed-off, delegated and uncertain attempts (cancelling and cancelled were exercised in round 2); a retry of a cancel whose response was lost; a refused or uncertain routing step; a second workspace; a runtime that does not own execution; a 401, 409 or 429 from the service. The sample team cannot delegate on this runtime (team tools are not offered to members).

## Design critiques (2026-11-18)

Two isolated critics (Impeccable critique method, Operate mode) reviewed Playwright screenshots of the dev instance at 1440 px and 390 px plus the source. Neither saw this conversation or the handoffs. The light-theme screenshots rendered dark (the app keeps its own stored theme), so the light theme was not judged.

| Critic | Nielsen score | MUST findings |
|---|---|---|
| A (source and screenshots) | 24/40 | pinned header takes ~265 px desktop and ~390 of 844 px mobile; lanes come after the whole test form; definitions under every team; teams indistinguishable in the list; cancel hides its cost |
| B (screenshots only; could not list directories) | 20/40 | teams indistinguishable; cancelled-with-full-charge understated; routing test does not state its cost before commit; result lands a screen below the button on mobile |

Accepted and assigned: every MUST, plus definition order, `Record status` and roster removal, launch-sheet labels and fixed counts, binding Step 2 opening while references are missing, one footer order across sheets, a styled file picker, mobile navigation reaching Teams, lane scannability (duplicate status, idle "Current task", member id, "Billed at the rate of", clamped output).

Not accepted: lowering the default routing-test reservation. A reservation below the model's output budget is refused at admission (`output_budget_unavailable`, observed earlier); the form states the cost instead. A first-send confirmation dialog: the consequence line above the button carries that information without an extra step.

## Round 3: design refinement (2026-11-18)

Two writers applied the accepted critique items (page and sheets; run view). The lead fixed what verification found.

| Step | Command (from `frontend/`) | Result |
|---|---|---|
| Type-check | `tsc -b`; `tsc -p packages/uar-ui/tsconfig.json` | exit 0, exit 0 |
| Lint | `eslint .` | FAIL first: 8 errors. 6 were the console's Flat 2.0 rule (borders and a shadow used for the selected row, the warning bars and the submit button); 2 were `react-hooks/immutability` on a ref prop named `armed`. Fixed with fills instead of borders and the prop renamed `armedRef`; rerun exit 0 |
| Tests | `vitest run src/platform/entities/teams src/features/teams src/app/shell` | 9 files, 90 of 90 pass |
| Build | as before | exit 0, same 5 pre-existing warnings; 5.0.0 `dist` restored, checksums verified |

Browser and screenshots (Playwright at 1440 px and 390 px, then the in-app browser with the tab in front):

| Check | Result |
|---|---|
| Desktop list | FAIL first: the Teams/Definitions tabs rendered as a tall side column and the list overlapped the detail pane. The shared `Tabs` keys its column layout on a `data-horizontal` attribute that is never set; other pages pass `flex-col` themselves. Fixed in the Teams list; rerun PASS |
| Desktop header and rows | PASS. One header row; rows show binding, `#` short id and relative time; selected row has a filled marker |
| Lanes and routing test | PASS. Lanes before the folded test form; consequence line above "Reserve and send"; formatted reservation hints |
| Charged-in-full cancel | PASS. Warning bar in the lane header and on the usage line; plain explanation, code behind "Runtime detail" |
| Phone (390 px) | PASS. Teams in the bottom bar and highlighted; header scrolls away; detail starts with "Back to list" |
| Live routing test after the redesign | PASS. Sent to the writer; the form folded, focus moved to the writer's lane heading, lane Succeeded (996 tokens, 5 s) |

Open from round 3:

- A team whose latest attempt failed or was charged in full still reads "Idle" in the list: the domain exposes liveness and attempt counts per team, not failures.
- The clamped output shows a sliver of its fourth line under the "Show more" fade.
- Visual baselines in `e2e/runtime-console-visual.spec.ts` that include the phone bottom bar will change (Teams added). Not run.
- Light theme was not captured: the app keeps its own stored theme.
