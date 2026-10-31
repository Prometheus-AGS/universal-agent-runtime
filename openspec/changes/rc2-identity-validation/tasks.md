## 1. Implementation

- [x] 1.1 Invalid bearer → 401 and JWKS unavailable → 503, independent of `jwt_required` (#422 J07/J09, #392)
- [x] 1.2 Explicit `typ` restricted to `JWT` / `at+jwt` (#422 J08)
- [x] 1.3 Opt-in `security.local_loopback_owner`: loopback no-credential callers on TrustedLocal → `local-host` principal; peer address via ConnectInfo (#392)
- [x] 1.4 J01–J10 named tests (authored) (#422)
- [x] 1.5 Anthropic SSE byte-partition test (authored) (#380)
- [x] 1.6 `docs/security/jwt-validation.md`
- [x] 1.7 BDD `local-identity.feature`

## 2. Verification

- [ ] 2.1 server-full and postgres-backend builds with zero warnings
- [ ] 2.2 `openspec validate rc2-identity-validation`
- [ ] 2.3 Named cargo tests for #422 and #380 (operator-approved exception)
- [ ] 2.4 `local-identity.cucumber.mjs`
