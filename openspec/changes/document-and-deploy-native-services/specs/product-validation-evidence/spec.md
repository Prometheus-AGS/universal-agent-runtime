## ADDED Requirements

### Requirement: Native release evidence separates API, UI, provider, and platform claims
Evidence SHALL record API and shipped-UI observations for local proxy, Kimi K3, and MiniMax M3 through installed UAR, along with source SHA, server-full profile, platform, timeout/output limits, and redaction. macOS runtime, Linux-template, and Windows compile/template results SHALL be separate.

#### Scenario: Required provider is unavailable
- **WHEN** a credential, endpoint, capacity, or model prevents a required genuine response
- **THEN** the phase stops before reflection and reports that exact inference requirement as unmet without substituting synthetic evidence
