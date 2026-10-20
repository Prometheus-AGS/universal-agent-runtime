# INVENTORY — approved embedding provider/model surface (condition 1)

Date: 2026-10-19 15:25 UTC. Candidate: v4 sha256 `96bf4868…5154`
(server-full feature set). Secret VALUES are never recorded — presence only.

## Compiled backend catalog (`src/uar/rag/embeddings/mod.rs:109-124`)

| Backend | Compiled in v4? | Evidence |
|---------|-----------------|----------|
| fastembed | YES (feature `local-models` ∈ `server-full`, Cargo.toml:224,267) | serving today; honored |
| candle | NO (`candle-embeddings` NOT in `server-full`, Cargo.toml:273) | build → `BackendUnavailable` |
| openai | YES (unconditional arm, mod.rs:117) | not credentialed (below) |
| voyage | YES (mod.rs:118) | no credential |
| cohere | YES (mod.rs:119) | no credential |

## Per-backend approved identity + credential presence

1. **fastembed** — honored model: `bge-small-en-v1.5` ONLY, dimension 384
   (hardcoded canonicalization, `fastembed.rs:124-127`); endpoint: local
   (none); credential: none required. Persisted live row on 1907:
   `llm.embedding.model = bge-small-en-v1.5` (REST readback 15:14 UTC).
2. **openai** — OpenAI-compatible `POST {base_url}/embeddings`
   (`openai.rs:25-28,92-93`); `model_id()` honors the configured model string
   (`openai.rs:55-57`); sends explicit `dimensions` param when
   vector_dimension ≠ 1536 (`openai.rs:83-87`). Credential resolution:
   `config.api_key` → env named by `config.api_key_env` → else
   `MissingApiKey` (`openai.rs:142-154`). On 1907: EmbeddingConfig has
   `api_key: None, api_key_env: None` (no `llm:` section in
   uar-r14/config.yaml; defaults at `mod.rs:139-160`) → **not buildable**.
   Process env name-presence check (pid 24103, names only): `OPENAI_API_KEY`
   present BUT nothing points `api_key_env` at it, and REST cannot set it
   (boot-config-only field, `settings.rs:787-790`). Use is also barred by
   child limits (paygo + unapproved third-party data transmission).
3. **voyage** — same resolution shape (`voyage.rs:114+`); env names
   `VOYAGE_API_KEY`/`VOYAGEAI_API_KEY` absent from server process env → not
   buildable.
4. **cohere** — same resolution shape (`cohere.rs:120+`); env absent → not
   buildable.
5. **candle** — not compiled; would additionally need safetensors weights in
   `models_dir`. `src/uar/runtime/matching/models/` holds only
   `bg-small-en-v1.5.onnx` (skill-matcher asset, same model family).

## Settings schema (`manager.rs:2841-2849`)

Only TWO mutable embedding keys exist: `llm.embedding.backend`,
`llm.embedding.model`. No backup/failover embedding keys.
`llm_failover.*` (manager.rs:2859-2919) is chat-MODEL failover, not
embeddings. Boot-only fields (api_key, api_key_env, base_url,
vector_dimension=384, batch_size, models_dir) are not REST-mutable
(`settings.rs:787-790`); persistence `vector_dimension: 384`
(uar-r14/config.yaml).

## Inventory conclusion

**Exactly one honored embedding space exists in this deployment:
`{fastembed, bge-small-en-v1.5, 384}`.** No approved compatible backup is
reachable within the child's hard limits (no new credentials, no paygo, no
unapproved data transmission, preserved config binding).

## Scope correction (2026-10-19 15:50 UTC, operator-directed)

This inventory covers the **RAG/KB embedding backend of the disposable
uar-r14 fixture only**. It does NOT cover the intended **memory-service
embedding configuration**, which is a separate subsystem:

- Operator-supplied (personal config NOT read by the agent — secret-risk
  boundary): `~/.uar/config.yaml` names OpenAI `text-embedding-3-small`/1536
  for memory/default KB; `~/.bash_profile` carries a `MISTRAL_API_KEY`
  export reference, but no Mistral selection appears in that YAML.
- Repo-verified (read-only, dirty working tree): `src/uar/memory/service.rs`
  has an explicit `"mistral"` arm requiring `MISTRAL_API_KEY`
  (service.rs:100-107) and REJECTS unknown providers — supported:
  `openai | anthropic | mistral | cohere | local` (service.rs:138).
- Operator-cited: GitHub issue #420 documented the older SILENT memory
  embedding fallback. The current dirty code rejects instead — i.e. the
  silent-fallback defect class is acknowledged and partially repaired in
  the working tree, and the personal config predates or disagrees with it.
- Fixture-local observation (boot-v4.log 13:23:03): the 1907 fixture's
  memory service initialised with `embedding_api=local`,
  `BAAI/bge-small-en-v1.5`, 384d (surreal-memory candle path) — a THIRD
  embedding identity distinct from both the RAG fastembed space and the
  personal-config OpenAI selection.

**Recorded mismatch**: three embedding configurations exist across
subsystems (RAG fixture fastembed/384; memory personal-config OpenAI/1536;
fixture memory-service local BGE/384), and the personal environment contains
a Mistral credential without a Mistral selection. No claim in this child
about the RAG fixture generalises to the memory subsystem.
