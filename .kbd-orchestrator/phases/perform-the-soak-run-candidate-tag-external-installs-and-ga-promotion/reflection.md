# Phase Reflection: perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

**Project:** universal-agent-runtime  
**Date:** 2026-08-22  
**Reporting scope:** server-full functional closeout only; no aggregate percentage or cross-profile verdict  
**Terminal changes:** 10 completed; 4 cancelled by operator course correction

## Delta, Root Cause, and Corrective Actions

The delivered closeout diverged from the original release-tail plan. The three-hour operational-resilience run, supply-chain artifacts, release-candidate certification, and GA publication were not completed and are not represented as passed. The operator superseded those gates after determining that synthetic and elapsed-time testing did not provide useful evidence for this closeout and required five bounded real-model functional paths through both the packaged API boundary and the shipped UI.

The implementation audit and live runs exposed five concrete code defects: configured Kimi credentials were not resolved from the catalog-declared environment variable; the provider form attempted create instead of update; dotted native skill IDs were invalid provider tool names; assistant-ui grouped leaf parts returned an invalid child value; and chat runtime creation occurred outside the selected-agent and memory providers. Each defect was corrected at its causal boundary. The shipped UI was rebuilt after the frontend corrections.

Corrective action was limited to the requested product paths. All ten functional checks were rerun against genuine model inference. The four superseded release-tail changes were transitioned to CANCELLED through the canonical KBD runtime rather than being mislabeled complete.

## Goals

| Goal | Status | Evidence and limit |
| --- | --- | --- |
| OpenAI proxy inference without UI | MET | Packaged UAR returned `UAR-PROXY-REAL-8181` from `openai/gpt-5.4-mini` through `http://localhost:8181`. |
| OpenAI proxy inference through UI | MET | Chat UI returned `437` for 23 × 19 through `openai/gpt-5.4-mini`. |
| Skill activation without UI | MET | `emberquartz-functional-skill` emitted activation selection and real inference returned `SKILL-ACTIVATED-EMBER`. |
| Skill activation through UI | MET | The UI showed the Emberquartz activation artifact and returned `SKILL-ACTIVATED-EMBER`. |
| Knowledge-base grounding without UI | MET | Added `Cobalt Heron 7319`; indexed search scored `0.8674219` and inference returned the fact. |
| Knowledge-base grounding through UI | MET | Created `UI Verification KB`, uploaded a file containing `Silver Mango 4826`, and UI inference returned the fact with the source visible. |
| Kimi k3 configuration and inference without UI | MET | Configured `kimi-for-coding/k3` with the environment-backed credential and observed `KIMI-K3-LIVE-UAR`. |
| Kimi k3 configuration and inference through UI | MET | Effective policy reported `kimi-for-coding/k3`; model `k3` returned `UI-KIMI-K3-LIVE`. |
| Basic agent creation and inference without UI | MET | Created `api-basic-agent` and observed `AGENT-BASIC-LIVE` through UAR. |
| Basic agent creation and inference through UI | MET | Created `UI Basic Agent`; effective policy reported `ui-basic-agent` with `openai/gpt-5.4-mini`, and a genuine model response was returned. |

## Delivered Changes

- Corrected catalog-backed provider credential resolution in `src/llm/registry.rs`.
- Corrected configured-provider update behavior in `frontend/src/features/providers/model/providers-store.ts` and its API documentation.
- Added safe provider-facing native-skill tool names in `src/uar/runtime/native_skill.rs`.
- Corrected assistant-ui grouped leaf rendering in `frontend/src/components/assistant-ui/enhanced-thread.tsx`.
- Moved chat runtime construction inside selected-agent and memory providers in `frontend/src/features/chat/chat-thread-view.tsx`.
- Rebuilt the shipped static frontend bundle.

## Technical Debt and Limits

- The cancelled soak, supply-chain, RC, and GA changes have no passing evidence and support no release-publication claim.
- Results apply only to the server-full configuration exercised locally. No claim transfers to minimal or embedded-mobile profiles.
- The UI-created OpenAI agent refused to reveal its hidden system prompt, which is acceptable model behavior; agent identity and model routing were verified from the effective run policy and a real response.
- No GitHub Actions job, synthetic provider, recorded provider, unit suite, soak, tag, publication, push, or PR was used for this closeout.

## Architecture Integrity

- The code corrections are surgical and remain within the observed provider, skill, assistant rendering, and chat configuration boundaries.
- The KBD record preserves the distinction between completed functional acceptance and cancelled release-tail work.
- No new public API or dependency was introduced.

## Cross-Tool Coordination Notes

- Canonical progress tracking remained available locally while the control plane at `127.0.0.1:7892` was unavailable; the CLI committed immutable events locally.
- The earlier testing plan caused substantial rework because it optimized for duration and broad certification rather than the operator's bounded functional acceptance.
- Future closeouts should lock the operator's acceptance matrix before running evidence and inspect the effective run policy whenever UI agent/model routing is under test.

## Lessons Learned

- A model response alone cannot prove provider routing; the effective run policy must match the selected agent and model.
- React context consumers must be instantiated below their providers, not merely rendered below them.
- Provider-facing tool identifiers need a compatibility name distinct from stable internal skill IDs.
- Configuration forms must distinguish create from update and preserve environment-backed credentials.
- Real knowledge grounding evidence should show both the returned fact and its selected source.

## Next Phase Focus

No successor phase is started here. The requested UAR 1.0 functional closeout is terminal. Any future supply-chain, publication, or UAR 1.1 work requires a separately authorized phase.

## Context for Any Future Phase

Use `functional-verification.md`, this reflection, and canonical decision `functional-real-inference-closeout-only` as prior context. Do not reinstate the cancelled release-tail gates without explicit operator direction.
