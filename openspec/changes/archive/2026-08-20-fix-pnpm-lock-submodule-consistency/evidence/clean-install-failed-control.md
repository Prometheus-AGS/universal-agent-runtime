# First clean-install correction control

Date: 2026-08-20

The first adversarial correction restored `y-webrtc` to `ws` 8.21.0 but
incorrectly removed the `ws` 8.21.1 package record. The advanced
`entity-graph-sync` importer pins 8.21.1 directly, so the clean full install
failed closed even though metadata-only validation had passed.

Command:

```bash
pnpm install --dir /Users/gqadonis/.claude/worktrees/lock-consistency-cert \
  --frozen-lockfile --ignore-scripts
```

Observed exit: `1`

Observed output:

```text
Scope: all 21 workspace projects
✓ Lockfile passes supply-chain policies (verified 13s ago)
Lockfile is up to date, resolution step is skipped
[ERR_PNPM_LOCKFILE_MISSING_DEPENDENCY] Broken lockfile: no entry for 'ws@8.21.1' in pnpm-lock.yaml
```

Correction: retain both package records, keep `y-webrtc` on 8.21.0, and keep
the changed importer on its direct 8.21.1 pin. The succeeding clean result is
recorded in `frozen-install-verification.md`.
