# Owner memory capture and removal

Status: production source authored in Dot Execute; no compilation, test, format,
lint, validator, probe or product review has run for this slice. The full expanded
Dot implementation boundary still governs acceptance.

## Public contract

Authenticate through the runtime's normal verified owner credential. The host
mounts the following routes beneath `/api/memory`:

- `GET /policy`: durable `enabled`, `retention`, `revision`, `consent_epoch`, and
  the effective host capability switch `host_capture_enabled`.
- `PUT /policy`: `{ "enabled": true, "retention": { "kind": "days", "days": 7 } }`.
  Day values are exactly 7, 30, 90, 365. The other choice is
  `{ "enabled": true, "retention": { "kind": "until_deleted" } }`.
  Unknown/mixed shapes and enablement without retention are rejected.
- `PUT /policy` with `{ "enabled": false }`: revoke future automatic capture,
  including already admitted work. Existing memories keep their retention.
- `GET /records`: owner export of currently visible memories, with protected
  `metadata.uar_capture` provenance on new records.
- `GET`, `PATCH { "content": "replacement" }`, `DELETE /records/{id}`:
  direct owner operations. Deletion acknowledges durable suppression and pending
  purge; it does not wait for physical index cleanup.
- `GET /records/{id}/history`: visible history only. Removed/expired memory
  returns no content (404), including imported legacy history.
- `GET /records/{id}/control`: non-content provenance and deletion/purge status,
  including after the memory itself is physically gone. Foreign IDs are 404.

MCP exposes `memory_capture_policy` and `memory_set_capture_policy` under the
existing authenticated owner session. Existing MCP memory read/write, history,
search and TaskStream routes use the same enforcement. The native
`memory_capture_policy` tool is read-only. Model output cannot construct a
capture admission or grant consent through the native tool. Consent writes are
explicit authenticated owner HTTP/MCP operations performed by host services.

`MemoryService::capture_policy`, `set_capture_policy`, `memory_control_status`,
`capture_admission`, and `auto_capture_admitted` expose the same host service
contract. A queue must obtain the opaque admission before enqueueing. Missing
admission means no capture; it is never silently reacquired by the worker.
Ordinary synchronous `auto_capture` admits its call before embedding starts.

Memory service enablement and `memory.auto_capture` remain host switches. The
owner policy defaults **off**, even on an existing store with memories. A true
host capability switch is not consent. A false switch is visible in policy
responses and rejects enablement rather than pretending capture is active.

## Durability and time

Controls use additive `uar_memory_policy` and `uar_memory_control` tables on the
same SurrealStorage engine and owner database (`<database>__owner_v1`) established
by Dot12. Supported deployment shapes are local durable SurrealKV and the same
shared durable network SurrealDB used by multiple runtimes. Every host must use
the same namespace/database and owner identity contract. No new database engine,
vendor changes, PostgreSQL assistant-memory parity, or durability inference from
an in-memory store is claimed. Builds without the Surreal backend return an
explicit unsupported error when memory controls are initialized.

First durable capture uses the database UTC clock in the atomic commit. Day
retention is exact 24-hour multiples. At `now >= effective_expires_at`, every
ordinary read, export, direct read, search/context and history path suppresses
content before physical cleanup. Shortening stores a smaller bound on existing
captures in the policy transaction. Later extension or until-deleted affects
new captures only. It cannot remove stored smaller bounds.

Capture keys are stable hashes of owner, agent, session, role and content.
Uncertain retries retain the same capture ID, first timestamp and expiry. A
content tombstone prevents automatic recapture under that identity. Explicit
user saves have fresh identities and remain allowed without automatic consent;
existing explicit/legacy records are never reclassified by inference. This host
adapter does not use the vendor's semantic deduplication mutation, which could
otherwise refresh a removed capture or mutate explicit memory during capture.

Policy mutation and capture write the same owner policy row. Revocation advances
a consent epoch; a previously admitted queue item is refused even after a later
re-enable. Retention changes without revocation use the policy current at
commit. Concurrent database write conflicts propagate; they are not disguised
as success or automatically retried with fresh consent.

## Deletion, history and derived records

Deletion first commits an owner tombstone, removes stored edit-history rows and
clears metadata (including Dot12 imported history). The record and its vector
are suppressed immediately by all public read paths. Startup and the existing
TTL worker then idempotently remove the physical record/vector/history and mark
`purged=true`. Cancellation after the tombstone cannot undo suppression. Tombstones
retain identity, owner, timestamps, reason and provenance, never memory content.

Compression and TaskStream summarization fence source versions and control rows.
Derived records inherit expiry and flattened source identities. Owner deletion
or expiry of any source suppresses descendants immediately; purge discovers and
removes them. Normal source retirement during compression is distinct from owner
deletion, so a newly committed summary remains usable. Earlier shortening cannot
be lost by embedding or summary work racing a policy update.

Dot12's original unsuffixed legacy database remains quarantined and is not an
owner read/export surface. Imported copies, including embedded legacy history,
are removed normally and stable import replay cannot bypass their tombstones.
Original quarantined operator source material follows the separate migration
retention procedure; this change does not silently destroy that evidence.

Rollback must retain these tables and must not serve this owner database through
an older binary that ignores them. Physical purge is irreversible content
removal; restoring an old content backup without its tombstones can reintroduce
removed content. Keep controls with backups and complete purge before exporting
raw database snapshots as user exports. Raw trusted-host database handles are
maintenance authority, not an owner export interface.

## Deferred evidence

Authored `tests/dot_memory_controls.rs` reuses the production MemoryService,
verified JWT path, native tool adapter and pinned real store. It joins processes
between local KV opens and holds an opaque admitted queue item in one shared
process while a second revokes and re-enables consent. It also covers default-off,
retention input rejection, immutable replay anchor, all four day intervals,
shortening/extension across restart, pending-delete recovery, history removal,
imported-history metadata, and repeated purge. Clock-boundary setup writes only
fixture timestamps; production mutations and read enforcement remain real.

After the lead records the entire expanded phase implementation inventory,
include this existing target in the serialized local gate:

`cargo test --locked --no-default-features --features server-full --test dot_memory_controls -- --nocapture --test-threads=1`

It requires an isolated durable `UAR_DOT_MEMORY_SURREAL_URL` and optional
`UAR_DOT_MEMORY_SURREAL_USER`/`UAR_DOT_MEMORY_SURREAL_PASS`, plus the configured
local embedding model. It does not contact a hosted completion provider. The
Dot12 fixture now explicitly opts in before its expected capture.

The uncomfortable thing: these are authored scenarios, not evidence. SQL/schema
compatibility, real concurrent transaction conflicts, exact boundary behavior,
HTTP/MCP composition, derived-summary races, failure during index cleanup and
the customer artifact still need the complete phase's production gate. No
previous passing receipt certifies these changes.
