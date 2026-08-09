---
type: SessionRecord
id: karpathy-session-72495abcd4c97ac8
title: Karpathy session 72495abcd4c9
tags:
- karpathy
- session-learning
sources:
- session:b64196be-b5a5-4f19-9e1f-794edd060536
timestamp: 2026-08-08T18:23:59.010062+00:00
created_at: 2026-08-08T18:23:59.010062+00:00
updated_at: 2026-08-08T18:23:59.010062+00:00
revision: 0
---

## Delta

Still queued. The other session's build is long-running; mine will start when the lock frees, and the monitor will report.

All three fixes are implemented. Here's the state:

## Changes

**Bug 1 — the 503** ([Cargo.toml:309](Cargo.toml#L309)): enabled `local-embeddings` on `surreal-memory`, with a comment recording why it's mandatory rather than optional (the `_` catch-all routes every non-openai/cohere provider to `EmbeddingProvider::Local`, which was a hard `bail!`). `Cargo.lock` is unchanged, as expected — activating a feature on a path dependency turns on already-locked optional deps without altering resolution, so `--locked` still holds.

**Bug 2 — `${VAR:-default}`** ([src/mcp/config.rs](src/mcp/config.rs)): replaced the env-var-iterating string replace with a real scanner. Two entry points:
- `expand_env_placeholders` (lenient) — unchanged behavior for unresolvable placeholders, because the Tavily flow *depends* on `${TAVILY_API_KEY}` surviving process-env expansion so the URL resolver can substitute it from the entry's `env` map.
- `expand_env_placeholders_strict` — errors, naming every unresolved variable.

[registry.rs:72](src/mcp/registry.rs#L72) now runs the Tavily substitution first, then requires the result to be placeholder-free before `Url::parse`. That ordering preserves the Tavily indirection while making an unexpanded placeholder a startup error, as you specified. I verified both real `mcp.json` values parse correctly, including that the Tavily URL yields no default and so stays intact through the lenient pass.

Empty-var handling follows shell `:-` semantics (set-but-empty takes the default), and `:-` splits on the first occurrence so `http://host:port/path` defaults survive.

**Bug 3 — harness `db_path`** ([harness.rs:170](tests/integration/live/harness.rs#L170)): now writes a bare path, with a comment explaining that `MemoryService::new` adds the scheme.

**Tests**: 9 unit tests for the expansion parser (including the exact `surreal_memory` URL), the config-layer probe that already passed, and a new [memory_cases.rs](tests/integration/live/memory_cases.rs) — the first test anywhere to boot with `ServiceNeeds { memory: true }`. It asserts specifically that the response is *not* the 503, since that was the common symptom of all three defects.

## Not yet verified

The build is queued behind another session's `cargo check` on the shared package-cache lock — I'm honoring the repo's single-writer discipline rather than adding contention. Nothing is confirmed compiling or passing yet, so treat every claim above as implemented-but-unverified. I'll report the unit-test result when the monitor fires.

Two caveats worth flagging now:
- The new `memory_cases` test will download an embedding model on first run (candle's `EmbeddingProvider::Local` fetches lazily from HF Hub). That may make it slow or network-dependent in CI — you may want it gated behind a feature or marked ignored there.
- `example.config.yaml` still has no `memory:` section at all, so the newly-working `"local"` provider remains undocumented in the primary config example. I left that alone as out of scope; say the word and I'll add it.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: b64196be-b5a5-4f19-9e1f-794edd060536
- Captured: 2026-08-08T18:18:48.119825Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/worktrees/infallible-taussig-29db4f

## Changed Paths

- Cargo.toml
- src/mcp/config.rs
- src/mcp/registry.rs
- tests/config_integration.rs
- tests/integration/live/harness.rs
- tests/integration/live/mod.rs
- tests/integration/live/memory_cases.rs
