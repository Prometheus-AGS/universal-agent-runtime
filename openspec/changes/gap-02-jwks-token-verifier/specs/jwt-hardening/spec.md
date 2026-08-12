## ADDED Requirements

### Requirement: Token verification is reached through a single TokenVerifier abstraction
The runtime SHALL verify presented credentials through one `TokenVerifier`
abstraction accepting a `Presented` value and returning a single `Principal`, so
that additional credential lanes are added without branching every call site.

#### Scenario: JWKS-signed token accepted
- **WHEN** a request presents an RS256 token whose `kid` matches a key in the configured JWKS document
- **THEN** the request is authenticated and the resulting `Principal` carries the token subject

#### Scenario: Unknown kid triggers exactly one refresh
- **WHEN** a token presents a `kid` absent from the cached key set
- **THEN** the runtime refetches the JWKS document once and, if the `kid` is still absent, rejects the request with 401

#### Scenario: Shared-secret lane still works
- **WHEN** no JWKS URL is configured and a valid HS256 token signed with `security.jwt_secret` is presented
- **THEN** the request is authenticated, unchanged from current behaviour

### Requirement: jwt_required is enforced at the point of verification
The runtime SHALL reject unauthenticated and invalid-token requests with 401
whenever `security.jwt_required` is true. The configured value SHALL be the value
used; no call site may substitute a literal.

#### Scenario: Invalid token is rejected rather than downgraded
- **WHEN** `security.jwt_required` is true and a request presents a malformed or bad-signature token
- **THEN** the response is 401 and the request is NOT processed as anonymous

#### Scenario: Absent token is rejected
- **WHEN** `security.jwt_required` is true and a request carries no Authorization header
- **THEN** the response is 401

#### Scenario: Anonymous still permitted when explicitly disabled
- **WHEN** `security.jwt_required` is false and a request carries no credential
- **THEN** the request proceeds with the anonymous principal

### Requirement: Signature validity alone does not establish token validity
When an issuer or audience is configured, the runtime SHALL verify the `iss` and
`aud` claims in addition to the signature, and SHALL reject a token whose
signature verifies but whose claims do not match.

#### Scenario: Correct signature, wrong audience
- **WHEN** a token is validly signed by the configured IdP but its `aud` names a different application
- **THEN** the response is 401

#### Scenario: Correct signature, wrong issuer
- **WHEN** a token is validly signed but its `iss` does not match the configured issuer
- **THEN** the response is 401

### Requirement: Verification fails closed when it cannot be performed
The runtime SHALL refuse to authenticate a request when the verification material
required to judge it is unavailable, rather than falling through to anonymous.

#### Scenario: JWKS endpoint unreachable with no cached keys
- **WHEN** a JWKS URL is configured, no keys are cached, the endpoint is unreachable, and `security.jwt_required` is true
- **THEN** the response is 401 and the failure is logged at error level

#### Scenario: Negative control for the fail-closed path
- **WHEN** the fail-closed test is run against a build where the closed branch is inverted
- **THEN** the test fails, demonstrating the assertion is capable of failing
