# Stale nested-lock negative control

Date: 2026-08-20
Worktree: `/Users/gqadonis/.claude/worktrees/frontend-lock-assess-a`
Source: `1274039a28f0072bc0e6629a9dab327bdcd9417d`
Entity-management pin: `0352c83d7b386db56ffea8304ffdf3e2edb00fc8`

Command:

```bash
test ! -e frontend/node_modules && echo CLEAN_DEPENDENCY_TREE_PASS
before=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
set +e
output=$(pnpm --dir frontend install --frozen-lockfile --ignore-scripts 2>&1)
code=$?
set -e
printf '%s\n' "$output"
after=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
printf 'EXIT_CODE=%s\nLOCK_BEFORE=%s\nLOCK_AFTER=%s\n' "$code" "$before" "$after"
test "$code" -ne 0
test "$before" = "$after"
echo STALE_LOCK_NEGATIVE_CONTROL_PASS
```

Observed exit: `0` for the enclosing control; pnpm exited `1`.

Observed output:

```text
CLEAN_DEPENDENCY_TREE_PASS
Scope: all 10 workspace projects
✓ Lockfile passes supply-chain policies (verified 11d ago)
[ERR_PNPM_OUTDATED_LOCKFILE] Cannot install with "frozen-lockfile" because pnpm-lock.yaml is not up to date with <ROOT>/packages/prometheus-entity-management/package.json

Note that in CI environments this setting is true by default. If you still need to run install in such cases, use "pnpm install --no-frozen-lockfile"

Failure reason:
specifiers in the lockfile don't match specifiers in package.json:
* 17 dependencies were added: @arethetypeswrong/cli@0.18.5, @cucumber/cucumber@13.2.0, @axe-core/playwright@4.12.1, @eslint/js@10.0.1, @playwright/test@1.62.1, @tauri-apps/cli@2.11.4, ajv@8.20.0, axe-core@4.12.1, eslint@10.8.0, eslint-plugin-react-hooks@7.1.1, eslint-plugin-react-refresh@0.5.3, globals@17.8.0, publint@0.3.22, semver@7.8.5, tsx@4.23.1, typescript-eslint@8.65.0, yaml@2.9.0
* 12 dependencies are mismatched:
  - @changesets/cli (lockfile: ^2.31.0, manifest: ^2.31.1)
  - @types/better-sqlite3 (lockfile: ^7.6.13, manifest: ^9.6.0)
  - @types/react (lockfile: >=19, manifest: 19.2.18)
  - @types/react-dom (lockfile: >=19, manifest: 19.2.4)
  - @vitest/browser (lockfile: ^4.1.7, manifest: 4.1.10)
  - better-sqlite3 (lockfile: ^12.11.1, manifest: ^13.0.2)
  - jsdom (lockfile: ^29.1.1, manifest: ^30.0.1)
  - loro-crdt (lockfile: ^1.13.6, manifest: ^1.13.9)
  - tsup (lockfile: >=8, manifest: 8.5.1)
  - turbo (lockfile: ^2.9.18, manifest: 2.10.8)
  - typescript (lockfile: >=6, manifest: 6.0.2)
  - vitest (lockfile: >=4, manifest: 4.1.10)
EXIT_CODE=1
LOCK_BEFORE=a8dd7d07c43aadb2e9809b6c80ae22184d0a41093165cae60083530d7bd846e4
LOCK_AFTER=a8dd7d07c43aadb2e9809b6c80ae22184d0a41093165cae60083530d7bd846e4
STALE_LOCK_NEGATIVE_CONTROL_PASS
```

Limit: this proves only that the nested lock committed at `1274039a` is stale
for the exact pinned frontend workspace. Lifecycle scripts were disabled; no
build or product behavior was asserted.
