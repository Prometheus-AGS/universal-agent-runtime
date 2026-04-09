## ADDED Requirements

### Requirement: UAR pods autoscale based on CPU and memory
A HorizontalPodAutoscaler SHALL scale UAR pods between configured min and max replicas based on CPU and memory utilization.

#### Scenario: Scale up on high CPU
- **WHEN** average CPU utilization across UAR pods exceeds 70%
- **THEN** the HPA scales up by adding pods (up to max replicas)

#### Scenario: Scale down on low utilization
- **WHEN** average CPU utilization drops below 30% for the stabilization window
- **THEN** the HPA scales down (to min replicas)

#### Scenario: Default replica range
- **WHEN** no overrides are provided
- **THEN** HPA is configured with min 2, max 10 replicas
