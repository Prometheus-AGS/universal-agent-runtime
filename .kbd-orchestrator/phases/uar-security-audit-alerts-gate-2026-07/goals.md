# Goals — uar-security-audit-alerts-gate-2026-07

Seeded from `uar-post-dependabot-followup-2026-07`'s `reflection.md`
(2026-07-08) — specifically its "§7 Next Phase Recommendations".

## Why this phase exists

`uar-post-dependabot-followup-2026-07` closed 4/4 changes and 4/4 goals,
but its own execution accidentally proved the case for this phase: while
investigating a confusing push-time vulnerability count, `cargo audit`
was found to systematically under-cover vs. GitHub's GHSA/Dependabot
database. Two real, reachable CVEs (`cmov` / CVE-2026-50185, and
`opentelemetry_sdk` / CVE-2026-48504) were caught only by chance — a
manual `gh api dependabot/alerts` check triggered by suspicion about the
count, not by any part of `security-audit.yml` (the scheduled CI
workflow this project ships specifically to catch exactly this class of
issue). Left as-is, `security-audit.yml` has a structural blind spot:
it can go green while real, disclosed, upstream-fixed CVEs sit unflagged.

## Goals

1. **Add a `gh api dependabot/alerts` check (or equivalent) to
   `security-audit.yml` as a required complement to `cargo audit`.**
   This is the single most consequential follow-up identified across the
   last two phases. The job should fail (or at minimum clearly report)
   when GitHub's Dependabot alert set contains anything `cargo audit` /
   `pnpm audit` didn't already surface, closing the exact gap that let
   `cmov` and `opentelemetry_sdk` go unnoticed by tooling this phase.
2. **Confirm the next real scheduled `security-audit.yml` run (Monday
   06:00 UTC) — or a fresh `workflow_dispatch` — stays green**, including
   the new Dependabot-alerts job, and that the currently-clean state (2
   open alerts, both already-disclosed/not-reachable `hickory-proto`
   findings) is still accurate at that point. Don't just trust the prior
   phase's one-time manual check.
3. **(Medium priority, time-permitting) Migrate `vite.config.ts`'s
   `manualChunks` from the deprecated function form to Rolldown's
   `codeSplitting` API** before Vite removes the deprecated form
   entirely — found necessary mid-phase during the prior phase's Vite
   7→8 merge resolution.
4. **(Medium priority, time-permitting) Grep for other potential
   Tailwind v4-only CSS syntax** (`--spacing(`, other theme functions)
   across less-frequently-built code paths, beyond the 6 call sites
   already fixed in shadcn-ui components — the prior phase's fix was
   scoped to what one specific build happened to emit, not a full sweep.

## Non-goals

- Re-litigating any of the 4 already-completed and archived changes from
  `uar-post-dependabot-followup-2026-07` — those are closed.
- The standing process/architectural question (carried across 3+ phases
  now) of whether OpenSpec needs a lighter-weight schema variant for
  hygiene-only changes — still a human call, not resolved unilaterally
  here.
- Broader dependency modernization beyond the specific items above.

## Product decisions required

None known yet — `/kbd-assess` may surface some (e.g., whether the new
Dependabot-alerts job should be blocking or advisory-only on first
rollout, mirroring how `--require-baseline` was introduced gated behind
an opt-in flag in the eval harness).
