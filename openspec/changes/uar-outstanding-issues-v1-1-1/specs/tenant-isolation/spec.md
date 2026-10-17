# Spec Delta

## MODIFIED Requirements

### Requirement: Provider credentials use verified complete owner identity

Credential ownership SHALL use host-verified issuer, tenant and subject in a namespace disjoint from legacy identifiers. Caller-supplied identity SHALL NOT confer ownership. Local and issuerless credential owners SHALL additionally use an operator-provisioned installation UUID, without changing general session, knowledge, schedule or followup owner identities. Shared credential use SHALL require a verified configured issuer and nonempty signed tenant.

#### Scenario: Identical subjects

- **WHEN** two authenticated principals share a subject but differ by tenant or issuer
- **THEN** their credential create, read, list, delete and use operations are isolated

#### Scenario: Legacy ownership

- **WHEN** a legacy credential lacks sufficient owner provenance
- **THEN** it is not granted to a matching subject and only an authenticated audited mapping can migrate it without key collisions

#### Scenario: Local installation separation and stability

- **WHEN** installations with different provisioned UUIDs admit the same local subject, including issuerless JWT and authenticated local-launch identities
- **THEN** their credential owners differ, local-launch and local-JWT namespaces remain distinct, and restart or signing-key rotation with the same UUID preserves each credential owner
- **AND** delegated identity retains its root credential owner without acquiring host privilege

#### Scenario: Missing local installation identity

- **WHEN** a local principal credential operation has no valid configured installation UUID
- **THEN** it fails with a safe actionable identity error before credential lookup or mutation and never reads an old fixed-local namespace implicitly
- **AND** issuer-qualified principal credentials remain usable without that UUID and retain their existing owner bytes

#### Scenario: No global identity rewrite

- **WHEN** installation identity is configured or changed for credentials
- **THEN** session, knowledge, schedule and followup owner keys remain unchanged, no database-global installation lock is introduced, and a different valid UUID identifies a different credential namespace

## ADDED Requirements

### Requirement: Credential deployment mode is explicit operator policy

The host SHALL accept the new `security.provider_credentials` configuration with `mode`, `installation_id`, `shared_fallbacks` and `principal_mapping_file`. Mode SHALL be `single_owner` or `shared`; omitted mode SHALL mean `single_owner`, including remote deployments. Request metadata and deployment profile SHALL NOT choose or broaden credential mode.

#### Scenario: Compatibility deployment

- **WHEN** mode is omitted or explicitly `single_owner`
- **THEN** callers admitted by existing ingress rules retain house-account compatibility, including on remote hosts, subject to local credential identity and migration safeguards
- **AND** documentation states that this mode does not provide per-person house-account isolation

#### Scenario: Invalid policy

- **WHEN** mode is unknown, an explicit installation UUID is malformed, or a fallback entry is incomplete or uses a wildcard
- **THEN** the host refuses that invalid configuration with a secret-safe error rather than weakening it
- **AND** absence of an installation UUID alone does not block unrelated issuer-qualified operation

### Requirement: Shared house credentials require exact issuer-qualified grants

Each new `shared_fallbacks` entry SHALL contain exact nonempty `issuer`, `tenant_id`, `provider_id` and `source`, where source is `system` or `config`. Shared fallback SHALL require equality with verified issuer/tenant, resolved provider identity and selected source. Empty grants SHALL deny house fallback. A grant SHALL NOT establish caller identity or administrative privilege.

#### Scenario: Tenant name reused by another issuer

- **WHEN** two verified issuers use the same tenant identifier and only one issuer/tenant/provider/source tuple is granted
- **THEN** only that tuple can consume the house credential, including when the provider has a catalog alias

#### Scenario: Independent source permission

- **WHEN** principal session, agent and user scopes miss in shared mode
- **THEN** system and config sources are considered in that order only when independently granted
- **AND** a system miss does not authorize config fallback, while identity, migration, storage and decryption errors stop resolution

#### Scenario: Incomplete or synthetic shared identity

- **WHEN** shared credential use has no verified configured issuer, no nonempty signed tenant, or an anonymous, synthetic or inconsistent identity
- **THEN** it is denied before credential lookup or provider effects even if a fallback grant exists

### Requirement: Credential source authority covers every provider capture

The trusted host SHALL authorize the exact provider, endpoint and credential source before constructing or accepting a credential-bearing client. The same policy SHALL govern primary, fallback, team, graph, summarization, SDK-supplied and inherited bindings, including absent encrypted credential service. A descendant SHALL only reuse or narrow its captured authorized bindings.

#### Scenario: Service unavailable or opaque supplied client

- **WHEN** shared mode has no encrypted credential service or receives a supplied driver without host-verifiable source provenance
- **THEN** absence of service or a driver label does not authorize ambient credentials, and an unproven binding is denied before provider effects
- **AND** an exact granted config source can remain usable without an encrypted service

#### Scenario: Explicit credentials and credential-free inference

- **WHEN** an existing request credential grant or credential-free local provider is selected
- **THEN** the host retains exact provider/endpoint checks and shared identity eligibility for credential use
- **AND** credential-free admission prevents later environment or config credential pickup

#### Scenario: Administration is distinct

- **WHEN** an authorized provider administration operation verifies or seeds a system credential
- **THEN** its existing host/settings authority and exact target checks remain required and its receipt grants no ordinary caller fallback permission

### Requirement: Fixed-local credentials require exact approved migration

The new fixed-local mapping file SHALL bind `source_owner_key`, `scope`, `source_scope_id`, `provider_id`, `source_revision`, `destination_owner_key` and `approval_id`. `POST /api/uar/credentials/migrate-principal` SHALL apply only that installed mapping to the authenticated destination. Only old fixed-local user, session and agent rows SHALL be eligible; source equality of subject alone SHALL confer no authority.

#### Scenario: Authenticated destination and exact resource

- **WHEN** a migration request selects an approved fixed-local source
- **THEN** the bound caller owner must equal the configured destination, user scope targets the verified subject, and session/agent scope preserves its exact source identifier
- **AND** request fields cannot select destination or approval, and system, external-issuer and already installation-bound source rows are rejected

#### Scenario: Quarantined source would trigger house fallback

- **WHEN** a local lookup misses in its bound namespace and an unresolved old fixed-local row could supply that exact scope/resource/provider
- **THEN** it returns `credential_migration_required` without reading its secret or falling through to a house account
- **AND** ambiguous or unmapped sources remain quarantined; there is no automatic adoption, retirement or ignore switch

### Requirement: Migration copies and journals are atomic and immutable

Both migration source kinds SHALL atomically copy ciphertext, hint and original timestamps to a vacant destination and append an immutable receipt while retaining the source. New fixed-local journal identities SHALL be disjoint from unchanged legacy journal identities. Replay SHALL return the original exact receipt without overwriting a rotation or resurrecting deletion.

#### Scenario: Drift or concurrent conflict

- **WHEN** an uncommitted source changes revision after approval, a destination already exists, or a conflicting mapping races for the source
- **THEN** migration fails without partial destination or receipt writes; a missing source remains missing, and equal timestamps do not conceal source change

#### Scenario: Response lost after commit

- **WHEN** a caller retries the same authorized operation after losing its response, including after destination rotation or deletion
- **THEN** the original receipt is returned and no second copy occurs
- **AND** changing destination, approval or the new operation's bound revision conflicts rather than rewriting the receipt

#### Scenario: Old journal compatibility

- **WHEN** an existing legacy receipt is read or its historical unconditioned call is retried
- **THEN** its original key, fields and replay semantics remain readable and immutable, and absence of new revision fields is not claimed as preview binding

### Requirement: Migration preview exposes metadata only to verified host authority

The new `GET /api/uar/credentials/migration-preview` SHALL require existing verified trusted-host authority. It SHALL return bounded metadata pages, namespace/scope/status counts, mapping revisions and exact source/destination status through metadata-only store projections. Ordinary principals and role strings alone SHALL NOT authorize inventory. Preview SHALL perform no writes and confer no migration approval.

#### Scenario: Nonsecret bounded inventory

- **WHEN** a trusted host requests preview with `limit` between 1 and 500, default 100, and an optional opaque continuation cursor
- **THEN** it receives exact source tuples, opaque metadata source revisions, configured destination/approval metadata, timestamps and conflict/journal status without plaintext, ciphertext, key hints, key fingerprints, raw backend errors or free-form reasons
- **AND** counts describe the current page observation and page-time totals; pages are explicitly not a frozen cross-page snapshot

#### Scenario: Restricted or cancelled preview

- **WHEN** an ordinary principal requests preview, or an authorized preview is cancelled
- **THEN** the ordinary request is denied before inventory access and cancellation leaves source, destination and journal unchanged

#### Scenario: Existing receipt with changed destination

- **WHEN** a journal exists but its destination is missing or has changed revision
- **THEN** preview reports that condition separately from ready-to-copy and never authorizes replaying the copy

### Requirement: Preview-guided application checks metadata preconditions

Credential stores SHALL maintain opaque per-row metadata revisions that change on writes and recreation without deriving them from secrets. Fixed-local mapping SHALL bind `source_revision`. Preview SHALL return a row `precondition` for the additive `If-Match` header on migration calls, binding source identity/revision and exact installed mapping. First application SHALL compare current metadata transactionally; preview is neither approval nor a lock.

#### Scenario: Stale preview or changed mapping

- **WHEN** an uncommitted preview-guided call presents a precondition for another source, an earlier source revision or a changed mapping
- **THEN** it returns HTTP 412 with `credential_migration_precondition_failed` before copy or journal commit
- **AND** wildcard, weak, multiple or malformed `If-Match` values are rejected with HTTP 400 rather than treated as unconditioned calls

#### Scenario: Existing legacy request remains compatible

- **WHEN** `POST /api/uar/credentials/migrate-legacy` uses its existing body without `If-Match`
- **THEN** its body and installed legacy mapping schema retain their existing strict fields and historical behavior
- **AND** documentation explicitly excludes preview/source-revision binding for that unconditioned call

### Requirement: Upgrade guidance preserves recoverable ownership evidence

The release SHALL document backup, all-replica writer/consumer quiescence, metadata inventory, independently approved exact mappings, transactional application and cold-read acceptance before shared traffic. It SHALL disclose local-owner compatibility changes, irreversible historical collisions and unsupported shared rollback to old identity semantics. Retained source rows SHALL NOT authorize old lookups.

#### Scenario: Missing ownership evidence or unsafe rollback

- **WHEN** historical provenance cannot distinguish owners or an operator proposes a rollback to subject-only/global-local credential lookup with shared traffic
- **THEN** guidance keeps ambiguous data quarantined and requires controlled recovery or a matched database/config/key restore with later rotations and deletions accounted for
- **AND** it makes no claim to recover overwritten history or detect a valid but unintentionally changed installation UUID automatically

### Requirement: Credential revision initialization is an explicit host operation

The runtime SHALL provide the local host command `credentials initialize-revisions --writers-quiesced` for the specified additive metadata upgrade. It SHALL require explicit operator acknowledgement, connect only to the configured credential store and exit without starting listeners, providers or MCP. Ordinary startup and migration preview SHALL NOT initialize missing revisions. Guidance SHALL require backup and quiescence of every relevant writer; the acknowledgement SHALL NOT be described as proof of exclusion.

#### Scenario: Explicit initialization preserves existing records

- **WHEN** the operator invokes the command with acknowledgement after backup and writer quiescence
- **THEN** missing row revisions are assigned without changing ciphertext, owner identity, hints, timestamps or existing migration receipts
- **AND** repeated invocation preserves already assigned revisions and reports only safe counts

#### Scenario: Missing acknowledgement or ordinary startup

- **WHEN** acknowledgement is absent, or the operator only starts the runtime or calls preview
- **THEN** no missing revision is initialized implicitly
- **AND** the explicit command refuses missing acknowledgement without opening listeners or contacting providers or MCP
