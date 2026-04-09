## ADDED Requirements

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

### Requirement: GitHub Actions build and deploy workflow
A workflow at `.github/workflows/deploy.yml` SHALL build the Docker image, push to Google Artifact Registry, and update the image tag in K8s manifests.

#### Scenario: Push to main triggers build
- **WHEN** code is pushed to the `main` branch
- **THEN** the workflow builds a Docker image tagged with the git SHA

#### Scenario: Image pushed to Artifact Registry
- **WHEN** the Docker build succeeds
- **THEN** the image is pushed to the configured Artifact Registry repository

#### Scenario: Manifest updated with new tag
- **WHEN** the image is pushed
- **THEN** the workflow updates the image tag in `k8s/base/uar-deployment.yaml` and commits the change

#### Scenario: Manual trigger supported
- **WHEN** a user triggers the workflow via `workflow_dispatch`
- **THEN** the workflow runs with optional image tag override

#### Scenario: Smoke test after deploy
- **WHEN** ArgoCD syncs the new manifests
- **THEN** the workflow waits for rollout completion and hits `/readyz` to verify the deployment
