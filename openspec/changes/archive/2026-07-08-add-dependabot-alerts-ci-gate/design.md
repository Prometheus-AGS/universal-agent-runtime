## Context

`security-audit.yml` has 4 jobs (`cargo audit`, `npm audit` root, `pnpm
audit` frontend, `npm audit` sdks/typescript), all keyed off ignore-lists
in the workflow that mirror disclosed advisories in
`docs/DEPENDENCY_MANAGEMENT.md`. None of them see GitHub's own
Dependabot/GHSA alert feed. The prior phase found real, disclosed CVEs
(`cmov`, `opentelemetry_sdk`) that only `gh api dependabot/alerts`
surfaced — `cargo audit`'s RustSec advisory database lagged behind.

Constraint discovered during planning (web-verified against GitHub
community discussion #60612 and the REST API docs): the default
`GITHUB_TOKEN` issued to an Actions job **cannot** call
`GET /repos/{owner}/{repo}/dependabot/alerts` under any `permissions:`
grant — this is a hard platform limitation of the Actions App, not a
scope you can widen in the workflow file. A token with `security_events`
scope (classic PAT) or "Dependabot alerts: Read" (fine-grained PAT) is
required. This repo already provisions `secrets.SUBMODULES_TOKEN` for
private-submodule checkout across every workflow — the user chose to
reuse it (2026-07-08, `AskUserQuestion`) rather than provision a new
secret, on the reasoning that a token broad enough for private-repo
submodule access is very likely also scoped for `security_events` (this
session's own interactive `gh` token, scoped only `repo` among others,
successfully read this same endpoint).

## Goals / Non-Goals

**Goals:**
- Add a CI job that fails when GitHub's Dependabot alert feed contains an
  open, undisclosed alert — closing the exact gap that let `cmov` and
  `opentelemetry_sdk` go unnoticed by CI last phase.
- Fail loudly and specifically if the reused token lacks the required
  scope, rather than passing green on a silently-broken check.
- Keep the existing 4 jobs and their ignore-lists untouched.

**Non-Goals:**
- Provisioning a new secret — out of scope per the user's explicit
  choice to reuse `SUBMODULES_TOKEN`.
- Replacing `cargo audit`/`npm audit`/`pnpm audit` — this is a
  complement, not a replacement (RustSec and GHSA data sources overlap
  but neither is a strict superset of the other, per this project's own
  `hickory-proto` disclosure history).
- Building a general-purpose alert-triage UI or dashboard — this is a
  CI gate, not a new product surface.

## Decisions

**1. Reuse `secrets.SUBMODULES_TOKEN` rather than provision a new secret.**
User's explicit choice (`AskUserQuestion`, 2026-07-08) over the
alternatives (new dedicated secret; defer the decision). Alternative
considered: a brand-new `DEPENDABOT_ALERTS_TOKEN` secret would have
cleaner separation of concerns (least-privilege: a token scoped only for
reading alerts, nothing else), but the user weighed the operational cost
of provisioning + rotating a second secret against the low marginal risk
of reusing one that's already broadly scoped and already trusted by
every workflow in this repo.

**2. Fail loudly, not silently, if the token lacks scope.**
The job's first step calls the Dependabot alerts endpoint and checks the
HTTP status before proceeding to the diff logic. A 401/403 produces an
explicit failure message naming the likely cause ("SUBMODULES_TOKEN may
lack security_events scope — see docs/DEPENDENCY_MANAGEMENT.md") rather
than a bare curl/gh error, or worse, an accidental pass. This mirrors the
project's established `--require-baseline` fail-loud precedent from the
eval harness (`eval-harness-hardening`): make a missing precondition an
unambiguous CI failure, never a quiet no-op.

**3. Diff logic: "new/undisclosed" not "any/all".**
The job doesn't fail on every open alert (this repo already has 2 open,
disclosed, not-reachable `hickory-proto` alerts as of 2026-07-08) — it
fails only when an open alert's GHSA ID isn't already listed in
`docs/DEPENDENCY_MANAGEMENT.md`'s disclosed-advisory sections. This keeps
the job's signal-to-noise high: it should only ever go red when there is
something genuinely new to triage, matching `cargo audit`'s existing
`--ignore <RUSTSEC-ID>` pattern in the same workflow.

**4. Implementation approach: inline `gh api` + `jq`/`python3` in the job step, not a separate script file.**
The existing 4 jobs are each a handful of inline steps; a ~20-30 line
diff check doesn't warrant a new `scripts/` file. If this logic grows
(e.g., per-ecosystem disclosure lists), extract to a script in a later
change — not preemptively here (Rule 2: simplicity first).

## Risks / Trade-offs

- **[Risk]** `SUBMODULES_TOKEN` may not actually have `security_events`
  scope, despite the plausibility argument above → **Mitigation**: the
  fail-loud preflight check (Decision 2) surfaces this immediately and
  specifically on the first real CI run (`verify-dependabot-alerts-gate-live`,
  the next change in this phase), rather than failing ambiguously or
  silently passing.
- **[Risk]** The disclosed-advisory list in `docs/DEPENDENCY_MANAGEMENT.md`
  is prose, not a structured list — parsing it programmatically to build
  the "already disclosed" set is fragile → **Mitigation**: maintain a
  small, explicit GHSA-ID allowlist inline in the workflow (same pattern
  as `cargo audit`'s `--ignore RUSTSEC-...` list already in this file),
  cross-referenced to the doc's prose rather than parsed from it. This
  avoids brittle markdown-parsing while keeping the two in sync by
  convention (same discipline already used for the RustSec ignore list).
- **[Risk]** A real new CVE could appear between this change landing and
  `verify-dependabot-alerts-gate-live` running → **Mitigation**: none
  needed — that's exactly the gate working as intended; the next change
  in this phase would then report it rather than assume green.

## Migration Plan

Additive only — new job in an existing workflow file, no removal or
behavior change to the 4 existing jobs. No rollback complexity: reverting
the commit removes the job with no other side effects.

## Open Questions

None blocking — the one open question from planning (token source) was
resolved by the user before this design was written.
