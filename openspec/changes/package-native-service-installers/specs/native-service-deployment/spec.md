## ADDED Requirements

### Requirement: Native packages provide reversible lifecycle entrypoints
Each native package SHALL provide install and uninstall entrypoints plus documented start, stop, restart, upgrade, and credential-refresh operations. Uninstall SHALL preserve mutable state by default.

#### Scenario: Operator uninstalls a native service
- **WHEN** the uninstall entrypoint runs without an explicit destructive-state option
- **THEN** the supervisor registration and program files are removed while configuration, database state, backups, and logs remain recoverable

### Requirement: Platform packages use native filesystem conventions
macOS SHALL install a user service beneath `~/.uar`; Linux SHALL use `/etc/uar` for configuration and `/var/lib/uar` for state; Windows SHALL separate `%ProgramFiles%` program files from `%ProgramData%` state. Every package SHALL use the platform log directory defined by the native-service contract.

#### Scenario: Installer renders a service definition
- **WHEN** a supported installer creates its native service definition
- **THEN** executable, working-directory, configuration, environment, state, and log paths match the platform contract

### Requirement: Native service definitions restart only after failure
The macOS and Linux native service definitions SHALL start automatically and restart after unexpected failure without turning an operator-requested graceful stop into an uncontrolled restart loop.

#### Scenario: Native process exits unexpectedly
- **WHEN** UAR exits unexpectedly after its supervisor considers it running
- **THEN** the native supervisor restarts it according to its platform failure policy
