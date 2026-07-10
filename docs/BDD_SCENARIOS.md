# BDD Chat Scenario Registry

Checked-in record of the behavior-driven chat scenario suite
(`tests/bdd/`) — what's covered, current pass/fail status, and where the
video-proof evidence lives. Generated per `bdd-chat-scenario-suite`
(`uar-production-ready-uiux-2026-07`, Round 3). Run locally with
`pnpm test:bdd` (root `package.json`).

Last run: 2026-07-10 (post `fix-embeddings-fastembed`,
`uar-final-production-hardening-2026-07`). **6/6 scenarios pass.**
Certification bundle (5/6-era, pre-fix): [`docs/certifications/bdd-chat/cf37653/report.html`](certifications/bdd-chat/cf37653/report.html).

## Scenarios

| # | Scenario | Feature file | Status |
|---|----------|--------------|--------|
| 1 | Plain chat with no knowledge base | [`chat-no-kb.feature`](../tests/bdd/features/chat-no-kb.feature) | ✅ PASS |
| 2 | Retrieval-influenced response (KB enabled) | [`chat-kb-retrieval.feature`](../tests/bdd/features/chat-kb-retrieval.feature) | ✅ PASS (fixed — see below) |
| 3 | Skill visibly activates mid-conversation | [`chat-skill-activation.feature`](../tests/bdd/features/chat-skill-activation.feature) | ✅ PASS |
| 4 | Tool call invoked and result surfaced | [`chat-tool-call.feature`](../tests/bdd/features/chat-tool-call.feature) | ✅ PASS |
| 5 | Agent selection / mid-session switching | [`chat-agent-switching.feature`](../tests/bdd/features/chat-agent-switching.feature) | ✅ PASS |
| 6 | Provider/model configuration affecting the answering model | [`chat-model-routing.feature`](../tests/bdd/features/chat-model-routing.feature) | ✅ PASS |

## KB retrieval bug: FIXED (2026-07-10, `fix-embeddings-fastembed`)

Scenario 2 was deliberately left failing when this suite first landed: the
root cause was `VectorMatcher::embed_batch` returning placeholder zero-vector
embeddings (`model.forward()` never wired), which made every embedding
consumer — KB search, chat RAG, skill embedding matching, LocalEmbedding
intent — structurally unable to match anything.

Fixed by `fix-embeddings-fastembed` (`uar-final-production-hardening-2026-07`):
real local BGE-small-en-v1.5 inference (384-dim, CLS-pooled, normalized) via
`fastembed`, loading the repo's on-disk ONNX model + tokenizer — no network
at runtime. Verified three independent ways: unit tests (non-zero,
discriminative embeddings), a live-server ingest→`POST /api/knowledge/{id}/search`
round trip returning the phrase-bearing chunk (score ≈0.84 on the exact query
that previously returned `{"results":[]}`), and this scenario passing
unweakened. The formerly-`#[ignore]`d `rag_ingest_then_retrieve` integration
case is re-enabled and passing. Knowledge bases ingested BEFORE the fix carry
zero-vector chunks that can never match — searches over them now log an
explicit stale-index error; re-ingest affected documents.

## Other findings disclosed (not fixed here — see design.md for full detail)

- **Fixed as part of this change** (user-approved scope expansion): the
  agent-selector popover's list was permanently broken for *every* agent,
  not just test-created ones — `loadAgentsIntoGraph()` never populated the
  entity-management library's list index `useAgents()` read from. Fixed by
  migrating `frontend/src/entities/hooks/use-agents.ts` off the deprecated
  `useEntityView` to the same `useGraphStore`-selector pattern already used
  by `useModels()`/`useAgentsByStatus()`.
- Two dead CLI/env passthroughs in `src/config.rs`: `Cli::port`
  (`PORT` env) and `Cli::jwt_required` (`JWT_REQUIRED` env) are parsed by
  clap but never applied anywhere — use `UAR_SERVER__PORT` /
  `UAR_SECURITY__JWT_REQUIRED` instead.
- The outgoing LLM request's `model` field is sometimes bare
  (`"gpt-5.4-mini"`) and sometimes `"provider/model"` depending on the
  resolution path taken (`src/llm/registry.rs`'s `has_explicit_base_url`
  branch). Not a routing bug — an implementation formatting detail.
- Root `tests/e2e/playwright.config.ts`'s bare `cargo run` webServer
  command is latently ambiguous (4 binaries now exist, no `default-run`
  set in `Cargo.toml`) — pre-existing, unrelated to this change.
- `tests/e2e/rag.spec.ts` (root Playwright suite) only asserts a
  file-upload `<input>` is attached — never uploads a document or verifies
  retrieval. `frontend/e2e/chat-agent-selection.spec.ts` only asserts the
  agent-selector button is visible — never opens the popover, which is how
  the list-loading bug above went undetected until this suite. Neither is
  deleted by this change (out of scope); this suite supersedes them in
  spirit for the scenarios it covers.

## Re-running

```bash
pnpm test:bdd
```

Boots a deterministic stub LLM (`src/bin/stub-llm.rs`,
`tests/bdd/fixtures/bdd-chat.json`) and the real app against an ephemeral
SurrealKV store — no live provider API key required. See the
`bdd-chat-scenario-suite` OpenSpec change (`openspec/changes/archive/`) for
the full design rationale and decision log.
