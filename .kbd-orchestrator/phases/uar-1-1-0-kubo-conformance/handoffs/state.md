# uar-state handoff — remote SKILL.md import

## Delivered source changes

- `src/uar/domain/skills.rs`
  - Added the persisted optional `Skill::skill_markdown` field. It carries the
    exact UTF-8 document submitted to the remote import API; the existing
    parsed `prompt_overlay` remains the activation representation.
- `src/uar/runtime/skills/service.rs`
  - Added `SkillContentImportError` and
    `SkillService::import_skill_markdown(String)`.
  - The method parses one manifest with the existing SKILL.md parser, derives
    the normal lowercased space-to-hyphen ID, records the exact-document
    `sha256:<hex>` digest, and marks the result as a user manifest skill.
  - It requires a configured persistence layer, rejects an in-memory or
    durable ID collision, and calls `SkillRegistry::register_if_absent_checked` while
    holding the registry write lock. Persistence completes before registry
    publication and before the API can return. It does not call a filesystem
    storage provider.
  - It now calls the existing governance mutation check before persistence;
    governance denial maps to `skill_mutation_denied`.
- `src/uar/runtime/skills/registry.rs`
  - Added the package-visible persistence-presence query and checked
    create-if-absent registry publication. A row reaches memory only after the
    storage provider atomically creates it.
- `src/uar/persistence/mod.rs`
  - Added `PersistenceLayer::create_skill_if_absent`, distinct from the
    existing update/upsert API.
- `src/uar/persistence/providers/memory.rs`
  - Added a lock-protected create-if-absent implementation for deterministic
    local fixtures.
- `src/uar/persistence/providers/surreal.rs`
  - Added a transaction that checks the named `skills` record and creates it
    only when absent; its explicit duplicate signal maps to `false`.
- `src/uar/persistence/providers/postgres.rs`
  - Added `INSERT ... ON CONFLICT DO NOTHING`, returning `false` without
    replacing an existing row.
- `src/uar/api/skills.rs`
  - Added `POST /api/uar/skills/import-content`, taking
    `{ "skill_markdown": "..." }`.
  - Returns `201` with the ordinary skill response, now including
    `artifact_digest` and, where available, exact `skill_markdown`.
  - Error mapping: malformed manifest `400 skill_markdown_invalid`; collision
    `409 skill_id_conflict`; governance denial `403 skill_mutation_denied`;
    unavailable or failed persistence `503`.
  - Existing agent binding routes remain the bind interface:
    `PUT /api/uar/agents/{agent_id}/skills` or
    `POST /api/uar/agents/{agent_id}/skills/{skill_id}`.
- `tests/fixtures/skill-upload/round-trip.SKILL.md`
  - Valid remote payload with trailing newline for exact-body comparison.
- `tests/fixtures/skill-upload/collision.SKILL.md`
  - Different body with the same derived ID for the `409` acceptance case.
- `tests/fixtures/skill-upload/invalid.SKILL.md`
  - Missing closing frontmatter delimiter for the `400` acceptance case.
- `tests/fixtures/skill_remote_import.py`
  - External-image fixture with no local process or database ownership. Its
    caller provides an authenticated absolute-URL request callback and a
    restart callback. It exercises upload/readback/binding/restart/readback,
    collision non-overwrite, malformed content, and a separate configured
    persistence-failure image that must return `503` without GET publication.

## Suggested acceptance path after the phase implementation boundary

1. Start the real authenticated runtime with a persistence provider and no
   shared skill filesystem between client and server.
2. POST the raw bytes of `round-trip.SKILL.md` in `skill_markdown`; require
   `201`, `skill_id = remote-receipt`, and a `sha256:<hex>` digest computed
   from those same UTF-8 bytes.
3. GET `/api/uar/skills/remote-receipt`; compare both `skill_markdown` and
   `artifact_digest` exactly to the POST source and computed digest.
4. Bind the returned ID using the existing agent binding endpoint, restart the
   runtime over the same persistence provider, and repeat both GET and binding
   readback.
5. POST `collision.SKILL.md`; require `409` and verify the first raw document
   and digest remain unchanged. POST `invalid.SKILL.md`; require `400`.
6. Use a real external SurrealDB fault: stop the pinned storage container,
   require `503` from a new import, restore the database, and verify that skill
   remains absent by GET. This proves no memory publish occurred after an
   unacknowledged write.

## Release acceptance runner handoff

- `scripts/release/uar_1_1_acceptance.py` is the authored, unexecuted
  installed-image orchestrator. It requires `--image` to be a digest, or an
  explicit `--local-tag`, and a new evidence directory. It starts pinned
  `surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20` without `--rm` so a real
  stopped-database import fault can restart the same durable RocksDB service.
- The runner invokes the Dockerfile `CMD` executable explicitly as
  `universal-agent-runtime --config …`; it does not attempt to execute
  `--config` as the image command. Runtime profiles override the Dockerfile's
  embedded `UAR_PERSISTENCE__DATABASE_URL` with the remote Surreal endpoint
  and set `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1`. MCP and each A2A tuple
  use distinct Surreal databases, so fixed protected skill IDs cannot collide.
- `tests/fixtures/skill_remote_import.py` runs against each real MCP runtime
  and exercises exact upload/readback, binding, restart, collision, malformed
  content, and a real stop/restore persistence failure. The runner invokes the
  MCP fixture for both `server.mcp_shared_transports_enabled = false` and
  `true`, including its restart and administrator callbacks. A private
  temporary authorization file supplies the run-grant sentinel. The peer
  records only credential-match booleans, and the runner removes the private
  file before returning.
- MCP peers are registered with a literal Docker host-gateway private IP,
  obtained from the candidate container. This satisfies the actual
  `allow_private_http` URL admission rule; `host.docker.internal` would not.
  The fixture-only protected manifest declares `qualification__elicit` and the
  A2A protected manifest declares `a2a-qualification__effect`; their agent
  artifacts explicitly select those tools. The independent remote SKILL.md
  round-trip payload itself remains unmodified.
- For each MCP sharing mode, the runner first retains that private-IP remote
  destination for the external-grant/full-elicitation profile. It then starts
  `scripts/release/uar_1_1_loopback_relay.py` with `docker exec` in the same
  candidate runtime namespace. The relay binds only `127.0.0.1` and forwards
  TCP bytes to the original controlled peer, so the fixture's localhost and
  `127.0.0.1` alias-denial variants are reachable without replacing the real
  peer, authorization sentinel, or trace. The loopback profile restarts both
  runtime and relay before its persistence assertion, records separately from
  remote grant coverage, and restores the private-IP destination afterward.
- `scripts/release/uar_1_1_loopback_client.py` passes
  `UAR_A2A_TENANT_ID` from the same verified `TENANT` claim used to mint both
  trusted and limited release credentials. The A2A fixture can therefore prove
  matching tenant admission and rejection of a mismatched tenant without task
  or peer side effects.
- `scripts/release/uar_1_1_defaults.py` is loaded through its exported
  `run_default_profile(image, evidence, docker=…, require=…, wait_for=…)`
  contract. Its default image path has no config file, model environment,
  command override, or port publication. It records startup refusal without a
  configured JWT secret, authenticated effective settings/version/CORS/no-model
  observations, and cumulative non-loopback transmit counters during 300
  seconds. This is observation evidence, not a claim about sub-millisecond
  startup activity.
- `scripts/release/uar_1_1_loopback_client.py` runs from the same production
  image with `--network container:<runtime>`. It supplies the only viable
  client path for the `server.host = 127.0.0.1` governance-disabled tuples,
  while the supervisor retains the runtime lifecycle. All eight A2A tuples
  (`governance={enforced,disabled} × transport={jsonrpc,grpc} ×
  decision={approve,deny}`) use a separate remote-persistence database,
  provider trace, and MCP trace. The protected skill body marker is
  `UAR_PROTECTED_PROMPT_BODY_379`, distinct from the dynamic request marker.
- Before A2A execution, the runner downloads the official pinned grpcurl 1.9.4
  checksum manifest, verifies its pinned SHA-256, verifies the selected Linux
  archive against that manifest, and writes both digests to
  `evidence/tools/grpcurl-receipt.json`. The checked binary is mounted at
  `/tools/grpcurl` in the candidate client container.
- Work and traces stay below `evidence/work`; the runner does not use a
  temporary directory that deletes paths cited by receipts. It redacts Bearer
  values and JWT-shaped values in subprocess failure receipts. Its client
  container passes runtime credentials by named inherited environment entries,
  so Docker command receipts never contain credential values.
- The runtime-loopback MCP relay validates that its recorded PID still belongs
  to `/opt/uar/uar_1_1_loopback_relay.py` before signaling it and removes its
  ready/PID files before every launch. A runtime restart therefore cannot make
  a stale readiness marker or a reused PID satisfy the loopback-alias phase.

## Verification status

No tests, compiler, formatter, build, validation, review, Docker runtime, or
other execution gate ran. The active phase has not reached its full
implementation boundary. The runner and fixtures are source-authored only.

## Shipped-profile authentication fixture

- `scripts/release/uar_1_1_profiles.py` owns shipped-profile materialization
  and the anonymous effective-settings check. It hashes the source manifests,
  records their exact paths and non-secret local substitutions, stages the
  file-based configurations with source credential values removed, and runs the
  supplied production candidate image directly. It does not run Compose,
  Kubernetes, OpenTofu, or a source build.
- The profile matrix has twelve installed-image projections:
  `config.embedded.yaml`, `config.remote.surreal.yaml`,
  `config.remote.postgres.yaml`, `config.test.yaml`, `example.config.yaml`,
  development/test/Surreal production/Postgres production Compose, Kubernetes
  base, Helm, and OpenTofu.
  Every projection generates the JWT and settings-admin secrets; it preserves
  each manifest's listener, provider, model, and memory selections while
  substituting runner-local service endpoints and writable embedded paths.
- PostgreSQL-backed projections start their selected image (`pgvector/pgvector:pg17`
  or `tribehealth/uar-postgres:pg17`) on the isolated release network. Redis
  projections start the selected `redis/redis-stack:latest` or `redis:7-alpine`
  image. Remote Surreal projections use the release runner's already pinned
  SurrealDB 3.3.0 service. The fixture waits for each service and runtime health,
  then makes unauthenticated Docker-namespace GETs to
  `/api/uar/settings/effective` and `/api/agents`, requiring HTTP 401 from
  each. The second probe catches a disabled JWT middleware configuration because
  effective-settings itself denies every anonymous request inside its handler.
  A startup refusal is a failing/incomplete profile, never a passing auth result.
- JWT and cache settings are derived from the shipped YAML, deployment
  expression, or the exact compiled default source and recorded with that source
  hash. File-based profiles retain their own `jwt_required` and
  `external_cache_enabled` values; only generated secrets and deliberate local
  endpoints or writable paths are overlaid. `docker-compose.dev.yaml` and
  `docker-compose.test.yaml` therefore keep the compiled `false` cache default.
  Embedded and upload paths are staged under `/var/lib/uar`, which is writable
  by the production image's non-root user.
- `scripts/release/uar_1_1_acceptance.py` loads the profile module through
  `shipped_profile_checks`. It now accepts repeatable
  `--gate {all,defaults,profiles,mcp,a2a}`. The default `all` runs the whole
  release program. A selected subset starts only its needed network and storage
  resources, records `full_acceptance: false`, and reports `partial-passed` so a
  rerun of one failed gate cannot be mistaken for release acceptance.

### Remaining setup limitation

The two Postgres-production projections request
`tribehealth/uar-postgres:pg17` first. If Docker reports that exact image is
unavailable, the fixture records and uses the repository-selected
`pgvector/pgvector:pg17` local PostgreSQL 17 dependency for the authentication
probe. Startup, health, or migration errors after an image starts remain
failures and do not trigger fallback. No profile or runner command has been
executed.
