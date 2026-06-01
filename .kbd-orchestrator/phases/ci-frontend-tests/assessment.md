# Assessment — `ci-frontend-tests`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `thread-topic-chat-sidebar` (100%) — entity migration project structurally complete

---

## 1. Phase goal

Wire CI gates that guard everything the entity-migration arc just produced. After 12 phases of frontend work — 10 entities migrated to direct/SSE-reconciler patterns, terminal aesthetic adopted across 4+ pages, 40/40 contract tests, bridge pattern retired — the gap is observability: a regression on `main` would land silently. CI needs to enforce the invariants this project lives on.

Six gates, in priority order:

1. `pnpm --filter ./frontend test` returns ≥ 40/40 vitest
2. `pnpm --filter ./frontend build` clean
3. `git grep useGraphBridge frontend/` empty (bridge pattern retired)
4. `git grep useSettingsStore frontend/` empty (settings store retired)
5. `git grep -rnE "(Inter|Roboto|Arial|Space Grotesk|system-ui)" frontend/src/admin/` empty in newly authored CSS (banned-fonts contract from `docs/admin-aesthetic-spec.md`)
6. `git grep "outline:\s*none" frontend/src/admin/` empty (a11y contract)

End-state: every PR runs these gates; main branch never regresses on the architectural invariants of the entity-migration project.

---

## 2. Current state inventory

### 2.1 Existing CI workflows

```
.github/workflows/
├── ci.yml                       (push/PR — rust check+test, frontend bun check+build)
├── tests-full.yml               (playwright via npx)
├── tests-quick.yml              (TBC)
├── quick-tests.yml              (TBC — duplicate?)
├── comprehensive-tests.yml      (frontend build + playwright chromium)
├── deploy.yml                   (release)
├── release.yml                  (release tag)
├── image-uar-toolchain.yml      (docker image)
└── template-cleanup.yml         (template scaffolding)
```

### 2.2 Current `ci.yml::frontend` job (lines 108–126)

```yaml
frontend:
  name: Frontend Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Setup Bun
      uses: oven-sh/setup-bun@v2
      with: { bun-version: latest }
    - name: Install dependencies
      run: bun install
    - name: Type check
      run: bun run check
    - name: Build assets
      run: bun run build
```

**Gaps:**
- Uses **bun** but the frontend migrated to **pnpm workspaces** (`frontend/pnpm-workspace.yaml`, `packageManager: pnpm@10.33.0`). The bun job runs the root `package.json`'s `build` script which proxies `cd frontend && bun run build` — but `bun install` at root won't install the pnpm workspace's `@prometheus-ags/prometheus-entity-management` workspace package correctly. The local dev flow uses pnpm. **Drift between CI and local.**
- **Vitest is not run.** `bun run check` is `tsc -b`; no vitest invocation. The 40 contract tests have no CI guard.
- **No grep gates** for the architectural invariants.
- **No playwright invocation** in `ci.yml` itself (it's in `tests-full.yml` and `comprehensive-tests.yml`, but those may not be triggered on every PR).

### 2.3 Frontend `package.json` scripts (already in tree)

```json
"scripts": {
  "dev": "vite",
  "build": "vite build",
  "lint": "eslint .",
  "typecheck": "tsc -b",
  "test": "vitest run",
  "test:watch": "vitest",
  "test:ui": "vitest --ui",
  "test:e2e": "playwright test",
  "test:e2e:ui": "playwright test --ui"
}
```

All scripts the CI needs already exist. The work is workflow plumbing.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|---|---|
| G1 | `ci.yml::frontend` job runs `pnpm --filter ./frontend test` and `pnpm --filter ./frontend build` (instead of bun) | workflow diff |
| G2 | A new `ci.yml::frontend-invariants` job (or step) runs the 4 grep gates and fails if any fires | workflow diff + dry-run test |
| G3 | All grep gates use plain shell `grep -rnE` so failures show line numbers in the CI log | workflow diff |
| G4 | The job uses the same pnpm version as `frontend/package.json::packageManager` (10.33.0) | workflow diff |
| G5 | Cache pnpm store between runs (perf) | workflow diff |
| G6 | The job is required for PR merge (settings change OR documented as required) | repo settings or doc note |
| G7 | Existing `bun` job either retired or kept only for bun-specific tasks (e.g. tauri build) | workflow diff |
| G8 | Local invocation is identical: a contributor can run `pnpm --filter ./frontend test && pnpm --filter ./frontend build && bash scripts/ci-grep-gates.sh` and see exactly what CI sees | new shell script |
| G9 | Existing `tests-full.yml` / `comprehensive-tests.yml` aren't broken by the change | their diffs |
| G10 | Documentation in `docs/migration-stale-data-audit.md` adds a "CI gates" subsection citing the contract | file diff |

---

## 4. Gap analysis

### 4.1 Bun → pnpm migration in CI

The root `package.json` still has bun-based scripts. The frontend is now a pnpm workspace. There are two valid resolutions:

- **A. Replace the bun job with a pnpm job in `ci.yml`.** Cleanest. Root scripts can be left intact for any non-CI uses; CI invokes pnpm directly.
- **B. Update root scripts to call pnpm, then keep `ci.yml::frontend` semantically the same.** More invasive but keeps the dev `npm run` muscle memory.

**Recommendation:** A. Smaller surface, more honest about reality.

### 4.2 Grep gates as a CI step

The four invariant greps need to be expressed in a way that:
1. Fails the job on match (grep returns 0 = match)
2. Surfaces all matches in the log (don't short-circuit)
3. Allows allowlisted exceptions (the memory-page bulk-delete uses `useGraphStore.getState()` intentionally — that's a different grep, not in the gate list, but the pattern matters)

A standalone `scripts/ci-grep-gates.sh` is the right shape:

```sh
#!/usr/bin/env bash
set -uo pipefail
status=0
check() {
  local label="$1"; local pattern="$2"; local path="$3"
  if git grep -nE "$pattern" -- "$path" >/dev/null 2>&1; then
    echo "❌ $label — matches found:"
    git grep -nE "$pattern" -- "$path"
    status=1
  else
    echo "✅ $label"
  fi
}
check "useGraphBridge retired"      "useGraphBridge"           "frontend/src/"
check "useSettingsStore retired"    "useSettingsStore"          "frontend/src/"
check "no banned admin fonts"       "(Inter|Roboto|Arial|Space Grotesk)" "frontend/src/admin/"
check "no outline:none"             "outline:\s*none"           "frontend/src/admin/"
exit "$status"
```

### 4.3 Pnpm caching

`pnpm/action-setup` + `actions/setup-node` with `cache: pnpm` is the standard recipe. ~30 s saved per run.

### 4.4 Playwright

Out of scope for this phase. `tests-full.yml` already runs playwright; that's adequate. Adding playwright to `ci.yml::frontend` would slow PR feedback. **Decision:** leave playwright in its existing workflow.

### 4.5 Risks

- **Existing PRs.** Adding gates as required will block any in-flight PRs that drift. Recommendation: ship the new job as **non-blocking informational** for one merge cycle, then promote to required after a clean week.
- **Workspace install on a fresh runner.** `pnpm install` for the workspace pulls `@prometheus-ags/prometheus-entity-management` as `workspace:*`. CI must run `pnpm install` from the repo root (not the frontend subdir) so the workspace resolves. Verified by reading `frontend/pnpm-workspace.yaml`.
- **Bun job removal.** If any other workflow depends on `bun run build` in `ci.yml`, removing it could break downstream jobs. Audit `comprehensive-tests.yml` and `deploy.yml` before deletion.

---

## 5. Sequencing recommendation

3 changes, ordered:

1. **`author-ci-grep-gates-script`** — new `scripts/ci-grep-gates.sh` executable shell script. Locally runnable. Pure utility; no workflow changes yet.
2. **`replace-bun-with-pnpm-in-ci-frontend-job`** — modify `.github/workflows/ci.yml::frontend` to:
   - swap bun setup for `pnpm/action-setup@v4` + `actions/setup-node@v4` with `cache: pnpm`
   - run `pnpm install --frozen-lockfile` from repo root
   - run `pnpm --filter ./frontend test`, `pnpm --filter ./frontend typecheck`, `pnpm --filter ./frontend build`
   - run `bash scripts/ci-grep-gates.sh`
3. **`document-ci-gates`** — append a "CI gates" subsection to `docs/migration-stale-data-audit.md` describing each gate, link to the script, note the contract is enforced.

Each change runs locally (script can be invoked; workflow diff can be linted by `yamllint` if available).

---

## 6. Open questions

1. **Required vs informational status.** Promote to required immediately, or run informational for one merge cycle first? Default: **informational first** for one week, then promote.
2. **Retire root `bun` scripts?** They're now misleading (root `npm test` runs `bun test` which misses the new pnpm vitest tests entirely). Default: **deprecate** them in this phase by updating to call `pnpm --filter ./frontend ...`.
3. **Add a clippy/cargo-test step to the frontend job?** No — those have their own job; this phase is frontend-only.
4. **Lint step?** `pnpm --filter ./frontend lint` exists but the existing codebase may not currently pass clean. Default: skip lint in this phase to avoid scope creep; address in a follow-up.

---

## 7. Progress signal

Assessment complete. Defaults are sensible. Next: `/kbd-plan ci-frontend-tests`.
