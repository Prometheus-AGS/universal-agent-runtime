---
type: SessionRecord
id: karpathy-session-9d35b0b664d31f45
title: Karpathy session 9d35b0b664d3
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T18:38:42.311579+00:00
created_at: 2026-08-07T18:38:42.311579+00:00
updated_at: 2026-08-07T18:38:42.311579+00:00
revision: 0
---

## Delta

Phase 1 complete. Here's what the triangulation found — and it substantially changes the question you asked.

## The three consumers demand three different things

| | BossFang | San Saba | KnowMe |
|---|---|---|---|
| **How it consumes UAR** | LLM driver only (1 file) | Not at all — security-fenced | **Links UAR as a Rust library** |
| **Live traffic today** | OpenAI-compat `/v1` via a *fallback* | none | embedded kernel + HTTP SDK |
| **Needs knowledge/RAG, memory, skills?** | No — has its own | Not exercised | **Yes, all of it** |

That last row resolves the question I'd flagged as premature: those large modules aren't vestigial, they're **KnowMe's**. A denominator built from BossFang and San Saba alone would have wrongly marked them dead.

## Four findings I verified against UAR's own source

**1. `GET /.well-known/uar-runtime` does not exist.** BossFang's k8s readiness probe (its *newest* artifact, 2026-07-30) gates on it plus three capability tokens. UAR has `/.well-known/uar-config`, and of the three tokens only `a2ui.registry` appears anywhere. **BossFang pods cannot reach Ready against this build.** Possibly a rename rather than missing work — worth checking before scoping it as new.

**2. San Saba's blockers are real.** `src/config.rs:284` is a shared `jwt_secret` with no JWKS/RS256; `a2a/task_store.rs:16` is a flat `HashMap<String, Task>` with no tenant key. These gate San Saba adopting UAR *at all*.

**3. A documented contract violation.** `docs/compatibility-policy.md:16` states the Rust library API is **not** a public compatibility contract. KnowMe imports ~30 UAR Rust paths and pins four feature flags. Either the policy is wrong or KnowMe is knowingly outside it — and for the denominator it means **the Rust API is a real contract surface**, not just HTTP.

**4. A parity bug, confirmed in UAR's source.** `register_builtins` is called only from `server.rs` — the HTTP composition root. Embedded consumers boot with an empty skill registry. KnowMe documented this; UAR's code confirms it.

## What this means for the denominator

It isn't one surface, it's **three**: the HTTP API, the Rust library API, and the protocol contracts (AG-UI events, ContentBlock kinds, A2UI). San Saba's AG-UI union is a compile-time exhaustive switch — an unknown event is a *build failure*, not a dropped frame. That makes it the strictest authority on event vocabulary.

Also worth flagging: San Saba's docs say the TS types should be generated from `gen_ui_core` / `@prometheus-ags/gen-ui-react`, **which don't exist** — so they hand-authored a mirror. UAR's own migration plan references that same package.

Moving to Phase 2 — reconciling this into the capability denominator with should-have-UI classifications. I'll bring you the manifest before anything moves.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T18:33:29.099967Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
