# Spec Delta

## ADDED Requirements

### Requirement: Configured filesystem sources satisfy durable admission invariants

UAR SHALL reconcile configured filesystem skill sources with durable selection metadata before admitting team member runs, preserving existing API-owned overrides and scoped enablement. It SHALL retain safe errors instead of bypassing failed reconciliation.

#### Scenario: Configured sources admit the math and writing members
- **WHEN** the repaired runtime starts with its configured filesystem skills and the existing math/writing project is submitted
- **THEN** both members enter their genuine configured-model runs without the configured-source metadata mismatch
- **AND** the existing TEAM-01 scenario can demonstrate their concurrent results and final artifact integration
