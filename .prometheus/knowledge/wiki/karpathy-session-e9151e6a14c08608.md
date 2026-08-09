---
type: SessionRecord
id: karpathy-session-e9151e6a14c08608
title: Karpathy session e9151e6a14c0
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-08T13:27:07.240199+00:00
created_at: 2026-08-08T13:27:07.240199+00:00
updated_at: 2026-08-08T13:27:07.240199+00:00
revision: 0
---

## Delta

Both reviews are in, and they changed the method substantially. Here's what happened.

## Verdicts

**Judge: INSUFFICIENT AS WRITTEN.** Q1 and Q3 sound; Q2 and the exclusion set unsound.
**Critic: fails the same way the killed version failed** — three fatal flaws, two verified against code.

## The finding that matters most — my fix was wrong

The judge said I hedged on whether OpenAPI was available instead of checking. I checked, found `/api/openapi.json` mounted at `server.rs:838`, and declared the route surface "authoritative, not grepped, not probed."

**The critic opened the file.** `src/uar/api/openapi.rs` is a 103-line hand-typed `json!` literal declaring **8 paths** against ~124 routes — **6%** — with **zero** `#[utoipa::path]` attributes anywhere. Its own comment says it exists to dodge "utoipa builder API version issues." It's a Swagger-UI demo.

I made the same mistake twice in an hour: **confirmed a thing exists, inferred it was fit for purpose, never opened it.**

## Two more verified fatal flaws

**The 404-probe returns 200 for missing routes.** `server.rs:1114` has `fallback_service(ServeDir…not_found_service(index.html))`, so any missing non-`/api` path returns **200 + HTML**. The critic's example is exact: `/a2ui/v1/surfaces/assemble` — spec-verified as GAP-08 orphaned — would score as UI-reachable.

**My test command targeted the wrong profile.** No `--features` means `minimal`, and I verified `a2a-transport`, `cedar-governance`, `document-intelligence`, `api-docs`, `response-quality` are all **absent** from it. Any `cfg`-gated capability test would silently not run — a false pass by absence, violating my own constraint.

## Q2 is now blocked, and correctly so

Numerator and denominator both come from "46 paths the UI calls," so Q2 **cannot detect a capability the UI fails to expose** — the entire question. That is verbatim the flaw that killed the first method: *omission reads as coverage*.

It needs a **spec-derived expected-UI-surface map**, authored from `SPECIFICATION.md` before looking at the frontend. Until then Q2 can only report a defect scan, never a coverage number.

## Two things I was getting wrong about honesty

**C-22 was missing from my exclusion list entirely** — it's `[V] ABSENT` in my own spec. And grouping C-23 (also ABSENT) under "excluded, needs two devices" **launders a known zero into "probably fine, couldn't check."** Meanwhile C-25/26/27 are BUILT with 47 tests and are testable *now*.

**The method silently measures no L4 at all** — the harness uses a fresh temp DB per boot with no shutdown hook. C-12 persistence and C-13 sessions are precisely where L3 without L4 is nearly worthless.

## One caveat on this review

Both reviewers were **same-model** — the liter-llm gateway is unauthenticated, so MiniMax-M3 and k3 were unavailable. Artifact-only isolation held; model diversity didn't. That's recorded, and the round should be re-run externally when the gateway is fixed.

Q1 can proceed with features pinned and results profile-labelled. Q2 needs the surface map first. Want me to author that map, or start Q1?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-08T13:27:00.317413Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
