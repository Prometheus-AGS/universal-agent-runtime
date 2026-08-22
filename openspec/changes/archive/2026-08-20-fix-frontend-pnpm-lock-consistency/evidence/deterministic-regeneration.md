# Deterministic nested-lock regeneration

Date: 2026-08-20
Package manager: pnpm 11.15.0
Source: `1274039a28f0072bc0e6629a9dab327bdcd9417d`

Independent worktrees:

- `/Users/gqadonis/.claude/worktrees/frontend-lock-assess-a`
- `/Users/gqadonis/.claude/worktrees/frontend-lock-assess-b`

Command in each worktree after initializing the entity-management submodule:

```bash
set -euo pipefail
before=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
pnpm --dir frontend install --lockfile-only --ignore-scripts
after=$(shasum -a 256 frontend/pnpm-lock.yaml | awk '{print $1}')
test "$before" = a8dd7d07c43aadb2e9809b6c80ae22184d0a41093165cae60083530d7bd846e4
test "$after" = 0a7145d678283ac45de05ffd6773e1a3ba939ac915cd1c2673383c50242f472a
printf 'LOCK_BEFORE=%s\nLOCK_AFTER=%s\n' "$before" "$after"
```

Observed run A tail:

```text
Progress: resolved 1327, reused 0, downloaded 4, added 0, done
[WARN] Issues with peer dependencies found. Run "pnpm peers check" to list them.
Done in 7.5s using pnpm v11.15.0
LOCK_BEFORE=a8dd7d07c43aadb2e9809b6c80ae22184d0a41093165cae60083530d7bd846e4
LOCK_AFTER=0a7145d678283ac45de05ffd6773e1a3ba939ac915cd1c2673383c50242f472a
```

Observed run B tail:

```text
Progress: resolved 1327, reused 0, downloaded 4, added 0, done
[WARN] Issues with peer dependencies found. Run "pnpm peers check" to list them.
Done in 7.4s using pnpm v11.15.0
LOCK_BEFORE=a8dd7d07c43aadb2e9809b6c80ae22184d0a41093165cae60083530d7bd846e4
LOCK_AFTER=0a7145d678283ac45de05ffd6773e1a3ba939ac915cd1c2673383c50242f472a
```

Comparison command and observed output:

```bash
set -euo pipefail
cmp -s \
  /Users/gqadonis/.claude/worktrees/frontend-lock-assess-a/frontend/pnpm-lock.yaml \
  /Users/gqadonis/.claude/worktrees/frontend-lock-assess-b/frontend/pnpm-lock.yaml
echo INDEPENDENT_REGEN_MATCH_PASS
```

```text
INDEPENDENT_REGEN_MATCH_PASS
```

The twice-identical resolver output was not adopted unchanged. Direct HEAD
audit found three common-snapshot body movements unrelated to the pinned
manifest delta. The exact accepted-to-raw patch is retained as
`accepted-to-raw-resolver.patch`; its SHA-256 is
`a82c39b935376bda494f70ab98b81bc2853d2da0725c4c3ebb538b5046c49a36`.

The uncomfortable limitation is that the two scratch worktrees were reused
after their outputs were recorded. The historical raw bytes were reconstructed
by reversing the three recorded restorations; the reconstruction matches the
previously observed raw SHA-256 `0a7145d6…`. The retained patch, final audit
script, and machine-readable classification make the transformation replayable
without claiming that a later registry resolution will reproduce the same raw
semver choices.

Fail-closed transformation and classification command:

```bash
set -euo pipefail
tmp=$(mktemp -d)
mkdir -p "$tmp/frontend"
cp frontend/pnpm-lock.yaml "$tmp/frontend/pnpm-lock.yaml"
patch -s -d "$tmp" -p1 < openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/accepted-to-raw-resolver.patch
raw="$tmp/frontend/pnpm-lock.yaml"
test "$(shasum -a 256 "$raw" | cut -d ' ' -f 1)" = 0a7145d678283ac45de05ffd6773e1a3ba939ac915cd1c2673383c50242f472a
output=openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/lock-delta-classification.json
node openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/audit-lock-delta.mjs --raw "$raw" --output "$output"
test "$(node -e 'const x=require("./'"$output"'"); process.stdout.write(String(x.unclassifiedMutationCount))')" = 0
shasum -a 256 openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/accepted-to-raw-resolver.patch "$output"
echo FAIL_CLOSED_LOCK_DELTA_AUDIT_PASS
```

Observed exit: `0`

```text
RAW_TO_ACCEPTED_TRANSFORMATIONS=3
DIRECT_MANIFEST_EDGES=44
CLASSIFIED_MUTATIONS=693
UNCLASSIFIED_MUTATIONS=0
AUDIT_SHA256=e986720672df17d0c2c826e6b42fa630554d0405cff68b1866a6703818d2ce87
LOCK_DELTA_CLASSIFICATION_PASS
a82c39b935376bda494f70ab98b81bc2853d2da0725c4c3ebb538b5046c49a36  openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/accepted-to-raw-resolver.patch
e986720672df17d0c2c826e6b42fa630554d0405cff68b1866a6703818d2ce87  openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/lock-delta-classification.json
FAIL_CLOSED_LOCK_DELTA_AUDIT_PASS
```

The retained patch applies forward from the accepted candidate to the observed
raw resolver bytes. The audit then installs the three HEAD bodies into the raw
object and asserts semantic identity with the accepted candidate; equivalently,
replaying the patch in reverse transforms raw back to accepted.
