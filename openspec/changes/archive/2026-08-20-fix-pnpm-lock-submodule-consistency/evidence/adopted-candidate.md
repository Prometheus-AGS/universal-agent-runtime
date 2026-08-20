# Adopted lock candidate

Date: 2026-08-20

Commands:

```bash
shasum -a 256 pnpm-lock.yaml
git diff --name-only HEAD -- package.json pnpm-workspace.yaml frontend/packages/prometheus-entity-management
git ls-tree HEAD frontend/packages/prometheus-entity-management
git status --short -- .claude/settings.local.json frontend/pnpm-lock.yaml static docs/certifications openspec/changes/screen-by-screen-validation pnpm-lock.yaml
```

Observed output:

```text
645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350  pnpm-lock.yaml
160000 commit 0352c83d7b386db56ffea8304ffdf3e2edb00fc8 frontend/packages/prometheus-entity-management
 M .claude/settings.local.json
 M frontend/pnpm-lock.yaml
 M openspec/changes/screen-by-screen-validation/evidence/validation-matrix.md
 M openspec/changes/screen-by-screen-validation/tasks.md
 M pnpm-lock.yaml
 D static/.vite/manifest.json
 M static/.vite/markdown-engine-graph.json
 M static/index.html
?? docs/certifications/product-screens/
?? openspec/changes/screen-by-screen-validation/evidence/artifact-refiner-validation.md
?? openspec/changes/screen-by-screen-validation/verification.md
```

The adopted candidate was subsequently corrected by restoring HEAD's
`@eslint/config-array`/`minimatch` and `y-webrtc`/`ws` edges while keeping the
new direct `ws` 8.21.1 record. The empty `git diff --name-only` result proves the root manifest, workspace
definition, and entity-management Git link are unchanged by this child. The
listed settings, secondary lock, static, parent change, and old certification
paths are pre-existing exclusions. Only the root lock is adopted from this
inventory.
