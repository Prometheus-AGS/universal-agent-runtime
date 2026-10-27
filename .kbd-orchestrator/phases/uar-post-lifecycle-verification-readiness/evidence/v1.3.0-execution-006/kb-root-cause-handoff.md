# Packet 006 KB diagnosis — source inspection only

Phase: uar-post-lifecycle-verification-readiness / Execute. Operator verification lock CLOSED. No tests, checks, builds, probes, imports, formatting, service calls, or external mutations were run. No production file was changed. This report is evidence interpretation, not acceptance or an implemented fix.

## Retained facts

- Packet source-build-boot.json binds sourceInventoryDigest `9ef107844e009bf977cedd6ebf411cfa3751e815247cf836b8f0288529786b11` and built/frozen main SHA256 `2310bd4ee834a094f31e5449f6cbbbea7f42e9068993cbb628c112286b164b6c`.
- Packet teardown.json event122 records concurrent_indexed_d poll HTTP200, observedState failed, errorMessageSha256 `8633498ab48df72d75ce4dd318966597af27c3e4038d7271f4634753077f84de`.
- Packet raw/peer-events.jsonl sequences807/808 record both kb-race embeddings arriving at monotonic187024.498121141 and187024.548087881. Release sequence809 occurred187025.492132193. Sequences810/811 record replies187025.492640906 and187025.493060047. Final sequence812 reports arrived2, expected2, released true, expired0. Thus the20second fixture hold did not expire for this race.
- Inspection of retained main1/main2 structured WARN/ERROR records found no worker error revealing this document failure. The worker persists TaskError text but does not log its primary executor error; logs in persist_failure concern settlement failure only.

## Source mechanism and limits

`src/uar/persistence/providers/ingestion_records/surreal.rs`: accept uses at most3 attempts, retrying only typed TransactionConflict after25ms. publish executes one transaction. Each publication updates its own job/document/chunks and also shared KB record config.indexed_embedding/updated_at. Therefore separate document publications can contend on the KB fence. This fence participates in KB deletion ordering and embedding-space consistency; removing it or splitting the atomic publication would weaken real contracts.

`src/uar/rag/ingestion_worker.rs`: process_document calls prepare_ingestion then publish_ingestion. Executor failure passes through TaskError::executor and persist_failure. `src/uar/persistence/ingestion_records/mod.rs`: failed_attempt marks attempts below5 Retryable, schedules next_attempt_at30*attempts seconds later, and persists its error. transition projects both Retryable and Failed to public DocumentStatus::Failed. A failed poll therefore does not establish terminal durable exhaustion.

This is a localized contention hypothesis, not evidence requiring an architectural rewrite. Existing durable source retention, generation CAS, transactional publication and shared deletion fence are the relevant architecture. The uncomfortable limit is that the packet retains only a hash of this error and the primary typed cause/stage is absent from logs. A plain source-literal SHA scan did not match the retained hash; dynamic/error-wrapper formatting remains unresolved. No upstream transaction-behavior claim is made beyond the source predicates inspected.

## Delivery and next ownership

Changed only this handoff. No unsolicited production guards or retries were added. No production fix is claimed. Root cause remains unresolved pending exact reconstruction of the retained error text or separately authorized evidence collection. Do not treat an added publication retry as proven repair without identifying a typed rolled-back conflict. Parent owns authoritative plan, ledger, source inventory and further operator authorization; no sixth repair, signed packet, or rerun is authorized here.

Inspection commands used cat/sed/rg/find and Python standard-library reads/JSON traversal/string SHA computation only. Literal hashing was offline evidence interpretation, not execution of application code. Output established retained identities, event ordering and source control flow; it did not establish compilation or runtime success.
