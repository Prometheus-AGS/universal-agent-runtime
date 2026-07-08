ASSESSMENT: uar-post-dependabot-followup-2026-07
Project: universal-agent-runtime
Date: 2026-07-08
Codebase baseline: `cargo check --lib` clean (2 pre-existing warnings, unrelated); all 8 changes from `uar-dependabot-remediation-2026-07` committed and archived; working tree otherwise clean except 3 pre-existing, unrelated uncommitted files carried over from before that phase (`.github/workflows/ci.yml`, `.claude/settings.local.json`, `.kbd-orchestrator/memory-outbox.jsonl` — none touched this phase either).
Cross-tool progress: NONE — no other tool (Roo/Cursor/Codex/Antigravity) has touched this phase; `progress.json` is a fresh skeleton (`changes_total: null`, `changes_completed: 0`).

## 1. Goal-by-Goal Findings

### Goal 1 — Correct `docs/ARCHITECTURE.md`'s D-D bullet

**STUB** (not yet fixed; confirmed exactly where and how).

Current D-D text (`docs/ARCHITECTURE.md:320`):
> *"pinned to specific commit SHAs (or, for `kreuzberg`, tracking `branch = "main"` deliberately)"*

Live `Cargo.toml` state (verified this session):

| Crate | Actual pin | D-D's claim | Match? |
|---|---|---|---|
| `rmcp` | `rev = "26b65b6b88c5552447905923f683b6e4720a5600"` | SHA-pinned | ✅ correct |
| `surreal-memory` | `branch = "main"` | implied SHA-pinned | ❌ **wrong** — this is the floating one |
| `kreuzberg` | `tag = "v4.9.8"` | "tracking `branch = "main"`" | ❌ **wrong** — it's a stable tag, not a floating branch |
| `prometheus_parking_lot` | `rev = "ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0"` | SHA-pinned | ✅ correct |

`docs/DEPENDENCY_MANAGEMENT.md`'s own "Current Pinned Versions" table already
has this right (corrected during `kreuzberg-reachable-vulns` in the prior
phase) — only `ARCHITECTURE.md`'s D-D bullet still has the swap. A
straightforward text fix, no code/behavior change.

### Goal 2 — Decision on `surreal-memory`'s floating `branch = "main"` pin

**Requires human input — not something to decide unilaterally.**

Facts gathered this session:
- `surreal-memory` is the *only* one of D-D's 4 pinned git dependencies on a
  floating branch rather than a fixed commit/tag. Every `cargo update`
  (or even every fresh `cargo check` if the lockfile isn't already
  resolved) can silently pull in whatever `main` currently points to.
- This already caused a real, observed effect: the prior phase's
  `surreal-memory-transitive-vulns` change found `ammonia`/`crossbeam-epoch`
  advisories reachable through `surreal-memory` → `surrealdb-core`, and its
  own `findings.md` explicitly noted the branch pin was investigated (but
  deliberately *not* resynced, since a scoped `cargo update -p <crate>` was
  lower-blast-radius for that specific fix).
- No CI check currently fails or warns if `surreal-memory`'s resolved
  commit drifts between builds — `Cargo.lock` pins the actual commit that
  got resolved at last `cargo update` time, so builds *are* reproducible
  build-to-build, but the *next* `cargo update` (routine or accidental)
  can silently pull in arbitrary upstream changes, unlike `rmcp`/
  `prometheus_parking_lot` which require an explicit `rev =` edit to move.
- `surreal-memory` is `Prometheus-AGS/surreal-memory-server` — an internal,
  unpublished library (per D-D's own stated reason for pinning it at all),
  so unlike `kreuzberg` (third-party, tagged releases exist) there may not
  be a natural "stable release tag" to pin to instead of a raw commit SHA.

This assessment does **not** recommend a specific resolution — that's the
point of Goal 2. Two viable options exist (pin to a specific SHA capturing
today's `main` HEAD, controlled explicitly like the other 3; or explicitly
re-affirm the floating pin with a documented reason, e.g. "we want
`surreal-memory` changes to land automatically since we control both
repos") — this needs the user's judgment before `/kbd-plan` commits to one.

### Goal 3 — Verify `security-audit.yml` fires on GitHub

**MISSING — confirmed via live check, root cause identified.**

```
$ gh run list --workflow=security-audit.yml
HTTP 404: workflow security-audit.yml not found on the default branch
```

Root cause: **none of this repository's local commits have been pushed
to `origin/main`.** `git status -sb` shows `## main...origin/main [ahead
15]` — all 15 commits from `uar-dependabot-remediation-2026-07` (including
the commit that added `.github/workflows/security-audit.yml`) exist only
in the local working copy. GitHub Actions can only discover/run workflow
files that exist on a pushed branch (for `schedule`) or via the API (for
`workflow_dispatch`, which also requires the file to already be on the
default branch) — this workflow literally cannot have fired yet. This
directly confirms the prior phase's own disclosed verification gap: "only
locally simulated, not observed on GitHub" was accurate, and the reason
is now concretely identified (push status), not a mystery.

`gh auth status` confirms an authenticated `gh` session with `repo` scope
is available in this environment, so once pushed, a manual
`gh workflow run security-audit.yml` dispatch is possible to verify without
waiting for the Monday cron.

### Goal 4 — Triage the 9 never-assigned unmaintained/unsound warnings

**PARTIAL** — reachability traced live for all 9 this session; no fixes
applied yet (that's plan/execute work).

| Crate | Reachability (traced via `cargo tree -i --target all --all-features`) | Notes |
|---|---|---|
| `atomic-polyfill` | **Orphaned** — zero reverse dependencies found under any feature/target combination | Same class as `quinn-proto`/`proc-macro-error2`'s sibling case from the prior phase — likely a stale `Cargo.lock` entry, not really in the build |
| `bincode` (2.0.1, unmaintained) | **Reachable, always-compiled** — `burn-core` → `burn` (not feature-gated; `burn` is a normal dependency, only `burn-import` is behind the optional `model-build` feature) | Real, in every build |
| `instant` | **Reachable, always-compiled** — `notify-types` → `notify` (plain non-optional dependency, likely used for skill hot-reload) | Real, in every build |
| `number_prefix` | **Reachable, but deep and feature-adjacent** — `indicatif` → `hf-hub` → `fastembed` → `mempalace-core` → `surreal-memory` | Same floating-branch dependency as Goal 2; worth checking if `fastembed`'s embedding path is actually exercised at runtime |
| `paste` | **Reachable, always-compiled** — via both `kreuzberg`→`biblatex` and `burn`-family crates | Already disclosed in the prior phase (`grcov-toolchain-refresh`'s plan-correction) as unrelated to that change; still unresolved itself |
| `rustls-pemfile` | **Reachable only behind the optional `sandbox-microsandbox` feature** — `microsandbox-network` | Same gating as `hickory-proto` (prior phase, disclosed not-reachable-by-default) |
| `ttf-parser` | **Reachable, always-compiled** — via `kreuzberg` → `lopdf` | Already covered by the prior phase's `kreuzberg-reachable-vulns` disposition (kreuzberg's `lopdf` pin is the same one implicated there) |
| `scc` | **Reachable, dev-only** — via `serial_test` (`[dev-dependencies]`) | Never ships in the release binary |
| `proc-macro-error2` | **Reachable only behind the optional `sandbox-microsandbox` feature** — `microsandbox` → `oci-spec`/`sea-orm-macros` | Already disclosed in the prior phase (`grcov-toolchain-refresh`'s findings) |

Summary: 5 of 9 (`bincode`, `instant`, `number_prefix`, `paste`, `ttf-parser`)
are reachable in a normal default-feature build; 2 of 9
(`rustls-pemfile`, `proc-macro-error2`) are feature-gated behind the
off-by-default `sandbox-microsandbox` feature (matching the
already-established `hickory-proto` disposition pattern); 1 (`scc`) is
dev-only; 1 (`atomic-polyfill`) is orphaned/unreachable. None have a
CVE-style "vulnerability" severity in `cargo audit`'s output — all 9 are
"unmaintained" or "unsound" *warnings*, which is why `cargo audit`'s
default exit-code behavior (and `security-audit.yml`'s design) doesn't
fail the build on them. This assessment does not attempt to fix any of
them — that's plan/execute scope — beyond noting that for `bincode`,
`instant`, `paste`, and `ttf-parser`, checking whether a maintained
alternative exists (mirroring `first-party-direct-dep-hygiene`'s
`serde_yml`→`serde_norway` precedent) is a reasonable next step, while
`atomic-polyfill`/`rustls-pemfile`/`proc-macro-error2`/`scc` likely warrant
"disclose, don't fix" dispositions similar to `quinn-proto`/`hickory-proto`.

## 2. Spec Alignment

No canonical `openspec/specs/*.md` capability governs "architectural
decision documentation accuracy" or "CI workflow verification" directly —
the closest existing capability, `dependency-security-posture` (introduced
last phase), covers dependency *vulnerability* triage, not documentation
accuracy or CI-trigger verification specifically. This phase's changes
will likely need `dependency-security-posture` deltas for goal 4 (fits the
existing pattern) but goals 1–3 are process/documentation work with no
natural existing capability — `/kbd-plan` should decide whether to extend
`dependency-security-posture` further or leave these changes with
"Capabilities: None" (and, per the established practice from last phase,
still add a token delta spec anyway to satisfy `openspec validate`/
`archive`, or explicitly scope them outside OpenSpec).

## 3. Cross-Tool Progress

NONE — this phase has not yet been worked by any tool.

## 4. Build Health

- Build check: **PASS** — `cargo check --lib` clean (2 pre-existing,
  unrelated warnings: a dead-code note on `UarMemoryMcpServer`, matching
  the baseline throughout the prior phase).
- Known violations: NONE newly introduced.
- Test coverage: N/A — this phase's goals are documentation/CI/decision
  work, not new application code; no new test surface to cover.

## 5. Constraint Compliance

No dedicated "Never Do" list or `.kbd-orchestrator/constraints.md` exists
in this project (confirmed absent again this session, same as last
phase) — checked against the Prometheus Base Rules Set in `CLAUDE.md`/
`AGENTS.md` instead. Nothing in this assessment's own investigation
violated any rule (read-only investigation this turn: `cargo tree`,
`cargo check`, `gh run list`, `gh auth status`, `git status` — no files
were modified).

## 6. Sycophancy Self-Check (S-02/S-03/S-06)

- **S-03 (surface at least one concern)**: satisfied — this assessment
  surfaces a concrete, unresolved problem for every one of the 4 goals
  (a factually wrong doc, an undecided risk, a workflow that has never
  run, and 5 reachable-but-untriaged unmaintained crates), not just a
  clean bill of health.
- **S-02 (independently evaluate, don't just agree)**: the D-D
  correction was independently re-verified against live `Cargo.toml`
  state rather than trusting the prior phase's reflection claim at face
  value — confirmed by direct inspection this session, not copied.
  Goal 2 explicitly declines to recommend a specific answer rather than
  assuming the "obvious" fix (SHA-pinning) is correct without the user's
  input on trade-offs specific to an internal, actively-co-developed repo.
- **S-06 (no unearned "clearly"/"obviously")**: avoided in the findings
  above; each claim is backed by a specific command's output (`cargo tree
  -i`, `gh run list`, `git status -sb`) rather than assertion.

Ran `detect_sycophancy` (standard strictness) against this draft:
**score 0.0, 0 patterns classified, correction not mandatory.** Saved to
`.kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/sycophancy/assess-2026-07-08T08-49-48Z.json`.

## Goal Progress Summary

| Goal | Status |
|---|---|
| 1. Correct D-D bullet | STUB — exact fix identified, not yet applied |
| 2. Surreal-memory pin decision | BLOCKED ON HUMAN INPUT — facts gathered, no default assumed |
| 3. Verify security-audit.yml fires | MISSING — root cause confirmed (never pushed); fix path is push + dispatch, not code |
| 4. Triage 9 unmaintained warnings | PARTIAL — reachability traced for all 9; dispositions/fixes not yet applied |
