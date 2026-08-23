---
sidebar_position: 7
title: Evidence Taxonomy
description: Match each UAR claim to the narrowest test that can actually support it.
source_records:
  - AGENTS.md
  - .prometheus/decisions.md
  - .kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md
current_authority: /docs/history/evidence-taxonomy
---

# Evidence taxonomy

Every retained result should name its command, observed output, source SHA,
environment, and profile. A label such as “integration” is not enough; the
exercised boundary determines the claim.

| Evidence class | What it proves | What it does not prove |
|---|---|---|
| Static/type/lint | Selected source parses, types, lints, or satisfies the named static rule | Execution or a completed workflow |
| Focused unit/component | A unit produced the asserted result under controlled inputs | Packaging, external providers, end-to-end behavior, or another profile |
| Synthetic/recorded integration | Protocol/orchestration/error handling against a deterministic double | Genuine inference, provider availability, soak, resilience, release, or readiness |
| Packaged functional | A named workflow crossed the supported packaged UAR boundary | A model boundary unless genuine model evidence is retained, or any untested path |
| Real-model functional | A named provider/model performed genuine inference through UAR for the bounded path | Broad quality, every provider, duration resilience, deployment, or another profile |
| Load/soak/resilience | The named workload exercised its failure model, volume, period, or statistical objective | Inference when traffic uses a double, or claims beyond the workload/environment |
| Deployment validation | The deployed artifact was reachable and passed named deployment checks | Routine code correctness, inference, or undeployed profiles |

## Profile and source boundaries

- `server-full` evidence applies to the named server-full build and configuration.
- `minimal` evidence is separate; absence of Cedar or packaged UI changes the
  surface under test.
- `embedded-mobile` evidence depends on host-provided inference, lifecycle, and
  persistence and does not inherit server deployment results.

A result also binds to its recorded **source SHA**. Later source can use the
result as history, not as fresh execution evidence. “All profiles” and “the
runtime is ready” are aggregate conclusions that require separately stated
criteria; one passing row cannot imply them.

## Inference boundary

Certifying inference must traverse UAR, use the configured provider path, reach a
real loaded model, and return that model's output through UAR. Retain the
provider/model identity and, when routing or an agent is under test, the effective
policy. Mocked, stubbed, recorded, replayed, hard-coded, or otherwise synthetic
responses remain non-certifying diagnostics.

## Evidence record minimum

Record the requirement, exact command or interaction, observed result, negative
control when fail-closed, source SHA, profile, environment, and explicit limit.
If a prerequisite is unavailable, record the claim unverified instead of
substituting a weaker test.
