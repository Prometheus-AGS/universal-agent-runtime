## ADDED Requirements

### Requirement: Configuration files reconcile into the skill database at startup
On startup the runtime SHALL reconcile configuration-provisioned skills against
stored skills, so that the database reflects the current configuration without
manual intervention.

#### Scenario: New skill in configuration is added
- **WHEN** a skill file is added to the configuration source and the runtime restarts
- **THEN** the skill is present in the database and available for matching

#### Scenario: Changed skill in configuration is updated
- **WHEN** an existing configuration skill's definition changes and the runtime restarts
- **THEN** the stored skill reflects the new definition

#### Scenario: Removed skill is tombstoned, not deleted
- **WHEN** a configuration-provisioned skill's file is removed and the runtime restarts
- **THEN** the skill is marked removed, is excluded from matching and default listings, and its record is retained

#### Scenario: Restoring the file restores the skill
- **WHEN** a tombstoned skill's configuration file is restored and the runtime restarts
- **THEN** the skill is active again and its prior scoped configuration still applies

### Requirement: Reconciliation removes only configuration-provisioned skills
Reconciliation SHALL determine candidacy from the skill's recorded source.
Built-in skills and skills created through the API SHALL NOT be tombstoned by
reconciliation under any circumstances.

#### Scenario: User-created skill survives an empty configuration
- **WHEN** a skill was created through the API and no configuration file mentions it
- **THEN** reconciliation leaves it untouched and active

#### Scenario: Built-in skill is never tombstoned
- **WHEN** reconciliation runs and no configuration file mentions a built-in skill
- **THEN** the built-in remains active

### Requirement: Reconciliation fails safe when the configuration source looks broken
The runtime SHALL NOT tombstone the catalogue when the configuration source
yields no skills while the database holds configuration-provisioned skills, since
an empty source is more consistent with a missing mount than with deliberate
removal.

#### Scenario: Empty configuration source is refused
- **WHEN** the configuration source resolves to zero skills and the database holds configuration-provisioned skills
- **THEN** no skill is tombstoned and the condition is logged at error level

#### Scenario: Every tombstone is attributable
- **WHEN** reconciliation tombstones a skill
- **THEN** it logs the skill id and the reason, so a deliberate removal is distinguishable from an environment fault

#### Scenario: Negative control for the fail-safe
- **WHEN** the empty-source test runs against a build with the fail-safe removed
- **THEN** the test fails, demonstrating the guard is doing the work
