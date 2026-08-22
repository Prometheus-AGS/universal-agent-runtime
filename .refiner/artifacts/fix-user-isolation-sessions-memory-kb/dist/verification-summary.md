# User isolation deterministic verification summary

Profile scope: `server-full`, with named provider feature paths where stated.
These results transfer to no other profile.

- Verified identity: PASS. Direct thread/run routes, ACP sessions/runs, agent
  configuration, conversation policy, legacy memory, and knowledge routes
  consume the verified `UserContext` subject. ACP also rejects anonymous calls
  when its existing `auth_required` setting is true.
- Thread-adjacent isolation: PASS. In the two-JWT live case, user B could not
  observe or mutate user A's run or policy/configuration, while user A's controls
  succeeded. ACP denied B's get/delete/run-create/run-get operations against A's
  IDs. Both users ran the same direct session ID without replacing each other.
- Memory isolation: PASS. Spoofed body/query identities were ignored; user B
  could not retrieve user A's memory.
- SurrealKV identity: PASS. Equal KB/document/chunk IDs coexisted for Alice and
  Bob, searches were owner-only, and Bob's deletion left Alice's graph intact.
- PostgreSQL 17 identity: PASS. All 18 migrations applied to a fresh database.
  The exact test reproduced the equal-ID controls, rejected a cross-owner parent
  foreign key, and preserved Alice's graph after Bob's deletion.
- Legacy compatibility: PASS. An authenticated lookup could not claim an
  ownerless legacy session; anonymous lookup preserved and lazily migrated it.
- RAG fail-closed: PASS within the stated limit. Inaccessible KB search returned
  no chunks and the runtime maps an empty accessible-KB set to an empty result.
  No model-prompt inspection was performed.
- OpenSpec: PASS. All eight tasks are complete and strict validation exits 0.
- Lint limit: Clippy exits 0 with 572 existing pedantic warnings; this is not a
  warning-free result.

Full phase Tier 2 and immutable-candidate Tier 3 remain deferred by tier timing.
