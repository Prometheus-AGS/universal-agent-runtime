## MODIFIED Requirements

### Requirement: Sidecar listener selection is deterministic and race-free

The supervised sidecar SHALL bind only to IPv4 loopback, SHALL use port 1906 as its default preferred port, and SHALL advance by one while a candidate port is already in use. An explicitly configured port SHALL replace 1906 as the preferred starting port. The sidecar SHALL retain the first successfully bound listener through server startup and SHALL report its actual port through the readiness channel.

#### Scenario: Default port is available

- **WHEN** the sidecar starts without an explicit port and `127.0.0.1:1906` is available
- **THEN** it SHALL listen on port 1906 and report `READY:1906`

#### Scenario: Preferred port is occupied

- **WHEN** the preferred port is already bound
- **THEN** the sidecar SHALL try each following port in order and retain the first listener that binds

#### Scenario: Operator selects another preferred port

- **WHEN** the supervisor starts the sidecar with `--port <port>`
- **THEN** scanning SHALL begin at that port and readiness SHALL report the effective listening port
