# skill-pack-distribution Specification

## Purpose
TBD - created by archiving change ship-skill-pack-install-path. Update Purpose after archive.
## Requirements
### Requirement: Skill pack installs from the public repository without a dev checkout
A documented installer SHALL fetch, build and install the prometheus skill
system onto an existing UAR installation, after which the admin UI lists all
pack skills eligible under the active loader policy. The default fetch SHALL
use public HTTPS and an immutable commit pinned by the matching UAR release. A
failed verification or build SHALL NOT activate a partial installation. The
installer SHALL preserve UAR's existing opt-in boundary for imported skills.

#### Scenario: Fresh install
- **WHEN** an operator runs the documented installer on a machine with UAR but no repo checkout
- **THEN** the pack is fetched over HTTPS, built, installed, and the pinned 1.7.0 pack's complete default inventory of 147 skills appears in the skills admin UI as non-deletable

#### Scenario: Source verification fails closed
- **WHEN** the fetched source does not match the UAR-pinned commit
- **THEN** installation stops before build and no pack version is activated

#### Scenario: Build failure preserves the active installation
- **WHEN** the locked Rust build fails
- **THEN** no partial version becomes visible to UAR and an existing installed version remains intact

#### Scenario: Upgrade follows the matching UAR pin
- **WHEN** an operator upgrades UAR and reruns the installer shipped with that release
- **THEN** the newly pinned pack version installs beside older versions and UAR selects the highest installed version
