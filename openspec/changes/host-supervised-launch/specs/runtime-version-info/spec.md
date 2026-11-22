## ADDED Requirements

### Requirement: The runtime reports its build identity
The runtime SHALL serve `GET /api/uar/version` with a JSON object of values fixed when the binary is built: package name and version, source commit, build profile, target triple, enabled Cargo features, wire protocol versions and the administration manifest revision. The response SHALL NOT contain a configuration value, filesystem path, host name, credential or secret.

#### Scenario: Installer checks the running version
- **WHEN** a client sends `GET /api/uar/version` to a running runtime
- **THEN** the response is 200 and `version` equals the package version of the binary that is running
- **AND** the body has `name`, `git_commit`, `build_profile` (`debug` or `release`), `target`, `features` (sorted), `protocols` (`a2a`, `ag_ui`, `ag_ui_profile`, `a2ui`, `mcp`) and `administration_schema`
- **AND** `protocols.a2a` is null when the build has no A2A transport

#### Scenario: Build with a recorded commit
- **WHEN** the binary was built with the `UAR_GIT_SHA` build variable set, or in the release profile from a git checkout
- **THEN** `git_commit` is that commit as a full lowercase hexadecimal object id

#### Scenario: Build without a recorded commit
- **WHEN** the binary was built in the debug profile without `UAR_GIT_SHA`
- **THEN** `git_commit` is null

### Requirement: Reading the build identity needs no credentials on a standalone server
On a standalone server the runtime SHALL answer `GET /api/uar/version` without credentials, including when JWT authentication is required for other routes. This exemption SHALL apply only to the `GET` method on that exact path. In supervised mode the launch-token guard SHALL apply to this route as to every other route.

#### Scenario: Standalone server that requires a JWT
- **WHEN** `security.jwt_required` is true and a client sends `GET /api/uar/version` with no credentials
- **THEN** the response is 200 with the build identity, while `GET /api/uar/capabilities` with no credentials is still 401

#### Scenario: Another method or a near-miss path without credentials
- **WHEN** `security.jwt_required` is true and a client sends `POST /api/uar/version` or `GET /api/uar/version/` with no credentials
- **THEN** the response is 401

#### Scenario: Supervised process without the launch token
- **WHEN** a client sends `GET /api/uar/version` to a supervised process without the launch token
- **THEN** the response is 401 and the body does not contain the version

#### Scenario: Supervised process with the launch token
- **WHEN** the host sends `GET /api/uar/version` to its supervised process with the launch token
- **THEN** the response is 200 with the build identity
