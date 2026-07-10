# BDD Chat Scenario Registry

Checked-in record of the behavior-driven chat scenario suite
(`tests/bdd/`) — what's covered, current pass/fail status, and where the
video-proof evidence lives. Generated per `bdd-chat-scenario-suite`
(`uar-production-ready-uiux-2026-07`, Round 3). Run locally with
`pnpm test:bdd` (root `package.json`).

Last run: 2026-07-10, commit `cf37653`. **5/6 scenarios pass.**
Certification bundle: [`docs/certifications/bdd-chat/cf37653/report.html`](certifications/bdd-chat/cf37653/report.html).

## Scenarios

| # | Scenario | Feature file | Status |
|---|----------|--------------|--------|
| 1 | Plain chat with no knowledge base | [`chat-no-kb.feature`](../tests/bdd/features/chat-no-kb.feature) | ✅ PASS |
| 2 | Retrieval-influenced response (KB enabled) | [`chat-kb-retrieval.feature`](../tests/bdd/features/chat-kb-retrieval.feature) | ❌ **FAIL — confirmed real product bug, see below** |
| 3 | Skill visibly activates mid-conversation | [`chat-skill-activation.feature`](../tests/bdd/features/chat-skill-activation.feature) | ✅ PASS |
| 4 | Tool call invoked and result surfaced | [`chat-tool-call.feature`](../tests/bdd/features/chat-tool-call.feature) | ✅ PASS |
| 5 | Agent selection / mid-session switching | [`chat-agent-switching.feature`](../tests/bdd/features/chat-agent-switching.feature) | ✅ PASS |
| 6 | Provider/model configuration affecting the answering model | [`chat-model-routing.feature`](../tests/bdd/features/chat-model-routing.feature) | ✅ PASS |

## Confirmed real bug: KB retrieval never influences a response

**Scenario 2 is failing for a real reason, not a test defect — left failing
deliberately rather than weakened.**

Ingesting a document containing a distinctive phrase, then asking a
question only answerable from that phrase, produces a response with **no**
retrieved content — verified by inspecting the actual outgoing system
prompt via `stub-llm`'s `/_stub/requests` introspection endpoint (the
prompt contains only the agent's base system prompt, no
`[RELEVANT KNOWLEDGE]` section).

Confirmed independently of the chat layer: `POST /api/knowledge/{id}/search`
with the ingested document's exact phrase as the query returns
`{"results":[]}`, even though the document's ingestion status reaches
`indexed` successfully.

Root cause: `VectorMatcher::embed_batch` (`src/uar/runtime/matching/vector.rs`)
returns placeholder zero-vector embeddings — `model.forward()` is still
commented out — a previously-flagged, still-open bug (tracked informally
as `task_188b4179`, owned by a separate session). Zero-vector cosine
similarity never clears the 0.7 match threshold in
`src/uar/runtime/manager.rs`'s RAG search, so `search_knowledge`/
`search_knowledge_scoped` always return empty, and the KB system-prompt
block is never appended.

**Not fixed here** — real embedding-pipeline work, out of this
test-infrastructure-only change's scope. Follow-up: fix
`VectorMatcher::embed_batch` and re-run `chat-kb-retrieval.feature` to confirm.

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
