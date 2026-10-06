# Scoped managed-local delegation grants

Contract: `scoped_delegation_grants_v1`, 2026-10-05. Native source implementation;
the C14.4 integrated, packaged runtime gate remains pending.

The Boss retains UAR's per-launch stdin credential in its main process. BossFang
receives only a separately minted grant. UAR stores grants in private native
memory as redacted `SecretString` values; it does not persist them. New runtime
startup creates a new authority and full-harness epoch, invalidating old grants.
External UAR continues to use its verified JWT subject and tenant; its transport
does not expose these issuance/revocation routes or accept host principal headers.

## Issuance and lifecycle

`POST /api/uar/delegation-grants` requires the original launch credential in
`Authorization: Bearer …`, no `Origin`, the exact managed-local authority, and
`x-uar-principal` identifying the host-approved authenticated session. Identity
comes from the authenticated `UserContext`, never from the JSON body. Anonymous
issuance, unknown fields, empty operations, and empty/wildcard workspace scope fail.

Request:

```json
{
  "workspace_ids": ["selected-workspace"],
  "operations": ["discovery", "model_read", "model_completion", "full_harness_delegation"]
}
```

The 201 response uses `Cache-Control: no-store` and contains `id`, `token`,
`token_type` (`Bearer`), `expires_at` (RFC3339), `expires_in` (900), `principal`,
`workspace_ids`, `operations`, `instance_id`, and `runtime_epoch`. The token is 32
random bytes encoded as 64 lowercase hexadecimal characters. `runtime_epoch`
matches `/api/uar/full-harness/v1/capabilities`. Expiry uses a monotonic clock
inside the native authority; moving the wall clock cannot extend a grant.

`DELETE /api/uar/delegation-grants/{id}` requires the original host credential.
It returns 204 even if the ID is already absent. Grants cannot issue, renew, or
revoke credentials. The Boss revokes the old ID when switching instances,
workspaces, principals, or closing the connection, and renews before the fixed
expiry. Its renderer receives connection status and expiry only. BossFang keeps
the grant in native memory, never in persisted settings, public status, traces,
errors, or renderer messages.

## Explicit allowlist

| Operation | Methods and paths |
| --- | --- |
| `discovery` | GET `/api/uar/capabilities`, `/api/openapi.json`, `/api/uar/full-harness/v1/capabilities`, `/readyz`, `/healthz`; POST `/api/uar/compatibility` |
| `model_read` | GET `/api/models`, `/api/providers`, `/v1/models`, `/v1/models/{model_id}` |
| `model_completion` | POST `/api/chat/completion` only |
| `full_harness_delegation` | POST `/api/uar/full-harness/v1/tasks`; GET `/api/uar/full-harness/v1/admissions/{admission_id}`, `/api/uar/full-harness/v1/tasks/{task_id}`, `/api/uar/full-harness/v1/tasks/{task_id}/stream`; POST task `tool-approval`, `cancel`, `detach`, `steer` |

Completion and full-harness requests must include exactly one
`x-uar-workspace-id` that exactly matches a recorded workspace. Discovery and
model catalog reads are global metadata reads, with their rights selected
explicitly. Every method/path outside this table fails closed. `/api/uar/test`,
`/v1/chat/completions`, native run administration, credentials, configuration,
and grant issuance/revocation confer no grant authority.

BossFang sends `Authorization: Bearer <grant>` and omits `x-uar-principal`.
The guard checks authority and `Origin` before credentials, strips the credential
before inner middleware/logging, and installs a separate grant marker. The auth
middleware injects the grant's captured subject and tenant; another principal
header is rejected. The host-authenticated marker is reserved for the original
launch credential.

Existing full-harness ownership, admission digest/revision contracts, approvals,
Cedar checks, effect authorization, and cancellation remain authoritative.
Grants do not bypass owner checks or grant paired-host tool-admission authority.
`steer` retains the existing unsupported result. Revoking or expiring a grant
prevents new requests; it does not cancel previously admitted native work. The
host must use the authorized cancellation path when cancelling that work.

## Delivery evidence

At the complete C14.4 delivery boundary, operate the managed-local API with a
real native sidecar: issue a grant, discover selected instance/models, complete
against the chosen model, admit/reconcile/observe/control a full-harness task,
and prove wrong principal/workspace, method/path, expired/revoked credentials,
Origin, authority, and nested issuance attempts fail. Reboot the native runtime
and prove old credential/epoch refusal. Confirm external JWT identity cannot be
replaced through a host header and confirm grants are absent from public status,
persisted settings, renderer traffic, and logs. Then build and operate the
packaged Boss path. These checks are deferred to the coordinator's single gate;
source changes alone establish no operational or release certification.
