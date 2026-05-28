# Design: treeshake-guard-audit

## Decision 1 — Check via grep, not bundle analysis

Full bundle analysis (rollup-visualizer, source-map-explorer) is heavyweight and
requires a rollup/vite build config. The package uses `tsc` for compilation.
Instead, we use a fast grep-based check on the compiled JS output:

```sh
NODE_ENV=production pnpm build
grep -r "notifyDevtools" dist/ && echo "LEAK DETECTED" && exit 1 || exit 0
```

This is reliable because `tsc` does NOT replace `process.env.NODE_ENV` — that is
the bundler's job. So we check the source guard instead:

```sh
grep -n "process.env.NODE_ENV" src/engine.ts | grep "production"
```

Exit non-zero if no match found.

## Decision 2 — prepublishOnly script stays minimal

We do not add a dedicated `check:treeshake` script. Instead, we:
1. Add the guard check inline in `prepublishOnly`
2. Document it in this design

`prepublishOnly` becomes:
```sh
node -e "const s=require('fs').readFileSync('src/engine.ts','utf8'); if(!s.includes(\"process.env.NODE_ENV !== 'production'\")) { console.error('devtools leak detected: NODE_ENV guard missing in engine.ts'); process.exit(1); }"
```

## Decision 3 — No runtime conditional on Explorer exports

Explorer components are React components; they have no side effects at module
load time (no global state mutation, no event listeners until mounted). Tree-shaking
by consumers works naturally. We document this but add no guard.

## Implementation Steps
1. Read `src/engine.ts` and verify the guard
2. Read `package.json` `prepublishOnly` and add the guard check
3. Run `pnpm prepublishOnly` to confirm it passes
4. Write the W7a commit
