# Embedding lifecycle fixture handoff

## Scope

`tests/embedding_lifecycle_failover.rs` is the source fixture for the active
`function-first-release-certification` Execute phase. It is intentionally
separate from the frozen `embedding-primary-backup-lifecycle` mirror evidence:
the prior A→B→A proof restarted between members and therefore cannot certify
the new no-restart requirement.

The fixture uses only local model directories supplied at execution through
`UAR_EMBEDDING_LIFECYCLE_PRIMARY_DIR`,
`UAR_EMBEDDING_LIFECYCLE_BACKUP_DIR`, and
`UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIR`. It makes no hosted request, creates
no credential, and never reads personal configuration. It records sanitized
member identity and asset-manifest diagnostics only.

## Required behavior

The runtime integration must provide real pinned local inference for all three
members. The fixture must then prove, through one owner-scoped durable knowledge
base, all of the following:

1. Primary ingest binds `{backend, model, dimension}` and primary query works.
2. A runtime-owned, explicit primary-member retirement closes its admission and
   joins accepted inference before it transitions in-process to the compatible
   backup. The backup ingests a second document and queries both documents.
3. An explicit, generation-checked recovery returns to the primary without a
   runtime restart. The retained compatible backup then serves a second
   primary retirement and a second recovery returns to the primary at
   generation 5. Query continuity remains intact across every generation.
4. An actual equal-dimension different local model with a distinct asset
   manifest and model-weight digest performs local inference before the
   incompatible attempt. The KB search is explicitly refused and the stored
   binding, documents, and sources remain unchanged.
5. Member transitions are coordinated through runtime events/barriers. The
   fixture owns cleanup through its parent scope on both ordinary failure and
   panic; it never deletes an operator-provided model directory.

The fixture registers its parent scope finalizer before acquiring local
members. The finalizer calls `LocalEmbeddingFailover::shutdown()` and the
retained incompatible backend's `EmbeddingBackend::shutdown()`, returning any
native cleanup error through the supervisor before fixture-owned SurrealKV is
closed. Constructor errors after a compatible member is acquired request that
same supervisor shutdown before returning the error.

## Current interface handoff

The production owner is implementing the local-only failover surface in
`src/uar/rag/embeddings`, config, server composition, and settings. The fixture
consumes `LocalEmbeddingFailover::build(primary, &failover, scope)`,
`retire_primary(expected_generation)`, `recover_primary(expected_generation)`,
and the serializable `status()` surface. It injects that one failover backend
through the same `SwappableEmbeddingBackend` into `IngestService` and
`VectorMatcher`. It must not replace that surface with a fake
`EmbeddingBackend`, a hashed vector, a hosted backend, or a restart-based
mirror scenario.

The three settings-router literals in `tests/settings_persistence.rs` retain
their no-embedding test posture explicitly with
`embedding_backend: None`, `embedding_base_config: None`, and
`embedding_failover: None`. They do not construct or substitute a backend.

The source fixture inputs are in
`tests/fixtures/local_embedding_lifecycle/`. They contain text and an
environment-variable contract only. A missing real compatible or incompatible
local model is an explicit failed prerequisite. It must neither skip nor pass,
and the fixture must not download assets.

## Verification status

The operator verification gate is closed. No test, Cargo command, formatter,
lint, compiler probe, or review gate has run for this fixture.

After the phase-wide implementation inventory is complete and the operator
explicitly opens the gate, the proposed single target is:

```text
cargo test --locked --no-default-features --features server-full --test embedding_lifecycle_failover
```

That command is only proposed. It has not been run and does not certify the
fixture until it observes all runtime receipts above.

## Uncomfortable limit

The fixture source cannot manufacture an equal-dimension different local model.
Until an operator-provisioned, actual local model and the runtime-owned
constructor support it, condition 4 remains unverified and the requirement is
not ready.

This fixture covers the frozen child RAG/knowledge-base contract only. It does
not certify the MemoryService subsystem's separate embedding-provider
configuration.

It also does not, by itself, certify process-restart hydration or
settings-mutation acceptance. Those frozen child legs require their own runtime
evidence and must not be inferred from this live in-process lifecycle fixture.
