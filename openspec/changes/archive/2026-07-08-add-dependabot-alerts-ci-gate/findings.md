# Findings: add-dependabot-alerts-ci-gate

**Date**: 2026-07-08

## What was built

A new `dependabot-alerts-gate` job in `.github/workflows/security-audit.yml`
that calls `gh api repos/<org>/<repo>/dependabot/alerts` and fails when an
**open** alert's GHSA ID isn't in an inline `DISCLOSED_GHSA_IDS` allowlist
(currently the 2 already-disclosed `hickory-proto` IDs). It fails loudly
with a specific diagnostic message if the API call itself errors (likely
insufficient token scope), rather than passing silently.

## Token source decision

Resolved via `AskUserQuestion` (2026-07-08): the user chose to reuse
`secrets.SUBMODULES_TOKEN` rather than provision a new dedicated secret.
Rationale documented in `design.md` Decision 1. This is unverified from a
real Actions run — the fail-loud preflight check (`design.md` Decision 2)
is the safety net if the reuse turns out not to work, and
`verify-dependabot-alerts-gate-live` (the next change in this phase) is
where that gets confirmed for real.

## Dry-run verification (local, not yet run on GitHub Actions)

Extracted the exact job logic and ran it locally against 3 scenarios,
using this session's own interactive `gh` token (not `SUBMODULES_TOKEN`,
which isn't available outside CI):

1. **Real current alert data** (2 open, both disclosed `hickory-proto`
   alerts): job logic printed `All 2 open Dependabot alert(s) are already
   disclosed.` and exited 0 — correct pass behavior.
2. **Injected fake undisclosed alert** (synthetic JSON, not a real API
   call): job logic printed an `::error::` line naming the GHSA ID,
   severity, and package, and exited 1 — correct fail behavior.
3. **API call failure** (queried a nonexistent repo to force a non-zero
   `gh api` exit): the preflight `if ! RESPONSE=...` block caught it,
   printed the "likely 401/403 / SUBMODULES_TOKEN may lack scope"
   diagnostic, and exited 1 — correct fail-loud behavior. (This test used
   a 404, not a real 401/403, since forcing an actual auth failure wasn't
   practical locally — the exit-code handling path is identical either
   way.)

YAML validity confirmed via `python3 -c "import yaml; yaml.safe_load(...)"`
(no `actionlint` available in this environment).

## What's NOT yet verified

Whether `secrets.SUBMODULES_TOKEN` actually has sufficient scope
(`security_events` for a classic PAT, or "Dependabot alerts: Read" for a
fine-grained token) to call this endpoint from inside a real GitHub
Actions job — that can only be confirmed by an actual CI run, which is
`verify-dependabot-alerts-gate-live`'s job, not this change's.

## Scope check

Only `.github/workflows/security-audit.yml` and
`docs/DEPENDENCY_MANAGEMENT.md` changed — matches `proposal.md`'s Impact
section exactly, confirmed via `git status --short`.
