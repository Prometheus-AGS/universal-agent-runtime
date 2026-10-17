# Durable knowledge ingestion

An authenticated document upload returns HTTP 202 only after the configured database commits its immutable source bytes, owner-scoped document, and ingestion job together. The existing multipart upload limits and owner authorization remain in force. The upload body is not an in-memory queue's responsibility after acceptance.

Supported stores are local SurrealKV, qualified durable networked SurrealDB, and PostgreSQL. Networked SurrealDB requires the existing trusted-host `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1` attestation. An in-memory or unsupported store returns 503 for uploads. A database failure returns a non-202 response; a transport error after a database commit can have an unknown outcome, so clients should list documents before resubmitting. There is no cross-request upload deduplication key.

The trusted host captures backing-store eligibility while it constructs the
provider: `local_durable`, `remote_attested`, `remote_not_attested`, or
`unsupported_backend`. An unattested remote endpoint is refused before source
bytes, a document, or an ingestion job are committed. A successful connection,
health response, writable account, replica count, or later environment change
does not upgrade that captured eligibility.

The server starts durable discovery alongside the ingestion pool. It scans on startup and every five seconds, and a successful upload sends an optional wake notification. Queue saturation or process death cannot remove the committed job. Acceptance without an available worker is intentional: processing begins when a compatible worker starts against that database.

## Job state and recovery

With the existing authenticated `/api/uar/knowledge-bases` prefix:

- `POST /{kb}/documents` accepts multipart field `file` and returns the existing document response.
- `GET /{kb}/documents/{document}/ingestion` returns job metadata: state, attempts, generation, retry timestamp, and last error. It never returns source bytes.
- `POST /{kb}/documents/{document}/retry` resets a failed or retryable job to pending, using its retained source bytes and a new generation. Concurrent changes return 409.
- `DELETE /{kb}/documents/{document}` cancels work by atomically deleting the source/job, document and chunks. Deleting the KB applies the same revocation to all its documents.

Jobs progress from pending to processing and then indexed. Processing failures enter retryable with a delay of 30 seconds multiplied by the attempt number; the fifth failed attempt enters failed. The document's existing failed status includes the error while job metadata distinguishes retryable from failed. An explicit retry begins a new bounded attempt sequence after the operator repairs extraction or embedding configuration. Startup reconciliation marks legacy pending/processing metadata without retained bytes as failed with `source_bytes_missing: reupload required`; those records need a new upload.

A crashed processing attempt remains discoverable. Workers may repeat extraction and embedding for the same immutable generation, including on different runtimes. Preparation has no chunk writes. Only a fenced database transaction can replace the complete document chunk set, bind its embedding identity, update its count, and publish Indexed. Duplicate, deleted, or superseded attempts cannot publish. The job's source is retained through indexed and failed states and is collected by document or KB deletion.

The uncomfortable tradeoff: crash recovery may repeat hosted extraction or embedding requests and therefore incur additional provider cost. This is not exactly-once provider invocation. Fencing guarantees one visible committed chunk generation, not one computation. Database transaction size scales with the accepted source and complete chunk generation; operational database limits can reject a commit. Such rejection never produces a successful indexing status.

## Authenticated reupload procedure

1. Establish an operator-attested durable backing and capture the verified
   principal with its selected namespace and database identity.
2. Inventory legacy and principal knowledge rows read-only. Legacy rows remain
   preserved and are not candidates for ownership adoption.
3. Create or reuse a knowledge base in the verified principal's partition, then
   upload retained source bytes through the authenticated upload route.
4. Wait for the durable job to index and retrieve using that same verified
   principal.

Missing original bytes require a new source upload. Storage eligibility does
not reverify provider or model configuration; that remains a separate
operational check.

## Qualified recovery boundary

Recovery documentation covers only authenticated quiesce/reclaim with its exact
joined receipt, or qualified trusted-supervisor exclusion with single-use
evidence. Active effects and accounting remain uncertain until their existing
reconciliation completes. Recovery stays blocked without that qualified proof;
there is no force operation or timeout, deletion, Pod absence, or replica count
substitute for exclusion.

## Migration and rollback

PostgreSQL applies `20261012000000_durable_ingestion.sql` through the existing migration runner. Surreal host initialization applies `migrations/surrealdb/durable_ingestion.surql`; existing tables and history are preserved. Both migrations make missing-source legacy rows explicit instead of pretending recovery is possible.

Before rolling back to a server that does not understand durable jobs, stop accepting new uploads and drain jobs or preserve the database with jobs paused. Do not drop the ingestion table or erase retained sources. Old servers cannot discover these jobs and may create unrecoverable uploads. Resume with a compatible server before reopening upload admission.

## Deferred acceptance

The source-only fixture `tests/durable_ingestion.rs` covers HTTP acceptance without a worker, joined fresh-process recovery before dispatch and after claim, source/owner readback, duplicate/stale publication, deletion revocation, rejected admission and KB deletion through real storage adapters. It reuses existing isolated database support (`UAR_API_KEY_TEST_SURREAL_URL`, optional user/password, and `UAR_API_KEY_TEST_POSTGRES_URL`). PostgreSQL needs the `postgres-backend` feature; remote stores must be isolated durable test infrastructure.

No fixture, build, compiler, formatter, linter, or other gate was executed during this slice. At the full expanded Dot implementation boundary, the lead must run the planned serialized production integration gate and customer build. The bounded fixture does not by itself qualify actual worker extraction, embedding outages, concurrent two-runtime preparation, or the customer artifact.
