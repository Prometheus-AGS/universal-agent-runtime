## ADDED Requirements

### Requirement: Green Main Branch
Every GitHub Actions workflow triggered on `main` SHALL conclude successfully
or be explicitly advisory (non-blocking) with that status stated in the
workflow file.

#### Scenario: All workflows green after a main push
- **WHEN** a commit lands on `main` and all triggered workflows complete
- **THEN** every workflow conclusion is `success`

### Requirement: Warning Policy Owned By Cargo Config
CI clippy steps SHALL derive their failure policy from `Cargo.toml`'s
`[lints]` configuration rather than CLI-level blanket escalations.

#### Scenario: Pedantic warnings do not fail CI
- **WHEN** clippy reports warnings whose configured level is `warn`
- **THEN** the CI step reports them without failing the build

### Requirement: Audit Steps Share One Disclosure List
Any CI step running dependency audits SHALL apply the same documented
ignore/disclosure list as `security-audit.yml`, so a finding is either fixed
or disclosed once — never failing one workflow while passing another.

#### Scenario: Disclosed advisory does not fail secondary workflows
- **WHEN** a RUSTSEC advisory is disclosed in the canonical ignore list
- **THEN** no other workflow fails on that same advisory
