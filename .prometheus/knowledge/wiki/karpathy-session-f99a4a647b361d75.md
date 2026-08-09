---
type: SessionRecord
id: karpathy-session-f99a4a647b361d75
title: Karpathy session f99a4a647b36
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T19:58:56.741583+00:00
created_at: 2026-08-07T19:58:56.741583+00:00
updated_at: 2026-08-07T19:58:56.741583+00:00
revision: 0
---

## Delta

`embedded-mobile` check complete. It found something bigger than the capability failure it was aimed at.

## The structural finding

**[V] `embedded-mobile = ["host-persistence"]` — nothing else.** It doesn't enable `server`, and `src/lib.rs:40` gates `pub mod server` behind `#[cfg(feature = "server")]`.

> **The entire 124-route denominator does not exist on this profile.**

This is worse than "routes behave differently per profile." On the platform KnowMe ships to mobile there are **no routes at all** — every capability reaches the consumer through the Rust library API, the exact surface `compatibility-policy.md:16` says is *not* a contract. GAP-04 stops being a policy nicety; on mobile it's the whole contract.

A route-based measure scores this profile 0/124 or 124/124 depending on whether you notice. Neither number means anything.

## Results

| Check | Result |
|---|---|
| Compiles | **PASS** — clean, 8m37s |
| **Builtin skills (GAP-05)** | **FAIL — confirmed** |
| SSRF guard | **PASS** (with a correction) |
| Persistence | **PASS** |
| Host remediation path | **PASS** |

**GAP-05's mechanism is now precise.** `register_builtins` is called only at `server.rs:436`, and since `server` isn't compiled here, **that call site is absent from the binary** — a compile-time exclusion, not a runtime ordering bug. Capability C-07 sits at 0% here while all 124 routes stay "present and correct" elsewhere.

But the remedy exists: `SkillService::register_builtins` is `pub` and the SDK exposes `Runtime::native_skills()` — KnowMe uses exactly that. So the honest restatement is **"builtins are not registered by default on embedded profiles; the host must do it explicitly, and this is undocumented"** — not "skills are broken on mobile."

## A correction I caught on myself

The compile surfaced dead-code warnings for `MAX_BODY_BYTES` and `MAX_REDIRECTS` in `fetch_guard.rs`. My first inference was that SSRF protection might be inert here — **that was wrong**, and I verified before recording it. `web_fetch.rs` enforces both independently: `redirect::Policy::none()` at :193 (commented as stopping `169.254.169.254` redirects) and a `max_size_kb` cap at :239.

Actual finding, minor: those constants have no consumer on *any* profile and `TooManyRedirects` is never constructed. Vestigial, not a hole — worth deleting so the warning stops implying one.

## Coverage I did *not* achieve

Memory, knowledge/RAG, model routing, A2UI, and tenant isolation were **not** exercised. A compile check plus call-site tracing isn't a behavioral test — that needs an embedded harness that constructs a `Runtime` and drives it. Worth building before certifying the remaining three profiles.

One new failure class for the method: **"default-off with an undocumented opt-in"** is invisible to both route enumeration and module inventory. It only surfaced by tracing a call site against a feature gate.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T19:53:46.206465Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
