## 0. Read first

- [x] 0.1 Read `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md`.
- [x] 0.2 Confirm `gap-02-jwks-token-verifier` is complete. **This change must not
      start before it.** A tenant claim read from an unverified token is worse
      than no tenant field at all.

## 1. Tenant claim (GAP-03a)

- [x] 1.1 Add `tenant_id: Option<String>` to `UserClaims`
      (`src/uar/security/claims.rs:4-9`) and surface it on `UserContext`.
      Optional by design: the HS256 lane issues tokens without it.
- [x] 1.2 Populate it in the verifier from `gap-02`'s `Principal`. It must be
      impossible to construct a populated tenant from an unverified token —
      prefer a type that makes this a compile error over a runtime check.
- [x] 1.3 Unit test: a tenant supplied in body, query, or header is ignored.

## 2. Partition the store (GAP-03b)

- [x] 2.1 Key `TaskStore::tasks` by `(tenant, task_id)`
      (`src/uar/api/a2a/task_store.rs:17-21`).
- [x] 2.2 Key `context_index` by `(tenant, context_id)`. **Do not skip this.**
      `get_by_context` is reached from `handler.rs:104` and is a cross-tenant
      read path if left flat.
- [x] 2.3 Thread tenant through `A2AState` to every `task_store` call site in
      `handler.rs` (14 sites: :104, :117, :132, :137, :154, :164, :168, :186,
      :190, :208, :226, :236, :242 and the store construction) and `grpc.rs`.
- [x] 2.4 Fail closed on a tenant-scoped surface when no tenant is established
      and `jwt_required` is true.

## 3. Convert the published exclusion

- [ ] 3.1 Replace `excluded_c21_tenant_isolation_no_cross_read_surface`
      (`tests/integration/live/capability_cases.rs:873`) with a real two-tenant
      test. Rename off the `excluded_` prefix per the label taxonomy — an
      exclusion that outlives its blocking condition is the failure the
      self-invalidating design exists to prevent.
- [ ] 3.2 The new case asserts denial by task id, by context id, and on cancel.
- [ ] 3.3 Same-tenant access still succeeds — the negative claim alone is not
      evidence the store still works.

## 4. Proof

- [x] 4.1 Defer the contract's pinned Tier 2 command to phase completion, after
      all six changes are implemented. Running it during A2 is prohibited by
      the phase tier discipline; A2 uses focused tests at change completion.
- [ ] 4.2 **Negative control.** Demonstrate the cross-tenant test fails when the
      partition key is ignored. Record the command and its failing output.
- [x] 4.3 Record current results in the contract's verification-record format;
      retain the live C-21 row as explicitly deferred until phase Tier 2.

## 5. Stop conditions

- [ ] 5.1 The change appears to require scoping runs, memory, or knowledge bases
      → stop. That is `fix-user-isolation-sessions-memory-kb`'s scope.
- [ ] 5.2 A task appears to require renaming or editing the
      `multi-tenant-isolation` capability → stop. Recommended, but not sanctioned
      in this change.
- [ ] 5.3 The tenant claim's name or shape appears to conflict with an external
      issuer contract → stop and report; this was an open question.
- [ ] 5.4 A pre-existing unrelated failure appears → stop and report.
