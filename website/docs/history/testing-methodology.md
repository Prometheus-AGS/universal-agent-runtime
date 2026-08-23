---
sidebar_position: 6
title: Testing Methodology History
description: Understand why UAR changed what it accepts as evidence.
source_records:
  - docs/adr/0003-coverage-baseline-60-percent.md
  - .prometheus/decisions.md
  - .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/reflection.md
current_authority: /docs/history/testing-methodology
---

# Testing methodology history

UAR did not stop using small tests when it adopted genuine-model functional
verification. **The evidence changed because the claim changed.** A unit test is
appropriate for a pure decision. A recorded provider is useful for deterministic
protocol behavior. Neither proves that a configured provider reached a real
model and returned inference through the packaged runtime.

## Coverage and isolated correctness

ADR-0003 established a 60% local line-coverage target for Rust and React. The
retained baseline table is still unpopulated, so the repository does not claim an
observed 60% baseline. Its old statement that CI would enforce coverage is
superseded: routine verification is local and GitHub Actions are deployment-only.

Coverage answers which lines ran under a suite. Focused unit and component tests
answer whether a selected decision produced its asserted result. Neither answers
whether the packaged application, external provider, or user workflow worked.

## Recorded-provider diagnostics

The 1.0 readiness phase used a deterministic **recorded provider** for a serial
server-full capability matrix. That was valid for the stated authentication,
tenancy, skill, and orchestration assertions. Negative controls showed that the
fail-closed cases could actually fail. The results transferred to no other
profile and did not certify real model inference.

## The synthetic-soak failure

A later operational plan spent hours sending deterministic traffic to a local
provider double. The run exercised runtime plumbing, then a shutdown test exposed
a real deadline defect. But elapsed time did not repair the workload's missing
model boundary. The evidence could not support the requested inference or broad
readiness claim.

The project therefore rejected duration-only synthetic evidence. A long run now
needs a named failure model, traffic-volume target, operating-period target, or
statistical detection objective. If it uses a double, it still cannot certify
inference.

## Bounded real-model closeout

On 2026-08-22, UAR 1.0 closed on five functions—proxy inference, skill
activation, knowledge grounding, Kimi k3 configuration/inference, and basic-agent
creation/inference—each exercised without and through the shipped UI. The
retained evidence identifies genuine models and the effective policy/routing
where relevant.

The soak, supply-chain, release-candidate, and GA-publication changes were
**cancelled**, not called passed. The functional result was local `server-full`
evidence only. It made no `minimal`, `embedded-mobile`, supply-chain, publication,
or universal production-readiness claim.

## Current rule

Choose the narrowest test that crosses the boundary named in the requirement.
Keep cheap diagnostics for fast fault localization. Add genuine external
integration only when the claim reaches that boundary. Retain what the result
does not prove as carefully as what it does.
