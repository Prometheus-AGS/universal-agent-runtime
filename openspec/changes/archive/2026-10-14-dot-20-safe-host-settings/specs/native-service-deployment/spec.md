# Spec Delta

## ADDED Requirements

### Requirement: Shipped host profiles deny unlisted browser origins

UAR SHALL apply an explicit browser-origin allowlist to its server routes. The shipped default list SHALL be empty and SHALL emit no allow-origin header for an unlisted origin. A deployment MAY deliberately configure allowed origins without turning an empty or malformed list into an allow-any policy.

#### Scenario: Empty default

- **WHEN** a browser-origin request reaches a shipped UAR profile with no configured allowed origins
- **THEN** the response has no allow-origin permission for that origin

#### Scenario: Explicit origin

- **WHEN** an operator configures one allowed origin and requests arrive from that origin and another origin
- **THEN** only the configured origin receives the corresponding allow-origin permission

### Requirement: Installable profiles declare safe network and authentication posture

The native local default SHALL bind every UAR listener to loopback and require JWT authentication for private routes. A shared-host profile MAY bind beyond loopback only through an explicit documented deployment override while retaining authentication and owner isolation. An anonymous public agent-card route SHALL NOT relax private route authentication.

#### Scenario: Native default starts

- **WHEN** an installable local profile starts without operator overrides
- **THEN** HTTP and A2A listeners bind only to loopback and private routes require JWT

#### Scenario: Shared deployment override

- **WHEN** the shared-host profile binds a non-loopback listener for its deployment
- **THEN** the bind is explicit in the deployed profile and private routes remain authenticated
