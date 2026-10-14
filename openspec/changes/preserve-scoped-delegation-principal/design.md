# Design

VerifiedIdentity is a private, non-deserializable proof bound to subject, tenant and roles. Extend it with an optional privately captured CredentialPrincipal for local-host delegation. Normal JWT/API-key construction leaves this field empty and retains existing behavior. Scoped-grant issuance captures the principal while the authenticated host proof is still present, then removes host_authority. CredentialPrincipal resolution returns the captured identity only after VerifiedIdentity.matches succeeds. A delegated local-host identity remains unclassified, with no API-key or issuer credential privilege.

Reject unverified capture rather than guessing identity from strings. Grant routes remain protected by the existing outer scoped token guard. Do not edit ActorOwner equality or delegated-host-context admission checks.

Complete both production edits, then rebuild server-full payloads and repeat the failed installed-app registration/prompt operation. No unit/per-edit suites. Preserve prior unsuccessful candidate receipts and separate code and qualification.
