# Automatic memory capture qualification

`scripts/dot/memory_capture_journey.py` exports
`run_memory_capture(config, evidence)` for the full-phase acceptance runner.
It is source-only until that runner opens the complete production boundary.
There is no standalone execution entrypoint. No VM provisioning, certificate
installation, process startup, clock change, or product check has been performed
while authoring this driver.

This is a separate dedicated-VM stage, with no ordinary or calendar stage host
processes running concurrently. It imports the existing `GuestClock` from
`calendar_journey.py` without changing its authorization or isolation checks.
Clock evidence uses that class's `calendar-*` names inside this stage's separate
evidence directory. The actual guest wall clock advances from January 1, 2027
through February 2, 2029. Fixture credentials, grants and certificates must span
that interval. See [the calendar provisioning contract](dot-calendar-qualification.md)
for machine/boot identities, isolated guest receipts and clock restoration limits.

The caller supplies these fields:

| Field | Contract |
| --- | --- |
| `hosts` | One local-KV or two shared-Surreal standard `Host` specifications; the stage owns child lifecycle. |
| `guest_clock` | The existing exact isolated VM configuration required by `GuestClock`. |
| `provider_port` | The stage's controlled model provider loopback port. |
| `chat_model` | Actual configured model identity for that provider, such as its admitted provider-scoped `gpt-4o-mini`. |
| `chat_agent_id` | Actual compatible-route agent identity admitted through ordinary host configuration/catalog. |
| `database_provisioning_evidence` | `{ "path": "absolute provision receipt", "sha256": "digest" }` showing real durable storage and the database clock in this guest. |
| `embedding_peer.certificate` | Digest-bound absolute PEM certificate path. |
| `embedding_peer.private_key` | Digest-bound absolute matching PEM private-key path. |
| `embedding_peer.provisioning_evidence` | Digest-bound absolute guest-only DNS and trust provisioning receipt. |

Host configuration must enable memory and automatic capture, disable context
injection for deterministic capture-only observations, and select
`embedding_provider: "openai"`, `embedding_model: "text-embedding-3-small"`,
`openai_api_key: "dot-memory-fixture-only"`. This dummy key is required. The
controlled model provider must also use fixture credentials. These settings do
not alter production memory persistence or consent. They choose the actual
existing external embedding and model collaborators.

The pinned memory client's URL is fixed at
`https://api.openai.com/v1/embeddings`; it has no configurable base URL. Therefore
the disposable guest must already resolve that name only to `127.0.0.1`, trust
the supplied fixture certificate for that hostname, and permit the stage to
listen on guest loopback port 443. Configure no outgoing proxy for that hostname.
The driver verifies the loopback DNS and file digests and uses a real TLS listener.
It never disables certificate verification, edits DNS/trust configuration, or
forwards requests to the public provider. The native installed client's actual
request must reach the peer and parse its dimension-correct embedding response;
a missing or unsupported guest trust setup blocks the scenario.

Both shared memory clients must use the same durable SurrealDB namespace/database
at a guest-local endpoint. An external database in a different clock domain
cannot qualify this driver: retention transactions use database `time::now()`.
Embedded local KV uses the runtime's guest clock. Provisioning receipts document
the actual storage setup; they do not supply scenario results. The driver never
edits memory rows, tombstones, retention controls or policy rows through database
APIs. All policy and content reads/mutations use authenticated production HTTP.

The actual capture route is `POST /api/chat/completion` with `stream: true`,
`stream_mode: "openai"`, an exact session header, and `memory_enabled: true`.
The driver consumes the finished SSE response and validates its assistant text
and native run/session correlation. Production stream completion queues capture;
the actual memory service embeds the user and assistant content and commits it
through its normal consent transaction. Exported `uar_capture` metadata and
`/api/memory/records/{id}/control` provide first-capture and expiry evidence.

Authored scenarios cover:

- A complete default-off chat emits no automatic records or embedding requests
  during a bounded eight-second observation.
- Separate actual captures under 7, 30, 90, 365 and until-deleted policies expose
  the committed revision and exact first-capture expiry. Increasing policy does
  not extend existing records. Advancing the guest and database clock across
  each fixed boundary removes expired records from read, export and history;
  until-deleted records survive.
- Shortening a new 365-day capture to 7 days changes its existing bound. Extending
  to 90 days leaves that bound unchanged and gives only new captures 90 days.
  Repeating the expired session/content through the real stream and embedding
  path cannot revive its retained identity.
- A real user-content embedding request is held before replying. Another
  authenticated host changes policy to 30 days; release then produces captures
  using the current committed revision and retention.
- A second held request is disabled and re-enabled before release. Its old
  consent epoch cannot commit. The sequential assistant embedding request proves
  the first save returned after revocation; bounded absence checks and a fresh
  successful capture distinguish revocation from a disabled fixture.
- An until-deleted capture survives more than a year and joined runtime restart,
  then disappears from all readers after the actual owner deletes it.

The embedding peer is an external protocol collaborator. It records requests
independently before replying and holds only explicitly selected fixture content.
It supplies deterministic 1536-dimensional vectors; it never implements memory
storage, retention or authorization. Capture has no production queue-pause API,
and this real embedding I/O boundary makes one unnecessary. The gate proves
host control semantics, not embedding-model quality.

The uncomfortable thing: asynchronous capture has no public task-completion
receipt. Revoked final-content absence is observed for a bounded interval, with
the preceding sequential request establishing the first transaction boundary;
it is not a proof over infinite future execution. Clock cleanup can also be
interrupted, so disposable-guest isolation remains mandatory. Nothing in this
document authorizes provisioning or execution before the full-phase gate.
