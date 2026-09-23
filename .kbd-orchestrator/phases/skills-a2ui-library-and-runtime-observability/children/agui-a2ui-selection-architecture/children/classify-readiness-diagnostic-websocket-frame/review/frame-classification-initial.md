# Initial fresh-context artifact review

**Disposition:** BLOCK

The reviewer received only the final-artifact packet and no generation history. It reported two priority-one findings:

1. The classification's observer and evidence digests referred to versions that preceded allowlist minimization.
2. The observer checked the 30-second total deadline before the sign-in write and during the later frame read, but did not enforce it while awaiting a stalled sign-in write callback.

The reviewer otherwise accepted the one-attempt and zero-retry structure, header-only evidence boundary, RFC 6455 Ping classification, exact old-collector incompatibility, historical uncertainty, and narrow next-action recommendation. It identified as unverified the continuity comparisons, independently observed operation counts and memory retention, and the prior collector control flow outside the initially supplied line.

## Resolution

- Non-allowlisted authorization and historical-assertion fields were removed from the observer schema and retained evidence.
- Digests in `classification.md` were updated after the final observer and evidence edits.
- `sendSignin` now takes the total deadline, times out and destroys a stalled socket, and clears the masked credential-bearing frame on every completion path.
- The phase-end suite now includes a synthetic stalled-write deadline fixture.
- A second fresh-context artifact-only critic received the full current artifacts and the prior collector implementation for final acceptance.
