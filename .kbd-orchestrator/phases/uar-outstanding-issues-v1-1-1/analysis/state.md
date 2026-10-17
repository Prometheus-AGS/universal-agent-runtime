# #385 state, durability, and recovery decision

Phase: `uar-outstanding-issues-v1-1-1`. Stage: Analyze. Written 2026-10-15.

This is a source-design decision for the active phase. It authorizes no
database operation, migration, deployment update, recovery attempt, test, build,
or external query. The full-phase verification boundary remains closed.

## Decision

Support the existing remote-Surreal durability contract and make its refusal
actionable. A bounded run with any configured token, cost, remote, or scoped
limit continues to require durable accounting. A remote SurrealDB endpoint is
eligible only after trusted-host configuration has explicitly attested its
durability; a successful connection, one replica, HTTP health, or a writable
database user is not an attestation. The host must reject an ineligible store
before provider or tool effects.

For #385, the supported knowledge-base recovery is authenticated reupload into
the verified principal namespace after the deployment operator establishes the
durable backing and the exact principal/storage identity. Preserve legacy data
in place and do not automatically adopt it. The issue asks whether a KB
migration is needed; it does not require preservation of old knowledge-base
IDs, document IDs, chunk IDs, embeddings, or ingestion jobs. Therefore a
privileged KB import is out of scope for this phase.

The existing fenced team recovery paths remain the only supported paths:
orderly quiesce followed by reclaim with the recorded joined receipt, or a
qualified trusted-supervisor exclusion followed by reclaim with its single-use
evidence. There is no recovery by timeout, replica count, absent Pod, claim
deletion, a Boolean bypass, or forced takeover. A dead unregistered predecessor
remains blocked pending a separately designed infrastructure-fencing capability.

The uncomfortable constraint is that the reported deployment may remain
blocked after the code improves diagnostics. The supplied evidence does not
prove the backing store survives a cold restart, that the repository agent
matches the persisted agent, that a legacy KB belongs to the current verified
principal, or that a held team owner was safely stopped. Reporting a more
precise reason must not turn any of those unknowns into authority.

## Source evidence and conclusion

| Concern | Source evidence | Analyze conclusion |
| --- | --- | --- |
| Budgeted remote chat | `ModelCallBudget::has_configured_limits` includes `max_tokens_per_turn`; `with_durable_accounting` rejects a store without durable accounting; manager currently collapses the error to `Run model binding failed`. The diagnostic snapshot identifies a repository `max_tokens_per_turn: 16000` and no visible remote durability attestation. | Keep durable accounting required. Add a safe category/stage at the run-admission boundary. Do not remove the budget, make accounting process-local, or infer durability from connectivity. |
| Durable upload | `SurrealDbProvider` captures a remote endpoint as durable only with `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1`; `upload_document` and the durable dispatcher already reject otherwise. `tests/durable_ingestion.rs` treats committed source bytes/job state and restart discovery as the durable contract. | Keep the existing eligibility guard. Surface the captured ineligibility reason consistently for upload and bounded run admission. |
| KB data after upgrade | `KnowledgeNamespace` routes verified principals to `principal_knowledge_*`; missing provenance remains legacy even if a legacy subject equals a modern owner key. `principal_knowledge.surql` creates disjoint tables without adoption. | Reupload is sufficient for the reported ask. No privileged KB import, table rename, owner inference, ID reuse, or legacy-job activation belongs in v1.1.1. |
| Existing import precedent | `MemoryService::import_legacy_memory` is an explicit administrator-mediated, source-digested, owner-mapped memory import. There is no corresponding KB import API. | Treat this only as a future design precedent. Do not generalize it into a KB import under #385. |
| Team recovery | Team ownership is fenced and nonexpiring. Quiesce records joined evidence; supervisor exclusion records a protected proof; reclaim consumes exact evidence once, transfers queued work, and marks active effects/accounting uncertain. | Document the established paths. No lifecycle or recovery code is justified by the startup warning alone. |

The open-issue snapshot asks for the changed deployment requirement, safe
diagnostics, and confirmation about model reverification and KB migration. It
does not ask to preserve legacy KB/chunk identity. Model verification has a
separate configuration-fingerprinted contract and is not the causal repair for
durable-accounting admission; it is required only when its own captured
provider/credential inputs changed.

## Bounded production work

### 1. Durable-store eligibility and run-admission diagnostics

The state implementation should introduce one host-owned, safe durability
status for the existing `PersistenceLayer` capability boundary. It must describe
only a category such as local durable, remote attested, remote not attested, or
unsupported backend. It must not disclose database URLs, namespaces, users,
credential material, storage internals, or a raw error chain.

| Owner | Exact production files | Required behavior |
| --- | --- | --- |
| `uar-state` | `src/uar/persistence/mod.rs`, `src/uar/persistence/providers/surreal.rs` | Replace the boolean-only explanation available to callers with a typed, captured-at-construction eligibility reason while retaining the current durable-accounting and durable-ingestion gates. Local SurrealKV and attested remote Surreal retain their existing eligible behavior. The Surreal provider must not re-read mutable environment state per request. |
| `uar-runtime` | `src/uar/runtime/cost_budget.rs`, `src/uar/runtime/manager.rs` | Translate only approved persistence/accounting failures into an allowlisted run-admission category and stage. Preserve safe public failure behavior while emitting the category and protected diagnostic reference in logs/receipts. Keep raw constructor error chains out of logs because credential handling can precede the scrubber. |
| `uar-state` with `uar-runtime` review | `src/uar/api/knowledge.rs`, `src/uar/rag/ingestion_worker.rs` | Use the same safe eligibility category for the existing HTTP 503 and dispatcher diagnostic without changing `202` acceptance rules. Preserve the current client-compatible public message unless the runtime owner explicitly versions the response contract. |

Required category vocabulary is bounded to the observed failures:
`remote_surreal_durability_not_attested`, `durable_storage_unsupported`,
`accounting_verified_principal_required`, and existing accounting reconciliation
states. Do not add generic "retry" treatment for an accounting refusal.
`accounting_scope_reconciliation_required` retains its existing liability and
requires reconciliation; it is not made equivalent to a new/empty ledger.

### 2. Operator recovery is documentation and evidence collection

No KB import code, data migration, or team-recovery implementation is selected.
The following documentation updates make the supported route operationally
legible:

| Owner | Exact documentation files | Required content |
| --- | --- | --- |
| `uar-state` | `docs/operations/durable-ingestion.md`, `docs/operations/knowledge-principal-isolation.md` | State that remote durable ingestion and bounded accounting both require an operator-attested durable profile. Give the reupload sequence: establish backing durability; capture the current verified principal and namespace/database identity; inventory legacy and principal rows read-only; create/reuse a principal KB; reupload retained source bytes; wait for the durable job to become indexed; verify retrieval under the exact principal. State that legacy rows remain preserved and invisible to principal reads. |
| `uar-trust-tools` | `docs/operations/supervisor-recovery.md`, `docs/agents/collaboration/team-execution-profile-migration.md` | Clarify the owner view, authenticated quiesce, trusted supervisor exclusion, exact reclaim receipt, uncertainty reconciliation, and blocked state when no qualified proof exists. Explicitly reject `force`, timeout, manual record deletion, Pod-absence, and replica-count procedures. |
| `uar-runtime` | `docs/operations/durable-ingestion.md` | Document the safe run-admission diagnosis and why provider/model reverification is separate from storage eligibility. Preserve #384 timeout inheritance as an independent contract. |

If a later issue requires imported IDs/chunks, it needs a separate proposal and
must specify a trusted administrator, independently verified owner mapping,
legacy writer quiescence, source snapshot digest, full base/document/chunk/job
closure, source-byte availability, embedding provenance, atomic publication or
an explicitly durable staged protocol, collision refusal, idempotent replay,
and rejection of changed source or competing mappings. It must not promote a
legacy pending/processing job merely by changing its namespace marker.

### 3. Planned fixture ownership after implementation completes

These fixtures are the narrow final-boundary evidence additions. They are not
authorized to run while the phase remains incomplete.

| Owner | Exact fixture files | Required scenario |
| --- | --- | --- |
| `uar-state` | `tests/durable_ingestion.rs`, `tests/principal_knowledge_namespace.rs` | Remote Surreal without attestation rejects durable acceptance and bounded-accounting admission with the safe category; an attested remote profile commits source bytes/job state and recovers them through a joined restart. Principal and legacy tables stay disjoint for same-subject/different-issuer and tenant cases; reupload produces readable principal content without adopting legacy rows. |
| `uar-runtime` | Existing model-binding coverage at `src/uar/runtime/manager.rs` or a new targeted integration fixture only if no existing public run-admission fixture can exercise the API | A budgeted run reports the allowed `remote_surreal_durability_not_attested` category and stage without raw error material; unsupported accounting and missing verified principal remain distinguishable. Confirm a provider is never called after refusal. |
| `uar-trust-tools` | `tests/integration/live/collaboration_conformance_cases.rs` | Clean shutdown/restart reacquires only after joined release; reclaim accepts a one-time exact qualifying receipt, transfers only queued work, and retains running/cancelling effects as uncertain with reserved-unknown accounting. No proof remains refused. |
| cross-owner integration | `tests/support/fresh_process_resume_process.rs` and the phase-selected live integration target | A real configured remote profile is used to establish the restart and durable-accounting boundary. This runs only as part of the one serialized full-phase production gate, not as a standalone early check. |

No fixture should be added for a privileged KB import because the feature is not
selected. A fixture that seeds legacy data remains valid only to prove isolation
and reupload recovery; it must not make the source tables appear migrated.

## Interface handoff

### To `uar-runtime`

Input: a typed durable-store eligibility category from `PersistenceLayer`; no
database error text and no mutable environment lookup.

Output: a stable, allowlisted run-admission category and stage in protected
diagnostics/logging, with a generic public failure retained unless a separately
approved response version changes it. Runtime must preserve the existing
`with_durable_accounting` ordering: refusal before model/tool effects, exact
owner partitioning, and unreconciled accounting liabilities.

Decision requested from runtime: identify the existing run admission/API
receipt shape that can carry a protected diagnostic reference without changing
the AG-UI public error contract.

### To `uar-trust-tools`

Input: documentation-only recovery scope and the assertion that no new
authority is required for reupload or team reclaim.

Output: authenticated operational procedures that use the existing
administrator and trusted-host boundaries. Trust must keep the privileged
supervisor endpoints restricted to verified installation/service hosts and
must not accept user-provided exclusion proof JSON as recovery authority.

Decision requested from trust: confirm that documentation can refer operators
to the existing authenticated owner/quiesce/reclaim APIs without exposing a
new public recovery capability.

## Rollout and closure prerequisites

These facts are not Analyze blockers. They are required before an operator
attests a deployment as durable or the phase declares #385 recovered.

1. A sanitized, read-only capture of the live persisted agent's effective
   budget, provider route, and relevant run policy; repository JSON is not
   enough.
2. Remote Surreal engine/version, namespace/database identity, durable volume
   or server storage evidence, and a cold-restart survival observation before
   setting the existing remote durability attestation. This evidence is
   deployment-specific and does not result from a source fixture.
3. Read-only legacy/principal KB inventory and the exact issuer/tenant/subject
   owner mapping before recovery. Reupload needs source material; missing source
   bytes mean that document must be obtained again, not reconstructed from a
   chunk claim.
4. Actual post-change diagnostic category or a phase-authorized equivalent
   production-path result linking the failed run to the safe cause.
5. Current team fence/owner state and, if recovery is needed, either joined
   shutdown evidence or a qualified supervisor exclusion receipt. Absence of
   either blocks team takeover.
6. Final full-phase, serial production-path evidence for the selected durable
   profile, owner isolation, reupload retrieval, and team recovery boundary.

## Version and scope limits

The repository's authoritative `versions.toml` currently pins SurrealDB,
`surrealdb-types`, and the server/image at 3.3.0, while the role brief mentions
3.2.4. This analysis introduces no SurrealQL, dependency, migration, or
vendor change. The lead must resolve that contradictory version statement before
any version-sensitive implementation or fixture claim; the checked-in pin is
the only source-backed value observed here.

No production source has been changed by this analysis. The proposed typed
eligibility interface is intentionally limited to the observed #385 failure
class. It does not authorize a general persistence-health API, a new operator
write capability, blind accounting reconciliation, or a cross-project fix.
