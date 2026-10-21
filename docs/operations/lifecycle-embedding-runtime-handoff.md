# Local embedding failover runtime handoff

## Delivered runtime contract

`llm.embedding` remains the primary embedding member. An optional
`llm.embedding.failover.backup` creates a local-only FastEmbed backup with only
`backend`, `model`, `vector_dimension`, `batch_size`, and `models_dir` fields.
It has no endpoint or credential fields. When this configuration is present,
both member directories are strict: `UAR_MODELS_DIR`, `/app/models`, and source
tree fallbacks are not consulted.

The runtime accepts the pair only after both members complete a real local
readiness inference. It requires the exact same `{backend, model, dimension}`
and the same SHA-256 asset manifest. A different model is refused even when it
has the same vector dimension. A member's five FastEmbed assets are
`bg-small-en-v1.5.onnx`, `tokenizer.json`, `config.json`,
`special_tokens_map.json`, and `tokenizer_config.json`.

The published backend remains one `SwappableEmbeddingBackend`, shared by
ingestion and query. The failover wrapper retains the compatible static
fingerprint, so existing knowledge-base bindings remain valid across the
primary and byte-identical backup. Any non-compatible configuration is refused
before it can become the live serving backend.

## Lifecycle and control surface

Server composition registers `local-embedding` with the trusted runtime
supervisor before it builds the pair. The resource finalizer retains only a
weak reference to the shared swappable handle. It closes the active failover
pair or legacy FastEmbed engine after consumers drain. FastEmbed initialization
and inference use the host `ResourceScope` blocking helper and keep their lease
until their join completes. Member shutdown failures are returned through the
controller and resource finalizer, so the lifecycle close report cannot claim a
released embedding resource after a native engine cleanup error. A failed
primary/backup construction and a discarded recovery candidate each close
already-initialized local engines before returning their failure. A failed
configured pair is recorded as a failed `local-embedding` resource and is
published only as an unavailable embedding backend, never as ready.

Each accepted member inference is also retained in `local-embedding` with an
owned reply channel. A client that cancels its request only abandons that reply;
the member admission remains held until the inference and its blocking join
finish, so retirement cannot clear the native engine underneath accepted work.

An administrator can inspect `GET /api/uar/settings/embedding/provenance`.
The response includes the active member, generation, static embedding space,
active/primary/backup provenance, last transition, and last failure. The
authenticated settings routes are:

- `POST /api/uar/settings/embedding/retire-primary` with
  `{ "expected_generation": <u64> }`
- `POST /api/uar/settings/embedding/recover` with the same payload.

Retiring the primary closes its member admission and joins accepted work. The
next actual embedding request receives the retirement error, promotes the
compatible backup, and retries once there. Recovery rebuilds and proves the
primary, checks the caller's expected generation, then publishes a new primary
generation while retaining the compatible backup as the next standby. Both HTTP
controls and the public controller methods submit retained tasks to
the `local-embedding` scope; dropping the HTTP waiter does not cancel accepted
retirement or recovery. Settings mutations for `llm.embedding.backend` and
`llm.embedding.model` are rejected while this pair owns the active space, so a
raw hot swap cannot discard the backup contract.

## Explicit local configuration

An operator-managed configuration can opt into the pair below. The two model
directories must already contain the same five assets. They can be separate
copies; neither directory is downloaded or populated by startup. These paths
are examples, not changes to personal or deployed configuration.

```yaml
llm:
  embedding:
    backend: fastembed
    model: bge-small-en-v1.5
    vector_dimension: 384
    batch_size: 32
    models_dir: /srv/uar/models/primary
    failover:
      backup:
        backend: fastembed
        model: bge-small-en-v1.5
        vector_dimension: 384
        batch_size: 32
        models_dir: /srv/uar/models/backup
```

Omit hosted credential and endpoint fields. This controls RAG ingestion and
knowledge queries; the separately configured MemoryService embedding provider
does not acquire this failover contract. Use the provenance response's current
generation for each explicit control request. Retirement acknowledges actual
member drainage. Recovery acknowledges validated primary publication; it does
not require a restart, a health probe loop, or a fixed waiting interval.

## Changed paths

- `src/config.rs`
- `src/server.rs`
- `src/uar/api/settings.rs`
- `src/uar/rag/embeddings/mod.rs`
- `src/uar/rag/embeddings/fastembed.rs`
- `src/uar/rag/embeddings/failover.rs`
- `src/uar/rag/embeddings/swappable.rs`
- `docs/operations/lifecycle-embedding-runtime-handoff.md`

## Source-only status and limits

No verification command ran. The phase gate is CLOSED, so compilation,
formatting, tests, builds, linting, and review remain unverified. In particular,
the real local primary-to-backup-to-recovery receipts, durable ingestion/query
continuity, incompatible-model refusal with supplied assets, and shutdown DAG
integration still require the authorized whole-phase gate. This implementation
does not create hosted calls, credentials, personal configuration, retries,
polling, or timeout-based correctness behavior.

