# C14.2 approval contract handoff

This is source delivery on isolated branch `codex/uar-c14-durable-approvals`, based on `e23348760b776f058f704e2fe965833353005a90`. It repairs the existing approved C14.2 durable approval criterion. It is not verified completion of that criterion.

## Routes and identity

- Existing `POST /api/uar/runs/{run_id}/tool-approval` keeps `{approved: boolean, approval_id?: string}`. Middleware authenticates the principal; the manager checks exact live-run subject and tenant before resolution. Legacy missing IDs remain supported only for ordinary root requests; child requests require their exact ID. Caller headers never establish approval authority.
- Additive `GET` on that same route returns `{version: 1, runId: string, durable: boolean, records: ApprovalRecordView[]}`. It reads durable storage without requiring a live run. Storage filters by the encoded verified subject-and-tenant owner key and root run. For workspace-bound records, the existing `x-uar-workspace-id` must match the admitted workspace. The header filters already owner-scoped data; it does not grant ownership.
- Existing `GET /api/uar/runs/{run_id}/tool-approval/pending` keeps its existing envelope and adds `issuerId` and `challengeId` inside `pending.approval`. `approvalId === challengeId` is the same broker UUID. The live-run auth check is unchanged.
- Successful POST retains `{resolved: boolean, decision: "allow" | "deny"}` and adds `record: ApprovalRecordView` with `resolvable: false`. `resolved` means delivery to the live waiter. Persistence may commit while cancellation closes that waiter, so a recorded decision and effect outcome are separate facts. No matching waiter remains HTTP 404; persistence failure is HTTP 503 with `approval_persistence_unavailable`. History failure is HTTP 503 with `approval_history_unavailable`.

## Record schema

Every record has `version: 1`; strings `issuerId`, `challengeId`, `ownerKey`, `rootRunId`, `toolCallId`, `toolName`; nullable strings `workspaceId`, `admissionId`; existing `admissionOwner: "paired-host" | "uar-runtime"`; RFC3339 strings `createdAt`, `expiresAt`, `updatedAt`; `state: "pending" | "approved" | "denied" | "cancelled" | "expired" | "interrupted"`; and nullable `decision`.

A human decision contains strings `decisionId` and `actor`, boolean `approved`, and RFC3339 `decidedAt`. Actor is the host-captured authenticated tenant-aware owner key, not a caller assertion. Record identity is immutable. One atomic compare-and-set from pending persists either this decision or a nonhuman terminal outcome. The read/POST view adds `resolvable: boolean`; this value is computed by the serving broker, not persisted. The existing StatePatch stream emits `/approval` with the raw correlated durable record; clients refresh GET for current resolution availability.

The issuer is the actual UAR broker runtime epoch that issued this challenge. Paired-host admission remains a separate existing execution authority. A readable challenge or decision never bypasses that authority, grants replay, or proves execution. The broker captures workspace from the trusted collaboration binding; descendants inherit the root channel. It does not persist raw tool arguments or credentials.

## Persistence and lifecycle limits

PostgreSQL stores one row per owner/issuer/challenge and uses a pending-state/data compare-and-set. SurrealDB stores the same immutable record under the existing tenant storage-key encoding and uses a conditional record update. Additive migrations are registered in existing startup paths. Memory-backed operation explicitly reports `durable: false`; persistent configured stores report their existing durability capability. Configured persistence errors do not fall back to memory.

Only the issuing broker's matching live, uncancelled and unexpired waiter is resolvable. A different runtime epoch is not evidence of retirement in shared storage. Historical or orphaned pending rows remain inspectable and non-resolvable on this runtime. Observed gate cancellation, timeout and channel closure settle as cancelled, expired and interrupted. Abrupt process loss can leave pending history without terminal evidence; no scheduler or lease has been added to infer otherwise.

Ordinary detach remains local presentation/subscription ownership. Explicit stop continues through existing authoritative attempt/run cancellation. This change adds no lifecycle action or endpoint.

## Remaining delivery evidence

No tests, compiler checks, formatting/lint gates, builds, migrations or operation scripts were run during implementation. Root must complete the full source delivery first, then run the approved packaged operation: two isolated authenticated clients read the same challenge and one decision, a competing decision cannot replace it, detach leaves execution alive, explicit stop produces cancellation evidence, and reopened clients retain the outcome. Exercise the selected persistent backend and actual packaged Boss/UAR integration; source inspection alone cannot prove durability or race behavior.

Inherited test-source limitation: the inline test in `src/uar/runtime/thread/approvals.rs` calls pre-existing register/request signatures with missing owner/admission arguments. Its old synchronous resolver calls will also need alignment with the persisted asynchronous resolver when test work is authorized. The production implementation retains no synchronous or test-only decision bypass.

Commit-hook activity: the normal mandatory pre-commit hook ran the GitHub Actions policy validator and pnpm automatically installed workspace dependencies. It rewrote only the unstaged pnpm lockfile (+69/-2496), saved as `/tmp/uar-c14-hook-lockfile.patch` for inspection and excluded from this source delivery. Source pins are preserved by restoring that hook-created lockfile delta. Frontend lint/typecheck hooks skipped this Rust-only change; no Rust build or test ran. Commit-message validation required wrapping the first attempted body.
