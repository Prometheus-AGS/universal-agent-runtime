## ADDED Requirements

### Requirement: A presented bearer never falls back to anonymous
When a request carries a bearer credential, UAR SHALL either verify it or refuse the request. This SHALL hold whether or not `jwt_required` is set. An invalid credential SHALL get 401, and an unavailable configured JWKS source SHALL get 503.

#### Scenario: Invalid bearer with optional JWT
- **WHEN** `jwt_required` is false and a request carries a bearer that fails signature, issuer, audience or expiry checks
- **THEN** UAR responds 401 and no verified or anonymous context is created.

#### Scenario: JWKS unavailable with optional JWT
- **WHEN** `jwt_required` is false, a JWKS URL is configured and cannot be fetched, and a request carries a bearer
- **THEN** UAR responds 503.

### Requirement: Explicit token type is restricted to access tokens
When a JWT header declares `typ`, UAR SHALL accept only `JWT`, `at+jwt` or `application/at+jwt` (case-insensitive) and SHALL reject any other declared type.

#### Scenario: ID token presented as an access token
- **WHEN** an otherwise valid token declares a `typ` other than an access-token type
- **THEN** UAR rejects it with 401.
