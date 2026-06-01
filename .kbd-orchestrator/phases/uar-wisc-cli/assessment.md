# Assessment — `uar-wisc-cli` (salvage assessment of `origin/feature/providers`)

- Generated: 2026-05-29
- Author: claude-code (`/kbd-assess`)
- Backend: openspec
- Scope: forensic read of `origin/feature/providers` vs current `main`; determine
  what was being built, whether it had value, and whether `main` already achieved
  it another way.
- Discipline note: sycophancy-correction applied **in both directions** — the
  initial dismissal was wrong, but this assessment also resists over-correcting
  into "merge everything." Findings are stated against the actual code on both
  sides, not against either prior position.

---

## Correction of the record

My earlier verdict ("don't merge, the only novel thing is a CLI, the provider
system was likely superseded by liter-llm") was **based on file names and commit
subjects, not the code.** Having now read `wisc.rs` (671 lines), `providers/mod.rs`,
`credential_resolver.rs`, and `encryption.rs` on the branch, and compared them
against `main`, that verdict was **wrong on the substance of the provider system**
and **only half-right on the CLI.** Detail below.

The branch was doing **two distinct things**, not one:

1. `uar-wisc` — a WISC (Write / Isolate / Select / Compress) context-management
   CLI for coding agents.
2. A **multi-tenant, multi-scope provider & credential system** (`src/providers/`)
   — runtime models.dev catalog sync + per-user AES-256-GCM-encrypted BYO API
   keys + a 5-level scoped credential resolution chain + REST API.

These have very different salvage verdicts.

---

## Finding 1 — WISC CLI (`uar-wisc`)

### What it was trying to do
Expose UAR's memory stack to coding agents (Claude Code, etc.) as ergonomic shell
commands, organised around the WISC mental model:

| Phase | Commands |
|-------|----------|
| **W**rite | `remember`, `decide`, `resolve`, `checkpoint` |
| **I**solate | `research` |
| **S**elect | `recall`, `prime`, `scout`, `graph {search,expand,path,read}` |
| **C**ompress | `handoff`, `compact` |
| (status) | `status`, `streams` |

"Same binary + env vars = any agent identity, shared memory" — agent identity is
just `UAR_AGENT_ID`, memory is scoped per-agent in shared SurrealDB.

### Did `main` achieve it another way? — **MOSTLY YES, via MCP.**
Every underlying primitive the CLI composes **already exists on `main`** and is
exposed through `src/uar/memory/mcp_server.rs` — an in-process **memory MCP server**
with 18+ tools:

| WISC command | Equivalent on `main` |
|---|---|
| `remember` | `memory_add` |
| `recall` | `memory_search` (hybrid BM25+vector) |
| `decide` | `memory_add` + `kg_create_entity` + `kg_create_relation` (composed) |
| `resolve` | `memory_add` (procedural) |
| `checkpoint` | `task_stream_create` + `task_stream_add` |
| `research` | `memory_add` |
| `graph *` | `kg_search` / `kg_expand_neighbors` / `kg_find_path` / `kg_read` |
| `handoff` | `memory_add` + `memory_compress` |
| `compact` | `task_stream_auto_summarize` |
| `status`/`streams` | `memory_list` + `task_stream_list` |

For the **Claude Code use case this is the better architecture** — MCP tools are
typed, discoverable, and run in-process without spawning a subprocess per call.
So the CLI surface itself is largely redundant *for MCP-capable agents*.

### What is GENUINELY NOT replicated on `main`
1. **`scout`** — filesystem signature extraction (pub fns/structs/traits for
   `rs`/`ts`/`tsx`/`js`/`py`/`go`). `wisc.rs:525` explicitly comments this is
   *"the one piece not already in UAR."* Confirmed true. (Overlaps with Claude
   Code's native file tools, so its value is for non-Claude-Code agents or as an
   MCP tool, not for Claude Code itself.)
2. **Composite recipes** the raw MCP tools don't bundle:
   - `decide` = one call → episodic memory + KG entity + `DECIDED_IN` project relation.
   - `prime` = token-budgeted assembly of {default queries + decisions + TaskStream context} for one model.
   - `handoff` = structured memory (importance=1.0) + compress + optional `.claude/handoff.md`.
3. **The WISC framing as documented workflow** — a mental model, not code.

### Verdict — Finding 1
**Low–medium salvage value.** The capability exists via MCP. Worth porting only:
`scout` (if non-MCP agents matter) and the `decide`/`prime`/`handoff` composite
recipes (as either thin MCP tools or a small CLI over current APIs). The 671-line
branch binary should **not** be merged — it targets older library signatures.
Re-author against today's `MemoryService` if pursued.

---

## Finding 2 — Multi-scope provider & credential system (`src/providers/`)

### What it was trying to do (`providers/mod.rs` verbatim intent)
- Runtime **TOML sync** of the models.dev submodule (96 providers, ~3000 models).
- **Per-user BYO API keys, AES-256-GCM encrypted at rest** (`encryption.rs`:
  `base64(nonce ‖ ciphertext)`, key from `CREDENTIAL_ENCRYPTION_KEY`).
- **5-level scoped credential resolution** (`credential_resolver.rs`):
  `session → agent → user → system → env var → None`.
- **REST API** `/api/providers`, `/api/models`.
- Backend-agnostic catalog store (Postgres **and** SurrealDB impls).

This is a **multi-tenant SaaS capability**: end users bring their own encrypted
provider keys, resolved by scope at request time.

### Did `main` achieve it another way? — **PARTIALLY. The core of it: NO.**

| Sub-capability | `main` today | Achieved? |
|---|---|---|
| Model/provider **catalog** | `src/llm/catalog.rs` — models.dev catalog embedded at **build time** via `build.rs` (`provider_catalog.json`) + liter-llm registry | ✅ (static, compile-time) |
| Provider **routing/drivers** | `src/llm/` — `liter_driver`, `anthropic_driver`, `orchestrator`, `registry`, `router` (liter-llm, 142+ providers) | ✅ |
| Provider **key resolution** | `src/config.rs` — precedence over a **single process-wide key**: CLI > `UAR_LLM__*` > `LLM_*` > provider-env > config file | ⚠️ single-tenant only |
| **Per-user encrypted BYO keys** | `src/session/encrypted.rs` is an explicit **`stub`** ("full implementation pending") | ❌ **NOT BUILT** |
| **Scoped resolution chain** (session→agent→user→system→env) | none — one key per process | ❌ **NOT BUILT** |
| **Runtime** catalog mutation/sync | catalog is compile-time embedded; can't add a provider without rebuild | ❌ (by design) |
| Provider/credential **REST API** | none | ❌ |

> ⚠️ `src/uar/security/api_keys.rs` exists but is **unrelated** — it's a PAT→JWT
> auth pattern (user authenticating *to UAR*), **not** BYO provider keys
> (user→OpenAI/Anthropic). I conflated these in my first pass; they are different.

### Verdict — Finding 2
**This is the real reason the branch existed, and `main` did *not* achieve it
another way.** liter-llm solved provider *routing*; `build.rs` solved the *catalog*;
but **multi-tenant per-user encrypted credential storage with scoped resolution
is genuinely unbuilt** — `session/encrypted.rs` being a stub is the proof. The
branch's `encryption.rs` + `credential_resolver.rs` + catalog store are a
thoughtful, self-contained design for exactly this gap.

---

## DECISION (resolved 2026-05-29 by user): **BOTH**

UAR must support **both** single-tenant/self-hosted **and** multi-tenant SaaS.

This does not create a fork — the branch's scoped resolution chain already encodes
both modes in one design:

```
session → agent → user → system → env var → None
```

- **Single-tenant**: no per-user keys exist; every request falls through to the
  **env-var step** → identical to today's `src/config.rs` precedence behaviour.
- **Multi-tenant**: user/session credentials resolve first; the env-var step
  becomes the operator's house-account fallback.

Therefore the multi-tenant encrypted-credential subsystem (G6+G7+G8) is **layered
on top of** the existing single-tenant model, not a replacement. Finding 2 is
**confirmed high-priority salvage.**

## Gaps vs. goals (structured)

| # | Capability | On `main`? | Salvage priority |
|---|------------|-----------|------------------|
| G1 | WISC primitives for agents | ✅ via memory MCP server | n/a (done) |
| G2 | `scout` filesystem signature extraction | ❌ | Low (Claude Code has own file tools) |
| G3 | `decide`/`prime`/`handoff` composite recipes | ❌ (primitives exist, not bundled) | Low–Med |
| G4 | Provider routing (142+ providers) | ✅ via liter-llm | n/a (done, better) |
| G5 | Model catalog from models.dev | ✅ build-time embed | n/a (done) |
| G6 | **Per-user AES-256-GCM BYO credentials** | ❌ (`encrypted.rs` is a stub) | **High IF multi-tenant** |
| G7 | **Scoped credential resolution chain** | ❌ | **High IF multi-tenant** |
| G8 | Runtime provider/credential REST API | ❌ | Med IF multi-tenant |

## Recommendation

1. **Do not merge `origin/feature/providers` wholesale** — it's branched from a
   pre-liter-llm `main`, carries regressing dep pins (`axum-test 18.4.1`, old
   surreal-memory rev, no `uar-jwt-proxy` workspace member) and a duplicate
   provider catalog that conflicts with today's `build.rs` approach. That part of
   the original verdict stands.
2. **Preserve the branch — do NOT delete it** until the decision below is made.
   It is the only copy of the encrypted-credential design.
3. **Make the deployment-model call (single- vs multi-tenant).** If multi-tenant,
   open a change to port **G6+G7 (+G8)** — `encryption.rs` + `credential_resolver.rs`
   + a SurrealDB catalog store — onto current `src/llm/` + `src/uar/security/`.
   This is the high-value salvage.
4. **Optionally** port `scout` (G2) and composite recipes (G3) as MCP tools if
   non-Claude-Code agents are in scope. Low priority.

## Honest self-correction summary
- ❌ Earlier: "the provider system was likely superseded by liter-llm." — **False.**
  liter-llm superseded *routing*, not *multi-tenant encrypted credentials*. That is a real gap.
- ✅ Earlier: "don't merge the stale branch wholesale; re-author on current APIs." — **Still correct.**
- ⚠️ Earlier: "the only novel code worth saving is the CLI." — **Understated.** The
  encrypted-credential subsystem is the more valuable artifact; the CLI is the more redundant one.
