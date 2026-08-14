# A2 fail-closed negative controls

Date: 2026-08-14
Profile: `server-full` only.

The positive `TaskStore` diff was hashed before the partition controls:

```text
02b3394f571e5691b5913fcc48020e319f90448e34049d81b9a1ee9a119c4593  -
```

After each control was restored with `apply_patch`, the same command returned
the same hash:

```bash
git diff -- src/uar/api/a2a/task_store.rs | shasum -a 256
```

A later comment-only Clippy cleanup changed the final TaskStore diff hash to:

```text
a2f5c7b7d1de1cb715ef39f79bc6c5bbdcfe6c3ea7412d59addb70b4ffef1e96  -
```

No tenant-key expression changed in that cleanup. The focused tenant group was
rerun after it and passed.

The positive handler diff was independently hashed before and after its
fail-closed control:

```text
ff22913f6753085d6945b531230b09be4686199628cda334f11426662bf01749  -
```

## Task-ID read and cancel with tenant keys ignored

Only `TaskStore::get`, `get_by_context`, and `cancel` were temporarily changed
to search records without tenant keys. The focused tenant group was run:

```bash
cargo test --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib tenant \
  -- --test-threads=1
```

Observed output and exit status:

```text
running 9 tests
test uar::api::a2a::grpc::tests::grpc_task_access_is_partitioned_by_verified_tenant ... FAILED
test uar::api::a2a::handler::tests::body_query_and_header_tenant_values_cannot_override_verified_tenant ... ok
test uar::api::a2a::handler::tests::required_jwt_without_verified_tenant_is_rejected ... ok
test uar::api::a2a::task_store::tests::cross_tenant_cancel_does_not_mutate_task ... FAILED
test uar::api::a2a::task_store::tests::partitions_task_and_context_lookup_by_tenant ... FAILED
test uar::runtime::manager::credential_layer_tests::multi_tenant_no_user_credential_falls_back_to_env ... ok
test uar::runtime::manager::credential_layer_tests::multi_tenant_user_key_overrides_env_key ... ok
test uar::runtime::manager::credential_layer_tests::single_tenant_no_service_keeps_env_key ... ok
test uar::security::verifier::tests::verified_tenant_claim_becomes_typed_principal_identity ... ok

---- uar::api::a2a::grpc::tests::grpc_task_access_is_partitioned_by_verified_tenant stdout ----
cross-tenant task get must fail: Response { metadata: MetadataMap { headers: {} }, message: TaskResponse { task_id: "f72758de-2378-47c7-bbbc-0bc226c8a3d6", status: "working", messages: [Message { role: "user", parts: [Part { content_type: "text/plain", content: Some(Text("tenant task")) }] }, Message { role: "agent", parts: [Part { content_type: "text/plain", content: Some(Text("Welcome! I'm the UAR Compiler Agent. I can help you compile a UAR-AGENT-MD specification into a signed agent descriptor.\n\nYou can:\n• **Paste a complete spec** — I'll compile it immediately.\n• **Describe what you want** — I'll guide you through building one step by step.\n\nWhat would you like to do?")) }] }], artifacts: [] }, extensions: {} }

---- uar::api::a2a::task_store::tests::cross_tenant_cancel_does_not_mutate_task stdout ----
assertion failed: !store.cancel(Some(&tenant_b), &task.id).await

---- uar::api::a2a::task_store::tests::partitions_task_and_context_lookup_by_tenant stdout ----
assertion failed: store.get(Some(&tenant_b), &task.id).await.is_none()

test result: FAILED. 6 passed; 3 failed; 0 ignored; 0 measured; 563 filtered out; finished in 0.01s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

Exit code: `101`.

The methods were restored, the complete diff returned the original
`02b3394f571e5691b5913fcc48020e319f90448e34049d81b9a1ee9a119c4593`
hash, and the same focused command passed:

```text
running 9 tests
test uar::api::a2a::grpc::tests::grpc_task_access_is_partitioned_by_verified_tenant ... ok
test uar::api::a2a::handler::tests::body_query_and_header_tenant_values_cannot_override_verified_tenant ... ok
test uar::api::a2a::handler::tests::required_jwt_without_verified_tenant_is_rejected ... ok
test uar::api::a2a::task_store::tests::cross_tenant_cancel_does_not_mutate_task ... ok
test uar::api::a2a::task_store::tests::partitions_task_and_context_lookup_by_tenant ... ok
test uar::runtime::manager::credential_layer_tests::multi_tenant_no_user_credential_falls_back_to_env ... ok
test uar::runtime::manager::credential_layer_tests::multi_tenant_user_key_overrides_env_key ... ok
test uar::runtime::manager::credential_layer_tests::single_tenant_no_service_keeps_env_key ... ok
test uar::security::verifier::tests::verified_tenant_claim_becomes_typed_principal_identity ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 563 filtered out; finished in 0.01s
```

## Context-ID read with the context partition ignored

Only `get_by_context` was temporarily changed to select both the context entry
and resulting task without tenant keys. This exact test was run:

```bash
cargo test --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib \
  'uar::api::a2a::task_store::tests::partitions_task_and_context_lookup_by_tenant' \
  -- --exact --test-threads=1
```

Observed output and exit status:

```text
running 1 test
test uar::api::a2a::task_store::tests::partitions_task_and_context_lookup_by_tenant ... FAILED

---- uar::api::a2a::task_store::tests::partitions_task_and_context_lookup_by_tenant stdout ----
assertion failed: store.get_by_context(Some(&tenant_b), "shared-context").await.is_none()

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 571 filtered out; finished in 0.00s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

Exit code: `101`.

The source was restored, the diff returned the original
`02b3394f571e5691b5913fcc48020e319f90448e34049d81b9a1ee9a119c4593`
hash, and the same exact command passed:

```text
running 1 test
test uar::api::a2a::task_store::tests::partitions_task_and_context_lookup_by_tenant ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 571 filtered out; finished in 0.00s
```

Two earlier context-control attempts were explicitly discarded: the first
command matched zero tests because `--exact` received an unqualified name; the
second inversion still passed through the correctly partitioned task map and
therefore did not exercise the intended failure. Neither is counted as
negative evidence above.

## Required JWT with no verified tenant

Only the handler guard for `jwt_required && tenant_id.is_none()` was
temporarily disabled. This exact test was run:

```bash
cargo test --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib \
  'uar::api::a2a::handler::tests::required_jwt_without_verified_tenant_is_rejected' \
  -- --exact --test-threads=1
```

Observed output and exit status:

```text
running 1 test
test uar::api::a2a::handler::tests::required_jwt_without_verified_tenant_is_rejected ... FAILED

assertion `left == right` failed
  left: Number(-32001)
 right: -32600

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 571 filtered out; finished in 0.01s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

Exit code: `101`.

The guard was restored, the handler diff returned the original
`ff22913f6753085d6945b531230b09be4686199628cda334f11426662bf01749`
hash, and the same exact command passed:

```text
running 1 test
test uar::api::a2a::handler::tests::required_jwt_without_verified_tenant_is_rejected ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 571 filtered out; finished in 0.00s
```

## gRPC verified token without a tenant claim

Only the gRPC match arm for a successfully verified, tenant-less token with
`jwt_required=true` was temporarily changed to return the unpartitioned lane.
This exact test was run:

```bash
cargo test --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib \
  'uar::api::a2a::grpc::tests::grpc_required_jwt_without_verified_tenant_is_rejected' \
  -- --exact --test-threads=1
```

Observed output and exit status:

```text
running 1 test
test uar::api::a2a::grpc::tests::grpc_required_jwt_without_verified_tenant_is_rejected ... FAILED

assertion `left == right` failed
  left: NotFound
 right: Unauthenticated

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 572 filtered out; finished in 0.01s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

Exit code: `101`.

The guard was restored and this complete-diff hash returned to its pre-control
value:

```text
b8a32bc808c309f171e472264d3a100996e58fa4532d52a89226d2ddb5c6f619  -
```

The same exact command then passed:

```text
running 1 test
test uar::api::a2a::grpc::tests::grpc_required_jwt_without_verified_tenant_is_rejected ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 572 filtered out; finished in 0.01s
```

## Deferred live control

The phase-level C-21 assertion and its live negative control are not represented
by these library controls. They remain deferred to the one permitted Tier 2 run
after all six changes are implemented. The integration target containing C-21
was compiled, not executed:

```bash
cargo check --locked -p universal-agent-runtime \
  --no-default-features --features server-full --test integration
```

Observed output:

```text
Checking universal-agent-runtime v1.0.0 (/Users/gqadonis/.claude/worktrees/uar-1-0-readiness)
warning: `universal-agent-runtime` (lib) generated 3 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 32s
```

Exit code: `0`. This is compile evidence only and is not reported as a live
C-21 result.
