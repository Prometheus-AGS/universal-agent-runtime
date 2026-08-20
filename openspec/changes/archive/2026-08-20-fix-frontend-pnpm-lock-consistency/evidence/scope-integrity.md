# Scope and immutable-input integrity

Date: 2026-08-20

Command:

```bash
set -euo pipefail
test "$(shasum -a 256 frontend/package.json | cut -d ' ' -f 1)" = 9d1102638af55c6c681b0f951d69dc9272a7fbe97b5370d4d03bfccb07fef934
test "$(shasum -a 256 frontend/pnpm-workspace.yaml | cut -d ' ' -f 1)" = e1487b173b603d242ea94b2b64ed6f9406b24b8bdc681763390a7d2d0193899f
test "$(shasum -a 256 pnpm-lock.yaml | cut -d ' ' -f 1)" = 645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350
test "$(shasum -a 256 frontend/pnpm-lock.yaml | cut -d ' ' -f 1)" = 43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593
test "$(git ls-files -s frontend/packages/prometheus-entity-management | awk '{print $2}')" = 0352c83d7b386db56ffea8304ffdf3e2edb00fc8
git diff --quiet -- frontend/package.json frontend/pnpm-workspace.yaml pnpm-lock.yaml frontend/packages/prometheus-entity-management
git diff --check -- frontend/pnpm-lock.yaml openspec/specs/frontend-build-tooling/spec.md
node <<'NODE'
const { execFileSync } = require('node:child_process');
const { readFileSync } = require('node:fs');
const roots = [
  'openspec/changes/fix-frontend-pnpm-lock-consistency',
  '.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-frontend-pnpm-lock-consistency',
  '.refiner/artifacts/fix-frontend-pnpm-lock-consistency',
  '.refiner/history/fix-frontend-pnpm-lock-consistency',
];
const output = execFileSync('git', ['ls-files', '--others', '--exclude-standard', '-z', '--', ...roots]);
const files = output.toString('utf8').split('\0').filter(Boolean);
for (const file of files) {
  const buffer = readFileSync(file);
  if (buffer.includes(0)) continue;
  const text = buffer.toString('utf8');
  if (!file.endsWith('.patch') && /(^|\n)[^\n]*[ \t]+(?=\n|$)/.test(text)) throw new Error(`trailing whitespace: ${file}`);
  if (/(^|\n)(<<<<<<<|=======|>>>>>>>)( |$)/.test(text)) throw new Error(`conflict marker: ${file}`);
}
console.log(`UNTRACKED_TEXT_FILES_INSPECTED=${files.length}`);
console.log('UNTRACKED_DIFF_CHECK_PASS');
NODE
git diff --cached --quiet
echo MANIFEST_ROOT_LOCK_GITLINK_UNCHANGED_PASS
echo FAIL_CLOSED_SCOPE_INTEGRITY_PASS
git status --short -- frontend/pnpm-lock.yaml \
  openspec/changes/fix-frontend-pnpm-lock-consistency \
  .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-frontend-pnpm-lock-consistency \
  openspec/specs/frontend-build-tooling/spec.md \
  .refiner/artifacts/fix-frontend-pnpm-lock-consistency \
  .refiner/history/fix-frontend-pnpm-lock-consistency .prometheus
git status --short -- .claude/settings.local.json pnpm-lock.yaml \
  frontend/package.json frontend/pnpm-workspace.yaml frontend/src frontend/packages \
  src tests static docs/certifications openspec/changes/screen-by-screen-validation \
  .refiner/registry.json
```

Observed exit: `0`

```text
UNTRACKED_TEXT_FILES_INSPECTED=37
UNTRACKED_DIFF_CHECK_PASS
MANIFEST_ROOT_LOCK_GITLINK_UNCHANGED_PASS
FAIL_CLOSED_SCOPE_INTEGRITY_PASS
 M frontend/pnpm-lock.yaml
?? .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-frontend-pnpm-lock-consistency/
?? .refiner/artifacts/fix-frontend-pnpm-lock-consistency/
?? openspec/changes/fix-frontend-pnpm-lock-consistency/
 M .claude/settings.local.json
 M .refiner/registry.json
 M openspec/changes/screen-by-screen-validation/evidence/validation-matrix.md
 D static/.vite/manifest.json
 M static/.vite/markdown-engine-graph.json
 M static/index.html
?? docs/certifications/product-screens/
?? openspec/changes/screen-by-screen-validation/evidence/artifact-refiner-validation.md
?? openspec/changes/screen-by-screen-validation/verification.md
```

The untracked validator reads every child OpenSpec, KBD, and refiner text file;
it is not a `git diff --check` call that silently ignores untracked artifacts.
Unified patch context lines are exempt from trailing-whitespace rejection
because their leading diff marker is part of the patch format. Conflict markers
remain forbidden in every text artifact.

The denied-path status is pre-existing parent/operator state and is an explicit
commit exclusion. The child does not claim that the whole worktree is clean.
