---
sidebar_position: 7
title: Deployment
---

# Deployment

The supported customer distribution is the `server-full` profile. Run it as a
container, with Docker Compose, or with the repository Helm chart. Pin a release
tag or signed digest in production; do not deploy `latest`.

## Docker Compose quickstart

```bash
cp .env.example .env
# Set UAR_LLM__MODEL, a provider key, UAR_SECURITY__JWT_SECRET,
# SURREAL_USER, and SURREAL_PASS in .env.
docker compose -f docker-compose.prod.yaml --env-file .env up -d
curl -sf http://localhost:1906/healthz
```

Persist the SurrealDB data directory or named volume before putting the service
under load. The default HTTP port is `1906`; the A2A gRPC port is `50051`.

## Kubernetes

The chart under `k8s/helm/uar` deploys UAR with its data services. Supply
production secrets through your secret manager rather than committing values:

```bash
helm install uar k8s/helm/uar \
  --namespace uar \
  --create-namespace \
  --set uar.image.tag=v1.0.0
```

Before promotion, confirm liveness, readiness, persistence after restart, and
that the configured JWT issuer/audience and tenant boundary match the edge
gateway. See [Installation](./installation), [Configuration](./configuration/intro.md),
[Security](./security), and the repository's
[deployment reference](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/DEPLOYMENT.md).
