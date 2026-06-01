# Reflection — `ci-frontend-tests`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-reflect`)
**Status:** reflect_complete

---

## 1. Goal achievement

**Phase goal:** wire CI gates that guard everything the entity-migration arc produced — vitest contract tests, frontend build, and four architectural greps (bridge retired, settings store retired, banned fonts, `outline: none`).

| # | DoD criterion | Verdict |
|---|---|---|
| G1 | `ci.yml::frontend` job runs pnpm test + build (not bun) | **MET** |
| G2 | Grep gates step runs `scripts/ci-grep-gates.sh` and fails on match | **MET** |
| G3 | Grep gates surface line numbers in CI log | **MET** — `git grep -nE` with `sed 's/^/    /'` indent |
| G4 | pnpm version pinned to 10.33.0 | **MET** |
| G5 | pnpm store cached between runs | **MET** — `actions/setup-node@v4` with `cache: pnpm` |
| G6 | Required for PR merge | **DEFERRED-BY-DESIGN** — informational for one clean week first (locked decision) |
| G7 | Existing bun job retired / repurposed | **MET** — bun job gone; root scripts now pnpm-based |
| G8 | Local invocation identical to CI | **MET** — `pnpm run ci-gates` is the same script CI runs |
| G9 | Existing `tests-full.yml` / `comprehensive-tests.yml` unbroken | **MET** — only `ci.yml::frontend` touched; other workflows independent |
| G10 | Audit doc cites the contract | **MET** — new "CI gates (enforced)" section |

**Goal achievement: 100%.** G6 is intentionally deferred (informational rollout) per the locked assessment decision.

---

## 2. Delivered changes

| # | Change | Status | Files |
|---|---|---|---|
| 1 | `author-ci-grep-gates-script` | DONE | `scripts/ci-grep-gates.sh` (new, executable, ~30 LOC) |
| 2 | `replace-bun-with-pnpm-in-ci-frontend-job` | DONE | `.github/workflows/ci.yml` (frontend job rewritten); `package.json` (root scripts modernised + `ci-gates` alias) |
| 3 | `document-ci-gates` | DONE | `docs/migration-stale-data-audit.md` (new "CI gates (enforced)" section) |

---

## 3. Code shape

| Metric | Value |
|---|---|
| Files touched | 4 |
| New files | 1 (`scripts/ci-grep-gates.sh`) |
| Lines added (across all 4 files) | ~80 |
| Lines removed | ~14 (bun job + bun-based root scripts) |
| Net LOC | +66 |
| New CI gates enforced | 6 (typecheck, vitest, build, 4 greps) |

---

## 4. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with QA (artifact-refiner) | 0/3 (refiner not configured) |
| First-pass pass rate | n/a |

Substitute gates ran at every change boundary:
- `bash scripts/ci-grep-gates.sh` → 0 (✅ × 4)
- `pnpm --filter ./frontend test` → 40/40
- `pnpm --filter ./frontend build` → clean
- `python3 -c "import yaml; yaml.safe_load(...)"` → YAML OK

---

## 5. Technical debt introduced

**Effectively none.** The phase ADDS guard rails and removes drift (bun vs pnpm). One genuinely deferred item:

- **G6 — gate promotion to required.** Gates run informationally for the first week. After one clean merge cycle, branch protection should be updated (manual step) to require the `frontend` job. Tracked in carry-over; not a code change.

Two micro-debts worth noting:

- `tests-full.yml` and `comprehensive-tests.yml` still use bun for some frontend operations. Out of scope here; they're orthogonal to the PR-feedback path that `ci.yml` provides. Cleanup is cheap if/when needed.
- The grep gates are scoped to `frontend/src/admin/` for fonts + `outline: none`. The chat surface intentionally retains its existing fonts (Ember/UAR Dark theme). If future entities migrate the chat to a new aesthetic, the gate scope will need to widen.

---

## 6. Lessons captured for the knowledge base

1. **Word boundaries matter in font-bans.** First attempt of the banned-fonts grep matched `setInterval`, `clearInterval`, `Interrupts` as substrings of `Inter`. The fix is the standard `\bInter\b` form — but only ERE word-boundaries (`\b` in `grep -E`) are reliable; PCRE-only tricks would have hidden the issue. Always run a new grep gate against the *clean* tree before committing, expecting exit 0.

2. **CI grep gates are cheap to author, high in long-term value.** ~30 LOC of shell stands between the project and silent regression on every architectural invariant achieved this session. Every entity migration we did was effectively undoable by a single bridge-import; the gate makes that visible.

3. **Local-equivalent commands matter more than CI internals.** `pnpm run ci-gates` runs *exactly* what CI runs. Contributors don't have to wait for a CI cycle to know if they passed. This is the same lesson as "contract tests as the QA harness" from earlier phases.

4. **Bun → pnpm drift was an unobserved bug.** Local dev used pnpm; CI used bun. They diverged on workspace install (bun doesn't resolve `workspace:*` the way pnpm does), and the vitest harness was untested in CI. The fix isn't just plumbing — it's stopping a class of "works on my machine" issues entirely.

5. **Informational-first rollout is a reasonable migration pattern.** New gates can block in-flight PRs. Running them as advisory for one merge cycle lets contributors observe the contract before it bites. The same approach would work for any future invariant additions.

6. **Document the gate contract in the same doc as the patterns it guards.** The "CI gates (enforced)" section lives in `migration-stale-data-audit.md` alongside the Direct Migration Playbook, SSE-Reconciler Pattern, and Form-Cache Pattern. Future contributors find the contract next to the pattern, not in a separate ops doc.

---

## 7. Cross-phase status

This phase **does not migrate** any entity — it guards what's already migrated. The entity migration scoreboard from `thread-topic-chat-sidebar` stands unchanged: 11/12 entities direct/SSE-reconciler, 1 non-realtime (ApiKey, intentional), bridge pattern permanently retired.

What changed is the **observability/regression posture**:

| Before this phase | After this phase |
|---|---|
| Local pnpm-managed; CI bun-managed → drift | Both pnpm 10.33.0 |
| Vitest had no CI run | 40/40 enforced on every PR |
| Bridge could silently come back | `useGraphBridge` grep gate blocks it |
| Settings store could resurrect | `useSettingsStore` grep gate blocks it |
| Banned fonts could appear in admin | grep gate blocks (admin surface only) |
| `outline: none` could appear | grep gate blocks (admin surface only) |

---

## 8. Recommended next phase

The waypoint's remaining seeds:

1. **Browser smoke walkthrough** (manual, two-tab Chrome) — still owed since `browser-smoke-providers-and-agents` (35% PARTIAL). Now covers 8+ migrated pages. Strong candidate now that all the data plumbing is stable.
2. **`readme-architecture-diagram`** — capture the now-stable architecture (SSE → graph → direct/SSE-reconciler patterns) for new contributors. Cheap; high knowledge-transfer value.
3. **`knowledge-page-aesthetic-pass`** — visual-only follow-up; deferred since `direct-entity-migration-models`.
4. **`runs-checkpoint-persistence-realtime`** — net-new feature (out of the migration arc).

**Recommendation:** `readme-architecture-diagram` next — cheap, captures everything this session built before it ages out of fresh memory. Then browser smoke walkthrough (requires the user's hands; not autonomously runnable).

---

## 9. Progress signal

Reflection complete. The entity migration project's CI guard rails are in place. Four sequential phases this session — `settings-store-retirement`, `add-push-channels-backend`, `thread-topic-chat-sidebar`, `ci-frontend-tests` — all 100% MET. 40/40 contract tests now CI-enforced; 4 architectural greps now CI-enforced; pnpm/bun drift eliminated.
