# Dual-license policy

## Purpose

Define the dual-licensing framework for the UAR runtime, the SDKs,
and the documentation. The runtime is AGPL-3.0 + commercial; the
SDKs are MIT; the documentation is CC-BY-4.0.

## ADDED Requirements

### Requirement: Runtime license is AGPL-3.0 + commercial
The UAR runtime server (the `universal-agent-runtime` and
`uar-sidecar` binaries) SHALL be licensed under AGPL-3.0-only, with
a separate `LICENSE-COMMERCIAL.md` providing a commercial
alternative for SaaS deployers who do not accept the AGPL network
clause.

#### Scenario: A developer self-hosts the runtime
- **WHEN** the developer runs the runtime on their own infrastructure
- **THEN** they accept the AGPL-3.0 terms (no commercial license
  required)
- **AND** the `LICENSE-COMMERCIAL.md` is informational, not required

#### Scenario: A SaaS deployer offers UAR as a hosted service
- **WHEN** the operator offers UAR as a network-accessible service
  to third parties
- **AND** they do not wish to open-source their modifications under
  the AGPL network clause
- **THEN** they require a commercial license
- **AND** the `LICENSE-COMMERCIAL.md` provides a contact path for
  licensing inquiries; public pricing bands are deferred (operator
  decision, 2026-07-13) and not required for this change to close

### Requirement: SDKs are MIT
The UAR SDKs in `sdks/python`, `sdks/rust`, and `sdks/typescript`
SHALL be licensed under the MIT License. Every `Cargo.toml`,
`pyproject.toml`, and `package.json` in those subtrees MUST declare
the MIT license. A `LICENSE` file MUST exist at the root of each
SDK subtree.

#### Scenario: An enterprise builds a closed-source product that calls a UAR SDK
- **WHEN** the enterprise integrates the MIT-licensed SDK into
  their proprietary product
- **THEN** the MIT terms permit this use without royalty or
  open-source obligation
- **AND** the runtime is still AGPL-3.0 + commercial, so the
  runtime instance they self-host requires the same license
  posture as any other self-hosted UAR

### Requirement: Documentation is CC-BY-4.0
All documentation under `docs/` SHALL be licensed under
Creative Commons Attribution 4.0 International (CC-BY-4.0). The
CC-BY-4.0 notice MUST be included in the docs site footer.

#### Scenario: A third party reuses a documentation page
- **WHEN** a third party copies or adapts a page from `docs/`
- **THEN** `LICENSE-CC-BY-4.0.md` at the repo root governs their
  reuse, and attribution to the UAR project satisfies the license
- **AND** the code license (AGPL-3.0-only for the runtime, MIT for
  the SDKs) is unaffected by their documentation reuse

### Requirement: CLA-lite forward-going clause
`CONTRIBUTING.md` SHALL state that by submitting a contribution
to UAR, the contributor agrees to dual-licensing under the project's
then-current license terms (AGPL-3.0 + commercial for the runtime;
MIT for the SDKs; CC-BY-4.0 for documentation). The clause MUST be
visible before the contribution submission instructions.

#### Scenario: A new contributor opens a pull request
- **WHEN** a contributor submits a change to a path outside `sdks/`
- **THEN** `CONTRIBUTING.md`'s CLA-lite clause governs that their
  contribution may also be distributed under the commercial terms
  in `LICENSE-COMMERCIAL.md`
- **AND** a contribution scoped to `sdks/python`, `sdks/rust`, or
  `sdks/typescript` is MIT-licensed only, with no AGPL or commercial
  dual-license obligation

### Requirement: CI guard for license consistency
A CI step MUST verify that every `Cargo.toml`, `pyproject.toml`,
and `package.json` declares a license and that the declared license
matches the LICENSE file in the same directory. The CI step MUST
fail on mismatch.

#### Scenario: A PR changes a manifest's license field without updating the LICENSE file
- **WHEN** `tools/license-check.sh` runs in CI
- **AND** a manifest's declared `license` field no longer matches
  the expected value for its component (runtime AGPL-3.0-only, SDKs
  MIT/MIT OR AGPL-3.0) or the expected LICENSE file is missing
- **THEN** the CI step fails and blocks the PR

### Requirement: Open-letter process for past SDK contributors
Past SDK contributors SHALL be given a one-time window to either
(a) consent to MIT relicensing of their contributions, or (b) have
their contributions removed from the SDK before the MIT license
file is added. The window SHALL be at least 30 days. The process
SHOULD be documented in `docs/governance/license-migration.md`.

#### Scenario: A git history audit finds only maintainer-owned commits
- **WHEN** `git log --all -- sdks/python sdks/rust sdks/typescript`
  is audited for contributor identities
- **AND** every commit is authored by the project's own maintainer
  or CI automation identities, with no third-party contributors
- **THEN** the open-letter send (task 1.3) reduces to operator
  self-authorization, and the removal step (task 1.4) does not apply
- **AND** the operator must still explicitly confirm this reading
  before the consent step is treated as resolved

#### Scenario: A git history audit finds a third-party contributor
- **WHEN** the audit finds a commit under `sdks/` authored by
  someone other than the project's maintainers or CI automation
- **THEN** the open letter in `docs/legal/sdk-relicense-open-letter.md`
  MUST be sent to that contributor before the SDK is published
  under MIT
- **AND** if no consent is received within the 30-day window, that
  contributor's code MUST be removed from the SDK source before
  release
