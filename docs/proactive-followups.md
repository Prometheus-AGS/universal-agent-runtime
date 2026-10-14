# Durable conversation follow-ups

Follow-up rules consume committed logical-agent occurrence metadata and deliver
it to the owner's durable conversation inbox. Creating a rule defaults to
disabled. The owner must opt in and select event kinds, a source instance, an
owned destination instance, and its exact conversation/session ID. Creation
starts after the source's current durable high watermark; it does not import
past events.

The first release sends no email, push notification or external connector
message. It does not create a model turn, observer agent or second executor.
The inbox is an authenticated API surface; there is no new assistant UI.

## API

All routes require an authenticated owner and `x-uar-workspace-id` header.
The base is `/api/uar/followups`.

| Method | Suffix | Action |
| --- | --- | --- |
| POST | `/` | Create a rule |
| GET | `/` | List owned rules and decisions |
| GET | `/{id}` | Read a rule |
| PUT | `/{id}` | Revise policy |
| POST | `/{id}/enable` | Opt in |
| POST | `/{id}/disable` | Disable and suppress queued work |
| POST | `/{id}/revoke` | Permanent tombstone and queued suppression |
| GET | `/inbox/{conversation}` | Read currently authorized metadata |
| POST | `/inbox/{conversation}/{item}/acknowledge` | Idempotent acknowledgement |

Create body:

```json
{
  "sourceInstanceId": "source-agent",
  "destinationInstanceId": "assistant-agent",
  "conversationId": "owned-session-id",
  "eventKinds": ["turn_completed", "turn_failed"],
  "enabled": false,
  "quietHours": {"start": "22:00", "end": "07:00", "timezone": "America/Chicago"}
}
```

An edit takes `{"expectedRevision":1,"rule":{...create fields...}}`. Controls
take `{"expectedRevision":1}`. Editing source, destination or conversation
requires a new rule. Edits can refresh current binding and representation
references after owner authorization; pending decisions recheck the new policy.
A revoked rule cannot be re-enabled. Disabling does not replay suppressed events
when enabled later. Quiet hours may be `null` or omitted.

The native tools are `followup_create`, `followup_list`, `followup_get`,
`followup_update`, `followup_enable`, `followup_disable`, `followup_revoke`,
`followup_inbox`, and `followup_acknowledge`. All take `workspaceId`; create and
update wrap the request in `rule`. Inbox tools take `conversationId` and
acknowledgement also takes `itemId`. Mutations require the normal host tool
approval; reads do not. Only verified native execution context establishes owner.

Errors use `followup_invalid` (400), `followup_conflict` (409),
`followup_not_found` (404), `followup_denied` (403), or
`followup_unsupported`/`followup_storage_unavailable` (503). Malformed JSON may
use the framework's 422 response. Unsupported storage never reports success.

## Authority and persistence

Rules retain the complete verified issuer, tenant and subject key in a protected
host-issued grant. The restore operation accepts protected storage state and
exact rule/workspace/destination scope; caller JSON cannot supply it. It does
not mint roles, JWTs or a general user context. Current source and destination
binding, representation revision and grant references are checked before
admission, delivery and inbox reads. A failed source can still report its failure;
a disabled instance is unavailable. Source payload and credentials never enter
the inbox; the item contains occurrence identity, event kind and timestamps.

The existing durable catalog CAS commits cursor advancement and a queued or
suppressed decision together. Queued/deferred decisions are the outbox. One CAS
commits the logical inbox item and delivered decision, keyed by rule and durable
occurrence. Binding and grant state, consent and policy revision are checked
again in that transaction. Racing scanners cannot create a second logical item.
Restart after decision commit retries delivery; restart after inbox commit sees
the existing item. Acknowledgement updates the same protected record idempotently.
It is not proof of human receipt or reading.

Suppression records distinguish disabled consent, revocation, unselected events,
and unavailable source/destination authority. A source retention gap is recorded
as `source_gap:<first>-<last>` rather than silently skipping history. A storage
outage leaves work pending rather than treating the outage as consent revocation.
Rule inspection exposes decisions and reasons without source event content.
Revoking authority or consent also hides previously delivered metadata from reads.

Each existing observer wakeup visits up to 128 rules and consumes at most one
new source event and one queued decision per visited rule. Multiple runtimes may
perform this metadata-only work under catalog CAS; no supervisor execution lease
or second timer is created. Schedules still use their separate supervisor fence
before admitting agent execution.

Local durable SurrealKV and shared networked SurrealDB are the required delivery
profiles. Memory and providers without durable observer occurrences return
unsupported. This does not assert Postgres observer parity. Existing snapshots
default the new rule/inbox maps to empty; source occurrence storage is reused.
The `supported()` predicate is identical to the operations' storage predicate.

## Quiet hours

Times are exact `HH:mm`, with an explicit available IANA timezone. Start is
inclusive and end exclusive. Start greater than end crosses midnight; equal
times are rejected. Both real instants of a repeated wall time are quiet when
their local time is inside the interval. Release is the earliest future real
instant outside the interval, including a timezone transition if the specified
end does not exist. Every wakeup rechecks current policy rather than trusting
a previously computed release timestamp.

Dated fixtures use America/Chicago: on 2027-03-14, `01:00–02:30` releases at
08:00Z (03:00 local); on 2027-11-07, `00:30–01:45` releases a 06:30Z candidate
at 06:45Z and a 07:30Z candidate at 07:45Z. A 22:00–07:00 winter window includes
04:00Z and excludes 13:00Z. System timezone database provenance must be retained
with final evidence. Windows timezone availability remains explicitly
experimental until its installed artifact is qualified.

## Deferred production evidence

The uncomfortable thing: source inspection does not prove crash recovery or
revocation races across real runtimes. No test, check, compiler or build was run
for this slice before the full phase boundary. `tests/dot_followup_quiet_hours.rs`
and `tests/fixtures/dot15_followups_real_path.py` are authored fixtures only.

After the lead declares the entire phase implemented, the real-path runner must
exercise local cold restart and two shared runtimes, crash before/after decision
commit and inbox acknowledgement, race scanners, revoke source/destination
grants, edit quiet policy while deferred, confirm suppression, and prove one
logical inbox item. Generate events through the installed logical-instance API;
do not inject protected records. The HTTP fixture takes explicit environment
inputs and preserves observations between invocations. Its caller owns runtime
lifecycle, controlled clock and grant changes. No external transport is supported,
so no unknown external send may be retried or presented as delivered by this host.
