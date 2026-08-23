# Network Policies Specification

## Purpose

Define default-deny Kubernetes network isolation and the explicit service and gateway communication paths required by UAR.

## Requirements

### Requirement: Default deny ingress policy
The namespace SHALL have a default-deny ingress NetworkPolicy preventing all inbound traffic except explicitly allowed.

#### Scenario: Unauthorized pod cannot reach UAR
- **WHEN** a pod in a different namespace attempts to connect to UAR on port 3000
- **THEN** the connection is denied by the NetworkPolicy

### Requirement: Inter-service communication allowed
NetworkPolicies SHALL allow UAR to communicate with PostgreSQL, Redis, and SurrealDB within the namespace.

#### Scenario: UAR connects to PostgreSQL
- **WHEN** UAR pod initiates a connection to PostgreSQL on port 5432
- **THEN** the connection is allowed by NetworkPolicy

#### Scenario: UAR connects to Redis
- **WHEN** UAR pod initiates a connection to Redis on port 6379
- **THEN** the connection is allowed by NetworkPolicy

#### Scenario: UAR connects to SurrealDB
- **WHEN** UAR pod initiates a connection to SurrealDB on port 8000
- **THEN** the connection is allowed by NetworkPolicy

### Requirement: Ingress allowed from gateway
The UAR service SHALL accept ingress traffic from the Envoy Gateway pods.

#### Scenario: Gateway routes to UAR
- **WHEN** Envoy Gateway forwards a request to UAR on port 3000
- **THEN** the connection is allowed by NetworkPolicy
