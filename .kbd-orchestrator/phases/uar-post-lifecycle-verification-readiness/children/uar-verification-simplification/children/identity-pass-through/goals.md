# Goals — uar-post-lifecycle-verification-readiness› identity-pass-through

- Implement the adopted pass-through identity policy (decisions.md 2026-10-28): no authentication or roles inside UAR, a required trust declaration (trusted-front, validation hook, or single-user), (iss, sub) partition key with optional tenant
- Hold presented tokens in memory only; forward only to owner-opted destinations, preferably via a token-exchange hook; never persist; keep the scrubber
- Map existing v1:k rows to the new key; replace removed roles with the admin key, host launch token, or owner hook
- Local test identities via a non-secret subject-selector header on the local identity proxy only, unblocking E09 and gRPC I15-I18
- Assess and Plan only; stop before Spec or implementation
