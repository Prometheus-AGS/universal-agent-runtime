# Session and adjacent state ownership

The authenticated host derives history, ACP sessions, conversation policies and
knowledge access from the complete verified credential principal: issuer kind,
issuer, tenant and subject. A caller-selected owner or subject cannot substitute
for that identity. Catalog and Presentation ownership retain their existing
contracts; their keys are not reused for private conversation history.

New authenticated session records use a separate `principal-session:` physical
namespace in the existing sessions table. Conversation policies use the existing
principal policy table with the complete principal key. APIs, native session
search, actor context restoration, logical-instance restoration and final history
persistence use the same key. Session-to-current-run lookup and cancellation check
the same verified owner. The evidence API requires either a live run owned by the
caller or protected checkpoint provenance bound to that complete owner/run before
reading its underlying audit records.

Only the actual anonymous context may read anonymous compatibility history or
policies. Bare legacy rows must themselves be anonymous before migration into the
anonymous owner partition. Authenticated callers never adopt a row through its
old subject or logical session ID. Signed-in users with the same subject under a
different issuer or tenant receive independent histories and policies.

The uncomfortable upgrade behavior: old authenticated subject-only histories and
policies are preserved but no longer automatically attached to signed-in users.
This change cannot prove which issuer/tenant owned those rows. Any future migration
requires independent ownership evidence and explicit authorization. Signing in with
a matching subject is insufficient.

Session erasure deletes the exact owned session and its corresponding policy.
Accounting, cost and admission records have separate trusted-host lifecycles and
are retained. A session ID does not authorize broad audit or accounting deletion.

## Authored qualification contracts

All fixtures remain unexecuted until the complete Dot production boundary opens.
No source-authored observation is acceptance evidence.

- `tests/dot_owner_adjacent_contract.rs` is one whole integration target for
  anonymous compatibility and equal KB/document/chunk identifiers (cases41/44),
  using actual SurrealKV and shared SurrealDB providers. Its seeded records are
  fixture setup through production contracts, not proof of HTTP authorization.
  It also places an old session in the legacy physical namespace with an owner
  string identical to the verified principal key. Principal reads and deletion
  cannot adopt it; an independent principal write/delete leaves its history intact.
  Deleting one of three same-ID sessions preserves the other owners and the actual
  provider's independent session cost history.
  Shared configuration uses `UAR_DOT_MEMORY_SURREAL_URL`, optional
  `UAR_DOT_MEMORY_SURREAL_USER`/`UAR_DOT_MEMORY_SURREAL_PASS` and a unique database.
- `scripts/dot/owner_isolation_journey.py` exposes
  `run_owner_isolation(host, spec, evidence)` and
  `run_unresolved_knowledge(host, spec, evidence, provider)` for the existing fixed runner.
  It uses real authenticated native/ACP/session/policy APIs and actual worker
  upload, indexed search, run retrieval and captured upstream provider requests.
  Credential labels are `owner`, `other_tenant`, `other_issuer`, all the same subject.
  Owner and other_tenant authenticate under the configured issuer. Other_issuer
  is explicitly rejected with401 and recorded as authentication rejection, never
  accepted-principal isolation. Cross-issuer authenticated storage isolation uses
  the separate issuer-configured nodes of `credentials_api_integration_test` and
  the verified-principal real provider contracts. IDs are
  `owner-thread-isolation`, `owner-adjacent-isolation`, and
  `knowledge-unresolved-selection` (cases39/40/43). Provider capture uses the real
  local or Kubernetes adapter's `rows()` method, not a controller-local trace path.
  `session-erasure-authority-scope` additionally deletes the completed real run's
  session, verifies own policy removal and other principals' policy retention,
  and requires unchanged nonempty actual accounting and admission records.
  `owner-untrusted-issuer-rejection` records the distinct authentication control.
  Canonical-looking legacy knowledge subjects are covered separately by the whole
  `principal_knowledge_namespace` target, including its preserved old-table rows.
- Case42 remains the whole real `dot_memory_owner_contract` target. Its spoofed
  owner is rejected at the real verifier/service boundary; it is not proof of an
  installed binary.
- `tests/dot_stale_epoch_contract.rs` is a separate whole target for case33, after
  qualified native/CRI supervisor exclusion. `UAR_DOT_STALE_EPOCH_CASE` points to
  JSON with `database_url`, `namespace`, `database`, optional credential variable
  names `user_env`/`password_env`, `qualified_observation`,
  `qualified_observation_sha256`, `effect_trace`, `evidence_owner`,
  `accounting_owner`, and `root_run_id`. The observation must be the actual
  supervisor runner's exclusion file containing `data.http` and
  `data.observation`, bound to the same registered predecessor and consumed
  challenge. It loads an existing predecessor admission, attempts new stale
  admissions through the production store, and requires `ExecutorClosed` with
  unchanged evidence, canonical receipts, accounting and effect trace. It never
  constructs supervisor proof or calls a new public write endpoint. The trace
  and predecessor scope must be quiescent during this target. A local KV case
  requires its owned runtime stopped and joined before opening the database;
  shared remote cases use their actual isolated durable database. Qualification
  metadata alone cannot pass: the actual provider must reject the closed epoch.

The final validation locations selected by the operator are this machine and the
existing know-me LKE environment through its Envoy TLS gateway. No fixture authoring
provisions, deploys, changes clocks or reboots either environment.
