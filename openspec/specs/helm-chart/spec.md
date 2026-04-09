## ADDED Requirements

### Requirement: Helm chart packages all K8s resources
A Helm chart in `k8s/helm/uar/` SHALL generate all Kubernetes resources needed for a complete UAR deployment.

#### Scenario: Helm template renders all resources
- **WHEN** `helm template uar k8s/helm/uar/` is run
- **THEN** the output includes Deployment (UAR), StatefulSets (PostgreSQL, SurrealDB), Deployment (Redis), Services, ConfigMap, Secrets, PVCs, HPA, ServiceAccount, and NetworkPolicies

#### Scenario: Values override works
- **WHEN** `helm template uar k8s/helm/uar/ --set uar.replicas=3` is run
- **THEN** the UAR Deployment has `replicas: 3`

### Requirement: Helm chart has sensible defaults
The `values.yaml` SHALL provide production-ready defaults matching the Kustomize base manifests.

#### Scenario: Default resource limits
- **WHEN** no values overrides are provided
- **THEN** UAR pod has CPU request 250m, limit 1, memory request 256Mi, limit 1Gi

### Requirement: Chart passes lint
The Helm chart SHALL pass `helm lint` without errors.

#### Scenario: Lint clean
- **WHEN** `helm lint k8s/helm/uar/` is run
- **THEN** the output shows 0 errors
