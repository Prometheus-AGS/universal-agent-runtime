---
type: SessionRecord
id: karpathy-session-efa5587af99837fc
title: Karpathy session efa5587af998
tags:
- karpathy
- session-learning
sources:
- session:b64196be-b5a5-4f19-9e1f-794edd060536
timestamp: 2026-08-08T17:39:22.816035+00:00
created_at: 2026-08-08T17:39:22.816035+00:00
updated_at: 2026-08-08T17:39:22.816035+00:00
revision: 0
---

## Delta

Build still running (cargo alive, output buffered). The monitor is armed and will report. 

The investigation is complete regardless of that probe — it's a confirmatory check on the one question I answered by reading the loader, and my two primary conclusions were verified independently.

## Findings

All three investigation questions are answered.

**1. Does `memory.enabled: true` reach `MemoryConfig`? Yes.** The config layering is correct — the YAML file source is added at [config.rs:1090-1106](src/config.rs#L1090) and nothing overrides `memory.*`. This was not the bug. (A confirmatory test is still compiling; I'll report it when it lands, but the loader code is unambiguous.)

**2. What made `MemoryService` `None`? A Cargo feature gap — not config.**

`surreal-memory` is declared `default-features = false` with no feature list ([Cargo.toml:309](Cargo.toml#L309)), and `local-embeddings` is never enabled from UAR's side. So `EmbeddingProvider::Local` compiles to the `#[cfg(not(feature = "local-embeddings"))]` arm — a hard `anyhow::bail!` ([embeddings/mod.rs:80-86](vendor/git/surreal-memory-server/crates/surreal-memory/src/embeddings/mod.rs#L80)). That fires inside `MemoryService::new` before any DB connection, `server.rs` logs `error!` and sets `None`, and the endpoint returns 503.

Verified by feature-resolution graph: crates with active features render explicit `feature "…"` nodes (`surrealdb feature "kv-surrealkv"`); `surreal-memory` renders with **none**. Note `Cargo.lock` is *not* evidence here — it lists optional deps regardless of activation, which is why candle/hf-hub appear despite being inactive.

**The blast radius is wider than reported.** [service.rs:77-85](src/uar/memory/service.rs#L77) routes `embedding_provider` through a `_` catch-all to `Local`, so *any* value except exactly `"openai"` or `"cohere"` — including typos — hits the same bail. Memory is unreachable without an OpenAI or Cohere API key, on every profile. And [docs/MEMORY_SYSTEM.md:265](docs/MEMORY_SYSTEM.md#L265) documents `"local"` as supported, so docs contradict the build.

**3. Is the `${VAR:-default}` placeholder the cause? No — an independent second bug.** [config.rs:72-84](src/mcp/config.rs#L72) iterates set env vars doing exact `${NAME}` replacement, with no `:-default` parser. `UAR_MEMORY_MCP_URL` is unset, so the literal string reaches `Url::parse`. The doc comment concedes the design gap: *"If env var is missing, leaves the placeholder unchanged by default. (You can choose to error instead—recommended for prod.)"*

There's also a **harness bug** that would bite next: `db_path` is contractually a bare path (`MemoryService::new` prepends `surrealkv://` itself), but [harness.rs:173](tests/integration/live/harness.rs#L173) writes it scheme-qualified, producing `surrealkv://surrealkv:///tmp/...`. It's currently masked by the feature bug.

## Two corrections to the report's premises

- **The verification command cannot fail as described — that test doesn't exist.** There is no `capability_cases.rs` and no `shape_only_c06` in the repo. Related: `ServiceNeeds { memory: true }` is dead code; every call site passes `ServiceNeeds::default()`. No test has ever exercised the memory boot path, which is why this shipped. The cited reproduction came from outside this worktree.
- **The 503 is not a config-parsing failure**, so the fix doesn't belong in the config layer.

## I have not written a fix

Root cause is confirmed; implementation is the next phase and it needs your call, because the obvious fix is a real dependency-surface change:

Enabling `local-embeddings` pulls in candle-core, candle-nn, candle-transformers, tokenizers, and hf-hub. [Cargo.toml:307](Cargo.toml#L307) shows that crate's optional features were deliberately trimmed ("the optional palace graph extension is intentionally excluded"), so I won't assume the exclusion of `local-embeddings` was accidental rather than the same deliberate policy.

Which do you want?
- **Enable `local-embed

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: b64196be-b5a5-4f19-9e1f-794edd060536
- Captured: 2026-08-08T17:34:05.718519Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/worktrees/infallible-taussig-29db4f

## Changed Paths

- No changed paths detected.
