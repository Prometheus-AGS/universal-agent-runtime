# Plan — `fix-pnpm-lock-submodule-consistency`

1. Bind the artifact to the retained root-lock digest and exact submodule pin.
2. Replay the stale-lock failure and corrected frozen-install controls.
3. Verify unrelated dependency resolutions were not moved by regeneration.
4. Verify Tier 0, strict OpenSpec, schema, reference, hash, and scope checks.
5. Submit the frozen artifact alone to history-free critic and judge review.
6. Persist and finalize only if every blocking constraint passes.
