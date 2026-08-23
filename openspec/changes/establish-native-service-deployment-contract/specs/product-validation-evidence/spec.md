## ADDED Requirements

### Requirement: Native inference evidence is genuine and bounded
An inference claim for a native service SHALL identify the actual provider and model, traverse the installed UAR boundary, and retain an observed model-produced response. Mocked, recorded, replayed, stubbed, or hard-coded responses SHALL NOT satisfy the claim. Verification SHALL use short requests rather than a soak.

#### Scenario: Native inference is certified
- **WHEN** an installed-service inference requirement is evaluated
- **THEN** evidence identifies the provider/model, source SHA, profile/platform, command or UI action, observed response, timeout, and request limit without retaining credentials
