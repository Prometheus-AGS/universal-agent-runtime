## ADDED Requirements
### Requirement: Team member skills resolve during binding preflight
Team binding preflight SHALL resolve every agent member's declared SkillRefs through the same canonical exact installed skill resolver used by ordinary agent bindings and team execution. It SHALL retain required/optional disposition and immutable source definition attribution. An unavailable required skill SHALL prevent activation; portable package preflight SHALL remain independent of private installed locations.

#### Scenario: Required member skill is stale
- **WHEN** a team binding references an agent requiring a skill whose version, digest, installedLocation, entrypoint, requiredTools, enabled or tombstone state does not match current installed metadata
- **THEN** preflight returns the existing skill.installed-artifact-mismatch required-unsupported diagnostic attributed to the agent definition and does not admit activation

#### Scenario: Required member skill is not bound
- **WHEN** a member declares a required SkillRef absent from private skillBindings
- **THEN** preflight reports skill.binding-missing for that definition before team activation
