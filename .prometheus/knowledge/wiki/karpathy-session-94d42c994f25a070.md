---
type: SessionRecord
id: karpathy-session-94d42c994f25a070
title: Karpathy session 94d42c994f25
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T14:53:23.502713+00:00
created_at: 2026-08-07T14:53:23.502713+00:00
updated_at: 2026-08-07T14:53:23.502713+00:00
revision: 0
---

## Delta

This changes the picture substantially, and it means my earlier advice was wrong in an important way.

## `max_retries` semantics — now verified

`attempt` starts at 0, and `should_retry` returns `None` when `attempt >= max_retries`. So:

| Value | Behavior |
|---|---|
| `0` | 1 attempt, no retries |
| `1` | 1 attempt + **1 retry** |
| `3` (current) | 1 attempt + **3 retries** |

So `max_retries = 1` **does** give you a real retry — your concern about "effectively no retries" would apply to `0`, not `1`. That resolves the ambiguity in your favour as a question, but against your worry as a fact.

## The more important finding: this retry never fires for your problem

Look at line 34 of `with_retry`:

```rust
let resp = send().await?;    // ← `?` propagates transport errors immediately
```

Transport errors — **timeouts, connection resets, DNS failures** — hit that `?` and return straight out. They never reach `should_retry`. And `should_retry` only retries on `429 | 500 | 502 | 503 | 504`.

So `max_retries` governs **HTTP-level rejections only**: rate limits and upstream 5xx. It does nothing for:

- the 300s timeout you actually observed
- a dropped connection
- "a simple communications issue that resolves itself" — the exact case you want covered

**The 300s→600s escalation in your screenshot is not this retry.** That's the adversarial-review skill's own logic (`dispatch-judge.sh`), operating a layer above liter-llm. Two independent retry mechanisms, and I conflated them when I estimated a "35-minute worst case." That estimate was wrong.

## What this means for your decision

Lowering `max_retries` would **not** shorten the stall you saw, because that stall came from the skill's timeout escalation, not from liter-llm's retry budget. It *would* reduce tolerance for 429s — and with two subscription-based vendors (Kimi, MiniMax), 429s are the one thing you genuinely want retried.

My recommendation reverses: **leave `max_retries = 3`.** It's doing useful work on exactly the failure class you care about, and it isn't the source of the latency.

The latency lever is the packet size (240KB, 89% raw diff) and, if you want it, the skill-level timeout escalation in `dispatch-judge.sh` — a different file from the one I was about to edit.

Want me to look at how `dispatch-judge.sh` decides its 300s/600s windows before we change anything? That's where the actual stall lives, and I'd rather read it than guess twice.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T14:49:28.647271Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
