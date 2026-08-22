# Decisions — `fix-user-isolation-sessions-memory-kb`

## Iteration 1 — 2026-08-18T11:16:42Z

- **Decision:** await independent review before termination.
- **Iteration:** 1 of 5.
- **Blocking violations remaining:** 0 in local deterministic checks.
- **Rationale:** verified identity, owner-scoped providers, fail-closed KB resolution, and the shared-admin compatibility boundary have implementation and focused evidence.
- **Uncomfortable result:** the repository's Clippy command exits 0 while emitting 573 warnings, and no live PostgreSQL migration was executed. Neither is represented as a clean or runtime-verified result.

## Iteration 2 — 2026-08-18T12:18:22Z

- **Decision:** continue to independent rereview before termination.
- **Iteration:** 2 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** the first review's durable identity, thread-adjacent state, and
  legacy-session blockers now have implementation and observed positive/negative
  controls, including a real PostgreSQL 17 migration and test.
- **Supersedes:** iteration 1's statement that PostgreSQL execution was absent.
- **Uncomfortable result:** the first candidate's passing receipt was wrong; its
  tests did not exercise the durable collision or all session-adjacent surfaces.

## Iteration 3 — 2026-08-18T12:29:07Z

- **Decision:** continue to independent rereview before termination.
- **Iteration:** 3 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** ACP now receives the verified request principal, respects its
  existing authentication-required setting, partitions sessions, and creates
  and retrieves only owner-matching runs; live negative and positive controls
  passed.
- **Uncomfortable result:** ACP's `auth_required` configuration existed but was
  inert because the router was mounted after the main authentication layer.

### Final independent decision — 2026-08-18T12:37:17Z

- **Decision:** terminate refinement and permit the OpenSpec/KBD completion
  transition.
- **Evidence:** critic PASS 0/0/0 and judge PASS on the exact staged candidate.
