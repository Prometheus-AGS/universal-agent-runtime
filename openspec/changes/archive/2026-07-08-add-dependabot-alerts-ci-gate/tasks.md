## 1. CI job

- [x] 1.1 Add `dependabot-alerts-gate` job to `.github/workflows/security-audit.yml`: call `gh api repos/${{ github.repository }}/dependabot/alerts --jq '...'` using `secrets.SUBMODULES_TOKEN`, filtering to `state == "open"`.
- [x] 1.2 Add a preflight check on the API call's exit status / HTTP response that fails with an explicit "token likely lacks security_events scope" message on 401/403, per design.md Decision 2.
- [x] 1.3 Add an inline GHSA-ID allowlist (currently: `GHSA-q2qq-hmj6-3wpp`, `GHSA-3v94-mw7p-v465`, both `hickory-proto`, already disclosed in `docs/DEPENDENCY_MANAGEMENT.md`) and fail the job on any open alert not in that list.
- [x] 1.4 Confirm the job's YAML is valid (`gh workflow view` or equivalent parse check) and its logic dry-runs correctly against this session's already-fetched alert data (2 open, both in the allowlist) using local `gh api` + the same jq filter, without needing a live CI run yet.

## 2. Documentation

- [x] 2.1 Update `docs/DEPENDENCY_MANAGEMENT.md`'s Dependabot/GHSA section to state the check is now automated in CI (replacing "check manually" language), and note the job reuses `SUBMODULES_TOKEN`.
- [x] 2.2 Note in the same doc that the GHSA allowlist in `security-audit.yml` must be updated whenever a new advisory is triaged and disclosed, mirroring the existing `cargo audit --ignore` convention already documented there.

## 3. Verification

- [x] 3.1 Run `cargo fmt --check` / no-op check that only `.github/workflows/security-audit.yml` and `docs/DEPENDENCY_MANAGEMENT.md` changed (docs + CI only, per proposal.md Impact).
- [x] 3.2 Record findings (dry-run output, allowlist rationale, any surprises about `SUBMODULES_TOKEN`'s actual scope) in `findings.md`, per this project's established per-change convention.
