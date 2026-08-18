# Verification: `fix-user-isolation-sessions-memory-kb`

Scope: `server-full`, with named provider feature paths where stated. Results
transfer to no other profile.

The PostgreSQL rows are replayable on this macOS/Homebrew toolchain with:

```bash
pg_bin=/opt/homebrew/opt/postgresql@17/bin
pg_replay_dir=$(mktemp -d /tmp/uar-pg-replay.XXXXXX)
"$pg_bin/initdb" -D "$pg_replay_dir/data" --auth=trust --no-locale
"$pg_bin/pg_ctl" -D "$pg_replay_dir/data" \
  -l "$pg_replay_dir/postgres.log" -o "-p 55432 -h 127.0.0.1" start
"$pg_bin/createdb" -h 127.0.0.1 -p 55432 uar_test
```

After the two recorded commands, stop that isolated cluster with
`"$pg_bin/pg_ctl" -D "$pg_replay_dir/data" -m fast stop`.

| Requirement | Evidence command | Observed result | Limit |
|---|---|---|---|
| Process-local session ownership | `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib session::thread::tests::session_store_partitions_identical_ids_by_owner -- --exact --test-threads=1` | Exit 0; 1 passed, 0 failed. The same logical session ID was independently readable by its two owners. | Process-local `SessionStore` only. |
| In-memory policy ownership | `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::persistence::providers::memory::tests::conversation_policy_round_trips_without_durable_storage -- --exact --test-threads=1` | Exit 0; 1 passed, 0 failed. Alice's same-ID policy was readable to Alice, absent to Bob, and survived Bob's delete negative control. | In-memory provider only. |
| SurrealKV durable identity | `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::persistence::providers::surreal::tests::knowledge_rows_with_identical_ids_remain_partitioned_by_owner -- --exact --test-threads=1` | Exit 0; 1 passed, 0 failed. Alice and Bob stored the same KB, document, and chunk IDs; each search returned only its owner, and Bob's delete left Alice's graph intact. | One temporary embedded SurrealKV database. |
| Legacy session compatibility | `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::persistence::providers::surreal::tests::legacy_session_is_preserved_as_anonymous_without_becoming_claimable -- --exact --test-threads=1` | Exit 0; 1 passed, 0 failed. Authenticated lookup of the ownerless row returned none; anonymous lookup migrated and preserved it. | SurrealKV lazy migration path; PostgreSQL uses the migration described below. |
| ACP session ownership | `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::api::acp::handler::tests::session_store_denies_cross_owner_get_and_delete -- --exact --test-threads=1` | Exit 0; 1 passed, 0 failed. Bob's get returned none and delete returned false; Alice's session remained readable and deletable. | Process-local ACP session store; HTTP authentication and run ownership are covered by the live row. |
| PostgreSQL schema migration | `DATABASE_URL=postgres://127.0.0.1:55432/uar_test sqlx migrate run --source migrations` against an isolated PostgreSQL 17 cluster | Exit 0; all 18 repository migrations applied, including `20260818000000_knowledge_tenant_ownership`. | Fresh temporary PostgreSQL 17 database; no upgrade of a production dataset. |
| PostgreSQL durable identity and foreign keys | `DATABASE_URL=postgres://127.0.0.1:55432/uar_test cargo test --locked -p universal-agent-runtime --no-default-features --features server-full,postgres-backend --test knowledge_base_integration test_equal_ids_are_partitioned_by_owner_in_postgres -- --exact --test-threads=1` | Exit 0; 1 passed, 0 failed. Same KB/document/chunk IDs coexisted for two owners. A cross-owner document-to-KB reference failed, and Bob's delete left Alice's graph intact. | Isolated PostgreSQL 17 with the migration above; not another PostgreSQL version. |
| All provider implementations compile | `cargo check --locked -p universal-agent-runtime --no-default-features --features server-full,postgres-backend,in-memory-backend` | Exit 0. | Compile evidence only for feature combinations not exercised by the focused rows above. |
| JWT-derived thread-adjacent, memory, and knowledge isolation | `UAR_LIVE_INTEGRATION_BACKEND=recorded cargo test --locked --no-default-features --features server-full --test integration live::capability_cases::l3_c21_threads_memory_and_knowledge_are_partitioned_by_verified_user -- --exact --test-threads=1` | Exit 0; 1 passed, 0 failed. ACP rejected an unauthenticated request; user B could not get/delete A's ACP session, create a run in it, or get A's ACP run, while A's controls succeeded. Direct API controls returned A's thread context, run stream, policy/configuration, KB, document, and memory. B's direct negative controls returned 404 or empty results and the same session ID did not replace A's run. Spoofed memory identifiers were ignored. | One recorded-backend server-full runtime with embedded SurrealKV and local embeddings. |
| Unresolved or inaccessible KB selection fails closed | The live command above plus `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full,in-memory-backend --lib uar::persistence::providers::memory::tests::knowledge_rows_are_partitioned_by_owner -- --exact --test-threads=1` | Exit 0; 1 passed, 0 failed for the provider control. Bob's inaccessible KB search returned no chunks; Alice's same-row control remained readable after Bob's delete. The runtime maps zero accessible configured KBs to an explicit empty result instead of all-KB search. | No model-prompt inspection was performed. |
| Compatibility boundary | `git diff --check` and inspection of `docs/compatibility-policy.md` | Exit 0. The policy distinguishes private user resources, anonymous legacy sessions, and installation-wide skills/agents/settings. | Shared administrator-resource role enforcement remains a deployment gateway responsibility in 1.x. |
| Tier 0 | `cargo check --locked -p universal-agent-runtime --no-default-features --features server-full` | Exit 0. | Three unrelated existing warnings remain (`MAX_BODY_BYTES`, `MAX_REDIRECTS`, `WasmHostState` Debug). |
| Clippy | `cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps` | Exit 0; 572 warnings from the existing pedantic baseline. | This is not a warning-free lint result; adjacent warnings were not changed. |
| OpenSpec | `openspec validate fix-user-isolation-sessions-memory-kb --strict` | Exit 0; change is valid and all 8 tasks are complete. | Verification covers this change only. |

The first PostgreSQL build attempt stalled inside the configured compiler cache
with zero CPU. It was terminated, the isolated database was restarted, and the
same exact test completed successfully after a real eight-minute dependency
compile. The test result above is from the completed retry.

Phase-level Tier 2 and release-level Tier 3 were not run. Their timing remains
reserved for completion of the active phase and the immutable release candidate.
