# S4 installed-image recovery adapter handoff

Phase `uar-outstanding-issues-v1-1-1`, Execute. Sole writer for this packet:
`execute_recovery_integration`. The full-phase verification boundary remains
closed. This is an implementation handoff, not acceptance or independent review.

## Changed paths and interface

- `scripts/release/uar_1_1_1_recovery.py`: replaces the incomplete adapter with
  executable real HTTP/database/restart cases and a configured real supervisor
  path. Keeps `REQUIRED_SCENARIOS["recovery"]` and `run_group(context)`.
- `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/coordination/handoffs/S4-integration.md`:
  this receipt and private prerequisite contract.

No Rust support file was needed. Existing `tests/durable_ingestion.rs`,
`tests/principal_knowledge_namespace.rs`, source state/provider changes, shared
runner/support, manifests, pins, and canonical phase state were not edited.

The adapter uses the lead's existing `Resources` context, including `base_config`,
`new_runtime`, `http`, `jwt`, `wait_for`, `surreal_query`, `postgres_query`,
`start_process`, `relay`, and `docker`. Its only executable child mode is the
source-defined `--embedding-peer <owned-directory>` on this same Python file.
There is no manifest command, SQL, script, or arbitrary argv field.

## Implemented scenario assertions

1. `unattested_remote_upload_refusal`: real multipart route returns 503;
   principal-table before/after snapshots remain identical, and document,
   chunk, ingestion-job and retained-source counts remain zero.
2. `attested_upload_restart_owner_isolation`: Surreal and PostgreSQL each
   accept 202, commit exactly one source/document/job for the observed owner,
   and enter real embedding HTTP preparation. An independent Python HTTP peer
   holds that response so the first runtime cannot finish indexing. The
   adapter removes that exact owned runtime, releases the peer, restarts the
   actual database container, and starts a fresh installed-image runtime.
   The real dispatcher rediscovers the same generation and indexes it. The
   same owner retrieves the new document; other principal, same subject in
   another tenant, and same subject/tenant with another issuer cannot access
   document, job, retry or search. A second cold database/runtime boot must
   preserve the entire indexed principal inventory.
3. `legacy_principal_namespace_cold_readback`: during each backend scenario,
   seeds all four legacy tables using forms from
   `tests/principal_knowledge_namespace.rs`. Legacy owner bytes equal the
   verified principal key read from the real newly created KB. The legacy KB
   logical ID equals that current KB's ID. Document/job/retry routes cannot
   adopt the old record; search cannot return its chunk. Full rows, including
   old job data and source bytes, must remain identical through both cold
   boots. Receipts expose counts/digests, not source configuration or secrets.
4. `joined_quiesce_reclaim_controls`: two installed-image runtimes share the
   owned database. The replacement starts while the predecessor holds the
   catalog, preventing startup auto-acquisition. Missing and forged evidence
   reject with 409 and identical full durable catalog snapshots. The old
   runtime's quiesce API joins its execution roots; its exact durable proof
   must name `trusted-runtime-joined`, the original fence and stopped roots.
   The old container is then removed and its absence observed. Reclaim must
   advance exactly one epoch and name that proof. Reusing it at the current
   epoch rejects and leaves catalog effects/accounting unchanged.
5. `qualified_supervisor_reclaim_controls`: when provided the prerequisites
   below, uses the existing controller/recovery_scenarios contracts through
   actual installed-runtime APIs. It requires nonempty real queued and active
   predecessor work, observes absent/forged/ordinary/wrong-incarnation
   denials, obtains the durable challenge and requests physical exclusion.
   It reads the existing controller journal read-only and binds the actual
   observation to the retained registration/challenge/qualification. It checks
   actual whole-domain Kubernetes observations or a distinct actual native
   host boot. Reclaim transfers exactly the supplied queued attempts, keeps
   active work uncertain and reservations unchanged, advances one epoch, and
   rejects replay without epoch/effect changes.

The embedding peer returns controlled 384-dimensional vectors over the actual
UAR embedding HTTP path. This establishes transport/index/retrieval behavior;
it does not establish external embedding semantic quality. It is not a fake
persistence provider or a synthetic supervisor proof.

## Exact private supervisor input

`infrastructure.supervisor` is an object with these required fields:

| Field | Meaning |
| --- | --- |
| `qualification_file` | Absolute path to the operator's retained qualification JSON. |
| `qualification_sha256` | Exact file digest with `sha256:` prefix. |
| `identity_receipt_file` | Absolute path to the immutable private setup identity JSON below. |
| `identity_receipt_sha256` | Exact file digest with `sha256:` prefix. |
| `controller_journal` | Existing real controller SQLite journal accessible read-only, including its current WAL if applicable. The fixture opens `mode=ro`. |
| `runtime_url` | Replacement runtime's exact endpoint from the setup receipt; HTTPS with normal trust, or loopback HTTP. Redirects and ambient proxies are rejected. |
| `host_token` | Existing verified trusted-host JWT accepted for supervisor challenge/exclusion. |
| `ordinary_token` | Ordinary authenticated JWT that must fail supervisor authority with 403. |
| `admin_token` | Authenticated owner JWT **without** `uar_instance_id`; collaboration administration rejects host-instance tokens. |
| `admin_key` | Installed replacement's collaboration administration key. |
| `team_owner_token` | Verified owner of the prepared team scopes. |

Tokens are resolved only in the private infrastructure file and must never be
copied to public manifests/docs/receipts. Resources registers them for redaction.
`host_token` and `admin_token` have different authority contracts and are not
interchangeable.

Qualification JSON uses the existing controller contract, including:
`controller_id`, `qualification_id`, `mechanism`, `production_qualified: true`,
and `controller_sha256` (plain hexadecimal). The last digest must equal the
current `scripts/supervisor/controller.py` bytes. The adapter neither creates
nor promotes a qualification. The artifact's outer `sha256:` digest must match
`qualification_sha256` exactly.

The private immutable setup identity JSON contains:

- `phase: "uar-outstanding-issues-v1-1-1"`, `isolated: true`.
- `image`, `platform`, `source_tree`, `fixture_tree`, exactly matching the gate
  manifest. `runtime_image_id` equals the actual Docker image content ID
  observed for the candidate/published child (`docker image inspect --format
  '{{.Id}}' <manifest.image>`). The operator's setup receipt must bind both
  installed runtimes to these exact bytes, not merely a mutable image tag.
- `runtime_url`, exactly matching the private supervisor endpoint.
- `predecessor_registration`: the actual complete existing
  `SupervisorRegistration` from the controller `domains` journal. Includes
  `runtime_epoch`, `team_incarnation`, `controller_id`, `qualification_id`,
  `deployment_id`, and the actual immutable `domain`. No invented Pod UID,
  sandbox/container IDs, host boot identity, or sentinel may fill these fields.
- `predecessor_fence`: exact `catalogId`, `serviceInstanceId`, `incarnationId`,
  `epoch` from the prepared catalog's held claim. Its incarnation must equal
  registration `team_incarnation`.
- `replacement_fence`: exact same four fields from the replacement's
  `GET /api/v1/collaboration/execution-owner` `currentFence`, before reclaim.
  Replacement must not already own the predecessor's execution.
- `team_scopes`: nonempty array of objects with `team_id`, `workspace_id`,
  `queued_attempt_ids`, `uncertain_attempt_ids`. Collectively both ID arrays
  must be nonempty. Every expected ID is checked against real execution API
  records and the predecessor fence. Queued records must be queued; active
  records must be running, cancellation_requested or already uncertain.
  The scope inventory must include **all** predecessor queued/uncertain work
  because the returned reclaim receipt's full ID sets must match exactly.

The setup must already supply the installed runtime's private SupervisorConfig,
TLS identities, durable store and independently operated qualified controller.
These are prerequisites owned by the operator; the adapter does not accept
configuration/mount argv to build an imitation controller. The API controls and
journal observations reuse `scripts/supervisor/recovery_scenarios.py` and
`scripts/supervisor/controller.py` contracts directly in this installed-image
adapter. No broad Dot qualification or production customer mutation is invoked.

The target must be an explicitly disposable isolated deployment. Kubernetes
physical exclusion can mask the dedicated node's kubelet. Native exclusion
requires an already observed different whole-host boot; this fixture never
reboots a machine. Existing unrelated workloads must not share that containment
scope. Controller qualification and truthful operator setup receipts are trust
inputs, not something a fixture can manufacture from HTTP success.

## Verification and outstanding limits

Commands run for this work were source reads/searches and file edits only.
**No tests, builds, import probes, formatters, linters, validators or independent
review ran.** Therefore syntax, schema behavior, both backend integrations,
container restart sequencing, HTTP scenarios and the new supervisor path are
unverified. Lead must finish L4 and run the existing phase integration gate;
independent fresh-artifact review remains pending.

The operator currently has no known isolated qualified supervisor setup. The
lead's read-only production source inspection found no reusable controller or
qualification. The supervisor scenario therefore remains a concrete external
prerequisite, not an unconditional adapter stub. Missing private inputs produce
a `blocked` required scenario with the completed ordinary scenario observations
retained in the group result; the runner must not accept it. Invalid supplied
identity, physical observations, or production API outcomes fail assertions.
No scenario is skipped or credited from reference-only source assertions.

Unrequested additions: none. Guards enforce real boundaries: exact scratch DB
scope (support layer), verified owner/namespace, immutable source/image identity,
operator-qualified physical containment, nonreplaying proof, and nonsettlement
of uncertain effects. No customer recovery, remote-store durability attestation,
external-model quality, broad Dot certification, or release acceptance is claimed.
