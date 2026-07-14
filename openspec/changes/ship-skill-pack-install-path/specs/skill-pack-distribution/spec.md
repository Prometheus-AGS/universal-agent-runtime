## ADDED Requirements

### Requirement: Skill pack installs from the public repository without a dev checkout
A documented installer SHALL fetch, build and install the prometheus skill
system onto an existing UAR installation, after which the admin UI lists all
pack skills.

#### Scenario: Fresh install
- **WHEN** an operator runs the documented installer on a machine with UAR but no repo checkout
- **THEN** the pack is fetched over HTTPS, built, installed, and every pack skill appears in the skills admin UI as non-deletable
