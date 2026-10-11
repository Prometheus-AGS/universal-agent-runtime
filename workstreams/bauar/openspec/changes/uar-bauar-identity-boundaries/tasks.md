# Tasks

Parent mapping is numeric task 1 through 7 in order; child semantic IDs below are authoritative after driver registration. Author tests/docs with each production slice; run only at combined V1/V2. All remain unchecked until linked exit evidence.

## 1. Identity policy and keys

- [x] 1.1 Bind accepted source, dependency pins, exact ownership and sole in-memory key backend; verify the recorded child UUID and no database migration claim.
- [ ] 1.2 Implement explicit local/remote verifier and trusted tenant/workspace policy after D0; author migration docs and invalid-claim/leeway scenarios, verify source covers every bound admission caller.
- [ ] 1.3 Implement attenuated versioned API-key metadata and direct/exchanged issuer-subject-tenant authority; author reissue/unsupported-mode scenarios and verify all backend/constructor consumers are bound.
- [ ] 1.4 Require owner/tenant or exact scoped-admin key management and bounded exchange TTL; author denial and revocation-window scenarios/docs, verify denied requests cannot disclose or mutate keys.

## 2. Host and signing-key authority

- [ ] 2.1 Replace role-derived host trust with typed authenticated launch/service provenance; author legitimate-host and forged-role scenarios with real destination prerequisites, verify existing grant checks remain.
- [ ] 2.2 Implement single-flight monotonic JWKS refresh with 60-second target, 300-second hard age and five-second limits; author removal/replacement/outage/concurrency scenarios, verify failed refresh never renews age.

## 3. Shared acceptance

- [ ] 3.1 After complete selected production, consume the single real-router AUTH acceptance and applicable build receipts; verify every parent 01 scenario has actual revision-bound evidence and preserve C05/shipping gates.
