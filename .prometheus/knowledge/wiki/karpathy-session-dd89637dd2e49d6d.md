---
type: SessionRecord
id: karpathy-session-dd89637dd2e49d6d
title: Karpathy session dd89637dd2e4
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T14:58:48.536099+00:00
created_at: 2026-08-07T14:58:48.536099+00:00
updated_at: 2026-08-07T14:58:48.536099+00:00
revision: 0
---

## Delta

I now have the facts to evaluate your three tiers honestly. Here's the assessment — including where I think one of them is a bad idea.

## What's actually on this machine

| Resource | Status |
|---|---|
| **Ollama** | Installed, running, OpenAI-compatible at `:11434/v1` |
| Local chat models | **`llama3.2:1b` only** — everything else is `:cloud` (Ollama-hosted, needs their auth) or embeddings |
| Configured keys | `KIMI_CODING_KEY`, `MINIMAX_KEY`, `LITER_LLM_MASTER_KEY` — all set |
| Unconfigured keys in env | `MINIMAX_API_KEY` (duplicate of what you have) |
| Local gateways | `liter-llm :4000`, `openai-proxy :8181` |

## Tier-by-tier

**Tier 1 — already-configured-but-unlisted models: strong, and you're nearly there.**

Your config already declares `k3-256k` and `kimi-for-coding` beyond the two role bindings, and `gpt-5.4` via `:8181`. These are known-good, already-authenticated, and cost nothing to discover. This is the cheapest tier and the one I'd build first.

The catch is **E-2 isolation**: `k3-256k` is the same vendor and family as `k3`. Falling back judge→`k3-256k` gives you a same-family judge, which your own `models.toml` calls "a FAILURE, not a fallback." A legitimate fallback chain needs to cross vendors: judge `k3` → `MiniMax-M3` → `gpt-5.4`, never `k3`→`k3-256k`.

**Tier 2 — Ollama / env-key discovery: weaker than it looks here.**

Discovery works, but the inventory doesn't. `llama3.2:1b` is a **1-billion-parameter** model. Asking it to adjudicate a 60K-token adversarial review isn't a degraded verdict, it's a meaningless one — and worse, it would return `PASS` with confident-sounding prose. A silent-but-wrong judge is more dangerous than no judge, because the pipeline treats a verdict as evidence.

The `:cloud` models (`kimi-k2.5`, `minimax-m2`, `deepseek-v4-flash`, `mistral-large-3:675b`) *would* be capable — but they're Ollama's hosted tier, requiring their own auth, and two of them are the same vendors you already use directly. Worth probing for reachability, but they're not free local capacity.

**Tier 3 — web-search for a free provider: I'd advise against this one.**

Three problems, in order of severity:

1. **It sends your source diff to an unvetted third party.** The packet contains proprietary code. A fallback that auto-discovers an unknown endpoint and posts a 60K-token diff to it is an exfiltration path chosen by an automated heuristic, at the exact moment your normal path is failing and nobody is watching closely.
2. **It undermines what the review is for.** A judge whose provenance was decided by a web search minutes earlier can't be reasoned about. Your architecture treats judge identity as load-bearing — `models.toml` records the generator specifically so collision checks can prove judge ≠ producer.
3. **Free tiers are aggressively rate-limited**, so the fallback would likely fail too, having spent time and leaked the packet.

If you want a "last resort," a far better one is: **write a `pending-review` receipt and stop.** Your skill already has that contract — the screenshot shows Codex reasoning about exactly this. An honest "not reviewed" is worth more than a verdict from an unknown model.

## What I'd actually recommend

**A declared fallback chain, not discovery.** Per role, in `models.toml`, with vendor-diversity enforced:

```
judge:  k3 (Moonshot) → MiniMax-M3 (MiniMax) → gpt-5.4 (:8181)
critic: MiniMax-M3    → k3
```

Failover triggers only on transport failure or 5xx after the existing retry budget — never on a `Rejected` verdict, so a real BLOCK can't be laundered by retrying elsewhere. Every fallback gets **recorded in the findings receipt**, so a review adjudicated by the second choice is visibly different from one adjudicated by the first.

Two supporting pieces:

- **Probe on preflight, not mid-failure.** `preflight-models.sh` already runs and caches for 24h. Have it verify each chain member answers `/v1/models`, so failover targets a known-live endpoint rather than discovering live

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T14:58:34.658255Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
