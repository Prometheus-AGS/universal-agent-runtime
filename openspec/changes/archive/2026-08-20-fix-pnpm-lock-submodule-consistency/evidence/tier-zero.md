# TypeScript Tier 0

Date: 2026-08-20

Command:

```bash
pnpm typecheck && pnpm lint
```

Observed exit: `0`

Observed output:

```text
Scope: all 21 workspace projects
✓ Lockfile passes supply-chain policies (verified 5m ago)
Lockfile is up to date, resolution step is skipped
Packages: +3
Progress: resolved 3, reused 3, downloaded 0, added 1, done
Done in 1.2s using pnpm v11.15.0
$ pnpm -C frontend typecheck
$ tsc -b
$ pnpm -C frontend lint
$ eslint .
```

Limit: these commands validate the repository TypeScript and lint profile.
They do not replace the parent build or browser certification.
