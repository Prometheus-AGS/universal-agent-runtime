# Goals — uar-final-production-hardening-2026-07

## Binary success criterion

Deliver a production-ready `server-full` UAR sidecar for BossFang and complete all 24 phase changes. Tests, CI, and artifacts are completion evidence; they are not substitutes for implementation or release execution.

## Current truth

- Changes 1–19: complete.
- Changes 20–24: implementation/integration complete; formal completion requires immutable evidence, time-bound validation, and authorized release effects.
- Windows: Experimental and nonblocking.
- Embedded/library consumption: not a current release requirement; BossFang uses the sidecar.

## Remaining outcomes

1. Certify the supported Linux/macOS platform workflow from an immutable candidate.
2. Retain resilience evidence, including non-root operation and the required soak duration.
3. Produce and independently verify signed checksums, SBOMs, provenance, images, and the release manifest.
4. Certify clean installs and three external installations for the required operating period.
5. Promote the unchanged certified source to GA and verify public artifacts.

## Non-goals

- Fixing Experimental Windows during this round.
- Reopening completed changes 1–19 without a demonstrated regression.
- Repeated broad validation of known-incomplete work.
- Treating green workflow counts as the product objective.
