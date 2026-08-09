---
type: SessionRecord
id: karpathy-session-64001463972d4e00
title: Karpathy session 64001463972d
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-08T15:50:15.418163+00:00
created_at: 2026-08-08T15:50:15.418163+00:00
updated_at: 2026-08-08T15:50:15.418163+00:00
revision: 0
---

## Delta

Q1 is complete — 18 capabilities exercised against a live server.

## Results: 13 passed, 5 failed — but that is **not** 5 defects

The five failures have five different causes. Reporting them as five capability failures would have overstated the runtime's problems by 67%.

| Cap | Observed | Actually |
|---|---|---|
| **C-20** metrics | thread panic | **Runtime defect** |
| **C-06** memory | `503 "not enabled"` despite config saying `enabled: true` | **Runtime defect (probable)** |
| **C-09** compiler | `500 "table 'uar_specs' does not exist"` | **Runtime defect** |
| **C-04** credentials | `401 Authentication required` | **My test was wrong** — `credentials.rs:86` guards unconditionally |
| **C-13** sessions | `404 legacy_route_disabled` | **My test was wrong** — `server.rs:2280` deliberately retires it |

**3 defects, 2 test errors.** Distinguishing them required reading the source for each; the tests alone couldn't tell me.

The discriminator earned its place again: **none of the five returned the catch-all sentinel**, so every route is genuinely mounted — they reject for real reasons rather than being absent.

## Three defects filed

1. **`/metrics` panics when embedded** — `metrics::init()` only in the two binaries, never in `start_server`, while `/metrics` is registered unconditionally. **No test or SDK path calls it**, and `sdks/rust/runtime.rs:723` re-exports `start_server` — so every embedder inherits it. Same shape as GAP-05.
2. **`/api/compiler/specs` → 500 on a fresh DB.** An empty catalog isn't an error; `200 []` is correct. Likely a missing migration, possibly a pattern across other list endpoints.
3. **Memory disabled despite `enabled: true`**, with an unexpanded `${UAR_MEMORY_MCP_URL:-…}` in the logs — config placeholder expansion failing.

## The pre-registration did its job

Six of nine predictions held. **Three misses**: I called C-20 the *safest* capability in the set, and never predicted C-06 or C-09 at all.

Those three defects are invisible to the spec — which is precisely the outcome the original route-presence method could never have produced. The 124 routes were all "present and correct."

## What remains unmeasured

C-16/C-18/C-19 have zero routes (probing them would manufacture findings). C-21 needs two tenants. No L4 anywhere, no semantics, nothing outside `server-full`.

Q2 still needs an externally-authored denominator. Q3 is untouched.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-08T15:48:42.272343Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
