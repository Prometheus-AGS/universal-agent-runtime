# R10 action-leg receipt — 2026-10-19 ~17:05 UTC (v4, instance 1907, anonymous)

Candidate: v4 `96bf4868…5154` (running 1907, pid 24103, untouched).
Principal: anonymous, session `41732fd0-c23a-468f-a1d8-68484d5c26bd`
(x-uar-session-id header + cookie). No login, no admin key.

## Setup
Run `bbeac80f-bfbb-4256-9705-f8727b878ccf` (default-agent, status done,
model kimi-for-coding/k3) created via `POST /v1/chat/completions`
(reply "r10 probe ok"). A2UI surface ops driven through the no-login
`/api/uar/runs/{run}/a2ui/*` surface-test pipeline.

## Legs
1. **Render leg — PASS.** `surface-test-trigger` createSurface
   {"title":"R10 probe surface"} + updateComponents [btn1 button with
   action `r10-test-action`] → both ack `triggered`. `GET surface-replay`
   returned the exact durable ops (`add /a2ui/surfaces/r10-surface`,
   `replace …/components`), bound to run bbeac80f / default-agent /
   session 41732fd0. Receipts: r10-t1.json, r10-t2.json,
   r10-replay-before.json.
2. **Declaration enforcement — PASS.** POST action `not-declared` →
   **HTTP 422** `action is not declared by the active A2UI surface`
   (registry semantics enforced). Receipt: r10-action-neg.json.
3. **Action fires + state changes — PASS.** POST action `r10-test-action`
   (camelCase body: surfaceId/sourceComponentId) → durable replay gained
   `add /a2ui/actions/8a76222e-e376-4358-aeba-2f505954143e` with the
   interaction payload. State mutation observed via replay (3 ops).
4. **Continuation — truthfully refused (not phantom).** The same request
   returned **HTTP 403** `skill_forbidden` ("the verified principal cannot
   use a selected skill") from `continue_run` AFTER the action op was
   durably published (routes.rs:867-885 publishes before continuing).
   This is the R2(d) anonymous-skill governance boundary, not an A2UI
   defect; no phantom/ghost state appeared.

## Verdict contribution
R10 render + action/state legs: **PASS (class A, v4/1907)** — first
artifact-bound binding for this row (was PARTIAL/B). Continuation-reaction
leg (action → agent continuation → surface update): **UNVERIFIED** on the
anonymous surface — blocked by the same skill gate as R2(d)/R9, recorded,
not a new failure.

## OPERATOR CORRECTION ADDENDUM — 2026-10-19 ~17:45 UTC (supersedes the verdict line)

R10 stays **PARTIAL**. The action record (`add /a2ui/actions/8a76222e…`)
was published to the durable journal BEFORE `continue_run` returned 403
skill_forbidden — a journal append is not the registry-semantics,
callback-driven state change the frozen criterion requires, and none was
shown. Legs that hold: registry-backed render (replay of surface +
components ops) and action-declaration enforcement (422 negative).
Leg that does not: callback-driven state mutation (refused before any
such effect).
