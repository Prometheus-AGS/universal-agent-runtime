## Why

Every embedding consumer in the product — KB ingestion (`src/uar/rag/ingest.rs`,
`chunking.rs`), KB search (`src/uar/api/knowledge.rs:541`), agent-scoped chat
RAG (`src/uar/runtime/manager.rs:696`), skill embedding matching
(`src/uar/runtime/skills/registry.rs`), and the LocalEmbedding intent backend —
funnels through `VectorMatcher::embed_batch`, which returns **placeholder
zero vectors** (`vector.rs:212-215`; `model.forward()` was never wired).
Confirmed live: KB search returns `{"results":[]}` for an exact-phrase match
against a successfully-indexed document, so "chat with your documents" does
not work. Additionally, `KbConfig.embedding_provider` stores `"fastembed"`
but nothing consumes it — the API misrepresents what the system does. This is
the #1 blocker in `uar-final-production-hardening-2026-07`'s 100%-customer-
ready mandate.

Web-researched fix (July 2026): `fastembed` 5.17.2 (actively maintained)
supports BGE-small-en-v1.5 as its default model, loads the repo's **existing**
on-disk `bg-small-en-v1.5.onnx` + `tokenizer.json` via
`try_new_from_user_defined` (fully offline), and handles pooling +
normalization internally on `ort` 2.0.0-rc.12 prebuilt static binaries. The
incumbent burn ONNX-import path is confirmed still broken upstream for
BERT-family models (tracel-ai/burn#3412) — not viable.

## What Changes

- Add `fastembed` (5.x, offline features only — no hf-hub runtime downloads)
  and rewire `VectorMatcher::embed_batch` to real inference: local model +
  tokenizer from `src/uar/runtime/matching/models/`, sync inference wrapped in
  `tokio::task::spawn_blocking`, 384-dim normalized output.
- Remove the burn placeholder inference path (and the
  "generic placeholder mode" warn); keep tokenizer-independent public API
  unchanged so all nine call sites work without modification.
- Make `KbConfig.embedding_provider: "fastembed"` truthful; reject or warn on
  unsupported provider values instead of silently ignoring the field.
- Handle pre-existing zero-vector rows: detect-and-re-embed on ingestion-side
  (re-index path) or document mandatory re-upload in the upgrade guide —
  decided in design.md.
- Un-weaken nothing: the deliberately-red `chat-kb-retrieval.feature` BDD
  scenario must now pass as-is (bdd-chat 6/6).

## Capabilities

### New Capabilities
- `local-embedding-inference`: real, offline, local text-embedding inference
  (BGE-small-en-v1.5, 384-dim, normalized) powering KB retrieval, chat RAG,
  skill matching, and intent classification, with no runtime model downloads
  and no network dependency.

### Modified Capabilities
(none — `chat-bdd-coverage`'s KB-retrieval requirement already states the
correct behavior; this change makes the implementation satisfy it.)

## Impact

- **Code:** `src/uar/runtime/matching/vector.rs` (core rewrite), `Cargo.toml`
  (new dep; burn possibly demotable later — out of scope), possibly
  `src/uar/rag/ingest.rs` for the re-embed path.
- **Runtime UX:** KB search / "chat with your documents" starts actually
  working; skills' embedding/hybrid matching modes become meaningful; the
  LocalEmbedding intent backend becomes non-degenerate.
- **Provider compatibility:** none — local inference, no LLM-provider surface.
- **Realtime state:** none.
- **Build/CI:** ort-sys downloads prebuilt static ONNX Runtime at build time
  (cacheable in CI); ~30-50 MB binary-size impact — recorded in the change's
  verification notes.
- **KBD workflow state:** Round 1 change 1/9 of
  `uar-final-production-hardening-2026-07`; progress.json + waypoint updated
  per task via the kbd-apply driver.
