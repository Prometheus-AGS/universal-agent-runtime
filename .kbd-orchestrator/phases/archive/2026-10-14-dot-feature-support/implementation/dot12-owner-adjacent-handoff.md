# Dot12 task3 owner and adjacent-state source handoff

Status: caller/session/policy production source and bounded qualification fixtures
authored. No verification or acceptance claim. Knowledge namespace implementation
is the separate `dot12-knowledge-namespace-handoff.md` packet. Root owns canonical
inventory/phase transitions and runner composition remains with Dot17 worker.

## Observed requirement gaps and resulting contract

Same-subject identities under different tenants/issuers previously shared session
keys, ACP sessions, native session search and KB ingress/retrieval owner strings.
Conversation-policy loading fell back to ambiguous subject-only rows. Session
removal deleted cost rows by logical session ID without owner binding; Surreal also
removed all owner tool evidence. The tool-evidence HTTP route used raw subject
without establishing run ownership and reconciled unrelated runs before filtering.
These are concrete source trust boundaries implementing cases39–44, not adjacent
speculative changes.

`UserContext::session_owner_key` derives CredentialPrincipal.storage_key from a
verified context. Anonymous compatibility requires the exact anonymous subject,
no tenant, no verified authority and no trusted host authority. ActorOwner's
fallible equivalent requires its captured credential principal. Catalog and
Presentation keys remain unchanged. History uses `principal-session:` physical
namespace; authenticated policies use the existing principal policy table with
full principal key and no legacy fallback. Legacy bare-row migration verifies the
row itself is anonymous. Signed-in legacy rows are preserved without automatic
adoption. The evidence API requires a matching live owner/run or protected full
owner/run checkpoint before entering the underlying audit reader; reconciliation
is filtered to that run before any stale-state append.

Session deletion only removes the exact session and corresponding owned policy.
Independent accounting and tool evidence are retained. Knowledge caller keys pass
unchanged to the teammate's disjoint principal tables; ingestion owner comes from
the authenticated document, never request owner data. Anonymous admin save rejects
an authenticated owner string because this compatibility API has no principal.

## Files changed in this packet

- `src/uar/security/claims.rs`: strict complete session owner helper.
- `src/uar/runtime/actor/messages.rs`: strict ActorOwner helper; no catalog key change.
- `src/uar/api/acp/handler.rs`: canonical ACP session ownership; full-context run getter.
- `src/uar/runtime/manager.rs`: canonical history/current-session keys, retention protection,
  actor history persistence, strict missing-principal refusal, KB policy inventory/retrieval,
  current-run lookup by context, audit run filter before reconciliation.
- `src/uar/runtime/actor/system.rs`: A2A context durable history uses full owner key.
- `src/uar/runtime/thread/actor_host.rs`: durable session persistence passes verified owner.
- `src/uar/runtime/instance/controller.rs`: logical-instance history restoration uses full key.
- `src/uar/tools/session_search.rs`: native history uses captured full principal.
- `src/uar/api/discovery.rs`: current-run lookup, policy/cache read/write/delete, session
  deletion and KB inventory honor canonical private owner; no authenticated legacy fallback.
- `src/uar/domain/policy.rs`: authenticated policy load cannot adopt legacy subject rows.
- `src/uar/api/routes.rs`: tool evidence full-principal run/checkpoint authorization.
- `src/server.rs`: context-stats, chat history fallback and knowledge sync caller scopes.
- `src/uar/api/knowledge.rs`: all knowledge/doc/search/job/retry handlers derive verified key.
- `src/uar/admin/knowledge.rs`: anonymous compatibility save cannot select another namespace.
- `src/uar/persistence/mod.rs`: disjoint session storage key helper.
- `src/uar/persistence/providers/{surreal,postgres,memory}.rs`: session/policy spans only;
  storage namespace, anonymous migration checks, remove unbound cascades. Knowledge
  spans in these same files were owned by MCP teammate and preserved.
- `tests/durable_ingestion.rs`: actual production JWT verifier replaces synthetic context;
  canonical fixture owner, no execution.
- `tests/dot_owner_adjacent_contract.rs`: whole local/shared real provider cases41/44.
- `scripts/dot/owner_isolation_journey.py`: real HTTP39/40 and real worker/provider43.
- `tests/dot_stale_epoch_contract.rs`: separate whole33 using a qualified real closed epoch.
- `docs/operations/session-principal-isolation.md`: upgrade and qualification contracts.
- `openspec/changes/dot-12-memory-isolation/{design.md,specs/multi-tenant-isolation/spec.md}`:
  explicit namespace/non-adoption and erasure scope scenarios, no task checkbox changes.

## Exact runner packet

Call `run_owner_isolation(host, spec, evidence)` and
`run_unresolved_knowledge(host, spec, evidence, provider)` from the existing running-provider
profile. Same-subject credential labels are `owner`, `other_tenant`, `other_issuer`.
Observations: `owner-thread-isolation`, `owner-adjacent-isolation`,
`knowledge-unresolved-selection`, `session-erasure-authority-scope`. The unresolved scenario first proves an actual
worker-indexed accessible KB reaches a provider system message, then requires the
missing-name selection not to include that chunk; user-message query text is
excluded from this comparison.

The provider argument is mandatory and uses the actual local/Kubernetes adapter's
`rows()` trace reader. Session erasure runs only after a real approved run completes:
nonempty accounting/admission evidence must remain identical, own conversation
policy must disappear and both other principals' policies must remain intact.
The whole provider target additionally seeds the historical session physical key
with a canonical-looking legacy subject, refuses adoption/deletion, writes/deletes
an independent principal session and confirms the legacy messages survive. It also
preserves a real provider cost-history fixture entry across session deletion. Those
seeded rows are retention/collision setup, not host execution/authorization proof.
The distinct `principal_knowledge_namespace` target covers the analogous legacy KB
namespace collision; it must be included in the matrix for that obligation.

Whole source targets: `dot_owner_adjacent_contract` (41/44), existing
`dot_memory_owner_contract` (42), and separate `dot_stale_epoch_contract` (33).
The latter MUST follow qualified supervisor native/CRI completion. Its exact
`UAR_DOT_STALE_EPOCH_CASE` schema is documented in the operator document. It requires
actual registered epoch, consumed challenge, prior tool admission and exact
ExecutorClosed result, plus unchanged evidence/receipt/accounting/effect snapshots.
It never manufactures exclusions, writes supervisor proof or adds a public endpoint.
CRI runner must retain `observation: after` beside `http` in its exclusion observation;
this narrow runner change was requested from Dot17 owner. Local KV must be opened
only after its runtime is stopped/joined; shared case uses actual remote durable
store. Seeded anonymous/reused-ID records prove provider behavior, not HTTP authority.

## Remaining and unverified

All code, schema migrations, concurrency, HTTP status behavior and fixtures remain
unverified under the full-phase rule. No tests, checks, lint, formatter, build,
validator, probe, product review, deployment or infrastructure mutation ran. No
canonical phase acceptance was written. The final required environment target is
this machine plus existing know-me LKE Envoy TLS gateway, after root opens the
whole-phase boundary. Source-only fixture authoring does not authorize a reboot,
clock change, cluster mutation or test. No unrequested feature was added.

The uncomfortable thing: ordinary signed-in legacy history/policy/KB access now
fails closed, and restoring it safely requires provenance the old rows do not
contain. This packet does not invent a migration authority or claim that this
source change has passed the actual production path.

## 2026-10-13 prerequisite correction before validation

Source inspection found the live fixture required other_issuer to authenticate on
a single-issuer host. Dot02 requires complete owner isolation, not new multiissuer
trust; its actual credentials integration uses separately configured issuer nodes
against the same storage. Root authorized the minimum fixture-only correction:
owner_isolation_journey.py and mcp_journey.py retain authenticated other_tenant
controls and explicitly require401 for untrusted other_issuer. The distinct
owner-untrusted-issuer-rejection observation never counts as accepted-principal
isolation. MCP observations identify accepted tenant and rejected issuer separately.
Session erasure now retains the actually authenticated foreign tenant policy.
Operator documents session-principal-isolation.md and dot-mcp-lke-qualification.md
were corrected accordingly. No production changes, gates, probes or runtime calls
were performed. Root must rebind the changed source before runner execution.
