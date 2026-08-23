# GKE Deployment Specification

## Purpose

Define the container, Kubernetes, storage, routing, and deployment contracts for operating UAR on Google Kubernetes Engine.

## Requirements

### Requirement: Dockerfile uses latest Rust stable base image
The Dockerfile SHALL use `rust:1.87-slim-bookworm` (or latest stable) for the build stage and `debian:bookworm-slim` for the runtime stage.

#### Scenario: Build succeeds with current code
- **WHEN** `docker build -t uar:latest .` is run
- **THEN** the build completes successfully with the updated Rust base image and current frontend/backend code

#### Scenario: Multi-stage optimization
- **WHEN** the Dockerfile is built
- **THEN** it uses 3 stages: frontend build (Node 20 + Bun), Rust build (latest stable), runtime (minimal Debian)

### Requirement: K8s manifests in k8s/base/
The `k8s/base/` directory SHALL contain Kustomize-compatible manifests for all UAR components.

#### Scenario: Kustomize build succeeds
- **WHEN** `kubectl kustomize k8s/base/` is run
- **THEN** it produces valid YAML containing all required resources

#### Scenario: Resources include all components
- **WHEN** manifests are applied
- **THEN** the following resources exist: Namespace, UAR Deployment, PostgreSQL StatefulSet, SurrealDB StatefulSet, Redis Deployment, Services (4), ConfigMap, Secrets, PVCs (3+), StorageClass, HPA, ServiceAccount, NetworkPolicies, HTTPRoute

### Requirement: Storage uses immediate binding
All PersistentVolumeClaims SHALL reference a StorageClass with `volumeBindingMode: Immediate`.

#### Scenario: PVC binds immediately
- **WHEN** a PVC is created for PostgreSQL data
- **THEN** it uses the `uar-immediate` StorageClass which has `volumeBindingMode: Immediate`

### Requirement: HTTPRoute for Envoy Gateway
Traffic routing SHALL use Gateway API HTTPRoute (not Ingress) pointing to the existing Envoy Gateway.

#### Scenario: HTTPRoute routes to UAR
- **WHEN** an HTTPRoute is applied referencing the existing Gateway
- **THEN** HTTP traffic on the configured hostname routes to the UAR service on port 3000

### Requirement: ArgoCD-compatible directory structure
The `k8s/` directory SHALL be structured for ArgoCD Application watching.

#### Scenario: ArgoCD detects changes
- **WHEN** a manifest file in `k8s/base/` is modified and pushed to main
- **THEN** ArgoCD detects the change and triggers a sync (or marks as OutOfSync)

### Requirement: GitHub Actions deployment workflow
A workflow at `.github/workflows/deploy.yml` SHALL deploy a locally built and certified immutable Docker image, publish that exact image to Google Artifact Registry when required, and update the image reference in K8s manifests. It SHALL NOT build, unit test, integration test, lint, or perform other routine development verification in GitHub Actions.

#### Scenario: Accepted source triggers deployment
- **WHEN** deployment is requested for an accepted `main` source SHA with a locally certified image digest
- **THEN** the workflow verifies and deploys that immutable image without rebuilding it

#### Scenario: Image published to Artifact Registry
- **WHEN** the certified immutable image is absent from the configured Artifact Registry repository
- **THEN** the workflow publishes that exact image and verifies its digest

#### Scenario: Manifest updated with new tag
- **WHEN** the image is pushed
- **THEN** the workflow updates the image tag in `k8s/base/uar-deployment.yaml` and commits the change

#### Scenario: Manual trigger supported
- **WHEN** a user triggers the workflow via `workflow_dispatch`
- **THEN** the workflow runs with optional image tag override

#### Scenario: Smoke test after deploy
- **WHEN** ArgoCD syncs the new manifests
- **THEN** the workflow waits for rollout completion and hits `/readyz` to verify the deployment
