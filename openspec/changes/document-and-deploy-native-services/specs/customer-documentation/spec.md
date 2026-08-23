## ADDED Requirements

### Requirement: Native deployment documentation is operationally complete
The README and product documentation SHALL cover native installation, upgrade, uninstall, credential refresh, configuration precedence, program/config/state/log locations, service control, listener exposure, troubleshooting, and platform-specific verification for macOS, Linux, and Windows.

#### Scenario: Operator follows platform instructions
- **WHEN** an operator selects a supported native platform
- **THEN** the documentation identifies prerequisites, exact lifecycle commands, default paths/ports, preservation behavior, log inspection, and the limits of locally observed evidence
