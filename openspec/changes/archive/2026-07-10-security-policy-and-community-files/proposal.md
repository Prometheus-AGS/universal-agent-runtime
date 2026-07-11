## Why

2026-baseline expectations for customer-adoptable OSS (per the phase's web
research): a vulnerability disclosure policy with a private reporting channel
(OpenSSF baseline; EU CRA Art. 14 prerequisite — reporting obligations start
2026-09-11 and the commercial license likely makes this project a
"manufacturer"), a support policy, issue templates, and plain-language
licensing clarity (AGPL is auto-rejected by many enterprise legal teams
without an explicit dual-license statement). None of these files exist.
Operator decision (2026-07-10): manufacturer posture.

## What Changes

- `SECURITY.md`: manufacturer-posture vulnerability policy — GitHub private
  vulnerability reporting as the channel, 24h acknowledgement / 72h triage
  targets for actively-exploited reports, CVE response SLA, supported
  versions table, CRA-alignment note.
- `SUPPORT.md`: channels, response expectations, commercial support pointer.
- `.github/ISSUE_TEMPLATE/`: bug report + feature request forms, config.yml
  routing security reports to private reporting.
- `docs/LICENSING.md`: plain-language dual-license page (what AGPL does and
  does not require of a self-hosting customer; when the commercial license
  applies), linked from README.

## Capabilities

### New Capabilities
- `security-support-policy`: published security disclosure, support, and
  licensing-clarity policies meeting 2026 OSS production baselines.

### Modified Capabilities
(none)

## Impact

Docs/policy files only; README gains two links. KBD: change 4/9.
