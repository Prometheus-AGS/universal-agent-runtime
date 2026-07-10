## Context

`VectorMatcher` (`src/uar/runtime/matching/vector.rs`) owns tokenizer + model
state behind `tokio::sync::Mutex`es and exposes `embed_batch`,
`index_skills`, and query helpers consumed by nine call sites across RAG,
skills, and intent matching. Tokenization works; inference was never wired
(burn-import codegen for BERT was broken when written, and remains broken
upstream — tracel-ai/burn#3412). The repo already ships
`models/bg-small-en-v1.5.onnx` (34 MB), `tokenizer.json`,
`tokenizer_config.json`, `special_tokens_map.json`.

## Goals / Non-Goals

**Goals:** real 384-dim normalized embeddings, fully offline, minimal blast
radius (public `VectorMatcher` API unchanged), CI-safe build, honest
`embedding_provider` config, bdd-chat 6/6.

**Non-Goals:** removing burn from the dependency tree (used elsewhere /
follow-up), GPU acceleration, alternative models, embedding-model
configurability beyond honesty checks, automated re-embedding migration (see
D4).

## Decisions

**D1 — fastembed 5.x, offline-only features.** Use
`TextEmbedding::try_new_from_user_defined(UserDefinedEmbeddingModel, ...)`
with the on-disk ONNX + tokenizer files; disable default `online`/hf-hub
features so no runtime download path exists. Rationale: only option using
assets already in-repo with pooling+normalization built in (assessment
research; burn dead upstream, direct-ort = reimplementing fastembed on the
same dependency). Fallback if the pinned `ort` rc conflicts: direct `ort`
rc.12 + hand-rolled mean-pool/L2.
Note: `UserDefinedEmbeddingModel` requires a model `config.json` alongside
the tokenizer files; the repo lacks one — add the canonical
`BAAI/bge-small-en-v1.5` config.json (tiny, static) to `models/`.

**D2 — concurrency.** fastembed inference is synchronous CPU work: hold the
`TextEmbedding` in the existing struct state and call it inside
`tokio::task::spawn_blocking`. Batch inputs as today (callers already batch).

**D3 — smallest-diff structure.** Keep `VectorMatcher`'s public surface
(`initialize`, `embed_batch`, `index_skills`, `find_matches`, threshold
logic) untouched so zero call sites change. Internally replace the burn
`model` slot with the fastembed handle. If the struct's `B: Backend` generic
is load-bearing across files, keep it as a phantom rather than refactoring
nine call sites; remove-the-generic is a follow-up cleanup, not this change.

**D4 — pre-existing zero-vector rows: document, don't migrate.** No release
has ever been cut (0 tags), so no external customer holds a persisted index;
internal/dev deployments re-upload documents. We add (a) an upgrade note in
the docs-site change's upgrade guide, and (b) a defensive runtime check —
`search_knowledge` logging an explicit error when stored embeddings are
zero-norm — so any stale index self-identifies instead of silently returning
nothing. Automated re-embed is disproportionate pre-1.0.

**D5 — config honesty.** `KbConfig.embedding_provider` keeps default
`"fastembed"` (now true). Unknown values log a loud warning and fall back to
fastembed (not a hard error — pre-existing KBs must keep loading).

## Risks / Trade-offs

- [ort-sys downloads prebuilt ONNX Runtime at build time] → CI caches cargo;
  first uncached build slower; record impact. If the download endpoint is
  unreachable in some environment, builds fail loudly (acceptable; offline
  *runtime* is the requirement, not offline build).
- [~30-50 MB binary growth] → recorded in verification notes; acceptable for
  a server binary.
- [fastembed pins `ort =2.0.0-rc.12` exactly] → pin fastembed minor version;
  revisit when ort 2.0 stabilizes.
- [Existing `#[ignore]`d live-integration cases reference disclosed product
  bugs] → check at apply time whether either is this bug; un-ignore if so.

## Migration Plan

Additive dependency + internal rewrite; no API/schema change. Rollback =
revert the commit (placeholder returns, tests re-redden). Stale-index
behavior covered by D4.

## Open Questions

(none — operator decisions all resolved at phase plan time)
