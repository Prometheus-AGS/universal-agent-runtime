---
name: entity-prisma-generator
description: >
  Slash command /entity-prisma-generator — add a Prisma generator package (prisma-entity-graph-generator
  pattern) to emit entity graph configs and registration from DMMF; configure prisma schema generator
  block, output path, and post-generate typecheck.
---

# /entity-prisma-generator

## Steps

1. Add devDependency for the generator package (or scaffold a local generator in `packages/`).
2. Edit `schema.prisma`:

```prisma
generator entityGraph {
  provider = "node ./node_modules/prisma-entity-graph-generator/lib/cli.js"
  output   = "../src/generated/entity-graph"
}
```

(Adjust `provider` to match actual package.)

3. Run `pnpm prisma generate`.
4. Import generated `registerAllSchemas()` (or equivalent) from app bootstrap.
5. Commit generated files **if** team policy requires reproducible builds without generate step; otherwise gitignore and run in CI.

## Guardrails

- Generator runs in Node only; never import output from Edge bundles if it pulls `fs`.
- Review diff for dangerous `endpoint` URLs or hardcoded secrets.

## Reference

- `../references/generator-guide.md`
