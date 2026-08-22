# Tier 0 and focused unit verification

Date: 2026-08-20

Command:

```bash
set -euo pipefail
nested_before=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
root_before=$(shasum -a 256 pnpm-lock.yaml | awk '{print $1}')
test "$nested_before" = 43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
test "$root_before" = 645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
pnpm --dir frontend install --lockfile-only --frozen-lockfile --ignore-scripts
pnpm typecheck
pnpm lint
pnpm -C frontend test src/entities/sync.test.ts
nested_after=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
root_after=$(shasum -a 256 pnpm-lock.yaml | awk '{print $1}')
printf 'NESTED_BEFORE=%s\nNESTED_AFTER=%s\nROOT_BEFORE=%s\nROOT_AFTER=%s\n' "$nested_before" "$nested_after" "$root_before" "$root_after"
test "$nested_before" = "$nested_after"
test "$root_before" = "$root_after"
echo FAIL_CLOSED_TIER_ZERO_FOCUSED_PASS
```

Observed exit: `0`

Observed output:

```text
Scope: all 10 workspace projects
Already up to date
Done in 256ms using pnpm v11.15.0
$ pnpm -C frontend typecheck
$ tsc -b
$ pnpm -C frontend lint
$ eslint .
$ vitest run src/entities/sync.test.ts

 RUN  v4.1.10 /Users/gqadonis/.claude/worktrees/uar-1-0-readiness/frontend

 Test Files  1 passed (1)
      Tests  4 passed (4)
   Duration  903ms (transform 250ms, setup 448ms, import 25ms, tests 7ms, environment 287ms)

NESTED_BEFORE=43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
NESTED_AFTER=43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
ROOT_BEFORE=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
ROOT_AFTER=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
FAIL_CLOSED_TIER_ZERO_FOCUSED_PASS
```

The unit command also performed a frozen nested-workspace materialization in the
warm active worktree and left the lock unchanged. The displayed result is
limited to the TypeScript/lint profile and the four SSE adapter unit tests; it
does not replace parent browser certification.
