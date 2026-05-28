# Plan — `ci-frontend-tests`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec
**Decisions locked from assessment §6:**

| Q | Answer |
|---|--------|
| Gate status | Informational first; promote to required after one clean week |
| Root bun scripts | Deprecate — update to pnpm equivalents |
| Frontend lint step | Skip (scope creep) |
| Playwright | Leave in `tests-full.yml`; not added to PR-feedback path |

---

## Ordered change list (3 changes)

| # | Change ID | Title | Effort |
|---|---|---|---|
| 1 | `author-ci-grep-gates-script` | New `scripts/ci-grep-gates.sh` (locally runnable; exit-1 on any gate failure) | XS |
| 2 | `replace-bun-with-pnpm-in-ci-frontend-job` | Modify `.github/workflows/ci.yml::frontend` to use pnpm 10.33, run vitest + build + grep gates. Update root `package.json` scripts to delegate to pnpm. | S |
| 3 | `document-ci-gates` | Append "CI gates" subsection to `docs/migration-stale-data-audit.md`; link to the script | XS |

Each change verifies: `bash scripts/ci-grep-gates.sh` succeeds locally (no false positives on the current tree).

---

## Per-change synopsis

### 1. `author-ci-grep-gates-script`

```bash
#!/usr/bin/env bash
# scripts/ci-grep-gates.sh
#
# Architectural invariants guarding the entity-migration project.
# Run locally before push; CI runs the same script.
#
# See docs/migration-stale-data-audit.md for the contract.

set -uo pipefail
status=0

check_grep_empty() {
  local label="$1"
  local pattern="$2"
  local path="$3"
  if git grep -nE "$pattern" -- "$path" >/dev/null 2>&1; then
    echo "❌ $label"
    git grep -nE "$pattern" -- "$path" | sed 's/^/    /'
    status=1
  else
    echo "✅ $label"
  fi
}

echo "=== Architectural invariants ==="
check_grep_empty "useGraphBridge retired" "useGraphBridge" "frontend/src/"
check_grep_empty "useSettingsStore retired" "useSettingsStore" "frontend/src/"

echo
echo "=== Aesthetic contract (admin surface) ==="
check_grep_empty "no banned fonts in admin CSS/TSX" "(Inter|Roboto|Arial|Space Grotesk)" "frontend/src/admin/"
check_grep_empty "no outline:none in admin" "outline:\\s*none" "frontend/src/admin/"

exit "$status"
```

Make executable: `chmod +x scripts/ci-grep-gates.sh`.

**Verification:** `bash scripts/ci-grep-gates.sh` exits 0 on the current tree.

### 2. `replace-bun-with-pnpm-in-ci-frontend-job`

Modify `.github/workflows/ci.yml::frontend`:

```yaml
frontend:
  name: Frontend Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Setup pnpm
      uses: pnpm/action-setup@v4
      with:
        version: 10.33.0

    - name: Setup Node
      uses: actions/setup-node@v4
      with:
        node-version: 22
        cache: pnpm
        cache-dependency-path: frontend/pnpm-lock.yaml

    - name: Install workspace
      working-directory: frontend
      run: pnpm install --frozen-lockfile

    - name: Type check
      run: pnpm --filter ./frontend typecheck

    - name: Vitest contract tests
      run: pnpm --filter ./frontend test

    - name: Build frontend
      run: pnpm --filter ./frontend build

    - name: CI grep gates
      run: bash scripts/ci-grep-gates.sh
```

Update root `package.json` scripts:

```json
"scripts": {
  "build": "pnpm --filter ./frontend build",
  "dev": "cargo run --bin universal-agent-runtime & pnpm --filter ./frontend dev",
  "test": "pnpm --filter ./frontend test",
  "lint": "pnpm --filter ./frontend lint",
  "format": "pnpm --filter ./frontend exec prettier --write src/",
  "tauri": "tauri",
  "tauri:dev": "tauri dev",
  "tauri:build": "tauri build"
}
```

**Verification:** `act` simulation (if installed) OR push to a feature branch and confirm green PR run.

### 3. `document-ci-gates`

Append to `docs/migration-stale-data-audit.md` just before the "Historical: bridge pattern" appendix:

```md
### CI gates (enforced)

Every PR runs `scripts/ci-grep-gates.sh` plus the standard frontend pipeline:

| Gate | Enforcement |
|------|-------------|
| `pnpm --filter ./frontend typecheck` | TS errors fail the job |
| `pnpm --filter ./frontend test` | Vitest must report ≥ 40/40 |
| `pnpm --filter ./frontend build` | Vite build must succeed |
| `git grep useGraphBridge frontend/` | empty (bridge permanently retired) |
| `git grep useSettingsStore frontend/` | empty (settings store retired) |
| `git grep -E "(Inter\|Roboto\|Arial\|Space Grotesk)" frontend/src/admin/` | empty (banned fonts; aesthetic spec) |
| `git grep "outline:\\s*none" frontend/src/admin/` | empty (a11y contract) |

Wiring: `.github/workflows/ci.yml::frontend`. Local: `bash scripts/ci-grep-gates.sh`.
```

**Verification:** doc renders; link works.

---

## Verification matrix

| Gate | Where | When |
|---|---|---|
| `bash scripts/ci-grep-gates.sh` exits 0 | local | every change |
| `pnpm --filter ./frontend test ≥ 40/40` | local | every change |
| `pnpm --filter ./frontend build` clean | local | every change |
| YAML valid (`yamllint` or visual) | change-2 | change-2 |
| First CI run on a real PR | post-merge of change-2 | informational for 1 week |
| Promote to required | follow-up | after 1 clean week |

---

## OpenSpec scaffolding

Three change directories created under `openspec/changes/`. Proposals + tasks are thin (each change is small enough to be its own commit).

---

## Risk register

| Risk | Mitigation |
|------|------------|
| pnpm install fails on a fresh runner | `frozen-lockfile` + `cache: pnpm` is the standard recipe; verified frontend has `pnpm-lock.yaml` |
| Existing PRs fail the new gates | Informational status for first week (assessment Q1 default) |
| Bun-based root scripts still referenced elsewhere | Audit before deletion — `comprehensive-tests.yml` may invoke them; if so, update there too |
| Banned-fonts grep false-positives | The grep is scoped to `frontend/src/admin/` (not the whole frontend); chat surface keeps its existing fonts |
| `outline: none` is used in some Radix primitives transitively | Scope grep to `frontend/src/admin/` only; the contract is for newly authored admin CSS |

---

## Next step

`/kbd-execute ci-frontend-tests` — proceed straight through; tiny phase.
