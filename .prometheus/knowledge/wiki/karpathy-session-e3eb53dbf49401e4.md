---
type: SessionRecord
id: karpathy-session-e3eb53dbf49401e4
title: Karpathy session e3eb53dbf494
tags:
- karpathy
- session-learning
sources:
- session:b64196be-b5a5-4f19-9e1f-794edd060536
timestamp: 2026-08-08T16:37:22.880716+00:00
created_at: 2026-08-08T16:37:22.880716+00:00
updated_at: 2026-08-08T16:37:22.880716+00:00
revision: 0
---

## Delta

The monitor is already armed for the probe result. Let me summarize the confirmed findings while it completes.

Both bugs are now root-caused. Bug #2 needed no build to confirm — it's decidable from source.

**Bug #2 — `${VAR:-default}` never expands** ([src/mcp/config.rs:72-84](src/mcp/config.rs#L72)):

`expand_env_placeholders` iterates over *set* env vars and does exact `${NAME}` replacement. It has no parser for `:-default`. With `UAR_MEMORY_MCP_URL` unset, nothing matches, and the literal string reaches `Url::parse`. The doc comment states the silent-failure policy outright: *"If env var is missing, leaves the placeholder unchanged by default. (You can choose to error instead—recommended for prod.)"*

**Bug #1 — the 503.** Confirmed as `state.memory_service == None` ([memory_admin.rs:154-160](src/uar/api/memory_admin.rs#L154)), reached via the `error!` branch in [server.rs:362-381](src/server.rs#L362). Two independent defects make `MemoryService::new` fail with `embedding_provider: "local"`:

1. **`local-embeddings` is never enabled.** `surreal-memory` is declared `default-features = false` with no feature list ([Cargo.toml:309](Cargo.toml#L309)). `local-embeddings` appears *only* inside the dependency's own manifest. So `EmbeddingProvider::Local` compiles to the `#[cfg(not(feature = "local-embeddings"))]` arm, which is a hard `anyhow::bail!`. This fires before any DB connection and is fatal under every profile including `server-full`.

2. **The harness double-prefixes the scheme.** `MemoryService::new` builds `format!("surrealkv://{}", config.db_path)` ([service.rs:119](src/uar/memory/service.rs#L119)), so `db_path` is contractually a bare path — documented as `./data/memory.db` in five places including [website/docs/configuration.md:198](website/docs/configuration.md#L198). The harness writes `db_path: "surrealkv://..."` ([harness.rs:173](tests/integration/live/harness.rs#L173)), yielding `surrealkv://surrealkv:///tmp/...`.

Defect 1 masks defect 2 — fixing only the harness path would still 503.

Two corrections to the premises in the report, both of which change what needs doing:

- **The verification command cannot fail as described, because that test does not exist.** There is no `capability_cases.rs` and no `shape_only_c06` anywhere in the repo. Relatedly, `ServiceNeeds { memory: true }` is dead code — every call site in the suite passes `ServiceNeeds::default()` (memory: false), so no test has ever exercised the memory boot path. The "found by a live capability test" reproduction came from somewhere outside this worktree. I'd want to know where before writing a test to match its expectations.
- **`memory.enabled: true` does reach `MemoryConfig`.** The config layering is correct — the file source is added and nothing overrides `memory.*`. I've added a probe test to prove this rather than assert it; it's still compiling and I'll confirm when it lands.

One process note: this worktree had uninitialized git submodules, so nothing could build. I ran `git submodule update --init --recursive` to fix it.

I have not written any fix yet — that's Phase 4, and I wanted the root cause confirmed first. The shape of the fix is:

- Enable `local-embeddings` on the `surreal-memory` dependency (this is a real dependency-surface change pulling in candle/tokenizers/hf-hub — worth your call, given [Cargo.toml:307](Cargo.toml#L307) shows deliberate trimming of that crate's optional features).
- Fix the harness to write a bare `db_path`.
- Make `expand_env_placeholders` support `${VAR:-default}` and return an error for unresolved placeholders instead of passing them through, so a failed expansion is a startup error rather than a bad URL — as you specified.

Before I implement: do you want `local-embeddings` enabled, or should the memory path use a different embedding provider? That's a dependency-weight decision I shouldn't make unilaterally, and it determines whether the 503 fix is a one-line feature addition or a config-contract chang

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: b64196be-b5a5-4f19-9e1f-794edd060536
- Captured: 2026-08-08T16:32:13.983127Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/worktrees/infallible-taussig-29db4f

## Changed Paths

- No changed paths detected.
