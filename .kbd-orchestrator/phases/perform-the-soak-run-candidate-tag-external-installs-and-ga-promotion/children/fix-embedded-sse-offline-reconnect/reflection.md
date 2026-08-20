# Reflection — `fix-embedded-sse-offline-reconnect`

## Delta between plan and delivery

The planned adapter correction was necessary but not sufficient. The first live
browser pass updated the normalized graph while the Knowledge view remained
stale because upstream React projections memoized full entities behind an
unchanged ID array. The first dependency preparation then exposed two more
source-workspace defects: the nested engine rejected UAR's pnpm 11.15.0, and a
direct React-only build consumed stale core declarations. Delivery therefore
includes the operator-approved upstream source repair, tested pnpm consumer
range, submodule advance, and dependency-aware source build. It does not include
a UAR screen refresh, store bypass, backend endpoint change, or offline replay.

Independent review also rejected the first final candidate. A closed fake
source retained its named listener, scalar `record` payloads passed validation,
the proposal understated the submodule impact, and five schema-valid refiner
checkpoints all falsely contained the same completed state. The corrected
candidate removes the listener and callbacks before close, rejects non-object
records, proves stale/late delivery cannot occur, states the dependency impact,
and retains five genuinely progressive correction-cycle checkpoints.

## Goal results

| goal | result | observed evidence | limit |
|---|---|---|---|
| Consume the server's named embedded event contract. | MET | Focused FakeEventSource file passed 3/0; valid mapping and transport/malformed/scalar controls passed. | Embedded frontend adapter only. |
| Recover without reload, parallel source, or retry after unsubscribe. | MET | Close-before-retry, one replacement, stale-predecessor rejection, late-event rejection, timer cancellation, and status assertions passed. | Resume-only; no checkpoint replay. |
| Prove visible application-bound delivery and recovery. | MET | Fresh-process Chromium observed the initial visible Knowledge update, a second real stream request/open, and one visible recovered update; 1/0 passed. | Local Chromium and `server-full` dev stack only. |
| Return truthful evidence to the parent screen certification. | MET | Verification retains the two failing pre-source-fix browser attempts, the unrelated full-suite 10-test failure, source hashes, submodule pin, and delivery limits. | This child does not recertify the parent screen bundle. |
| Repair the source package and deliver its version separately. | MET | Upstream hook negative 2 failed then positive 2/0; React 58/0 and publish-facing 203-export gate passed. PR #20 carries source/compatibility at `0352c83`; PR #21 carries generated rc.2 at `5afa07b`. | No npm publication, tag, or dist-tag occurred. |

## Artifact quality summary

| metric | value |
|---|---|
| Changes with artifact QA | 1/1 |
| First review | BLOCK |
| Corrected review | critic PASS; judge PASS |
| Refinement iterations | 3 |
| Final blocking constraints | 5 satisfied |

The recurring lesson is that schema validity is not evidence chronology. A
checkpoint must capture the state at its named phase, not merely parse as the
same schema. The artifact retains both first-review BLOCK verdicts and the
specific corrections instead of presenting a first-pass success.

## Technical debt and risk

- `pnpm test` remains red in two unrelated A2UI Storybook files: 328 tests
  passed and 10 failed. This child did not modify or claim those cases.
- EventSource recovery resumes future received events only. Durable replay of
  the disconnected interval would require a server cursor/checkpoint contract.
- Upstream rc.2 remains an open generated version PR and is not available from
  npm until the project publishes it.

## Handoff

Resume the outer `screen-by-screen-validation` change. Re-run its certification
against committed UAR source `6cc69cfd` (or the later child-closure commit),
submodule `0352c83`, fresh processes, and the corrected dependency-aware BDD
preparation command. Do not reuse the pre-child certification bundle.
