# EXECUTION: uar-uiux-full-migration-2026-08

**Backend:** OpenSpec (`openspec/` present, `project.json.specSystem = "openspec"`)
**Executing tool:** `codex` (registered in `project.json.tools`)
**Handed off:** 2026-08-07 by `claude`
**Plan:** `plan.md` — 21 changes, 6 waves, dependency-ordered
**Source of truth:** `.kbd-orchestrator/` — KBD remains authoritative regardless of
which tool executes.

---

## 0a. KBD is now CANONICAL, not legacy — read this before touching any state file

Migrated 2026-08-07 via `prometheus kbd migrate --apply`. Three consequences that change
how you interact with KBD state:

1. **`.kbd-orchestrator/current-waypoint.json` is now `generatedBy: "kbd-runtime"`.**
   It is a *projection*, rewritten on every runtime write. **Do not hand-edit it** — edits
   are silently discarded. Operator/dispatch context that the canonical schema has no field
   for now lives in `.kbd-orchestrator/operator-context.json` (a sidecar the runtime never
   overwrites).

2. **The waypoint's `implementationCompleted`/`implementationTotal` is a PROJECT-WIDE
   roll-up** across all 47 phases — *not* this phase. It currently reads **27/72** while
   **this phase is 1/21**. For phase-scoped counters always read
   `.kbd-orchestrator/phases/uar-uiux-full-migration-2026-08/progress.json`, where the
   structured `changes[]` array is authoritative (21 entries, C-00 `DONE`).

3. **`prometheus kbd` WRITE commands now WORK** — `decision record`, `stage`, `change`,
   `task`, `completion`, `revise`. Use them; they are the correct way to record progress.
   You will see this on stderr and it is **normal, not an error**:

   ```
   control plane unreachable (…7892…); committing locally via the canonical runtime
   ```

   The daemon serves over a Unix socket and never binds TCP, so the CLI commits through
   the local runtime instead — same journal, same signing, same validation, and the
   command carries a stable `command_id` so a later daemon replay deduplicates rather than
   double-applying. Check for `"committedLocally": true` in the output.

> **Correction (2026-08-07).** An earlier revision of this section said writes *fail* and
> told you to avoid them. That was true when written. Three control-plane defects have
> since been fixed and verified in `prometheus-skill-pack` — see
> `skill-system-findings.md` (F-001, F-002, F-004). Consequences for you:
>
> - **Record completions through `prometheus kbd change transition`**, not only in
>   `progress.json`. Projection-only completions are what caused C-00 to be silently
>   reverted by a later migration.
> - `migrate --apply` now **refuses** when a projection records more completed work than
>   canonical state, naming the phases. If you hit that, reconcile with
>   `prometheus kbd change register` / `transition`, then re-run migrate — do not
>   force past it.
> - A local commit now refreshes `current-waypoint.json` and the phase `progress.json`,
>   so the waypoint tracks canonical state after every write.

Verify current mode any time with:

```bash
prometheus kbd --path . status     # canonical => "Run: … revision N"; legacy => "KBD mode: legacy"
```

---

## 0. Read these first, in this order

1. `.kbd-orchestrator/current-waypoint.json` — exact position (read-only projection; see §0a)
1b. `.kbd-orchestrator/operator-context.json` — dispatch + operator context
2. `.kbd-orchestrator/phases/uar-uiux-full-migration-2026-08/plan.md` — the change list
3. `.kbd-orchestrator/phases/uar-uiux-full-migration-2026-08/handoffs/plan.handoff.json`
4. `docs/knowme-ui-ux-standard.md` — **rank-1 binding design authority** (vendored;
   read its header for the two recorded divergences)
5. `docs/ui/uar-frontend-migration-plan.md` — target architecture and render contract
6. `AGENTS.md` / `CLAUDE.md` — repo rules, build discipline, worktree convention

`assessment.md` and `analysis.md` are background. **`plan.md` is the work list.**

---

## 1. Non-negotiable execution rules

These come from the plan's adversarial review and from `CLAUDE.md`. Violating any of
them produces work that must be redone.

| Rule | Why |
|---|---|
| **C-14a → C-14b → C-14c → C-14d is strictly sequential.** Do not parallelise. | All four touch the same tree; C-14c's deletions are only safe after a/b move everything out. |
| **`rehype-raw` and `rehype-sanitize` land in the SAME change (C-08).** | A-3 trust boundary. Splitting them ships a window where agent output can inject script. |
| **C-03 (gate) before C-05 (codemod).** | Gate-then-purge, or violations re-accumulate. |
| **C-05 covers only the 30 non-admin `hsl(var())` occurrences.** | The other 307 live in admin pages C-14a rewrites — doing them twice is wasted work. |
| **cand-010 / cand-011 / cand-012 are `reference`, NOT adoption decisions.** | C-10, C-11, C-12 each must resolve their library choice before building. |
| **Every change needs ≥1 OpenSpec spec delta.** Capabilities are pre-named in `plan.md` §5.2. | `openspec validate` fails a change with zero deltas. Plan the delta up front. |
| **Never `cargo clean`.** Preserve Cargo caches. | `CLAUDE.md` execution lock. |
| **Worktrees go under `~/.claude/worktrees/`**, via `scripts/worktree-new.sh`. | `CLAUDE.md` worktree convention — never inside the repo tree. |
| **Frontend gates:** `pnpm -C frontend typecheck`, `pnpm -C frontend lint`, and `node scripts/check-frontend-boundaries.mjs` (**from repo root**, not `frontend/`). | The boundary script lives at repo root; running it from `frontend/` throws. |

---

## 2. Current position — C-00 DONE, C-01 is next

**C-00 `archive-completed-ui-changes` is COMPLETE.** All four changes are archived under
`openspec/changes/archive/2026-08-07-*`:

| Change | Tasks |
|---|---|
| `a2ui-uar-renderer-on-webcore` | 49/49 |
| `base-ui-foundation` | 24/24 |
| `a2ui-inspector-lit-svelte-renderers` | 21/21 |
| `a2ui-world-class-theming-a11y-i18n` | 20/20 |

**Do not re-archive them.** The `base-ui-foundation` zero-spec-delta blocker this section
previously described was resolved before archiving —
`specs/frontend-component-primitives/spec.md` was written, plus a
`verify-archive-readiness.sh` and `verification-output.txt`.

> **If you see C-00 as PENDING at 0/21, the state is stale — stop and re-read.** That
> exact reversion happened once (F-001/F-002) and has been fixed; KBD now records 1/21
> with C-00 `DONE`.

**Next work: C-01 `amend-goal4-base-ui-divergence`.** Nothing blocks it. Note it must
create a **new** OpenSpec capability, `frontend-design-authority`, which does not exist in
`openspec/specs/` yet — along with `docs/ui-design-authority.md`.

```bash
/opsx:new amend-goal4-base-ui-divergence
```

---

## 3. Per-change loop

For each change in `plan.md` order:

```
1. /opsx:new <change-id>          (or continue an existing change — see §1.2 of plan.md)
2. Implement against the plan row + its named capability
3. Cheap gates only during implementation:
     pnpm -C frontend typecheck
     pnpm -C frontend lint
     node scripts/check-frontend-boundaries.mjs    # from repo root
4. /opsx:verify <change-id>
5. Record completion in CANONICAL state (not only in progress.json):
     prometheus kbd --path . change transition \
       --command-id "complete-<change-id>:uar-uiux-full-migration-2026-08" \
       --phase uar-uiux-full-migration-2026-08 \
       --id <C-NN> --status complete
   Expect "committedLocally": true. The "control plane unreachable … committing
   locally" line on stderr is normal (§0a item 3). This also refreshes
   current-waypoint.json and progress.json for you.
6. /opsx:archive <change-id>
```

**Why step 5 changed.** It previously said "update `progress.json`." A completion written
*only* to the projection is invisible to canonical state, and the next
`prometheus kbd migrate --apply` rebuilds the projection from canonical — silently
reverting it. That is exactly how C-00 was lost once. Recording through
`change transition` writes both.

**Tier discipline (`CLAUDE.md` A-9):** during implementation run only type/lint checks.
Full `vitest run`, `playwright test`, and `pnpm build` belong at wave boundaries, not
after every edit.

---

## 4. Already done — do NOT redo

| Item | State |
|---|---|
| `chromatic` + `@chromatic-com/playwright` | Installed in `frontend/` (18.1.0 / 0.14.11) |
| 14 Playwright specs | Migrated to `@chromatic-com/playwright`; the 4 `playwright.config.ts` files correctly still import `defineConfig` from `@playwright/test` |
| `CHROMATIC_PROJECT_TOKEN` | In `frontend/.env.local` (gitignored). Token **authenticated** against Chromatic — a dry run reached "Build 1 initialized" |
| `frontend/project-token.md` | **Deleted.** Token preserved in `.env.local`; no `chpt_` string remains on disk elsewhere |
| npm scripts | `chromatic`, `chromatic:playwright`, `test:e2e:chromatic` — all read the token from env, no literal in `package.json` |
| `.github/workflows/storybook-visual-regression.yml` | Already existed and is correct; reads `secrets.CHROMATIC_PROJECT_TOKEN`, skips cleanly when unset |
| `docs/knowme-ui-ux-standard.md` | Vendored with provenance header + two recorded divergences (brand, and Base-UI-over-shadcn per D1) |
| `CHROMATIC_PROJECT_TOKEN` GitHub secret | **Set** on `Prometheus-AGS/universal-agent-runtime` (2026-08-07). The workflow's publish step now fires instead of skipping; its header comment was updated to match |
| **C-00 `archive-completed-ui-changes`** | **COMPLETE** — 4 changes archived, `base-ui-foundation` spec delta written. See §2 |
| Decisions D1, D2, D3 | Recorded in canonical KBD state (`prometheus kbd decision record`) at revisions 3–5, in addition to `decision-log.md` |

**Nothing is open for the operator.** The prior "add the GitHub secret" item is done.

---

## 4a. C-02 — dangling references left by deleting the Tailwind config

C-02 deletes `frontend/tailwind.config.ts` and `frontend/postcss.config.js`. Three files
outside `frontend/src/` reference them and **must be updated in the same change**, or they
become silently-dead config:

| File | Line | Reference | Required action |
|---|---|---|---|
| `.github/workflows/storybook-visual-regression.yml` | 23 | `'frontend/tailwind.config.ts'` in the `pull_request` path filter | Repoint at the new token source (target §4.2: `frontend/src/shared/theme/tokens.css`). A filter naming a deleted file never matches, so **token-only edits would stop triggering visual regression** — a silent CI gap, not a failure. |
| `.github/workflows/storybook-visual-regression.yml` | 31 | same, in the `push` path filter | same |
| `frontend/components.json` | 7 | `"tailwind": { "config": "tailwind.config.ts" }` — the shadcn CLI config | Tailwind 4 is CSS-first and has no JS config. Set to `""` (shadcn's documented v4 value) or drop the key. Left stale, the shadcn CLI errors on any future `add`. |

Verify after C-02 with:

```bash
grep -rn 'tailwind\.config\|postcss\.config' .github/workflows/ frontend/*.json frontend/*.ts 2>/dev/null | grep -v node_modules
```

Expected: no hits. Hits in `crates/prometheus-skill-system/**` are vendored skill
documentation for *other* projects — do not touch them.

> Also note the workflow header was updated 2026-08-07 to record that
> `CHROMATIC_PROJECT_TOKEN` is now configured. The empty-token guard on the publish step
> is deliberate (forks receive no secrets); do not remove it.

---

## 5. Operator decisions binding on execution

| ID | Decision |
|---|---|
| **D1** | Keep **Base UI**; Goal 4's "shadcn" is amended by C-01. This is a recorded **override** of KnowMe standard §6.1/§6.3 — not compliance. |
| **D2** | **Per-surface scoping**, not literal greenfield. Preserve completed A2UI work; rebuild only where there is no foundation. |
| **D3** | KnowMe standard **copied** into `docs/` (done). |

Full rationale in `decision-log.md`.

---

## 6. Known-wrong figures to distrust in older artifacts

`assessment.md` rev 1–2 and `analysis.md` pre-review contained errors that were caught and
corrected. If you see these numbers, use the corrected value:

| Stale | Correct |
|---|---|
| 765 border idioms, "+143 regression" | **630**, stable — zero border-line churn since 2026-08-01 |
| 237 `hsl(var())` | **337** across 11 files (307 of them in admin pages) |
| "24 of 103 routes consumed" | **~40**; `/stream`, `/cancel`, `/tool-approval` already wired |
| "no boundary gate" | One **exists and passes**; coverage is the red gate (19.45% vs 60%) |
| 188 unarchived changes | **187** (the 188th entry is `archive/`) |
| `base-ui-verification` 0/37 | **0/33** — verified twice |

`plan.md` carries the corrected values throughout.

---

## 7. Exit criteria for this phase

- 21/21 changes archived
- `pnpm -C frontend typecheck`, `lint`, `test`, `build` all green
- `node scripts/check-frontend-boundaries.mjs` passes from repo root
- Coverage no worse than the 19.45% baseline (target: 60% threshold in `vitest.config.ts`)
- WCAG 2.2 AA certification + responsive sweep at 320/768/1024/1440 in both themes (C-15)
- `/kbd-reflect uar-uiux-full-migration-2026-08` run and `reflection.md` written

---

## EXECUTION HANDOFF READY
