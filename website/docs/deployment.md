---
sidebar_position: 12
title: Deployment
description: Deploy pinned UAR server artifacts with explicit storage, secrets, probes, and ownership.
source_records:
  - docs/DEPLOYMENT.md
current_authority: /docs/deployment
---

# Deployment

## Boundary statement

**A deployment is healthy only when the exact pinned artifact, configuration,
secrets, persistence, and dependency path pass functional checks in the target
environment.** A local build, tag, image name, chart render, or liveness result
alone is insufficient.

```mermaid
flowchart LR
    Artifact[Verified image digest] --> Runtime[UAR server]
    Config[Config and secrets] --> Runtime
    Runtime --> Persistence[(Application persistence)]
    Runtime --> Memory[(Optional memory store)]
    Runtime --> Provider[Configured model provider]
    Runtime --> Live[/healthz]
    Persistence --> Ready[/readyz]
    Provider --> Functional[Representative inference and workflow]
```

## Diagram in words

A verified immutable image and operator configuration start the server. The
server connects to its application persistence, optional memory store, and
configured model provider. `/healthz` proves the process is alive. `/readyz`
checks configured dependencies. A representative authenticated workflow and
genuine model response establish the application path; the probes do not.

## Immutable pin

Prefer the digest recorded in the verified release manifest:

```text
ghcr.io/prometheus-ags/universal-agent-runtime@sha256:<verified-digest>
```

Never deploy a floating `latest` tag as the production identity. The committed
SurrealDB Compose file names `v1.0.0`, but the operator must still verify that
the tag exists and resolve it to a reviewed digest before deployment.

## Docker Compose

`docker-compose.prod.yaml` defines UAR, remote SurrealDB, and Redis with
persistent volumes. It exposes HTTP on `1906`, A2A gRPC on `50051`, and the
database/cache ports configured by the file.

The checked-in Compose defaults are a template, not production secrets. They
include development database credentials and currently default UAR JWT
authentication off. Before exposing the stack, override all credentials,
require JWT/JWKS authentication, restrict database/cache ports, and pin every
image by reviewed version or digest.

```bash
docker compose -f docker-compose.prod.yaml --env-file .env config
docker compose -f docker-compose.prod.yaml --env-file .env up -d
curl --fail http://127.0.0.1:1906/healthz
curl --fail http://127.0.0.1:1906/readyz
```

`docker-compose.prod.postgres.yaml` is a source-build preview and contains
floating third-party/application image tags. Do not treat it as an immutable
production deployment until every image is pinned, secrets are replaced, and
the `postgres-backend` build is verified.

## Kubernetes and Helm

`k8s/helm/uar` can bootstrap UAR with PostgreSQL, SurrealDB, Redis, probes, HPA,
network policies, and an optional Gateway API route. Its defaults are not
portable production values:

- `uar.image.repository` is a placeholder;
- bundled secret values are placeholders and must be replaced through a real
  secret-management path;
- the storage provisioner is GKE-specific;
- the default replica/HPA settings must match volume access modes and datastore
  topology.

Render and review before applying:

```bash
helm template uar k8s/helm/uar \
  --set uar.image.repository=<registry>/universal-agent-runtime \
  --set uar.image.tag=<immutable-version>
```

The repository's active deployment workflow targets an operator-managed Azure
AKS environment by replacing only the image and then performing rollout,
`/readyz`, and `/healthz` validation. It deliberately does not apply this Helm
chart or overwrite the cluster's existing configuration and secrets.

## Secrets

At minimum, provide a model/provider selection and credential, a deliberate JWT
or JWKS configuration, and persistence credentials/location. Store secrets in
the platform's secret manager, not in Git, Compose defaults, Helm values, image
layers, or command history. Per-user provider credentials have a separate
encryption-key boundary.

## Persistence

Name the owner of every state path before deployment:

| State | Typical owner | Required decision |
|---|---|---|
| application resources | SurrealDB/SurrealKV or PostgreSQL | durable volume/service, backup, restore, vector dimension |
| memory | optional Surreal memory service/store | enabled state, credentials, retention, backup |
| uploads | mounted filesystem path | volume, limits, cleanup, restore requirements |
| Redis cache | Redis | loss tolerance and external-cache setting |
| A2A task store | current UAR process memory | restart loss is expected |
| run/live projections | runtime manager and browser projection | reload/replay boundary, not an audit ledger |

See [Recovery and shutdown](./operations/recovery-and-shutdown.md) before
changing storage.

## Health and functional checks

- `/healthz` — process liveness;
- `/readyz` — configured dependency readiness;
- authenticated resource read/write — API, auth, and persistence path;
- representative tool-policy decision — governance and trusted-host path;
- genuine inference — provider/model route and response path.

Provider/model latency and failure are not UAR-owned liveness. Keep those
signals separate in dashboards and incident decisions.

## Deployment ownership

GitHub Actions in this repository are deployment execution and deployed-artifact
validation only. Routine unit, integration, lint, type, documentation, and
conformance checks run locally before commit and push. The Pages workflow may
assemble documentation and validate the deployed routes because those steps
are the deployment itself.

The AKS workflow owns only its image bump and deployment smoke checks. The
cluster configuration, secrets, databases, storage classes, gateway, backups,
and rollback decision remain operator-owned.

## Profile limits

This guide covers server deployment. `server-full` is the complete release
claim; `minimal` is a smaller server and needs its own feature/evidence record.
`embedded-mobile` has no server container or cluster listener and remains the
responsibility of its iOS/Android/application host.

Next: [Upgrade and rollback](./upgrade-guide.md).
