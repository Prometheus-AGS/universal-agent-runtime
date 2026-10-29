---
type: SessionRecord
id: karpathy-session-303fabb0875d1079
title: Karpathy session 303fabb0875d
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T14:23:52.473869+00:00
created_at: 2026-10-28T14:23:52.473869+00:00
updated_at: 2026-10-28T14:23:52.473869+00:00
revision: 0
---

## Delta

I found the root cause of both, and changed nothing. One correction to what I told you before: the timeout is not about restarts.

## 1. The model call that times out

**Cause:** UAR allows 15 seconds for the provider to start answering, makes only one attempt, and has no way to wait out the liter-llm proxy's longer retry window.

- **The limit:** `resilience.stream_start_timeout_ms` = 15000. It covers everything up to the proxy's response headers (`orchestrator.rs:161-173`).
- **No retry in UAR:** UAR caps its own retries at `min(retry_max_attempts, the model's max_retries + 1)` (`orchestrator.rs:2325-2331`). That cap was a deliberate decision on 2026-10-17. This fixture sets `llm.max_retries = 0`, so the result is exactly one 15-second attempt, even though the resilience settings say 3 attempts.
- **The proxy:** liter-llm has a 120-second timeout and up to 3 retries of its own. A slow MiniMax response, or a retry inside the proxy, can easily take longer than 15 seconds before UAR sees any headers. UAR then gives up, and the user gets an error.
- **Not about restarts:** today's two timeouts happened at iteration 1 of one run and iteration 2 of another, so my earlier "first call after a restart" claim was wrong.
- **Unverified:** whether the slow part was MiniMax or the proxy retrying. The proxy hasn't written any logs today.

## 2. "Remember this" ending in `PROTECTED_HISTORY_OVERFLOW`

**Cause:** the default context strategy keeps the last 20 messages, and it applies even within a single turn. It is not allowed to drop a tool call, so a turn that grows past 20 messages fails.

- **The name misleads:** this isn't about size. The reducer checks, byte for byte, that every tool call and its result survive trimming. If the window would drop one, the run fails instead (`reduce.rs:120-131`).
- **The setting:** `sliding_window` with `max_messages: 20`, and its source is `Default`, so this is the product default, not something a test left behind.
- **What happened in the failed run:** this fixture's Cedar policy rejected `memory_save`. The model then looped for 9 tool iterations within one reply, retrying the save, checking the policy and listing memories, and reached 25 or more messages. Preparing the 10th step meant dropping the first `memory_save` call, so the run failed.
- **Why it matters beyond this fixture:** with default settings, any turn that needs about 10 or more tool calls would fail the same way. The governance loop only made this one get there fast.

Both are recorded in `.prometheus/session-log.md`.

**What a fix involves:**
- **Timeout:** a configuration choice about which layer owns retries. Either the model gets a retry budget so UAR retries timeouts, or the start limit goes above the proxy's retry window.
- **Window:** a product code change. The window must not cut into the current turn's tool calls, so it would go through the normal plan process.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T14:23:35.797768Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
