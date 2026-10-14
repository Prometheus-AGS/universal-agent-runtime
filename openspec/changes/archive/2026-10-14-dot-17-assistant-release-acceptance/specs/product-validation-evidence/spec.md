# Spec Delta

## ADDED Requirements

### Requirement: Both required deployment profiles pass before first release

First-release assistant certification SHALL require evidence for all six capabilities on both local and shared-host profiles. Product verification, publication and deployment health SHALL remain separate claims.

#### Scenario: Only one profile passes

- **WHEN** local operation passes but shared-host evidence is missing or fails, or vice versa
- **THEN** first-release assistant certification remains incomplete

#### Scenario: Final release evidence

- **WHEN** all local product acceptance gates pass and artifacts are published
- **THEN** the final source revision, backend/entry-point matrix, both architecture digests and deployment health are recorded separately from product test results




## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


## Superseding delivery acceptance — operator decision 2026-10-14

The operator directed that code-complete UAR delivery stop being blocked by qualification suites and instead be accepted through actual use by agents and dependent projects. The 109-case qualification matrix, synthetic/conformance/fault suites, native reboot and clock/disk qualification, and review-suite certification are no longer prerequisites for this release. Preserve their actual historical passes/failures and unexecuted status. This decision does not mark a cancelled check passed and does not remove any implemented feature or any of the nine reliability or nine issue #379 requirements.

The release exit criteria are usable production implementation, a build of the actual current source, installation on existing authorized capacity, and a useful task executed by an existing configured agent/provider through UAR. Record the real turn, the actual tools/approvals used and their independently observed result, memory/document/accounting persistence actually exercised, exact artifact/source identity and consumer availability. Do not force unused features into artificial scenarios. Report unused protocols, adversarial edge cases and crash/reboot guarantees as unverified rather than blocking unrelated consumers. Keep actual defects, failed builds and failed consumer use actionable; code presence alone is not a passing consumer result.

Necessary artifact building and deployment health remain delivery work. All production implementation and this scope reconciliation precede any build or consumer-use execution. Gates remain serial; no new qualification scripts, matrix expansion, cache-copy workaround or paid resources. Existing node/volume/VM approvals and immutable cleanup deadlines remain. Independent teammates may inspect and fix observed code problems without making their review a new release gate. Merged-main releases for other projects may proceed using their explicitly selected source; unfinished Dot qualification is not a dependency-wide freeze.

Cargo cache IDs are reusable namespace identifiers, not source-attestation hashes. The delivery may reuse the genuinely populated existing architecture/toolchain/profile namespace. Actual source identity, dependency pins, Cargo fingerprints and produced artifact digests remain independently recorded. Do not relabel an old source archive, copy compiled outputs under a fabricated cache identity, prune caches or claim reproducibility from cache reuse.
