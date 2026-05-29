## ADDED Requirements

### Requirement: AES-256-GCM Encryption of Credentials at Rest
The system SHALL encrypt every provider API key before persistence using AES-256-GCM
with a key derived from the `CREDENTIAL_ENCRYPTION_KEY` environment variable, and SHALL
store ciphertext as `base64(12-byte-nonce ‖ ciphertext)`.

#### Scenario: Plaintext key is encrypted on write
- **WHEN** a caller stores a provider API key
- **THEN** the persisted record MUST contain only `base64(nonce ‖ ciphertext)` and MUST NOT contain the plaintext key.

#### Scenario: Identical plaintexts produce distinct ciphertexts
- **WHEN** the same plaintext key is encrypted twice
- **THEN** the two stored ciphertexts MUST differ (a fresh 12-byte random nonce is used per encryption).

#### Scenario: Round-trip decryption
- **WHEN** a stored ciphertext is decrypted with the same `CREDENTIAL_ENCRYPTION_KEY`
- **THEN** the result MUST equal the original plaintext key.

#### Scenario: Decryption fails under a wrong key
- **WHEN** a stored ciphertext is decrypted with a different `CREDENTIAL_ENCRYPTION_KEY` than was used to encrypt it
- **THEN** decryption MUST fail with an error and MUST NOT return corrupted or partial plaintext.

#### Scenario: Encryption key required only on credential access
- **WHEN** `CREDENTIAL_ENCRYPTION_KEY` is absent AND no per-user credential is written or read
- **THEN** the system MUST operate normally (single-tenant env-only mode is unaffected).

#### Scenario: Missing encryption key is a hard error on credential access
- **WHEN** `CREDENTIAL_ENCRYPTION_KEY` is absent AND a caller attempts to write or read a per-user credential
- **THEN** the operation MUST fail with a clear configuration error.

### Requirement: Scoped Credential Store
The system SHALL provide a SurrealDB-backed store for encrypted provider credentials
keyed by scope and provider, supporting at minimum `user`, `agent`, `session`, and
`system` scopes.

#### Scenario: Store and retrieve a user-scoped credential
- **WHEN** a credential is stored for `(scope=user, user_id=U, provider_id=P)`
- **THEN** a subsequent lookup for `(user_id=U, provider_id=P)` MUST return the decryptable credential row.

#### Scenario: Scope isolation between users
- **WHEN** user A and user B each store a credential for the same `provider_id=P`
- **THEN** a lookup for user A MUST return only A's credential and MUST NOT return B's.

#### Scenario: Delete removes the credential
- **WHEN** a stored credential for `(scope, id, provider_id)` is deleted
- **THEN** a subsequent lookup for the same key MUST return no credential.

#### Scenario: SurrealDB store available in default build
- **WHEN** the crate is built with default features (surreal-backend, no `postgres-backend`)
- **THEN** the SurrealDB credential store MUST compile and function without requiring the `postgres-backend` feature.

### Requirement: Credential CRUD API
The system SHALL expose JWT-protected REST endpoints for managing the authenticated
user's provider credentials, where raw API keys are accepted on write only and are
never returned on read.

#### Scenario: Store a credential via API
- **WHEN** an authenticated user POSTs a raw provider API key for a provider
- **THEN** the system MUST encrypt and persist it scoped to that user and MUST respond without echoing the raw key.

#### Scenario: Read never returns plaintext
- **WHEN** an authenticated user lists or fetches their stored credentials
- **THEN** the response MUST indicate the credential exists (e.g., provider id, masked/last-4, created-at) and MUST NOT include the plaintext or full ciphertext key.

#### Scenario: Rotate replaces the stored key
- **WHEN** an authenticated user submits a new raw key for a provider they already have a credential for
- **THEN** the system MUST replace the stored ciphertext such that subsequent resolution uses the new key.

#### Scenario: Delete a credential via API
- **WHEN** an authenticated user deletes their credential for a provider
- **THEN** the credential MUST be removed and subsequent resolution for that user+provider MUST fall through to lower-priority scopes.

#### Scenario: Unauthenticated access is rejected
- **WHEN** a request to any credential endpoint arrives without a valid JWT
- **THEN** the system MUST reject it via the existing auth middleware and MUST NOT read or mutate any credential.

#### Scenario: A user cannot access another user's credentials
- **WHEN** an authenticated user A targets a credential belonging to user B
- **THEN** the system MUST deny the operation (the authenticated subject scopes all credential access).

### Requirement: Provider and Model Catalog Read API
The system SHALL expose read-only endpoints listing available providers and models from
the existing build-time catalog, without introducing a second authoritative catalog.

#### Scenario: List providers
- **WHEN** a caller requests the providers endpoint
- **THEN** the system MUST return providers sourced from the existing `ModelCatalog` / liter-llm registry.

#### Scenario: List models
- **WHEN** a caller requests the models endpoint
- **THEN** the system MUST return models from the existing build-time catalog and MUST NOT depend on a runtime models.dev sync.
