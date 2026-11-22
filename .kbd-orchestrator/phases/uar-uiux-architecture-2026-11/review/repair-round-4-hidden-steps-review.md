# Review of the steps hidden behind the round 4 failures (artifact-critic, 2026-11-22, about 12:24 CST)

Transcribed by the lead from the reviewer's returned report. Read-only, on 9ad39030. **The lead has not verified
these findings.** Verify each in source before acting.

## Drafts
- `repair-round-4-draft.patch`: accept with one change: `checkVisibility({ visibilityProperty: true })`, because
  without options it also counts `visibility:hidden` text, which innerText excluded.
  - The reviewer also said the exact filter was never probed. The lead did probe the filter as drafted (3 replies
    counted, a display:none decoy ignored; unfiltered 6), but saved only the earlier probe as evidence. The
    amended option has not been probed.
- `repair-round-4-draft-thread-menu.patch`: accept, correct and complete. It also makes thread Delete
  reachable, and nothing tests Rename or Delete.

## Predictions with both drafts
- SYS-013 FAIL at "the operator changes the system default to openai/gpt-5.4-mini" (H1), and possibly earlier at
  "send r5 y2" (P1). Every other scenario and control: PASS predicted, with the H2 timing risk.
- Both mutant judgements: PASS if the worktrees are rebuilt at the new candidate. They cannot reuse 9ad39030 builds.

## Further defects
- **H1, HIGH, harness, deterministic.** The picker option is never mounted. `tests/bdd/support/s1-ui.ts:253-270`
  types the model id and waits without scrolling. The list is virtual above 1000 rows
  (`combobox-picker-model.ts:13`, `combobox-picker.tsx:480-530`). For "gpt-5.4-mini", openai's row is 44 of 68
  and about rows 0-16 mount. Typing the qualified id still gives row 23.
  Fix: after fill, scroll `[data-slot="combobox-list"]` a page at a time until attached or at the end.
  optionState (:273-296) needs the same, or an offered option reads as absent and passes a "must not offer" check.
- **P1, MEDIUM, product, race.** Send is enabled before the stream lease is released.
  `chat-message-store.ts:1108-1132` clears isStreaming, then awaits writes; `chat-stream-store.ts:1361-1368`
  awaits settleTurn (persists the graph, `platform/entities/chat-turns/index.ts`); `stream-ownership.ts:5` throws
  "Another response is active". Fix in the product: keep the composer busy until release, or await the settling lease.
- **H2, MEDIUM, harness, race.** A half-revealed reply can be captured: MarkdownText smooth reveal (about
  110-170 ms), and `s1-ui.ts:95-107` takes the first count increase. Fix: wait until the newest assistant turn
  has no `[data-status="running"]`.
- **H3, MEDIUM, harness, false step.** `world.ts` switchAgentViaUI clicks `getByText(title).first()`, which is the
  trigger's own span (`agent-selector.tsx:132`), so "Default Assistant" is never selected.
  Fix: scope to `[data-slot="command-item"]` and wait for the agent-config POST.
- **P2, MEDIUM, product, not reachable in this harness.** In postgres or remote-Surreal mode a rename is undone:
  `src/server/lifecycle_snapshot.rs:216-223` titles every thread "New conversation" and
  `use-thread-graph-sync.ts:67-69` copies it over. Fix: never apply the placeholder over a local title.
- **P3, LOW, product.** Only the first thread of a page load gets a title (`use-chat-runtime.ts:83,117,160`;
  titleGeneratedRef is per component and `chat-page.tsx:153` is not keyed by thread). Fix: track per threadId.

## Only execution can establish
checkVisibility on skipped turns in the real page; the P1 and H2 window widths; latency against the 15 s limit
(sends took 7.4-7.6 s); confirmClearOverride inside its 3 s fallback; two pages on one IndexedDB; the unexecuted
tails (routing after a thread switch, refused-override 422, served-model receipt); mutant JSON step titles.
