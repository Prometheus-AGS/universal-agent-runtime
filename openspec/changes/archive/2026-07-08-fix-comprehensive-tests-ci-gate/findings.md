# Findings: fix-comprehensive-tests-ci-gate

**Date**: 2026-07-08

## Root cause was deeper than the original assessment found

The assessment (this phase, earlier) diagnosed the gap as "`test-config.yaml` was never created." True, but incomplete: **`test-config.yaml` was also listed in `.gitignore`** (under a "Testing artifacts" section, grouped with genuinely-generated paths like `tests/coverage/`, `*.gcda`, `*.gcno`) — since the initial commit (`3bc4365`, confirmed via `git log -S`). This means even if someone had created the file locally at any point in this project's history, it could never have been committed or reached CI. Both the missing file and the gitignore entry needed fixing together.

## What was built

- Removed `test-config.yaml` from `.gitignore`.
- Created a real `test-config.yaml` with `environments`, coverage-tool selection (renamed to `coverage_tools` with suffixed keys — see bug below), and per-language `threshold` blocks for `rust`/`typescript`/`playwright`.
- Coverage thresholds are honest, low, explicitly-labeled interim placeholders (rust 15/15/10, typescript 6/6/5, playwright 10/10 for line/function/branch), not the abandoned spec's unmeasured 90%+/95% aspirational figures.
- No changes needed to `tools/test-all.sh` or either workflow YAML — `test-all.sh` already correctly separates `CONFIG_FILE` (server config) from `TEST_CONFIG_FILE` (this file) as two distinct env vars; the Codex assessment's config-drift concern was already resolved in the current script, contrary to that 2025-12-31 assessment's description of the codebase at the time.

## Bug caught during implementation (before it ever reached CI)

Task 1.3's dry-run step caught a real bug in the first draft of `test-config.yaml`: `tools/check-coverage.mjs`'s regex parser (`` `${section}:[\s\S]*?threshold:[\s\S]*?${key}:\s*(\d+)` ``) has no section-boundary awareness — it matches the *first* occurrence of a language name anywhere in the file (including inside comments) through to the *next* `threshold:` block anywhere after it. Two rounds of this bug were found and fixed:
1. A `coverage: { rust: {...}, typescript: {...} }` sub-structure (mirroring `quickstart.md`'s illustrative example) caused `typescript`'s threshold lookup to return `rust`'s values, since the parser found `typescript:` there first, then the *next* `threshold:` block in the file (which was `rust`'s). Fixed by renaming to `coverage_tools` with `rust_*`/`typescript_*` prefixed keys.
2. Even after that fix, an explanatory **comment** containing the literal text `` `typescript:` `` (documenting the first bug) reintroduced the exact same failure mode. Fixed by rewording comments to avoid the literal `language-name:` substring appearing anywhere before each section's real threshold block.

Verified via a standalone Node dry-run of the exact parser logic before ever touching CI: `rust: 15 15 10`, `typescript: 6 6 5`, `playwright: 10 10` — each language now resolves its own correct values.

## Live CI verification — first real runs in this project's history

### `comprehensive-tests.yml` (run [28966990812](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/28966990812))

| Job | Result |
|---|---|
| **Pre-flight Checks** | ✅ **PASS (47s)** — first time ever, confirms the fix |
| Security Audit | ❌ FAIL — `Rust security audit` step runs a bare `cargo audit` with no `--ignore` flags, unlike the properly-maintained `security-audit.yml` workflow. It fails on the same 11 vulnerabilities + 8 warnings already triaged and disclosed in `docs/DEPENDENCY_MANAGEMENT.md` — this inline audit step has simply never had that curation applied, since it never ran before today. |
| Code Quality | ❌ FAIL — `bun install --frozen-lockfile` fails: *"lockfile had changes, but lockfile is frozen."* The root-level `bun.lockb` has drifted out of sync with `package.json` — a real, previously-invisible issue. |
| Build Verification, Comprehensive Tests, Docker Integration Tests, Performance Benchmarks | ⏭️ Skipped — cascaded from the two failures above via job dependencies |
| Test Analysis & Reporting, Cleanup & Notifications | ✅ PASS |

### `tests-full.yml` (run [28967666152](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/28967666152))

Ran for real for the first time: checkout, Rust/Node/Bun setup, JS dependency install, Playwright browser install, grcov install all succeeded — then failed 8 minutes in (previously failed in under a minute at Pre-flight). Real failure: **Docker Compose test services never became healthy within the 120s timeout** — `postgres` and `redis` reported healthy, but `surreal` and `unstructured` were still `unhealthy` after 2 minutes. This is a genuine health-check/startup-time tuning gap for `docker-compose.test.yaml` on GitHub-hosted runners, never observed before because the workflow never got this far.

## Scope disposition

Per this change's own `design.md` (Non-Goals: "not every job goes green" is not this change's bar — "progress past Pre-flight" is), **none of the 3 newly-surfaced real failures above were fixed in this change**:
1. `comprehensive-tests.yml`'s inline `cargo audit` needs the same `--ignore` list as `security-audit.yml`, or should be replaced with a call to that workflow's logic to avoid drift between two audit configurations.
2. Root `bun.lockb` needs regenerating against current `package.json` (`bun install` without `--frozen-lockfile`, then commit the updated lockfile).
3. `docker-compose.test.yaml`'s `surreal`/`unstructured` health checks need either a longer timeout or a startup-time investigation specific to GitHub Actions runners.

These are real, concrete, newly-discovered gaps — exactly the kind of finding this fix exists to surface, not evidence the fix failed. Recommended as follow-up changes, not absorbed into this one (avoiding the same unbounded-scope-creep pattern this phase's own assessment flagged in `specs/001-testing-infrastructure`).

## Scope check

Files changed: `.gitignore` (1 line removed), `test-config.yaml` (new). Confirmed via `git status --short` — no accidental changes to `tools/test-all.sh` or either workflow YAML, matching `design.md`'s stated non-goals.
