## Why

GAP-03 (`docs/SPECIFICATION.md:443`): the A2A task store is a flat, process-wide
map. `src/uar/api/a2a/task_store.rs:17-21` holds **two** unpartitioned maps —
`tasks` (task_id → Task) and `context_index` (context_id → task_id) — constructed
once at `src/server.rs:652`. Any caller who learns a task or context id reaches
the record. `SPECIFICATION.md:392` states that tenant isolation of UAR's own
stores is **undelegatable**: Forge's RLS covers Forge's data, not UAR's.

### The blocking discovery: there is no tenant concept to partition by

`grep -rn "tenant_id" src/` returns **nothing**. `UserClaims` is
`{sub, name, roles, exp}` (`src/uar/security/claims.rs:4-9`). The A2A module
contains no tenant reference at all.

GAP-03 as written presumes an identity the runtime does not have. **Partitioning
a store by a key that does not exist is not implementable as specified**, so this
change introduces the claim first and partitions second, in that order.

### Vocabulary collision — read this before writing code

The codebase already uses "multi-tenant" to mean **per-user**:

- `src/lib.rs:120` — *"Multi-tenant provider credential service (per-user
  encrypted keys)"*
- `src/server.rs:944` — *"Per-user provider credential API (multi-tenant BYO
  keys)"*
- The change `fix-user-isolation-sessions-memory-kb` declares a capability named
  `multi-tenant-isolation` whose scenarios are entirely `user A` / `user B`
  against the JWT **subject**.

**Operator decision, 2026-08-12: `tenant_id` is distinct from `sub`.** A tenant is
an organization; a user belongs to one. The existing usages above are user
scoping and are out of scope here. This change therefore declares a **new**
capability `tenant-isolation` and does not touch `multi-tenant-isolation`.

> Recommended follow-up, not in this change: rename `multi-tenant-isolation` to
> `user-data-isolation` when that change is next worked. Two capabilities both
> claiming tenant isolation, one of which enforces only user scoping, reads as
> coverage where none exists.

### This closes a published exclusion

`tests/integration/live/capability_cases.rs:873`
(`excluded_c21_tenant_isolation_no_cross_read_surface`) documents precisely this:
*"the runtime's verified JWT claims carry a user id but no tenant id … the
harness cannot create two tenant identities."* It was written to fail the moment
the gap closes. **Converting it is part of this change**, not follow-up work.

## What Changes

- Add `tenant_id: Option<String>` to `UserClaims` and carry it on `UserContext`.
  Optional, because the HS256 lane and existing deployments issue tokens without
  it; absence is handled explicitly rather than defaulted.
- Populate it **only** from a token the verifier has authenticated. Depends on
  `gap-02-jwks-token-verifier`.
- Partition **both** `TaskStore` maps by tenant, keyed `(tenant, id)`. Covering
  only `tasks` would leave `get_by_context` (`task_store.rs`, used at
  `handler.rs:104`) as a cross-tenant read path.
- Thread the tenant through `A2AState` to every `task_store` call in
  `handler.rs` (14 call sites) and `grpc.rs`.
- Replace the C-21 exclusion with a real two-tenant cross-read denial test.

## Capabilities

### New Capabilities
- `tenant-isolation`

## Impact

`src/uar/security/claims.rs`, `src/uar/api/a2a/task_store.rs`,
`src/uar/api/a2a/handler.rs`, `src/uar/api/a2a/grpc.rs`, `src/server.rs:652`,
`tests/integration/live/capability_cases.rs`.

**Depends on `gap-02-jwks-token-verifier`.** Populating `tenant_id` from an
unverified token is worse than having no tenant field: downstream code would then
treat an attacker-controlled string as an isolation boundary.

## Non-goals

- Tenant-scoping runs, memory, or knowledge bases. Those surfaces belong to
  `fix-user-isolation-sessions-memory-kb`, and widening this change to reach them
  would collide with it.
- Tenant provisioning, directory, or lifecycle. This change consumes a claim; it
  does not manage tenants.
