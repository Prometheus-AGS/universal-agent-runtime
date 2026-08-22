# Specification — `fix-pnpm-lock-submodule-consistency`

- Artifact type: `content`
- Content type: `direct:content`
- Intent: certify that the root pnpm lock describes the exact committed
  workspace and entity-management submodule without unrelated resolution
  movement.
- Required outcome: replayable positive and negative evidence, strict
  OpenSpec validity, exact scope exclusions, and independent history-free
  review before archive.
- Deterministic execution: required for lock hashes, frozen installs, schema
  validation, Git-link replay, and scoped diff checks.
- Unknowns: none affecting implementation. Parent browser certification is a
  separate, deliberately deferred activity.
