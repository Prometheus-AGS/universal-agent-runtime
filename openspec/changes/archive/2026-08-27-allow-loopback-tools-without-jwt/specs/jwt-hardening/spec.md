## ADDED Requirements

### Requirement: Optional JWT permits an exact local governance-optional posture
The runtime SHALL classify governance as operator-optional only after boot proves all three conditions: the boot-effective configured `server.host` literal is exactly `localhost` or `127.0.0.1`; the authentication middleware installed for the process does not require JWT; and every declared tool-capable ingress has supplied a successfully bound loopback address to a sealed inventory. No other configured host spelling or address SHALL qualify. Requests without credentials SHALL continue with the anonymous principal in that posture.

#### Scenario: Localhost without required JWT is eligible
- **WHEN** boot configures `server.host` as `localhost`, installs authentication with JWT not required, and seals only successfully bound loopback ingress
- **THEN** the runtime classifies governance as operator-optional and permits unauthenticated requests to proceed with the anonymous principal

#### Scenario: IPv4 loopback without required JWT is eligible
- **WHEN** boot configures `server.host` as `127.0.0.1`, installs authentication with JWT not required, and seals only successfully bound loopback ingress
- **THEN** the runtime classifies governance as operator-optional and permits unauthenticated requests to proceed with the anonymous principal

#### Scenario: Required JWT keeps governance mandatory on loopback
- **WHEN** `server.host` is `localhost` or `127.0.0.1` and `security.jwt_required` is `true`
- **THEN** governance remains mandatory and a request without a credential is rejected under the existing JWT rules

#### Scenario: JWT-disabled non-local listener keeps governance mandatory
- **WHEN** `security.jwt_required` is `false` and `server.host` is any value other than exactly `localhost` or `127.0.0.1`
- **THEN** governance remains mandatory even though the request may proceed with the anonymous principal

#### Scenario: Installed authentication is authoritative
- **WHEN** stored or requested settings say JWT is disabled but the process installed JWT-required authentication at boot
- **THEN** governance remains mandatory for that process and unauthenticated requests follow the installed JWT-required behavior

#### Scenario: Every declared ingress must register before sealing
- **WHEN** a primary HTTP, companion HTTP, or enabled A2A gRPC ingress is declared but does not register a successfully bound address before the governance inventory is sealed
- **THEN** the runtime does not classify governance as operator-optional and does not admit a run through that ingress

#### Scenario: Non-loopback bound ingress overrides an allowed literal
- **WHEN** the configured host literal is `localhost` or `127.0.0.1` and JWT is not required but any registered bound ingress address is non-loopback
- **THEN** governance remains mandatory because loopback-only reachability was not proven

#### Scenario: Restart-pending security settings do not change active posture
- **WHEN** an operator saves a different `server.host` or `security.jwt_required` value while the process is running
- **THEN** governance eligibility continues to use the configured literal and authentication mode installed for the current boot until restart

#### Scenario: Configured IPv6 loopback is not eligible
- **WHEN** `server.host` is configured as `::1` and JWT is not required
- **THEN** governance remains mandatory because the configured literal is not exactly `localhost` or `127.0.0.1`

#### Scenario: Bound IPv6 loopback can prove an allowed configured literal
- **WHEN** the configured literal is exactly `localhost` or `127.0.0.1`, JWT is not required, and a declared ingress successfully binds `::1` as part of an otherwise loopback-only sealed inventory
- **THEN** that bound address satisfies the loopback-address proof and does not by itself make the posture ineligible

#### Scenario: Ingress cannot admit before governance finalization
- **WHEN** a tool-capable ingress has registered during boot but governance initialization, durable preference resolution, or admission-token activation is incomplete
- **THEN** the ingress cannot enter its serving path or admit a run and tool governance gates as enabled
