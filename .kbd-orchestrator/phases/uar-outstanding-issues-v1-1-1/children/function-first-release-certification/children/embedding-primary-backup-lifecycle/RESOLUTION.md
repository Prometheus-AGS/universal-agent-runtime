# RESOLUTION — primary/backup resolution + truthful effective status (condition 2)

## Resolution rule (as-built, with citations)

- **Primary** = the single active embedding slot:
  `SwappableEmbeddingBackend` holding one `ArcSwapAny` slot of
  `{backend, provider/model/dimension metadata}` (`swappable.rs`).
  Exactly two ordered writers: boot overlay (`server.rs:1025-1055`) then REST
  activation (`settings.rs:856-895`).
- **Backup** = *no automatic backup machinery exists for embeddings.*
  The only failover schema (`llm_failover.*`, manager.rs:2859-2919) governs
  chat-model routing, not embedding backends. A "backup" can therefore only
  mean: an alternate honored `{backend, model}` value persisted via the
  settings mutation contract and swapped into the same slot (manual,
  operator-driven). The inventory (INVENTORY.md) shows no such alternate
  honored value exists in this deployment.
- **Resolution precedence for the candidate at mutation/boot**:
  persisted rows (`llm.embedding.backend|model`) override boot config; boot
  config supplies the non-mutable fields (`settings.rs:791-819`).

## Truthful effective status

- Mutation responses carry an explicit `activation` map: `live` /
  `restart_required` / 422 `unsupported_setting` (`settings.rs:856-895`).
- `ensure_backend_honors_candidate` (`settings.rs:852-885`): the BUILT
  backend's `backend_name()`/`model_id()` must equal the candidate, else the
  mutation is rejected BEFORE persist — a value the backend canonicalizes
  away can never be reported `live`.
- Boot overlay applies only honored rows; unhonored out-of-band drift → warn
  + keep config base (`server.rs:1031-1053`; observed 13:23:03 boot log).
- REST readback of `llm.embedding.model` (15:14 UTC): effective ==
  saved == `bge-small-en-v1.5` == the actually-serving backend identity
  (fastembed fixed model). Effective status is truthful by construction.

## Evidence binding

P2 receipts (config-lifecycle-map §9, v4): unhonored-model PUT → 422 with
row byte-identical; bogus backend → 422; reset → `activation: live` →
honored swap → next-request search 0.6399; boot overlay refusal of
out-of-band bogus row.
