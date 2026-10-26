# Exact-owned Postgres fixture lifecycle

Source authored only. Gate CLOSED. This helper has not been imported or executed;
no Docker command, service, psql query, build, check, test or probe has run.
The future named acceptance packet must explicitly approve these container,
private credential-file, owned database/role/extension, stop/remove and cleanup
effects. Merely configuring argv does not authorize execution now.

Lifecycle argv, invoked by the existing runtime observer through its paired hooks:

```
[frozen_python, absolute_stores_fixture_py, "postgres-prepare", private_pg_config]
[frozen_python, absolute_stores_fixture_py, "postgres-cleanup", private_pg_config]
```

Root must require the helper's prepared receipt before CREATE SCHEMA, not infer
readiness from elapsed time. The runtime observer creates its actual run root and
ownership.json first and supplies UAR_ACCEPTANCE_CONTEXT. The helper derives
`uar_acceptance_<sha256(context.run_id)[:20]>`, requires backend postgres_pgvector,
and checks ownership.json run/backend/scope/nonce. Container name is that scope.
Labels bind exact run ID, scope, backend and ownership nonce. The configured
private state parent must match the runtime's selected state-parent env reference.

## Explicit private config

Regular absolute nonsymlink mode600 JSON, <=1MiB. Fields:

- schema_version1, executabletrue.
- docker and psql: `{path: absolute_frozen_regular_executable,sha256:"sha256:hex"}`.
  Use actual resolved tool binaries, not an unverified symlink shortcut. They are
  frozen/hashed later under the named invocation, never probed during authoring.
- state_parent_env: UAR_FIXTURE_POSTGRES_STATE_PARENT (or an explicit named ref).
- image: exact canonical repository@sha256:<64hex>, selected by the operator.
  Must already exist locally with that exact RepoDigest and include the real
  pgvector extension. There is no default image/version/digest, pull or build.
- host_port: explicit nonprivileged owned port matching private prep ports.postgres.
- container_pg_port: explicit actual PostgreSQL port for that image.
- mount_target: exact absolute container bind directory for the approved image.
- pgdata: exact version-appropriate absolute PGDATA strictly inside mount_target.
  No version-specific path is guessed. Mount must match the image's actual VOLUME
  contract; unexpected anonymous/other mounts fail ownership confirmation.
- username_env/password_env/database_env/connection_env: named generated inputs,
  normally UAR_FIXTURE_PG_USERNAME/PASSWORD/DATABASE/CONNECTION. Username/database
  must be generated `uar_fixture_<20hex>` names. Connection must match exact
  username/password/database and 127.0.0.1:host_port, with no query/fragment.
- docker_environment_env: explicit map of Docker client environment target to
  named source env ref. Supported targets HOME, DOCKER_HOST, DOCKER_CONTEXT,
  DOCKER_CONFIG, DOCKER_TLS_VERIFY, DOCKER_CERT_PATH, DOCKER_API_VERSION. No ambient
  secret environment is forwarded. Exact daemon/context selection belongs to the
  operator input; it is not inferred by starting a daemon or inspecting services now.
- limits: positive integer total_seconds<=110, command_seconds, ready_seconds,
  cleanup_seconds, stop_seconds, output_bytes<=1048576. Command/ready/cleanup must
  be strictly below total; stop below cleanup and command. Composer outer
  observer_seconds must be >=total_seconds+5 and<=120. A hard timer exits75 at
  total_seconds after the bounded initial config read, even if a CLI blocks.
  Root outer process deadline additionally covers initial config reading.

There are no arbitrary Docker arguments, SQL strings, image entrypoint overrides,
credential values, shell commands, role names or container names in config.
Credential values travel only through named environment references and a private
mode600 root/postgres-container.env (POSTGRES_USER/PASSWORD/DB and explicit PGDATA).
Neither argv nor stdout contains a password or credential URL. Docker inspection
contains environment internally but is parsed privately; only its opaque hash and
whitelisted identity fields leave the helper. Raw command errors are withheld.

## Prepare and actual handshake

Preparation refuses an existing same-name container, data directory or claim.
It inspects the exact locally present image, creates root/pg-data exclusively,
and fsyncs postgres-fixture-claim.json before creating a container. Docker run
uses --detach --pull=never, exact labels, --mount type=bind with source
root/pg-data and explicit target, --publish 127.0.0.1:host:container-port, and
--env-file. The image's actual POSTGRES_* initialization creates only this fresh
container's role/database. It never connects to an operator datastore.

Actual Docker inspect must confirm immutable returned container ID, exact name,
configured image, RepoDigest/image ID, ownership labels, exactly one writable bind
mount, exact owned data source/target and exact loopback port mapping. A configured
flag or a healthy-looking container state alone cannot stand in for readiness.

psql receives host/port/user/password/database via its child environment and a
fixed bounded SQL statement through stdin, with -X/-w/ON_ERROR_STOP. Startup waits
ONLY for exit2 with actual connection-refused stderr, within ready_seconds. Auth,
SQL, timeout, unavailable extension and arbitrary transport errors fail immediately.
A real SELECT confirms readiness, actual current_database/current_user and actual
pg_available_extensions entry for vector. Then CREATE EXTENSION vector runs in
that exact fresh database and a second SELECT returns the installed version.
No generic sleep/readiness flag earns success.

Success writes exclusive root/postgres-fixture-prepared.json and identical stdout:

- schema_version `uar.owned-postgres-fixture.v1`, status prepared.
- run_id, scope, config_sha256, ownership_nonce.
- actual: immutable container_id/name, image/image_id, labels, mount source/target,
  PGDATA, host/host_port/container_pg_port, running and opaque inspection SHA.
- ready: ready1, actual database/username, vector_availabletrue, server_version_num.
- pgvector: nonempty installed extension version plus actual database/username.
- data_retainedtrue.

Root must read this actual owned receipt and match config, nonce, scope, image,
mount and endpoint before schema setup and before reporting prepared. Its store
observer still checks real application data/schema counts independently.

## Cleanup and failed preparation

Cleanup validates the immutable claim and actual Docker ownership again. A
successful prepared receipt additionally binds its exact container ID. When
preparation failed after reservation, the exact claim/nonce/labels/image/mount
still permit scoped recovery of only that actual container; this is not a green
prepared receipt. Cleanup must never be skipped solely because readiness failed.
Root must arrange a bounded cleanup path for that partial-owned state instead of
requiring a successful application schema query before lifecycle cleanup.

The helper stops only that inspected immutable ID, reinspects it, removes only
that ID without volume removal, and confirms no exact-owned name remains.
It writes postgres-fixture-cleaned.json and prints status cleaned with
container_removedtrue and data_retainedtrue. It never removes PG data or the
private env file. Root collects retained data/identity evidence, then its separately
approved exact-owned root cleanup removes them. A failed or hard-killed helper
may leave a claimed container/data; retain that evidence and reconcile explicitly.
No retry resets a name, scope, credentials, image or previous consumption.

The uncomfortable limit is that image entrypoint and filesystem ownership are
operator-selected. A digest alone does not prove the image has compatible
Postgres/pgvector initialization or host-bind permissions. Actual SQL, inspect
and later approved cleanup must establish those properties. If the container
changes host-bind ownership so ordinary owned-root cleanup cannot read it, retain
and report that failure; do not delete an ambient path or introduce an implicit
privileged repair. Compatibility of these authored Docker/psql commands, actual
image/version, lifecycle timings and all application receipts remains unverified.

The actual readiness SELECT also records and checks PostgreSQL data_directory
against explicit PGDATA (strictly inside the inspected bind mount) and the actual
server port against container_pg_port. Merely mounting a directory while the
server stores its real data elsewhere cannot earn this prepared handshake.

## Partial creation with no container

Cleanup first validates the runtime ownership marker and any existing private
creation claim (config SHA, run/scope, image, exact mount source, nonce and labels).
It then performs a successful bounded exact-name `docker ps -a --filter
name=^/<scope>$ --format {{.ID}}`. Nonzero status, timeout, malformed/multiple IDs
or any command diagnostic blocks; none is treated as absence. A successful empty
listing is actual absence evidence, so cleanup returns status cleaned without
calling inspect, stop or rm and without inventing a container ID. This also covers
failure before the creation claim exists, provided the runtime-owned marker and
config are valid and the exact name is positively absent. A present container
without its creation claim is refused; it is never deleted by name.

Both successful cleanup variants include config_sha256, ownership_nonce,
container_absenttrue and absence_evidence (exact name, listing SHA, observed time,
container_absenttrue). The already-absent variant has container_removedfalse,
claim_validated reflecting whether an actual claim existed, and no
actual_before_stop/container_id field. The removal variant has container_removedtrue
and actual_before_stop from exact validated inspection. Data is retained in both.
Root must use container_absent rather than require a fabricated removal action;
this resource cleanup result never converts failed preparation into acceptance.
