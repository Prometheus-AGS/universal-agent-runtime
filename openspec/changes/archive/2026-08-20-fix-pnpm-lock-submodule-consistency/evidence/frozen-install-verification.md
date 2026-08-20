# Frozen installation verification

Date: 2026-08-20
Package manager: pnpm 11.15.0

Metadata command in the active worktree:

```bash
lock_before_metadata=$(shasum -a 256 pnpm-lock.yaml | awk '{print $1}')
printf 'LOCK_BEFORE_METADATA=%s\n' "$lock_before_metadata"
pnpm install --lockfile-only --frozen-lockfile --ignore-scripts
lock_after_metadata=$(shasum -a 256 pnpm-lock.yaml | awk '{print $1}')
printf 'LOCK_AFTER_METADATA=%s\n' "$lock_after_metadata"
test "$lock_before_metadata" = "$lock_after_metadata"
echo LOCK_ONLY_UNCHANGED_PASS
```

Observed exit: `0`

Observed output:

```text
LOCK_BEFORE_METADATA=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
Scope: all 21 workspace projects
✓ Lockfile passes supply-chain policies (verified 1m ago)
Done in 452ms using pnpm v11.15.0
LOCK_AFTER_METADATA=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
LOCK_ONLY_UNCHANGED_PASS
```

Clean full-install command in disposable external worktree
`/Users/gqadonis/.claude/worktrees/lock-consistency-cert`:

```bash
set -euo pipefail
CERT=/Users/gqadonis/.claude/worktrees/lock-consistency-cert
cp /Users/gqadonis/.claude/worktrees/uar-1-0-readiness/pnpm-lock.yaml \
  "$CERT/pnpm-lock.yaml"
test ! -e "$CERT/node_modules"
test ! -e "$CERT/frontend/node_modules"
echo CLEAN_DEPENDENCY_DIRS_PASS
printf 'SOURCE=%s\n' "$(git -C "$CERT" rev-parse HEAD)"
printf 'GITLINK=%s\n' "$(git -C "$CERT" ls-tree HEAD \
  frontend/packages/prometheus-entity-management | awk '{print $3}')"
printf 'SUBMODULE_HEAD=%s\n' "$(git -C \
  "$CERT/frontend/packages/prometheus-entity-management" rev-parse HEAD)"
before=$(shasum -a 256 "$CERT/pnpm-lock.yaml" | awk '{print $1}')
printf 'LOCK_BEFORE=%s\n' "$before"
pnpm install --dir "$CERT" --frozen-lockfile --ignore-scripts
after=$(shasum -a 256 "$CERT/pnpm-lock.yaml" | awk '{print $1}')
printf 'LOCK_AFTER=%s\n' "$after"
test "$before" = "$after"
echo LOCK_UNCHANGED_PASS
```

Observed exit: `0`

Observed output:

```text
CLEAN_DEPENDENCY_DIRS_PASS
SOURCE=fa4ffb96af63131b831c4f30a1b2c16aca599808
GITLINK=0352c83d7b386db56ffea8304ffdf3e2edb00fc8
SUBMODULE_HEAD=0352c83d7b386db56ffea8304ffdf3e2edb00fc8
LOCK_BEFORE=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
Scope: all 21 workspace projects
? Verifying lockfile against supply-chain policies (1482 entries)...
Lockfile is up to date, resolution step is skipped
Packages: +1345
✓ Lockfile passes supply-chain policies (1482 entries in 3.3s)
Progress: resolved 1345, reused 1305, downloaded 0, added 1345, done
Done in 9.9s using pnpm v11.15.0
LOCK_AFTER=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
LOCK_UNCHANGED_PASS
```

Limit: lifecycle scripts were deliberately disabled. This proves clean frozen
graph installation, supply-chain lock-policy acceptance, and lock immutability;
it does not prove package build scripts or product behavior.
