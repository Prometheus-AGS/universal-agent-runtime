# Repair round 4: proposal for the operator (2026-11-22)

Status: PROPOSED. Not granted, not implemented, not signed. Based on source reading of candidate 9ad39030
(no frontend, tests or src change since). No test, build or probe was run for this proposal.

## 1. Findings, verified by reading source

| ID | Defect | Verified? | Kind | Fix | rc.6? |
|---|---|---|---|---|---|
| T1 | Thread menu Rename and Delete do nothing. `left-sidebar.tsx:186,195` pass `onSelect`; Base UI 1.9.0 `Menu.Item` has no `onSelect` (MenuItem.d.ts lists only `onClick`). TypeScript accepts it as the DOM text-selection event | Confirmed | Product | Draft `repair-round-4-draft-thread-menu.patch` (onSelect to onClick). No other Base UI menu item uses onSelect; the other onSelect callers are cmdk `CommandItem`s | Yes |
| C1 | Reply counter uses `innerText`, which drops replies skipped by `content-visibility:auto` | Confirmed (attempt 4 probe) | Test | Draft `repair-round-4-draft.patch`, amended to `checkVisibility({ visibilityProperty: true })` | Yes |
| H1 | System-default picker option never mounts. `catalogModelOptions` lists the whole catalog: 8,170 models, above `VIRTUALIZE_ABOVE_OPTIONS` (1000), so the list is virtual. Typing `gpt-5.4-mini` leaves 35 matches across 34 provider groups (about 68 rows); openai is match 23, about row 44; about 16 rows mount (288 px viewport, overscan 8). `chooseModel` waits without scrolling | Confirmed | Test | After filling the search, scroll `[data-slot="combobox-list"]` a page at a time until the option is attached or the list ends. Same in `optionState`, else an offered option reads "absent" and a "must not offer" check passes falsely | Yes |
| H2 | A half-revealed reply can be captured. `MarkdownTextPrimitive` smooth reveal defaults to on and the BDD config does not set reduced motion; `/bdd-reply-[A-Za-z0-9._-]+/` matches a prefix such as `bdd-reply-gpt-5`; `captureReply` takes the first count increase | Confirmed as a race; window width needs execution | Test | Before reading the reply, wait until the newest assistant turn is no longer running | Yes |
| H3 | "a fresh conversation using Default Assistant" never selects it. On a fresh thread the trigger shows the fallback text "Default Assistant" (`agent-selector.tsx:121`); `switchAgentViaUI` clicks `getByText(title).first()`, which is the trigger span (the popover is portalled after it). The click closes the popover; no agent is applied | Confirmed | Test (false step) | Scope the click to `[data-slot="command-item"]` and wait for the agent-config request | Yes |
| P1 | Send re-enables before the stream lease is released. `finishStream` clears `isStreaming` in `set()` before awaiting PGlite writes; `settleTurn` is awaited after; the lease is released in `finally` (`chat-stream-store.ts:1506`). A send in that window throws "Another response is active" (`stream-ownership.ts:5`) | Confirmed in source; window width needs execution | Product | Keep the composer busy until the lease is released (release state drives `isRunning`), or have a new send await the settling lease | Yes: users who send quickly hit it, and SYS-013 may |
| P2 | Remote stores revert a rename. The thread snapshot (`lifecycle_snapshot.rs:217-223`) titles every thread "New conversation"; `use-thread-graph-sync.ts:67-69` copies any differing server title over the local one | Confirmed in source; which store modes reach it not traced | Product | Never apply the placeholder over a local title | Follow-up |
| P3 | Only the first thread per page load gets an auto title. `titleGeneratedRef` (`use-chat-runtime.ts:70`) is never reset; `LazyChatThreadView` is not keyed by thread (`chat-page.tsx:158`) | Confirmed | Product | Track per threadId, or key the view by threadId | Follow-up |
| D1 (new) | Delete is local only. `removeThread` deletes from the client store and PGlite; nothing deletes the server session. `use-thread-graph-sync` re-registers any server thread the client lacks, so a deleted thread may come back on the next snapshot | Found while verifying T1. Not traced end to end | Product | Delete the server session too, or keep a tombstone the sync respects | Decide after the Delete test runs: the reload step shows it |

## 2. New tests for Rename and Delete

A new feature `tests/bdd/features/thread-menu.feature`, with no credentials, on the existing BDD stack:

- **SCN-THR-001 Rename.** Send a message on a fresh thread; open the row's actions; choose Rename; enter
  "BDD Renamed Thread"; Save. Then the sidebar shows that name and not the old one; after a page reload it
  still does. Control: Cancel in the dialog leaves the old name.
- **SCN-THR-002 Delete.** Two threads, A and B, each with a message. Delete A through the row menu. Then the
  confirmation names A; after confirming, A's row is gone and B remains; after a page reload A is still gone.
  Control: Cancel in the confirmation keeps A.
- Each gets a negative control against the current candidate (onSelect): it must FAIL there, which proves the
  test can see the defect.

The reload step in SCN-THR-002 is where D1 shows up. If A comes back, that is a FAIL for rc.6, not noise.

## 3. Contract amendment, in plain words (not signed)

- Allow a fourth repair round (repairAttempts 3 to 4).
- Add the two thread-menu scenarios and their controls to the adopted set (13 to 15 scenarios), with
  `left-sidebar.tsx` and the new feature in their invalidation keys.
- Raise the boundary time budget from 900 to 1,350 minutes (section 4).
- Move the candidate to the new commit; all attempt-4 verdicts are void. A fresh artifact-critic confirms
  the new inventory.
- No change to oracles, warning rules or any existing scenario's assertions.

## 4. Time

Used: the current `boundary/run.jsonl` records 526 minutes, plus command 38, which was interrupted and has no
record. That leaves at most about 374 of 900.

A full rerun, from attempt-4 timings: candidate and baseline builds and checks 15-60 min; both mutant rebuilds
80-200 min (attempt 3 and 4 took 117 and 157); scenarios and controls 23-49, plus the two new ones, 90-180 min.
Total 3 to 7.5 hours. The low end fits the 374 minutes left; the high end does not, and the high end is what
happens when other projects build at the same time. Hence 1,350.

Before the rerun: implementation and an independent diff and step review, about 1.5-2.5 hours of work, no
build. Expect the rerun to find something more. Nine of the ten chat scenarios have never reached their own
assertions.
