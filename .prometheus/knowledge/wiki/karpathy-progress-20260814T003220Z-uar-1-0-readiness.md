---
type: Reference
id: karpathy-progress-20260814T003220Z-uar-1-0-readiness
title: "Skill installer renamed on collision and reported success; 19 skills unreachable"
tags:
- karpathy-progress
- uar-1-0-readiness
- complete
sources:
- conversation:operator-agent
timestamp: 2026-08-14T00:32:20Z
created_at: 2026-08-14T00:32:20Z
updated_at: 2026-08-14T00:32:20Z
revision: 1
---

## Intent

install-plugin-generation.js diverted placement to prometheus-<name> when a skill's canonical name was occupied, and targetDestination re-derived the same fallback so verifyTargets validated the renamed path. 19 skills across 14 targets were unreachable at the names tools search while every run printed a success checkmark. Sixteen were symlinks into a source checkout that resolve correctly, so any check asking 'does this symlink resolve' called them healthy; artifact-refiner served four-month-old content across six targets on that basis.

## Observed state and verification

Post-repair: verify-skill-install.js reports 2282/2282 placements current (163 skills x 14 targets), exit 0. Red/green suite scripts/tests/verify-skill-install.test.mjs: 13 passed, 0 failed, with six failure modes each observed failing before repair. deep-research now resolves in .claude/.codex/.opencode as the full 13519-byte skill (April stub was 4582 bytes, one file). install-plugin-generation.js --verify passes after moving provenance stamps out of the signed generation.

## Decision and lesson

Status: complete. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Rule promotion for 'completeness claims require a denominator' runs through AGENT_BASE_RULES.md D-6 gates (adversarial review, sycophancy gate, explicit approval) before any rule text lands. --allow-fallback path has never been exercised.
