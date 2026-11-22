## ADDED Requirements

### Requirement: The standalone binary offers the supervised entry point
The `universal-agent-runtime` binary SHALL run the supervised entry point when started with `--supervised`, with the same launch contract as the `uar-sidecar` binary, and SHALL select it before creating any thread. Without `--supervised` the binary SHALL start the standalone server exactly as before.

#### Scenario: Host starts the standalone binary as a supervised child
- **WHEN** a host starts `universal-agent-runtime --supervised` and writes a valid launch token line to its standard input
- **THEN** the process prints exactly one `READY:<port>` line and nothing else on standard output except lifecycle control replies
- **AND** it owns exactly one listening socket, at `127.0.0.1:<port>`
- **AND** it answers a request that carries the token, and answers 401 to a request without it or with another token

#### Scenario: Supervised launch without a token
- **WHEN** a host starts `universal-agent-runtime --supervised` and closes standard input without writing a token line
- **THEN** the process exits with status 2 and prints no `READY` line

#### Scenario: Host closes standard input after readiness
- **WHEN** a supervised `universal-agent-runtime` has printed `READY` and its host closes standard input
- **THEN** the process exits with status 0

#### Scenario: Standalone start is unchanged
- **WHEN** `universal-agent-runtime` is started without `--supervised`
- **THEN** it starts the standalone server with its configured listeners and authentication, and reads nothing from standard input

### Requirement: The standalone binary loads no environment file implicitly in supervised mode
When started with `--supervised`, the `universal-agent-runtime` binary SHALL NOT load a `.env` file from its working directory or any parent directory. It SHALL load an environment file only when `--env-file` or `UAR_ENV_FILE` names one, before it parses its arguments.

#### Scenario: A `.env` file sits in the working directory
- **WHEN** a host starts `universal-agent-runtime --supervised` in a directory whose `.env` file sets `PORT`
- **THEN** the process does not bind that port and `READY:<port>` reports the port selected without it

### Requirement: Port zero requests an operating-system-assigned port
In supervised mode, through either binary, an explicitly requested port of 0 SHALL bind an IPv4 loopback port chosen by the operating system, and the `READY:<port>` line SHALL report that port. An explicit 0 SHALL NOT be treated as the default preferred port. Starting without a requested port, or with a non-zero port, SHALL keep the preferred-port behaviour: begin at 1906 or at the requested port and advance by one while a candidate is in use.

#### Scenario: Host asks for any free port
- **WHEN** a host starts a supervised process with `--port 0`
- **THEN** the process listens on a loopback port assigned by the operating system and reports it in `READY:<port>`

#### Scenario: Two hosts ask for any free port at once
- **WHEN** two supervised processes are started with `--port 0` on the same machine
- **THEN** each binds a different port and neither fails because of the other
