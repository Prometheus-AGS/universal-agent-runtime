# Scope and Git-link integrity

Date: 2026-08-20

Commands:

```bash
git ls-tree HEAD frontend/packages/prometheus-entity-management
git diff --check -- pnpm-lock.yaml \
  openspec/changes/fix-pnpm-lock-submodule-consistency \
  .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-pnpm-lock-submodule-consistency
git status --short -- pnpm-lock.yaml \
  openspec/changes/fix-pnpm-lock-submodule-consistency \
  .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-pnpm-lock-submodule-consistency \
  openspec/specs/frontend-build-tooling/spec.md \
  .refiner/artifacts/fix-pnpm-lock-submodule-consistency \
  .refiner/history/fix-pnpm-lock-submodule-consistency \
  .prometheus
git status --short -- .claude/settings.local.json frontend/pnpm-lock.yaml \
  .refiner/registry.json frontend/packages/prometheus-entity-management package.json pnpm-workspace.yaml \
  src tests static docs/certifications openspec/changes/screen-by-screen-validation
git diff --cached --name-only
```

Observed exit: `0`

Observed output:

```text
160000 commit 0352c83d7b386db56ffea8304ffdf3e2edb00fc8 frontend/packages/prometheus-entity-management

 M pnpm-lock.yaml
?? .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-pnpm-lock-submodule-consistency/
?? .refiner/artifacts/fix-pnpm-lock-submodule-consistency/
?? openspec/changes/fix-pnpm-lock-submodule-consistency/

 M .claude/settings.local.json
 M .refiner/registry.json
 M frontend/pnpm-lock.yaml
 M openspec/changes/screen-by-screen-validation/evidence/validation-matrix.md
 M openspec/changes/screen-by-screen-validation/tasks.md
 D static/.vite/manifest.json
 M static/.vite/markdown-engine-graph.json
 M static/index.html
?? docs/certifications/product-screens/
?? openspec/changes/screen-by-screen-validation/evidence/artifact-refiner-validation.md
?? openspec/changes/screen-by-screen-validation/verification.md
```

`git diff --cached --name-only` and `git diff --check` emitted no output. The
first status block is the complete child surface before canonical spec sync and
append-only learning are produced. The second block is pre-existing
parent/operator state on denied paths and is an explicit commit exclusion,
including `.refiner/registry.json`, which this child did not update. The
entity-management Git link, root manifests, product source, generated output,
and parent certification artifacts are not modified by this child.
