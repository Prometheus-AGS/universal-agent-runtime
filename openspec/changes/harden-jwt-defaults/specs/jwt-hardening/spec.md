## ADDED Requirements

### Requirement: JWT verification uses a deliberate secret and full claim validation
The runtime SHALL refuse to start when JWT auth is required and the secret is
the built-in fallback, and SHALL support configurable issuer/audience validation.

#### Scenario: Fallback secret rejected
- **WHEN** security.jwt_required is true and jwt_secret equals the built-in fallback
- **THEN** startup fails with a clear configuration error

#### Scenario: Issuer/audience enforced when configured
- **WHEN** security.jwt_issuer or jwt_audience is set and a token lacks the matching claim
- **THEN** the request is rejected with 401
