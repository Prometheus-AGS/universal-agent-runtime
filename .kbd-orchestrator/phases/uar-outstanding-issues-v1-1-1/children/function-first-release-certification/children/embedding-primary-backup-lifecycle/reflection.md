# Reflection — embedding-primary-backup-lifecycle

Date: 2026-10-19 16:50 UTC. Verdict: **NOT READY** (frozen exit contract).

## Delta between plan and delivery (first, per contract)

The child was chartered to prove a primary+backup embedding lifecycle. It
delivered less than that and the shortfall is the point of this record:

- **Failover was never demonstrated.** What was demonstrated is restart
  resilience of two byte-identical local asset copies with truthful
  provenance. Every transition required a process kill and reboot. No
  in-process swap, no provider switch, no second space. The amended
  contract's own rule — blocker ⇒ NOT READY, never PASS — applies.
- **The query-time space guard remains unexercised.** Guard code exists
  (manager.rs:4658-4704) and the probe positively showed it NOT firing on
  an instance change (correct), but no mismatched-space query refusal was
  observed because no second honored space exists in this deployment.
- **The intended memory embedding configuration was never touched.** The
  subsystem mismatch (OpenAI/1536 named in personal config; Mistral
  credential present without a model/dimension selection — space identity
  UNKNOWN; #420 silent-fallback history) is recorded in INVENTORY and
  RECONCILIATION, not resolved. A credential in the environment is not a
  space; this child refused to treat it as one.

## What was actually proven (evidence-bound)

- Conditions 1, 2, 4, 6: PASS (INVENTORY/RESOLUTION artifacts; fresh v4
  422-rejection probes with unchanged revision and intact retrieval;
  equal-dimension different-model refusal).
- Condition 3: ingest/bind/restart legs PASS (P2 receipts); query-guard
  leg UNVERIFIED.
- Condition 5: restart resilience PROVEN on v5 with truthful provenance
  (boot fallback, backup continuity, recovery — three legs, manifest
  identical `f6dcb84ec0c04198…`); failover NOT MET.
- v5 correction shipped and exercised: eager resolve + per-file digest
  capture + load-time re-verification (TOCTOU closed), provenance in slot
  metadata, truthful status route, boot/swap logs. v5 binary sha256
  `978e2f8d…822a`; v4 `96bf4868…5154` remains candidate of record.

## Process defects caught by gates (not hidden)

- Minimal-features build discarded as evidence (feature-matrix correction).
- Probe script had two latent defects (tee-masked failures; digest capture
  polluted by log lines) — caught by operator review, fixed, self-tested.
- Two fixture defects at runtime (port collision with a foreign process;
  trailing-slash 404) — fail-fast worked, fixed, affected legs rerun.

## Uncomfortable fact

This child ran for hours of wall time and one full release build to
establish, rigorously, that the deployment has exactly one honored
embedding space and therefore cannot demonstrate the lifecycle it was
chartered to demonstrate. The value is the negative result with proof,
plus a truthful provenance surface that did not exist before. The blocker
is external and concrete: no second honored space/provider is reachable
within the no-new-credentials, no-paygo, no-personal-config-change limits.
