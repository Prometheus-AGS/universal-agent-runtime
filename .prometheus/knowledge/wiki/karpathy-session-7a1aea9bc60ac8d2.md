---
type: SessionRecord
id: karpathy-session-7a1aea9bc60ac8d2
title: Karpathy session 7a1aea9bc60a
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T20:40:39.239167+00:00
created_at: 2026-08-07T20:40:39.239167+00:00
updated_at: 2026-08-07T20:40:39.239167+00:00
revision: 0
---

## Delta

That's a documented landmine, and it answers your question with more precision than the function counts.

## Yes — and the pattern already exists, but it's incomplete and unenforced

`sdks/rust` has exactly the two-surface structure you're describing:

| | Functions |
|---|---|
| `client.rs` — HTTP (server/cloud) | **71** |
| `runtime.rs` — embedded (mobile/desktop) | **60** |

So the answer to "do we need APIs equivalent to routes on mobile" is **yes, and they're ~85% built.**

## Three problems with how it's built

**1. No shared trait.** There's no `trait Runtime` that both implement — parity is maintained by hand. That's structurally why KnowMe reported three *shipped* parity bugs (the `{total, items}` memory shape, the bare-`RunPolicy`-vs-wrapper break across Rust/TS/Dart, and the empty `NativeSkillRegistry`). Nothing forces the two surfaces to agree.

**2. Capabilities genuinely missing on embedded:**

| Missing | Consequence |
|---|---|
| `checkpoints`, `resume`, `resume_at` | No run resumption on mobile — where process death is *most* likely |
| `upload_document`, `ingest`, `embeddings` | Can't add documents to a knowledge base on-device |
| `execute` (direct tool invocation) | — |
| `promote_component`, `list_components` | A2UI catalog unreachable |
| `search_with_options`, `setting_types` | — |

The checkpoint/resume absence is the sharpest: mobile is precisely where you need it.

**3. A documented landmine.** From `runtime.rs:459`:

> `PersistenceLayer::save_memory` and `search_memory` are documented **NO-OP stubs** on `SurrealDbProvider` that return `Ok(())` / `vec![]`. Routing writes there would compile, run, and **silently discard the data**.

Someone already found this and routed around it — memory goes through `MemoryService`, not `PersistenceLayer`. But the trap is still armed for the next person, and it's invisible to compiler, tests, and route inventory alike.

## Why this matters for the measurement

This reframes GAP-04. The Rust API isn't merely "used as a contract despite policy saying otherwise" — **on mobile it's the only interface**, it's 15 functions short of the HTTP surface, and nothing structurally enforces parity.

That also means the spec's §2.2 understates the problem, and the denominator needs a second axis: not just *"is the capability there"* but *"is it there on every surface a consumer reaches it through."*

Two things I'd want your read on before I write this up:

**Is embedded/HTTP parity a requirement, or is a deliberate subset acceptable?** If some capabilities are intentionally server-only, that's a legitimate design choice — but it should be *stated*, because right now the difference reads as accidental.

**Should the missing eleven be treated as gaps, or as a fourth "not in scope" category?** Checkpoint/resume on mobile feels like a real gap; direct tool `execute` might not be.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T20:40:16.584691Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
