# Durable checkpoint resume

The checkpoint API can reconstruct an authorized run after an orderly runtime
shutdown. It uses protected persisted source metadata rather than a live run in
the old process. Local SurrealKV and qualified durable networked SurrealDB have
this capability. PostgreSQL, in-memory SurrealDB, an unattested remote endpoint,
and a runtime without persistence return explicit unsupported errors. Restorable
execution uses the ordinary host tool loop. Opaque host graph overrides,
host-supplied model callbacks, and host capabilities without a checkpoint-resume
renewal path remain inspectable but explicitly non-restorable. They cannot silently
become an artifact-default run.

The uncomfortable constraint: a saved checkpoint does not prove that its old
executor stopped or that its tools finished safely. Process disappearance,
timeouts, and wall-clock expiry do not authorize takeover. This implementation
denies abrupt takeover and never reassigns an existing continuation claim. A
supervisor exclusion and effect reconciliation capability is separate work.

## Persisted contract

The trusted host creates a version-1 `CheckpointResumeEnvelope` atomically with
each production checkpoint. It contains the complete credential-principal
storage key, source run/session identity, source runtime epoch, exact projected
artifact snapshot, current-policy authorization digest, binding/resource
markers, protected state/history bytes, and a revision binding all those values.
The protected source also records the actual versioned host execution definition,
which is separate from the artifact. The checkpoint list exposes
`execution_restorable` and an `unavailable_reason`; this describes the definition,
while predecessor exclusion and current admission are still checked at resume.
Issuer, tenant, and subject remain part of owner identity. A subject-only or
caller-supplied identity cannot read or resume another partition.

Only projected checkpoint state/history and safe source metadata are persisted.
Provider keys, run-owned credentials, executable transport capabilities, and
verified authentication objects are not serialized. Explicit resource markers
identify which provider and MCP resources must be reattached; MCP grant renewal
uses the existing same-owner destination/scope checks. Current policy and bindings
are resolved again before model or tool execution.

Ordinary catalog runs renew the original owner/workspace/binding revision through
the existing catalog resolver. Their context strategy and model/skill assignments
come from that current receipt. A selected model renews through the existing model
verification and target-admission APIs; definition, binding, configuration digest,
workspace and instance identity must still match. Verification timestamps may be
renewed. The saved target receipt never grants model or resource authority.

The current resume body has no renewal field for delegated full-harness admission,
remote usage/budget/sandbox grants, team attempts, private instance authority, or
inherited child execution bindings. Checkpoints from those specific source shapes
are non-restorable. An additional host policy is restorable only when it is exactly
the policy recreated by the current catalog-context or verified-target path.
Ordinary catalog runs are not disabled merely because they have a host policy.
Budget persistence and supervisor recovery remain separate planned capabilities;
this slice never deserializes their old grants or invents replacement authority.

Graph nodes return checkpoint intent. `GraphContext` has no persistence layer or
checkpoint-writer capability. The host executor handles the intent, validates
its captured source, projects secrets, and awaits the atomic write. Tool-loop
checkpoints use the same host boundary. A write failure produces an error rather
than an asynchronous success claim.

Three inspectable SurrealDB tables support recovery:

Both trusted provider constructor paths declare these tables and `checkpoints`
with `IF NOT EXISTS` before any transactional read. Existing definitions are
preserved; a missing store table is never treated as successful persistence.

| Table | Purpose |
| --- | --- |
| `checkpoint_resume_envelopes` | Immutable owner-bound metadata and exact protected bytes, committed with `checkpoints`. |
| `checkpoint_runtime_epochs` | Active execution incarnation or committed `orderly_excluded` proof. Transaction fences serialize checkpoint/claim writes against exclusion. |
| `checkpoint_resume_claims` | One immutable continuation identity per complete owner/source run, bound to the checkpoint revision and executor epoch. |

The claim transaction checks the exact persisted envelope/revision and the
source's committed exclusion proof before creating a continuation. Uniqueness
also prevents choosing a different checkpoint of the same source run to start
another concurrent continuation. A duplicate returns HTTP 409. Cancellation,
admission failure, or loss of the claiming process leaves the claim held.
Checkpoint saves, claim acquisition and epoch exclusion check transaction errors
and require matching durable readback before the host acknowledges the result.

After the durable CAS succeeds, the host mints a private, single-use admission
permit bound to that exact continuation, source/checkpoint revision, authenticated
owner, executor epoch, and reconstructed request. Shared execution consumes it
before touching history or executable resources. Compatibility adapters cannot
execute a checkpoint without this permit.

## Orderly exclusion

The production signal/shutdown path calls the manager's existing shutdown hooks.
The final shutdown hook cancels new execution, closes and joins graph roots,
waits for in-flight assemblies to publish their owned producers, joins ordinary
producer handles and existing actor-owned roots, and drains descendants,
terminals, and sandbox operations. A cancelled waiter does not drop an owned join
handle. Admission waiting behind the shutdown fence observes cancellation before
entering execution.

The host then checks durable admission and canonical tool outcomes. Any pending
claim, unknown effect, missing outcome, or failed cleanup prevents exclusion.
Unknown admission outcomes remain held even if a later unrelated state appears.
Only after those joins and certainty checks succeed does the store commit the
runtime epoch as `orderly_excluded`. A signal itself is not the proof. Inspect
that committed state before expecting a fresh process to resume.

Embedding hosts must cancel/drain through `RunManager::shutdown_sandboxes` before
dropping their runtime. Dropping a manager or killing a process does not create
exclusion evidence. Existing actor and graph shutdown hooks continue to own their
original handles and cleanup receipts.

## API

Use the existing authenticated routes:

- `GET /api/uar/runs/{run_id}/checkpoints` lists protected checkpoints for the
  complete verified principal, including in a fresh process.
- `POST /api/uar/runs/{run_id}/resume/{checkpoint_id}` resumes a named checkpoint.
- `POST /api/uar/runs/{run_id}/resume` selects the latest protected checkpoint.
  Missing metadata never falls back to a fresh execution.

The resume body accepts an optional new `input`, the original `session_id`, and
explicit `run_credentials`/`mcp_servers` renewal when the source used them. Omit
`input` to restore the saved dialogue without appending an empty user message.
Caller `history` is rejected. An optional artifact must match the protected
snapshot; it cannot replace it. Current deployment and service bindings must
still match. Host resources are admitted through the existing resource adapters.

A successful response contains the durable continuation's `run_id`, source and
checkpoint identifiers, restored state/history counts, and its stream URL. Run
status and error events determine success; a closing done frame alone does not.

| HTTP/code | Meaning |
| --- | --- |
| 503 `checkpoint_resume_unsupported` | The selected provider cannot prove durable resume. No execution starts. |
| 404 `checkpoint_resume_evidence_unavailable` | No protected envelope exists in this principal's partition. |
| 409 `checkpoint_predecessor_uncertain` | No trusted orderly exclusion/effect proof exists. |
| 409 `checkpoint_resume_conflict` | This source run already has a continuation executor. |
| 409 `checkpoint_resume_invalid` | Protected metadata, artifact/session identity, or reconstruction differs. |
| 409 `checkpoint_admission_denied` | Current authority refused the continuation. Its claim remains held. |
| 409 `checkpoint_executor_closed` | The executor incarnation has closed. |
| 409 `checkpoint_execution_unavailable` | The captured host definition/resources cannot be reconstructed. No claim or execution starts. |
| 409 `checkpoint_resume_unavailable` | A storage/current-admission result could not be confirmed. |
| Existing artifact/binding/resource errors | The source snapshot or explicit resource renewal failed the existing host checks. |
| 409 `run_target_changed` / 422 `target_model_unavailable` | Current target identity/configuration or model verification cannot renew the captured selection. |

Legacy raw checkpoints remain readable through their existing persistence
inspection methods. They have no complete owner-bound resume envelope and cannot
execute through the resume API. Do not infer legacy ownership or manufacture a
replacement envelope from a request body. Start a separately authorized new run
when the required proof is unavailable.

## Local production qualification

The `fresh_process_resume` integration target starts the actual production binary,
uses its JWT middleware and public run/approval/policy/resume APIs, and connects to
real disposable durable databases. It records separate OS process identities,
atomic envelope/epoch/claim readback, private provider requests and MCP effect
traces. Local KV is opened for readback only after its runtime has stopped; one
readback provider is reused for related assertions. The shared race uses two
independent runtime processes against the same networked database.

The fixture covers orderly restart and renewal of an explicitly verified model
selection; missing credentials; subject, tenant, and
issuer isolation; changed current policy and service binding; changed artifact,
session, or caller history; corrupt persisted metadata; duplicate and competing
resume; killed claimants before and after an effect; abrupt source loss; unknown
effects after orderly shutdown work; and explicit unsupported profiles with zero
execution. Synthetic HTTP/MCP peers prove protocol behavior, not genuine provider
inference or arbitrary remote storage durability.

Opaque host graph/callback and delegated/team/instance execution shapes are
explicit source limitations. This binary fixture exercises the public ordinary
tool-loop ingress; it does not certify embedding-specific callback renewal or
catalog installation workflows. The affected stream fixture retains its host
graph checkpoint/error semantics, and the legacy adapter fixture asserts that
raw checkpoint metadata cannot mint resume admission.

At the completed slice boundary, configure `UAR_DOT06_INFRA_FILE` with the private
disposable infrastructure manifest, `UAR_DOT06_NAMESPACE` with a fresh namespace,
and `UAR_DOT06_EVIDENCE_ROOT` with a preserved private directory. The qualified
remote constructor/readback environment uses
`UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1`; the negative child deliberately omits
it. Run locally, serialized with other Cargo work:

```bash
cargo test --locked --no-default-features --features server-full,postgres-backend --test fresh_process_resume -- --nocapture --test-threads=1
```

Checkpoint admission changed the stream fixture's authentication requirement.
Its helper now obtains a real verified context from a signed synthetic JWT; all
stream/protocol assertions remain. The affected `stream_terminal_contract`
target must also run at this boundary with its documented fixture environment.
The resume gate requires `postgres-backend` in addition to `server-full`; otherwise
its required unsupported-PostgreSQL proof is absent and the fixture refuses to run.
Neither target's authored source is evidence of a passing run. Record actual
commands, results, failed attempts, and retained artifacts separately.
