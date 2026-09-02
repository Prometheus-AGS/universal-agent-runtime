---
type: SessionRecord
id: karpathy-session-174d195e4c3f81a7
title: Karpathy session 174d195e4c3f
tags:
- karpathy
- session-learning
sources:
- session:473cad21-bddf-4449-8516-e75e1b8704c0
timestamp: 2026-09-01T16:08:23.435636+00:00
created_at: 2026-09-01T16:08:23.435636+00:00
updated_at: 2026-09-01T16:08:23.435636+00:00
revision: 0
---

## Delta

`HEL` → `LO` → ` FROM` — the local model streaming through UAR, token by token. `delta` is a nested object, not a string.

## The spike's core goals are proven end to end

**A complete agent turn ran through UAR against ferrox, locally, with no external connection.** Resolved model: `ferrox-local / Qwen2.5 Coder 7B Instruct`. Terminal event `agui.done`, no errors.

**R-1 is answered — and the answer is not what the question assumed.** The `provider/model` string was never the obstacle. The real mechanism is:

1. **The `llm:` block does not select the model.** I configured it, restarted, ran — and the run resolved to `kimi-for-coding / k3` anyway, failing on a Kimi quota error. That failure was the finding.
2. **The `providers:` catalog is what runs resolve against**, but catalog *order* doesn't decide either — I added ferrox first and it still chose Kimi.
3. **`provider_registry.default_id()` decides**, and it's set at runtime via **`POST /api/providers/{id}/default`**. One call, and the next run used ferrox.

That's a genuinely important architectural fact for ADR-010: **local-vs-remote switching is a runtime API call, not a config-file edit** — which is *better* for the app (no restart, no file writing) and would have been invisible without running it.

**Also found along the way:**

- **The tool-call parser defect I fixed earlier is real and now proven fixed** — `finish_reason: "tool_calls"` with proper OpenAI-shaped calls.
- **A route change in the new UAR build**: `/api/uar/agents` → `/api/agents` (404 vs 200). Our `registration.ts` targets the old path and will break.
- **UAR binds IPv6 `[::1]` only** — `127.0.0.1` probes silently fail. Worth knowing for the sidecar health check.
- **UAR logs nothing to stdout**, even with `RUST_LOG=info` — the old binary behaves identically, so it's not the new build.

Two things I got wrong and corrected mid-flight: I twice diagnosed the silent startup as a failure when it was working (once an IPv4/IPv6 mismatch, once `timeout` killing it early), and my SurrealDB-lock hypothesis was wrong — the server was running.

**Outstanding from your instruction:** you asked me to delete the UAR `target/` directory after building. I haven't yet — the `rm -rf` deny rule blocks me. It's ~15-20 GB:

```bash
cd /Users/gqadonis/Projects/prometheus/universal-agent-runtime && cargo clean
```

Next in the spike: tool calling *through UAR* against the code map (goal 4), then the 7B measurements (goal 5) and orphan behavior (goal 6). Want me to continue?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 473cad21-bddf-4449-8516-e75e1b8704c0
- Captured: 2026-09-01T16:08:16.584783Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
