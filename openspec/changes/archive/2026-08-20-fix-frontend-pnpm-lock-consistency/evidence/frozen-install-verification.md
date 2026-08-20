# Frozen nested-workspace installation verification

Date: 2026-08-20
Package manager: pnpm 11.15.0

Metadata command in the active worktree:

```bash
set -euo pipefail
nested_before=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
root_before=$(shasum -a 256 pnpm-lock.yaml | awk '{print $1}')
test "$nested_before" = 43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
test "$root_before" = 645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
pnpm --dir frontend install --lockfile-only --frozen-lockfile --ignore-scripts
nested_after=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
root_after=$(shasum -a 256 pnpm-lock.yaml | awk '{print $1}')
printf 'NESTED_BEFORE=%s\nNESTED_AFTER=%s\nROOT_BEFORE=%s\nROOT_AFTER=%s\n' "$nested_before" "$nested_after" "$root_before" "$root_after"
test "$nested_before" = "$nested_after"
test "$root_before" = "$root_after"
echo FROZEN_METADATA_LOCK_INTEGRITY_PASS
```

Observed exit: `0`

Observed output:

```text
Scope: all 10 workspace projects
✓ Lockfile passes supply-chain policies (verified 5m ago)
Done in 562ms using pnpm v11.15.0
NESTED_BEFORE=43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
NESTED_AFTER=43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
ROOT_BEFORE=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
ROOT_AFTER=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
FROZEN_METADATA_LOCK_INTEGRITY_PASS
```

Clean full-install command. The command creates a new detached external
worktree, initializes the exact gitlink, and explicitly injects the candidate
lock before asserting the empty dependency tree:

```bash
set -euo pipefail
candidate=/Users/gqadonis/.claude/worktrees/uar-1-0-readiness/frontend/pnpm-lock.yaml
cert=$(mktemp -d /Users/gqadonis/.claude/worktrees/frontend-lock-cert.XXXXXX)
rmdir "$cert"
test "$(shasum -a 256 "$candidate" | awk '{print $1}')" = 43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
git worktree add --detach "$cert" 1274039a28f0072bc0e6629a9dab327bdcd9417d
git -C "$cert" submodule update --init --recursive frontend/packages/prometheus-entity-management
cp "$candidate" "$cert/frontend/pnpm-lock.yaml"
cd "$cert"
test "$(git rev-parse HEAD)" = 1274039a28f0072bc0e6629a9dab327bdcd9417d
test "$(git ls-files -s frontend/packages/prometheus-entity-management | awk '{print $2}')" = 0352c83d7b386db56ffea8304ffdf3e2edb00fc8
test "$(git -C frontend/packages/prometheus-entity-management rev-parse HEAD)" = 0352c83d7b386db56ffea8304ffdf3e2edb00fc8
test ! -e frontend/node_modules
nested_before=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
root_before=$(shasum -a 256 pnpm-lock.yaml | awk '{print $1}')
test "$nested_before" = 43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
test "$root_before" = 645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
printf 'CERT_WORKTREE=%s\nSOURCE_COMMIT=%s\nENTITY_MANAGEMENT_PIN=%s\nCLEAN_DEPENDENCY_TREE_PASS\nNESTED_BEFORE=%s\nROOT_BEFORE=%s\n' "$cert" "$(git rev-parse HEAD)" "$(git -C frontend/packages/prometheus-entity-management rev-parse HEAD)" "$nested_before" "$root_before"
pnpm --dir frontend install --frozen-lockfile --ignore-scripts
nested_after=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
root_after=$(shasum -a 256 pnpm-lock.yaml | awk '{print $1}')
printf 'NESTED_AFTER=%s\nROOT_AFTER=%s\n' "$nested_after" "$root_after"
test "$nested_before" = "$nested_after"
test "$root_before" = "$root_after"
echo CLEAN_FROZEN_INSTALL_LOCK_INTEGRITY_PASS
```

Observed exit: `0`

Observed output:

```text
Preparing worktree (detached HEAD 1274039a)
HEAD is now at 1274039a fix(sync): reconcile embedded state after reconnect
Cloning into '/Users/gqadonis/.claude/worktrees/frontend-lock-cert.uBzmrG/frontend/packages/prometheus-entity-management'...
Submodule path 'frontend/packages/prometheus-entity-management': checked out '0352c83d7b386db56ffea8304ffdf3e2edb00fc8'
CERT_WORKTREE=/Users/gqadonis/.claude/worktrees/frontend-lock-cert.uBzmrG
SOURCE_COMMIT=1274039a28f0072bc0e6629a9dab327bdcd9417d
ENTITY_MANAGEMENT_PIN=0352c83d7b386db56ffea8304ffdf3e2edb00fc8
CLEAN_DEPENDENCY_TREE_PASS
NESTED_BEFORE=43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
ROOT_BEFORE=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
Scope: all 10 workspace projects
✓ Lockfile passes supply-chain policies (verified 7m ago)
Lockfile is up to date, resolution step is skipped
Packages: +1191
Progress: resolved 1191, reused 1169, downloaded 0, added 0
Progress: resolved 1191, reused 1171, downloaded 0, added 236
Progress: resolved 1191, reused 1171, downloaded 0, added 485
Progress: resolved 1191, reused 1171, downloaded 0, added 770
Progress: resolved 1191, reused 1171, downloaded 0, added 877
Progress: resolved 1191, reused 1171, downloaded 0, added 1021
Progress: resolved 1191, reused 1171, downloaded 0, added 1182
Progress: resolved 1191, reused 1171, downloaded 0, added 1188
Progress: resolved 1191, reused 1171, downloaded 0, added 1191, done

dependencies:
+ @assistant-ui/react 0.14.26
+ @assistant-ui/react-markdown 0.14.5
+ @base-ui/react 1.7.0
+ @electric-sql/pglite 0.5.4
+ @fontsource-variable/geist 5.3.0
+ @hookform/resolvers 5.7.1
+ @prometheus-ags/prometheus-entity-management 3.0.0-rc.1 <- packages/prometheus-entity-management/packages/entity-graph-react
+ @tanstack/react-table 8.21.3
+ @tanstack/react-virtual 3.14.9
+ @tauri-apps/api 2.11.1
+ @tauri-apps/plugin-shell 2.3.5
+ class-variance-authority 0.7.1
+ clsx 2.1.1
+ date-fns 4.4.0
+ dompurify 3.4.13
+ embla-carousel-react 8.6.0
+ immer 11.1.8
+ input-otp 1.4.2
+ katex 0.18.1
+ lucide-react 0.575.0
+ mermaid 11.16.1
+ next-themes 0.4.6
+ react 19.2.8
+ react-day-picker 10.0.1
+ react-dom 19.2.8
+ react-hook-form 7.85.0
+ react-markdown 10.1.0
+ react-resizable-panels 4.12.2
+ react-router 8.3.0
+ recharts 3.10.1
+ rehype-katex 7.0.1
+ rehype-raw 7.0.0
+ rehype-sanitize 6.0.0
+ remark-breaks 4.0.0
+ remark-gfm 4.0.1
+ remark-math 6.0.0
+ shiki 4.4.2
+ sonner 2.0.7
+ tailwind-merge 2.6.1
+ vaul 1.1.2
+ zod 4.4.3
+ zustand 5.0.13

devDependencies:
+ @chromatic-com/playwright 0.14.11
+ @chromatic-com/storybook 5.2.1
+ @eslint/js 10.0.1
+ @playwright/test 1.62.1
+ @storybook/addon-a11y 10.5.7
+ @storybook/addon-docs 10.5.7
+ @storybook/addon-mcp 0.7.0
+ @storybook/addon-vitest 10.5.7
+ @storybook/react-vite 10.5.7
+ @tailwindcss/vite 4.3.3
+ @testing-library/jest-dom 7.0.0
+ @testing-library/react 16.3.2
+ @testing-library/user-event 14.6.1
+ @types/node 26.1.2
+ @types/react 19.2.15
+ @types/react-dom 19.2.3
+ @vitejs/plugin-react 6.0.5
+ @vitest/browser-playwright 4.1.10
+ @vitest/coverage-v8 4.1.10
+ @vitest/ui 4.1.10
+ axe-core 4.13.0
+ chromatic 18.1.0
+ eslint 10.7.0
+ eslint-plugin-react-hooks 7.1.1
+ eslint-plugin-react-refresh 0.5.3
+ eslint-plugin-storybook 10.5.7
+ eslint-plugin-unicorn 73.0.0
+ globals 17.9.0
+ happy-dom 20.11.2
+ storybook 10.5.7
+ tailwindcss 4.3.3
+ tw-animate-css 1.4.0
+ typescript 5.9.3
+ typescript-eslint 8.66.0
+ vite 8.1.4
+ vitest 4.1.10

Done in 9.7s using pnpm v11.15.0
NESTED_AFTER=43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
ROOT_AFTER=645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
CLEAN_FROZEN_INSTALL_LOCK_INTEGRITY_PASS
```

Limit: lifecycle scripts were disabled. This proves exact nested graph
materialization, supply-chain policy acceptance, and both-lock immutability for
the named source and pin. It does not prove package build scripts or browser
behavior.
