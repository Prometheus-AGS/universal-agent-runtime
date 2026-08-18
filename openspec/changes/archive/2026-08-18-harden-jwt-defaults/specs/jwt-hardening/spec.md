## ADDED Requirements

### Requirement: JWT verification uses a deliberate secret and full claim validation
The runtime SHALL refuse to start when JWT auth is required and the secret is
the built-in fallback, SHALL support configurable issuer/audience validation,
and SHALL validate an optional not-before claim when configured to do so.

#### Scenario: Fallback secret rejected
- **WHEN** security.jwt_required is true and jwt_secret equals the built-in fallback
- **THEN** startup fails with a clear configuration error

#### Scenario: Issuer/audience enforced when configured
- **WHEN** security.jwt_issuer or jwt_audience is set and a token lacks the matching claim
- **THEN** the request is rejected with 401

#### Scenario: Not-before enforced when enabled
- **WHEN** not-before validation is enabled and a token's `nbf` is beyond the configured clock-skew allowance
- **THEN** the request is rejected with 401
