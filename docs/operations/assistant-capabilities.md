# Assistant operations

`GET /api/uar/assistant-capabilities` returns the installed host's operation
inventory. It requires the same verified issuer, tenant and subject as the
operations it describes. `/api/uar/capabilities` links to this inventory.
Discovery grants no authority and is not a release certification.

Every operation has a stable ID, contract revision, required authority,
persistence claim and separate HTTP, native-tool, MCP and direct-embedded
entries. An unavailable entry includes a reason. A route can be installed but
unavailable because its durable store is unsupported or its service is disabled.
Native-tool availability is intersected with the live registered descriptors.
MCP entries name the actual memory endpoint and tool, not a REST-shaped substitute.

Local durable SurrealKV and durability-attested networked SurrealDB use the
same owner-scoped contracts. Backend names alone do not enable operations.
Postgres preserves its supported receipts and approvals but does not acquire
missing checkpoint, observer or schedule support. In-memory and unattested
remote runtime stores cannot substitute ephemeral success for durable work.
Memory has its own explicitly configured Surreal store; its availability is
independent of the runtime catalog backend. Direct embedded `Runtime` does not
expose this host assistant contract and is reported unavailable. Use the
authenticated HTTP client for those operations.

`execution_owner` reports the current catalog execution owner. Another shared
runtime may support management while the owner performs scanning. A true
operation entry does not waive current binding, workspace, consent, budget,
approval, epoch or storage-health checks. An outage returns an error; it does
not silently select an in-memory store.

## Example requests

These examples assume an installed host, an operator-issued bearer token, and
an already-owned logical instance. Environment values are supplied by the
operator. They are examples, not commands run as part of implementation.

```bash
curl --fail-with-body -H "Authorization: Bearer $UAR_TOKEN" \
  "$UAR_URL/api/uar/assistant-capabilities"

curl --fail-with-body -H "Authorization: Bearer $UAR_TOKEN" \
  -H "x-uar-workspace-id: $UAR_WORKSPACE" -H 'Content-Type: application/json' \
  -d '{"instanceId":"owned-instance","recurrence":{"kind":"daily","time":"09:00","timezone":"America/Chicago"},"prompt":"Summarize my open work"}' \
  "$UAR_URL/api/uar/schedules/"

curl --fail-with-body -H "Authorization: Bearer $UAR_TOKEN" \
  -H "x-uar-workspace-id: $UAR_WORKSPACE" -H 'Content-Type: application/json' \
  -d '{"expectedRevision":1}' "$UAR_URL/api/uar/schedules/$SCHEDULE_ID/pause"

curl --fail-with-body -H "Authorization: Bearer $UAR_TOKEN" \
  "$UAR_URL/api/memory/records"

curl --fail-with-body -H "Authorization: Bearer $UAR_TOKEN" \
  "$UAR_URL/api/uar/runs/$RUN_ID/tool-outcomes"
```

The outcome endpoint reads the requesting owner's canonical receipts from the
durable host store. It exposes call ID, sequence, tool, outcome and acquisition
completeness, excluding raw arguments, result bytes and protected authority.
Missing historical outcome is `outcome_unknown` with `legacy_uncertain: true`.
New receipts carry verified issuer/tenant/subject provenance. Historical receipts
without that provenance are visible only while the live run proves the exact
verified owner; a subject-only legacy row is never disclosed after restart.
An empty list does not establish that a run succeeded or never performed work.
These reads cannot reconcile an uncertain effect or authorize replay.

Memory policy, export, record editing/deletion, history and purge state use the
existing `/api/memory` service. Schedule control uses a revision and requires the
workspace header. Follow-ups use `/api/uar/followups` and deliver only to the
owner's authorized conversation inbox. External connectors are unavailable in
this release. Approvals and checkpoint recovery use the existing run routes;
fresh authority and protected continuation checks remain mandatory.

Supervisor exclusion is host-only. Native recovery requires a confirmed reboot
of the same host unless a qualified implemented containment mechanism proves
descendant exclusion. Kubernetes support depends on the selected qualified node
controller. Pod deletion alone is not proof. Exclusion does not settle an
external action or release its accounting hold.

The uncomfortable thing: this inventory reports source-installed capabilities,
not successful end-to-end execution. The final phase gate must exercise every
advertised entry point, negative owner cases and unsupported profiles. New
source has not yet been compiled or executed.
