## Context

UAR's release evidence is commit-bound. Changing a candidate's source version
after certification would produce a different commit and invalidate that
evidence. An older `v1.0.0` tag also exists and must not be mistaken for the
certified release.

## Goals / Non-Goals

**Goals:**

- Make every public product surface report `1.0.0` before candidate testing.
- Certify and promote the same source commit and artifact digests.
- Publish verifiable evidence tied to that immutable commit.

**Non-Goals:**

- Rebuild or modify source between candidate certification and GA promotion.
- Expand Stable platform or capability claims during release promotion.

## Decisions

1. The `v1.0.0-rc.1` tag points to source whose manifests and CLI already
   report `1.0.0`. The tag denotes candidate status; it does not change product
   bytes or embedded version strings.
2. GA promotion retags the exact certified commit and reuses the certified
   artifacts. Any source, lockfile, catalog, workflow, or artifact change
   requires a new candidate and complete recertification.
3. The pre-existing `v1.0.0` tag cannot serve as GA evidence because it points
   to an older commit. Replacement requires an explicit, audited tag migration
   immediately before publication.

## Risks / Trade-offs

- Candidate binaries display `1.0.0` rather than a prerelease suffix → the
  candidate tag and draft/prerelease release state prevent GA discovery.
- Replacing the stale GA tag is irreversible for consumers that fetched it →
  record the old target, announce the correction, and verify the new signed tag.
- Any late fix invalidates evidence → automate digest comparison and rerun the
  full candidate workflow.

## Migration Plan

1. Align versions and policies on the intended final source commit.
2. Run the non-GA candidate tag and retain its evidence manifest.
3. Verify external installs and required operational evidence.
4. Replace the stale GA tag only after approval, pointing it at the certified
   commit; publish without rebuilding.
5. Download and verify every public artifact and image digest.

Rollback is withdrawal of the draft/prerelease candidate. A published GA is
never silently replaced; subsequent fixes use a new patch release.

## Open Questions

- Whether prior external operation qualifies for the one-week evidence gate
  must be resolved from auditable records, not inference.
