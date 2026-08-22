# Candidate comparison

Date: 2026-08-20

## Clean regeneration replay

The committed stale lock was restored in the disposable worktree before each
run. Both runs used the exact source and submodule Git links from `fa4ffb96`.

Command:

```bash
git restore --worktree pnpm-lock.yaml
pnpm install --lockfile-only --no-frozen-lockfile --ignore-scripts
shasum -a 256 pnpm-lock.yaml
```

Observed run 1 tail:

```text
Done in 6.8s using pnpm v11.15.0
8706080edcdbdd35c39f867a5af648aacb0ce484348e847be37a681f1b205af3  pnpm-lock.yaml
```

Observed run 2 tail:

```text
Done in 7.7s using pnpm v11.15.0
8706080edcdbdd35c39f867a5af648aacb0ce484348e847be37a681f1b205af3  pnpm-lock.yaml
```

## Retained candidate comparison

Command:

```bash
shasum -a 256 pnpm-lock.yaml /Users/gqadonis/.claude/worktrees/screen-cert-fa4ffb96/pnpm-lock.yaml
diff -u pnpm-lock.yaml /Users/gqadonis/.claude/worktrees/screen-cert-fa4ffb96/pnpm-lock.yaml
```

Observed digests:

```text
645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350  pnpm-lock.yaml
8706080edcdbdd35c39f867a5af648aacb0ce484348e847be37a681f1b205af3  /Users/gqadonis/.claude/worktrees/screen-cert-fa4ffb96/pnpm-lock.yaml
```

Observed diff:

```diff
       lucide-react:
         specifier: ^1.28.0
-        version: 1.32.0(react@19.2.8)
+        version: 1.33.0(react@19.2.8)

-  lucide-react@1.32.0:
+  lucide-react@1.33.0:

       '@eslint/object-schema': 3.0.5
       debug: 4.4.3(supports-color@10.2.2)
-      minimatch: 10.2.6
+      minimatch: 10.2.5

-  ws@8.21.0:
-    resolution: {integrity: sha512-Vsp28b7DRcimFQvrqu2Wek3z1iYxDCWqHYB8Qsnk/S4RfaCQzPGPyBNuVjJV3cd6UiKtUtp6sNM77gWvzcCH+g==}

-  lucide-react@1.32.0(react@19.2.8):
+  lucide-react@1.33.0(react@19.2.8):

-  ws@8.21.0:
-    optional: true

     optionalDependencies:
-      ws: 8.21.0
+      ws: 8.21.1
```

Conclusion limited to this comparison: the corrected candidate retains the
exercised `lucide-react` 1.32.0 resolution, the new causal
supports-color-10/minimatch 10.2.6 edge, and HEAD's `y-webrtc`/`ws` 8.21.0
edge. It separately retains `ws` 8.21.1 because the advanced
`entity-graph-sync` manifest pins that version directly. Direct HEAD audit is
recorded in `head-candidate-delta-audit.md`. This is not a general dependency
security or freshness verdict.
