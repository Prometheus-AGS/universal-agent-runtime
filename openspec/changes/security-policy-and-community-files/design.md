## Context
Phase assessment §2/§8; operator chose manufacturer posture (AskUserQuestion
2026-07-10). LICENSE (AGPL-3.0) + LICENSE-COMMERCIAL.md already exist; only
the clarity/policy layer is missing.

## Goals / Non-Goals
**Goals:** the four policy artifacts, accurate and dated.
**Non-Goals:** legal advice; changing either license's terms; CE-marking/
full CRA technical documentation (Dec 2027 obligations — out of scope).

## Decisions
- D1: GitHub private vulnerability reporting (no new infra) as the channel;
  enable the repo setting at apply time via gh api.
- D2: targets stated as goals, not contractual guarantees, with CRA Art. 14
  framing for actively-exploited vulnerabilities (24h early-warning intent).
- D3: supported-versions table starts at the 1.0.0 release this phase cuts.

## Risks / Trade-offs
- [Stated SLAs create expectations] → framed honestly as targets; commercial
  contracts govern guarantees.

## Migration Plan
Additive files; no rollback concerns.

## Open Questions
(none)
