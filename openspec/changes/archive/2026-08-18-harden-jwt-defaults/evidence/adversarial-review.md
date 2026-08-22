# Independent adversarial review

Date: 2026-08-18
Profile: `server-full`, plus the separately named `uar-jwt-proxy` package. These
results transfer to no other profile.

## History-free critic

Initial verdict: **BLOCK**.

The critic identified a real post-resolution bypass: with the `vault` feature,
the configuration was validated before Vault could replace a secret reference
with the published fallback. It also found an operator-home-contaminated
config-manager fixture, stale public documentation, imprecise `nbf` wording,
and stale verification hashes and counts.

The candidate was corrected to validate the effective secret after optional
Vault resolution on startup, watcherless startup, and reload; use explicit
temporary config files in config-manager tests; document jsonwebtoken's default
60-second clock-skew allowance in both documentation sets; and refresh all
receipts against the corrected source.

Final verdict: **PASS** with no findings.

The critic confirmed the effective fallback check on all three resolution
paths, the shared HS256/JWKS registered-claim policy, the current hashes and
configuration `20/0` result, strict OpenSpec validity, a clean scoped diff, and
the explicit exclusion of `.claude/settings.local.json`, `pnpm-lock.yaml`, and
unrelated KBD churn.

## History-free judge

The judge independently found stale Cargo `filtered out` denominators in the
otherwise passing receipt. The exact focused commands were replayed and the
receipt was corrected to the observed values: fallback 599, issuer/audience
600, `nbf` 600, API exchange 600, security 564, and config-manager 598.

Final verdict: **PASS**.

The judge confirmed pre- and post-resolution fallback rejection, hermetic
config-manager tests, configuration `20/0`, config-manager `3/0`, security
`37/0`, a passing Vault-enabled package check, accurate 60-second leeway docs,
matching hashes and result tails, strict OpenSpec validity, and the reported
three-warning Tier 0 and 572-warning scoped Clippy baselines.

Neither reviewer edited, staged, or committed files.
