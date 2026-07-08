# Findings: push-and-verify-security-audit-workflow

## The workflow genuinely fires now

`gh workflow run security-audit.yml` dispatched a real
`workflow_dispatch` run (28935118101). All 4 jobs passed:

```
✓ npm audit (sdks/typescript) in 26s
✓ cargo audit in 3m41s
✓ npm audit (root) in 27s
✓ pnpm audit (frontend) in 28s
```

This directly closes the verification gap disclosed at the end of the
prior phase ("only locally simulated, not observed on GitHub") — and the
gap in *this* phase's own assessment ("root cause confirmed: never
pushed"). Only informational annotations: Node.js 20 deprecation notices
on the underlying GitHub Actions runner (harmless, not this project's
concern to fix).

## Merge conflict with origin/main (unplanned but necessary)

`git push` was rejected on the first attempt — `origin/main` had
advanced by 8 commits (4 merged Dependabot PRs) while this phase's 3
earlier changes were being worked locally: `vite` 7.3.3→8.1.3 in
`frontend/`, `tiktoken-rs` 0.9.1→0.12.0, `jsonschema` 0.45.0→0.46.10, and
an 11-update "cargo-minor-patch" group.

The `vite` bump directly reopened a decision the prior phase's
`frontend-npm-remediation` had deliberately closed the other way (pinning
to `^7.3.6` specifically to avoid an untested major bump, after `pnpm
audit --fix` tried to jump to `8.1.3` unprompted). Surfaced this to the
user explicitly rather than silently picking a side; they chose to accept
origin's `8.1.3` as the new baseline given it was already merged/approved
via a separate PR.

**Conflict resolution, file by file:**

| File | Resolution |
|---|---|
| `Cargo.toml` | Auto-merged cleanly — my `notify`/`surreal-memory`/`opentelemetry` edits and origin's `tiktoken-rs`/`jsonschema`/`pgvector` bumps are in different regions. |
| `Cargo.lock` | Conflict was a stale `bollard`/`bollard-stubs` entry — `origin/main` predates `direct-network-facing-vulns`'s removal of the unused `testcontainers` dependency. Removed the conflicting block, regenerated via `cargo check`. |
| `frontend/package.json` | Single-line conflict on the `vite` version — took origin's `^8.1.3`. |
| `frontend/pnpm-lock.yaml` | **Not** a naive "take theirs" — origin's lockfile predates this session's `frontend-npm-remediation`-adjacent `undici`/`js-yaml` fixes. Restored *this session's* lockfile (already fixed) and ran `pnpm update vite` on top, which resolved cleanly to `8.1.3` within the newly-widened `package.json` range. A `pnpm audit` re-run after a naive "take origin's lockfile" attempt showed 8 vulnerabilities reappear — caught before committing, corrected via the restore-then-update approach. |

## Two real Vite 7→8 regressions (found via `cargo check`, which builds the frontend)

1. **`manualChunks` object form removed.** Vite 8 (via its new Rolldown
   bundler) removed the object form of `build.rollupOptions.output
   .manualChunks` entirely; the function form is deprecated but still
   works. Converted `frontend/vite.config.ts` accordingly.
2. **A stale, git-tracked duplicate config file was shadowing the fix.**
   `frontend/vite.config.js` — a compiled-looking duplicate, untouched
   since commit `396395c` (the original HTMX→React/Vite migration),
   referenced by no script or tool by name — was still present and took
   precedence over `vite.config.ts` in Vite's config-file resolution.
   Deleted it as the actual root cause once the `.ts` fix alone didn't
   resolve the error.
3. **`lightningcss` (bundled with Vite 8) now strictly rejects Tailwind
   v4's `--spacing()` theme function.** 6 occurrences across 4
   shadcn-ui-derived components (`sidebar.tsx` ×2, `calendar.tsx` ×1,
   `combobox.tsx` ×3, `toggle-group.tsx` ×1) used this v4-only syntax
   even though this project pins Tailwind **v3** (`^3.4.17`) — these
   classes were already producing invalid, no-op CSS (`v3` has no
   `--spacing()` function); the old `lightningcss` silently tolerated the
   invalid syntax, the new one errors. Replaced each with the literal
   `calc()`/`rem` equivalent (Tailwind's spacing scale is `N × 0.25rem`).

## Bonus fixes found via GitHub's Dependabot alerts (not `cargo audit`)

GitHub's push output reported "50 vulnerabilities," far more than local
tooling's count of 11. `gh api repos/.../dependabot/alerts?state=open`
showed only **4** actually open — the "50" was a stale, pre-push-scan
count. Of the 4: 2 were `hickory-proto` GHSA IDs not yet in RustSec's
database (same not-reachable disposition as the 2 already-disclosed
ones — see `docs/DEPENDENCY_MANAGEMENT.md`), and 2 were genuinely new,
reachable, patch-available CVEs `cargo audit` had never surfaced:

- **`cmov` `CVE-2026-50185`**: fixed via `cargo update -p cmov --precise
  0.5.4`.
- **`opentelemetry_sdk` `CVE-2026-48504`**: fixed by bumping the
  `opentelemetry` family in `Cargo.toml` — required also bumping
  `tracing-opentelemetry` 0.32.0→0.33.0 (its version number does not
  track `opentelemetry`'s; `0.32.0` failed to compile against
  `opentelemetry` `0.32.x`'s actual API).

This is a genuinely new lesson for `docs/DEPENDENCY_MANAGEMENT.md`:
`cargo audit`'s RustSec-sourced coverage can lag GitHub's own GHSA
database for the same crate. `security-audit.yml`'s `cargo audit` job
would **not** have caught either of these two CVEs — worth flagging for
a future phase (a `gh api .../dependabot/alerts` check, or GitHub's own
Dependabot alerts feature, is a necessary complement, not a redundant
duplicate, of the new workflow).

## Verification

- `cargo check --lib --tests`: clean.
- `cargo test --lib`: 387/388 pass (1 pre-existing ignore) — unchanged.
- `cargo clippy --lib`: 499 warnings — unchanged.
- `cargo audit`: 11 vulnerabilities (unchanged, all pre-existing
  disclosed); warnings 8→7 (`scc` incidentally resolved by Dependabot's
  merged patch-group bump).
- `pnpm audit` (frontend): 0 vulnerabilities.
- `cargo tree -i cmov` → `0.5.4`; `cargo tree -i opentelemetry_sdk` →
  `0.32.1` — both confirmed at patched versions.
- `gh run list --workflow=security-audit.yml`: real, non-404 run, all 4
  jobs passed.
