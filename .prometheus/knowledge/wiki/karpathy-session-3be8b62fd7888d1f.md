---
type: SessionRecord
id: karpathy-session-3be8b62fd7888d1f
title: Karpathy session 3be8b62fd788
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-08T13:21:41.600211+00:00
created_at: 2026-08-08T13:21:41.600211+00:00
updated_at: 2026-08-08T13:21:41.600211+00:00
revision: 0
---

## Delta

Directly answering what you asked: **I was asking for the liter-llm gateway's API key** — and I shouldn't have.

## Why the ask was wrong

Handling a plaintext secret isn't something I should do. The right fix is that the tooling authenticates itself, which is what the MCP server exists for. Asking you to paste a key was me routing around broken tooling instead of naming it.

## What's actually broken

The `liter-llm` MCP server is configured but **not authenticated**:

```
authentication failed: login fail: Please carry the API secret key
in the 'Authorization' field of the request header (1004)
```

That error text and the `1004` code come from an **upstream provider**, not the local gateway. So the gateway on `:4000` is running fine — the provider key it forwards upstream is missing or expired. My earlier `curl` 401s were the same problem seen from outside.

Fixing it is yours (it needs a secret): the MCP server needs its provider key in the environment that launches it. Then `/mcp` in an interactive session would show it healthy, and critic/judge calls would work with no key passing through me.

## What I'm doing meanwhile

Running the review with **two isolated same-model subagents**, each given only `method.md` and no conversation history.

This keeps the property that matters — **E-2 artifact-only isolation**, so the producer isn't the sole judge. It loses **model diversity**, which was the whole reason MiniMax-M3 and k3 were chosen: a critic with my weights has my blind spots.

I've recorded that degradation in the plan and marked the round for **re-run against the external reviewers** once the gateway is fixed. Treating a same-model pass as equivalent would be precisely the substitution error this session keeps catching.

Also confirmed: **D-7A is done** — instrumentation reverted, `src/` clean, zero compile errors.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-08T13:21:32.052509Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
