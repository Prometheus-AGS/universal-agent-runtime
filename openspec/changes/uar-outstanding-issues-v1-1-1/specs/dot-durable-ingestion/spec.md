# Spec Delta

## ADDED Requirements

### Requirement: Durable ingestion reports captured eligibility without accepting work

**Stable task:** S2. The knowledge upload API and durable ingestion dispatcher SHALL use the same host-captured eligibility category as durable accounting. Ineligible durable storage SHALL reject upload before a `202 Accepted` response and shall emit only an allowlisted diagnostic category. It SHALL NOT disclose a database endpoint, namespace, credential, source error, or mutable environment value.

#### Scenario: Unattested remote storage rejects upload

- **WHEN** `POST /api/knowledge/{id}/documents` reaches a remote SurrealDB provider constructed without the trusted-host durability attestation
- **THEN** the API returns its existing service-unavailable outcome without accepting source bytes, a document, or an ingestion job
- **AND** the protected dispatcher diagnostic identifies the captured `remote_not_attested` reason without exposing storage connection details

#### Scenario: Eligible upload keeps its acceptance meaning

- **WHEN** an eligible durable provider accepts an authenticated multipart upload
- **THEN** `202 Accepted` continues to mean that immutable source bytes, the owner-scoped document, and exactly one owner-scoped ingestion job committed together
- **AND** the acceptance response does not claim that indexing has completed

### Requirement: Dispatcher recovery preserves owner-scoped durable work

**Stable task:** S2. The durable ingestion dispatcher SHALL reconcile and discover only committed owner-scoped jobs, preserve source-byte and job identity across restart, and retain the existing publication fence. It SHALL NOT adopt legacy rows, create duplicate indexed chunks, or treat an uncertain transition as successful publication.

#### Scenario: Restart recovers one accepted job

- **WHEN** the process exits after an accepted upload and before dispatch completes
- **THEN** the restarted dispatcher discovers the committed source bytes and the exact owner-scoped job
- **AND** it resumes through the existing reconciliation and publication path without duplicating indexed chunks

#### Scenario: A principal cannot dispatch legacy or another principal's job

- **WHEN** a job or document identifier belongs to a legacy namespace or a different verified principal
- **THEN** the current principal cannot retrieve, retry, dispatch, or publish that work through the knowledge API
- **AND** no owner identity is inferred from a matching subject string
