# Durable approval recovery

An ordinary tool-loop approval can survive an orderly runtime restart. The
trusted host first commits a protected pre-effect checkpoint, then publishes
the pending challenge. Local SurrealKV and qualified durable networked
SurrealDB support this path. PostgreSQL and unattested remote stores retain
approval history but do not reconstruct executable checkpoint continuations.

The uncomfortable constraint: killing the original process does not prove
that it stopped executing or that its effects are known. A pending row, a
successful HTTP admission, or an expired timer is never proof of executable
authority. Abrupt takeover remains blocked by the checkpoint predecessor
exclusion contract. Unknown effects retain their consumed continuation claim.

## Persisted contract

New challenges use the complete verified issuer, tenant and subject principal
key. They retain their original issuer, challenge, source run, tool-call and
expiry identities. The optional `continuation` contains only `checkpointId`,
`checkpointRevision`, `bindingDigest` and `callIndex`. It contains no tool
arguments, executable callback, credentials, grants, receipts or future.

The protected checkpoint contains the unresolved assistant call block and
exact projected conversation bytes. Its pending marker binds the selected
call ID/index and current catalog, descriptor, payload, resource, effective
run-policy and governance digests. Reconstructing a different descriptor or
payload invalidates the challenge. Secret projection can make a call
non-reconstructible; recovery never restores redacted secrets from a row.

The checkpoint commit precedes approval-record creation. A crash between them
leaves an inert checkpoint without a decision record. Ordinary resume cannot
execute a checkpoint with a pending-approval marker; the exact pending decision
must enter through the authenticated approval route.

Orderly shutdown cancels and joins the run producer and descendants, explicitly
cancels the pending prepared host admission, and retains only a checkpoint-linked
pending challenge. Failed cancellation or any unknown/claimed effect prevents
the original runtime epoch from being excluded. User cancellation still cancels
the challenge. The original five-minute expiry does not restart after recovery.

Human decision CAS checks that immutable deadline at the storage serialization
point. A late approve or deny intent atomically commits `expired` with no human
decision and returns that it did not win the requested transition. SurrealDB
checks its current time within the conditional update. PostgreSQL locks the
exact pending row first, then checks its actual clock during the update;
transaction-start `NOW()` could predate a lock wait. PostgreSQL documents that
[actual-clock distinction](https://www.postgresql.org/docs/current/functions-datetime.html#FUNCTIONS-DATETIME-CURRENT).
Memory storage checks expiry while holding its writer lock. The host publishes
the actual row after a lost human CAS and cannot publish an invented approval.

The fresh host consumes the existing single-use checkpoint claim before
assembly. It assembles current resources and policy using the complete protected
history prefix, then reconstructs the unresolved block. It resolves descriptors
and validates arguments again, repeats current governance and host admission,
CAS-consumes the selected pending challenge, and obtains a new tool receipt.
The original grant and future are never replayed. A crash after the claim or
decision commit leaves the claim held, even if no effect was observed.

Sibling calls remain in original order. The selected challenge approves only
its matching call ID/index. Each remaining sibling goes through current
admission and its own approval decision. Model continuation receives the block
only after real canonical results complete it; the lossless history validator
is unchanged and no missing tool result is fabricated.

Opaque graph/callback, delegated/team, unreconstructible host-resource and
sandbox-renewal paths retain the checkpoint limitations documented in
[fresh-process-resume.md](fresh-process-resume.md). Old records without a
protected continuation cannot become executable. An explicit recovery attempt
invalidates an unbound record in the verified principal partition. Older
subject/tenant-only rows remain history; no issuer mapping is guessed for them.

## API

Use the existing authenticated routes and original run/challenge IDs:

- `GET /api/uar/runs/{run_id}/tool-approval` reads committed history. Expired
  pending challenges become `expired`; broken protected mappings become
  `invalidated`. Storage failures remain explicit errors.
- `GET /api/uar/runs/{run_id}/tool-approval/pending` reads the live waiter or,
  after restart, `{approvalId, record, resolvable, recoveryAvailable}`. A fresh
  record has `resolvable: false`: no original waiter exists.
- `POST /api/uar/runs/{run_id}/tool-approval` accepts the exact `approval_id`
  and `approved`. Fresh approval may also provide a nested `resume` object
  with the existing `run_credentials`, `mcp_servers`, `session_id` and host
  resource-renewal fields. Executable calls/history come from protected storage.
  Caller `history` and a new `input` cannot replace the unresolved continuation.

Denial needs only the current authenticated pending challenge. It is processed
before executable-continuation validation, including for an unbound record on
an unsupported resume backend. It starts no run or effect. Expiry still wins
over a denial that reaches the storage CAS after the original deadline.

`recoveryAvailable` describes a verified protected mapping. Trusted predecessor
exclusion, current resource/binding renewal and current admission are still
required. It is not an execution permission.

Successful reconstruction returns `continuation_admitted: true`, `run_id`,
`stream_url` and the checkpoint identifiers. `approval_record` and `resolved`
come from durable decision readback; `resolved` may be false while the async
current gate is still pending. Only a committed `approved` record sets it true.
Inspect the new run's events/status and canonical tool outcome for execution
success. `/approval` state patches on a live run publish committed pending or
terminal records; restart reads use the durable approval API.

| HTTP/code | Meaning |
| --- | --- |
| 400 `approval_id_required` | Supply the original opaque challenge ID. |
| 400 `approval_resume_input_invalid` | Recovery cannot append a new input before the pending call completes. |
| 404 | No matching challenge in this verified owner/workspace partition. |
| 409 `approval_not_pending` | Denied, expired, invalidated or consumed decisions cannot execute. |
| 409 `approval_continuation_unavailable` | An unbound challenge was invalidated; no executable mapping exists. |
| 200 `decision: deny` | A fresh pending rejection committed without starting a continuation. |
| 409 `checkpoint_predecessor_uncertain` | Original execution/effects lack trusted exclusion. |
| 409 `checkpoint_resume_conflict` | Another executor owns the source's immutable continuation claim. |
| 409 `checkpoint_admission_denied` | Current policy refused execution. Its claim remains held. |
| 422 `run_credential_required` | Reattach the original provider resource through current host admission. |
| 503 `approval_decision_unconfirmed` | Continuation was admitted, but durable decision readback failed; returned `run_id` remains inspectable. |
| 503 `approval_persistence_unavailable` | Required invalidation could not be durably confirmed; its consumed continuation claim remains held. The run also exposes the persistence failure. |
| Other approval/checkpoint/binding errors | The existing verified ingress, checkpoint or resource contract refused the operation. |

## Completed-slice qualification

The `approval_recovery` integration target launches the production binary with
signed JWTs, real durable databases, controlled provider/MCP protocol peers,
and independent client/readback processes. Local KV uses sequential runtime
owners and joined readback children. The local decision race uses two independent
authenticated clients against one current server; the shared race uses two
independent servers and clients against the same qualified networked store.

Each supported profile runs `pending-restart`, `race`, `deny`, `expiration`,
`revoked`, `tightened`, `binding`, `abrupt`, `unknown`, `siblings`, `legacy`,
`legacy-deny` and `invalidation-outage`.
The expiry case waits for the original real five-minute deadline. Identity,
missing-ID, missing-resource and caller-history negatives are included in
`pending-restart`. Unattested remote and PostgreSQL controls assert unsupported
resume with no new model request or effect. Protocol peers do not certify
genuine provider inference or arbitrary infrastructure durability.

`legacy-deny` proves a current unbound pending row can become denied with no
continuation claim. `invalidation-outage` installs a synchronous rejecting
event in its isolated approval table, then changes policy; HTTP 503, unchanged
pending decision, one held claim and no effect remain visible across retry.
Four `atomic-deadline` groups exercise real local/shared SurrealDB, PostgreSQL
and production memory storage. Pre-deadline unbound denial is a positive
control; stale approve/deny intents must become expired. PostgreSQL holds a
real row lock until after expiry to exercise the await/lock boundary. These
short storage-contract deadlines supplement the unchanged real five-minute
public API expiry case. The full authored inventory is 32 groups.

Set `UAR_DOT07_EVIDENCE_ROOT` to a private preserved directory and reuse the
private `UAR_DOT06_INFRA_FILE`/`UAR_DOT06_NAMESPACE` disposable infrastructure
manifest contract. Qualified readback requires
`UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1`. At the completed slice boundary,
serialize this whole target with the affected `fresh_process_resume` and
`stream_terminal_contract` gates:

```bash
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test approval_recovery -- --nocapture --test-threads=1
```

The acceptance target explicitly needs `in-memory-backend` for its production
memory deadline case. Integration tests compile the library without its
internal `cfg(test)` exports. Omitting the feature produced E0433 for
`providers::memory` before any scenario ran. The additive acceptance feature
does not qualify an exact `server-full,postgres-backend` customer artifact:
the fixture's `CARGO_BIN_EXE` inherits the acceptance feature set.

In the same serialized failed-gate rerun, after the complete approval target,
build the exact customer profile separately and retain its artifact/hash before
another feature profile can replace the build output:

```bash
cargo build --locked --no-default-features --features server-full,postgres-backend --bin universal-agent-runtime
```

Record acceptance and customer-build commands, feature sets and artifact hashes
separately. A passing acceptance target proves its exercised paths under the
additive profile; the exact-profile build proves that the customer artifact
compiles. It does not alone certify runtime behavior for that exact artifact.

The pre-approval checkpoint changes the accepted checkpoint inventory in the
earlier resume fixture. That fixture now requires exactly one named
`pending_tool_approval` snapshot and one `tool_loop_1` post-ToolEnd snapshot,
and selects the final checkpoint by committed identity. Earlier acceptance
does not certify this new source. Authored fixture source and this command are
not evidence of a passing gate; record actual commands/results and private
artifacts when the gate runs.
