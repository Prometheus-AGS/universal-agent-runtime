## Context

See proposal.md for the observed gap. This is a Spec-stage design boundary, not an implemented result. The phase analysis selects reuse of the existing host and pins; the following Plan prerequisites remain binding.

scope: `src/uar/runtime/context/`, `src/uar/context/strategy.rs`, `src/uar/runtime/manager.rs`, `src/llm/orchestrator.rs` and scenario fixtures under existing test roots. Plan must enumerate exact edit files before Execute. Shared files are edited in the phase index's total order; no concurrent writer to manager, orchestrator, prompt assembly or a build target.

## Goals / Non-Goals

Goals: satisfy this change's spec scenarios through existing governed production paths with local evidence.

Non-goals: replacement agent frameworks, dependency upgrades, UI redesign, kernel mutations, deployment or release. No product-code edit is performed by this Spec stage.

## Decisions

1. Extend existing typed history and context services with immutable receipt identity and host-owned protection metadata. Keep canonical storage separate from display projections. Pair-validity-only reduction is rejected because it can discard the entire required group.

2. Adapt the existing tiktoken-rs 0.12.0 service and Liter 1.18.2 access boundary; introduce no dependency. The pure planner consumes resolved bounds and returns a plan; only the trusted host executes optional summaries and writes receipts. A kernel-owned summarizer or storage writer violates capability inversion.

3. Keep a raw-byte digest separate from the existing newline-normalized fragment hash. Freeze canonical record ID/order/role/provenance plus typed-value identity for regression oracles. Only host-marked contiguous prose can enter a summary.

4. Use explicit invalid-history/incomplete/overflow outcomes instead of destructive normalization and acquisition truncation. Acquisition/storage quotas remain enforceable; this is not an unbounded storage promise. Authorization and retention take precedence over preservation.

5. Establish deterministic receipt and request-capture fixtures here, before any later routing or final evaluation work. Implement the common preparation and final envelope/output enforcement now at every entry point; verify successful dispatch under an explicitly resolved synthetic exact-destination contract. The next change extends these hooks with actual per-model profile resolution/settings. No protected-context completion scenario depends on waiting for that extension.

## Required planning prerequisites

Before implementation, Plan must resolve actual acquisition/storage limits, classify existing complete versus truncated receipts, and map every legacy/current reduction path and supported entry point. Exact destinations lacking verified counting bounds remain unsupported rather than guessed.

This change starts implementation after a valid Plan. It does not wait for the final evaluation change to establish fixtures.

## Risks / Trade-offs

The uncomfortable scenario: A valid protected receipt can exceed every allowed model → explicit overflow, with intact authorized data. Existing truncated checkpoints cannot be repaired → mark incomplete. Retaining raw receipts increases storage → enforce acquisition/storage policy with truthful incomplete outcomes.

## Migration Plan

Add receipt/protection metadata with explicit legacy completeness status; migrate only provably complete records. Switch production paths together after fixtures pass. Rollback must stop incompatible runs and retain new receipts; it must not silently reenable lossy compaction for protected histories. All rollout and rollback steps are future implementation work with local validation; this artifact authorizes no deployment.

## Verification contract

Map every scenario to an executable fixture during Plan. Run Tier 0 per edit, Tier 1 per completed unit and Tier 2 at phase completion. Tier 3 belongs only to the separately registered local certification milestone. Request-level fixtures and production-path traces are required where specified; module presence and build success are not behavior proof. Artifact validation in this stage cannot establish runtime correctness.


## Plan resolution (2026-09-16)

The historical Spec-stage prerequisite list above is retained. Its concrete resolution and remaining execution gates are in `.kbd-orchestrator/phases/runtime-harness-gap-closure/plan.md`, `plan-research.md`, `execution-map.json`, `acceptance-contract.json`, `provider-profile-matrix.json` and `profile-certification.md` (repository-relative paths). The Plan manifest records current hashes; the earlier Spec manifest remains a historical snapshot. No implementation checkbox is completed by these planning artifacts.

- library: cand-001 — Existing UAR typed prompt/turn, routing, persistence and governed evaluation seams; verdict: adapt. Preserve capability inversion and established host integration; implement missing contracts rather than replacing runtime.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/prompt/assemble.rs — Inspected local source renders typed fragments with two family preference booleans; extend existing seam.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/context/reduce.rs — Inspected reducer can drop severed tool groups; new protected preservation requires a stronger contract.
- library: cand-002 — tiktoken-rs (existing 0.12.0 dependency); verdict: adapt. Reuse existing dependency via a counting adapter; retain0.12.0 and verify exact pinned behavior before implementation.
  Evidence: https://github.com/zurawiki/tiktoken-rs — Repository search confirms Rust tokenizer project and MIT license.
  Evidence: https://github.com/zurawiki/tiktoken-rs/blob/main/_autodocs/api-reference-initialization.md — Context7 identifies model-to-tokenizer lookup; main documentation does not certify pinned-version whole-request or multimodal accounting.

Exact owned files, proposed fixtures, commands and per-task evidence directories are in execution-map.json. The reviewed order serializes shared files. No product behavior edit precedes its frozen baseline; no actual model profile is enabled before endpoint/count/output evidence; no phase completion precedes the separately registered local certification milestone.
