# Security Policy

_Last updated: 2026-07-12._

Universal Agent Runtime (UAR) is open-source software ([MIT](LICENSE))
developed by Prometheus AGS. We continue to treat ourselves as a
**manufacturer** in the sense of the EU Cyber Resilience Act and operate this
policy accordingly, because Prometheus AGS offers commercial services built on
UAR — the relicense to MIT changed the software's terms, not our security
obligations to the people running it.

## Reporting a vulnerability

**Please do not open a public issue for security reports.**

Report privately via **GitHub private vulnerability reporting**:
[Security → Report a vulnerability](https://github.com/Prometheus-AGS/universal-agent-runtime/security/advisories/new).

Include what you can: affected version/commit, reproduction steps, impact
assessment, and any suggested fix. Reports are welcome in English.

## What to expect

These are operating targets, not contractual guarantees (commercial support
contracts govern guarantees):

| Stage | Target |
|---|---|
| Acknowledgement | within **24 hours** |
| Triage & severity assessment | within **72 hours** |
| Fix or documented mitigation for confirmed HIGH/CRITICAL | within **14 days** |
| Coordinated disclosure | agreed with the reporter; default 90 days |

For vulnerabilities we assess as **actively exploited**, we align with EU CRA
Article 14 reporting practice (24-hour early warning / 72-hour notification
to the relevant CSIRT/ENISA channels, applicable from 2026-09-11).

## Supported versions

| Version | Supported |
|---|---|
| 1.0.x | ✅ security fixes |
| < 1.0 (unreleased development history) | ❌ upgrade to 1.0 |

The supported 1.0.x line covers the published server/archive artifacts and the
Stable rows in the product support matrix. Preview and Experimental
capabilities receive best-effort fixes unless a commercial agreement states
otherwise.

## Dependency security

- `cargo audit`, `pnpm audit`, and a GitHub Dependabot alert gate run in CI
  (`.github/workflows/security-audit.yml`); any suppressed advisory must be
  disclosed with rationale in
  [`docs/DEPENDENCY_MANAGEMENT.md`](docs/DEPENDENCY_MANAGEMENT.md).
- Dependabot is enabled for Rust and npm ecosystems.

## Hardening defaults

UAR ships secure-by-default: JWT auth required, rate limiting enabled,
prompt-injection/PII guardrails active, secrets redacted from logs. See the
deployment guide before changing these defaults.
