---
type: SessionRecord
id: karpathy-session-ebd3692638b0521e
title: Karpathy session ebd3692638b0
tags:
- karpathy
- session-learning
sources:
- session:b64196be-b5a5-4f19-9e1f-794edd060536
timestamp: 2026-08-08T17:59:15.958615+00:00
created_at: 2026-08-08T17:59:15.958615+00:00
updated_at: 2026-08-08T17:59:15.958615+00:00
revision: 0
---

## Delta

Clean: no orphaned cargo processes, and the only change is the diagnostic test I added.

## Investigation complete

All three questions answered, with the key claims verified rather than inferred.

**1. Does `memory.enabled: true` reach `MemoryConfig`? Yes — confirmed by a passing test.**

`test_memory_enabled_from_config_file_reaches_memory_config ... ok` ([tests/config_integration.rs](tests/config_integration.rs)). It asserts `enabled`, `embedding_provider`, and `db_path` all survive the file → `MemoryConfig` path. The config layer is not the bug.

**2. What made `MemoryService` `None`? A Cargo feature gap.**

`surreal-memory` is declared `default-features = false` with no feature list ([Cargo.toml:309](Cargo.toml#L309)); `local-embeddings` is never enabled. So `EmbeddingProvider::Local` compiles to the `#[cfg(not(...))]` arm — a hard `anyhow::bail!` ([embeddings/mod.rs:80](vendor/git/surreal-memory-server/crates/surreal-memory/src/embeddings/mod.rs#L80)) — which fires before any DB connection. `server.rs` logs `error!`, sets `None`, and the endpoint 503s.

Verified via feature-resolution graph: active-feature crates render explicit `feature "…"` nodes (`surrealdb feature "kv-surrealkv"`); `surreal-memory` renders with none. `Cargo.lock` is *not* evidence — it lists optional deps regardless of activation, which is why candle/hf-hub appear while inactive.

**Wider than reported:** [service.rs:77](src/uar/memory/service.rs#L77) routes `embedding_provider` through a `_` catch-all to `Local`, so any value except exactly `"openai"`/`"cohere"` — including typos — hits the same bail. Memory is unreachable without an OpenAI or Cohere key, on every profile, while [docs/MEMORY_SYSTEM.md:265](docs/MEMORY_SYSTEM.md#L265) documents `"local"` as supported.

**3. Is the unexpanded placeholder the cause? No — an independent second bug.** [config.rs:72](src/mcp/config.rs#L72) does exact `${NAME}` replacement over set env vars, with no `:-default` parser, so the literal string reaches `Url::parse`. Its own doc comment concedes it: *"If env var is missing, leaves the placeholder unchanged by default."*

Plus a **latent harness bug**: `db_path` is contractually bare (the service prepends `surrealkv://`), but [harness.rs:173](tests/integration/live/harness.rs#L173) writes it scheme-qualified → `surrealkv://surrealkv:///tmp/...`. Masked by the feature bug today.

**Two premises in the report were wrong**, and both matter: the cited verification command can't fail as described because `capability_cases.rs`/`shape_only_c06` don't exist in this repo, and `ServiceNeeds { memory: true }` is dead code — every call site passes `default()`. No test has ever exercised the memory boot path, which is how this shipped.

## Not yet fixed — one decision is yours

Enabling `local-embeddings` pulls in candle-core, candle-nn, candle-transformers, tokenizers, and hf-hub. Since [Cargo.toml:307](Cargo.toml#L307) documents deliberately trimming that crate's optional features, I won't assume this exclusion was accidental:

- **Enable `local-embeddings`** — memory works offline, at that dependency weight.
- **Drop `"local"`** — turn the unsupported provider into a clear startup error instead of a silent 503, and fix the docs. Memory then requires an API key.

Once you pick, I'll implement that plus the three items that don't depend on it: `${VAR:-default}` support with a hard error on unresolved placeholders, the harness `db_path` fix, and a test that actually exercises `memory: true`.

Two incidental notes: this worktree had uninitialized submodules (I ran `git submodule update --init --recursive` — nothing built before that), and another Claude session in worktree `epic-haslett-221a37` is running `cargo check` against a shared target dir, so expect lock contention. That session's MCP config also carries live API tokens in plaintext process args, visible to any local `ps` — worth rotating if those are real.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: b64196be-b5a5-4f19-9e1f-794edd060536
- Captured: 2026-08-08T17:57:24.504939Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/worktrees/infallible-taussig-29db4f

## Changed Paths

- tests/config_integration.rs
