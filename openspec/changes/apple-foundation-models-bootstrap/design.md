# Design

## Context

See [proposal.md](proposal.md) for motivation and the [provider delta](specs/provider-model-settings-certification/spec.md) for proposed behavior. Source inspection on 2026-11-15 found the existing liter-llm provider routing and persistent-default precedence, but no `SystemLanguageModel` adapter in UAR's `src/` tree. This is a design handoff, not a claim that the inspected checkout or installed UAR supports Apple inference. No build, model test, download, export, installation or service reconfiguration was performed.

## Goals / Non-Goals

**Goals:** use an available local Apple model to explain and assist first-run setup; retain trusted-host execution, explicit model identities, durable consent and state, and ordinary UAR routing/accounting.

**Non-Goals:** replacing liter-llm routing, selecting a universal model, changing existing operator defaults, granting the assistant install authority, changing embedding selections, benchmarking, or raising the whole product's deployment floor in this documentation change.

## Decisions

### Two distinct Apple paths

The system-model bootstrap uses Apple's available on-device `SystemLanguageModel`. Its availability API exists on macOS/iOS 26; eligibility still requires an implemented, compiled and qualified UAR adapter plus supported chip, OS, enabled Apple Intelligence and ready assets. A new Mac cannot be admitted merely because it is ARM. Runtime availability must be rechecked before use. Existing explicit settings and opt-outs win; do not reinterpret failed explicit selection as a fresh installation.

Custom-provider integration is an optional macOS 27 SDK/runtime lane. Apple documents the `LanguageModel` protocol and `MLXLanguageModel`/`CoreAILanguageModel` integrations. Qualify iOS/iPadOS separately; desktop evidence does not certify mobile memory limits or lifecycle behavior. Keep unsupported platforms on their existing deliberate selection/manual setup path.

### One provider seam, multiple supported execution backends

Register each backend under the existing UAR/liter-llm provider and model routing contract through a typed native bridge. The concrete registration identifier and wire mapping must be specified and pinned during implementation; this document introduces no configuration keys.

| Optional path | Resource and lifecycle contract | Admission |
|---|---|---|
| `MLXLanguageModel` | `weightsLocation` resolves an approved local directory; `load` binds the loader-owned acquisition/tokenizer operation instead of implicitly invoking the sample downloader. | macOS 27 SDK/runtime integration, compatible MLX architecture/weights/tokenizer, explicit capabilities and memory budget. |
| `CoreAILanguageModel(resourcesAt:)` | Consume a loader-verified export resource folder containing `.aimodel` and required tokenizer/resources. Export is an explicit setup action, not implicit format conversion. | Catalog-supported model/export and selected hardware/OS/precision; FP4/FP8 activation/KV-cache presets require macOS 27.2+. |
| Foundation Models bridge to flint-loader | Forward generation/cancellation to a declared loader service or native runtime, preserving operation identity, placement and lease ownership. | Capability support of the actual backend, not the bridge's API surface. |
| Direct MLX/Core AI/native execution | A selected backend may execute inference directly under a flint-loader lease instead of adding a redundant service hop. | Same explicit hardware, format, capability and lifecycle checks; UAR still owns every agent continuation and tool dispatch. |

The MLX adapter declares guided generation, tool calling, reasoning and vision capabilities explicitly; the model ID does not prove them. Treat model-family, tokenizer, format and precision support as separate evidence. Not all models are supported. The flint-loader custom-model registry retains artifact identity, source revision/digest, license, format, tokenizer, execution location and required memory. Apple manages its own system model assets; UAR must not pretend flint-loader owns or can install those assets.

### One agent/tool loop

UAR's trusted host owns policy checks, approval, effect classification, execution, timeout/cancellation, audit and continuation. The Swift/native provider returns generated text, structured output or tool proposals. Do not register directly executing Swift `Tool` implementations or start an independent `LanguageModelSession` tool loop. If the selected SDK path cannot return a proposal without executing it, disable tool capability for that adapter until a compatible bridge exists. The setup model's recommendation is data, not authority to act.

### Local first-run state and consent

Persist setup state and operation identity through the existing settings/domain authority and publish its entity projection. Resolve `checking` into `ready`, `unavailable` or `failed`; proposed changes enter `awaiting-consent`, then `acquiring`, `validating` and `ready`, with `canceled`/`failed` exits. Track current and proposed provider separately. A model/provider switch becomes live only after readiness and durable selection succeed. Reconcile partial effects on resume before offering retry; cancellation stops owned work and reports completed or uncertain effects without claiming rollback.

The first-run local system-model default needs no provider credential. Each download/export, provider switch, credential acquisition/use and off-device transition requires an explicit scoped decision. Private Cloud Compute is off-device and is not a local fallback. Credential entry belongs to the trusted credential surface, never the model prompt. OS-owned assets can be reported as preparing/unavailable; enabling Apple Intelligence or fetching assets remains an explicit user action. Failure retains manual setup and the previous selection; automatic cloud fallback is prohibited.

### Embeddings are a separate registry decision

Generation selection never rewrites memory or knowledge embedder identity, revision, vector dimensions, preprocessing, distance metric or index binding. Explicit embedder migration needs an approved resumable reindex and verified cutover; generation-model token embeddings are not automatically a retrieval embedding service. Existing local/Off defaults remain intact.

## Risks / Trade-offs

- A small system model may give poor setup advice or refuse a task → use bounded structured plans and trusted-host validation; preserve manual setup. Availability does not establish quality or capability.
- A custom artifact may fit disk but exhaust working memory → admit using model/context/KV-cache/runtime headroom, then qualify on the target device. Do not infer performance from parameter count or artifact size.
- API uniformity can hide remote placement or nested execution → expose real placement, disable unsupported tool paths, and collect egress and single-dispatch evidence at the authorized phase gate.
- OS updates can change the system model → record OS/model availability provenance and repeat relevant behavioral acceptance before claiming support.

## Future VLBV evidence gates

All product gates below are specified, not executed. A dedicated approved KBD phase, complete production implementation inventory, exclusive resource ownership and the repository's verification authorization are prerequisites; documentation completion opens none of these gates.

1. **Spec/plan gate:** approve platform/capability matrix, exact adapter/SDK revisions, provider schema mapping, consent model, implementation ledger and evidence plan. Keep system-model bootstrap separate from the macOS 27 custom lane and 27.2 precision presets.
2. **Implementation-complete gate:** finish native adapter, routing/selection persistence, loader bridge, setup state/UI, cancellation/recovery, embedding noninterference and production-path scenarios. Record exact source inventory before verification.
3. **Authorized integration gate:** demonstrate a fresh eligible device uses local system inference with zero credential/download/cloud effects; unsupported/disabled/not-ready states preserve manual setup; explicit choices and opt-outs survive restart; denied/canceled/resumed setup does not activate partial providers; tool calls dispatch only through UAR; generation switches leave embedding identities and indexes unchanged. Capture redacted routing, operation, availability and egress evidence.
4. **Device/release gate:** qualify an actual supported Mac and each claimed iOS/iPadOS profile, exact SDK/OS/model artifact/context/resource budgets, crash/restart recovery and migration/rollback. Run expensive builds and model evaluations only here with the required grant; do not claim benchmarks or all-model support from upstream examples.

## Migration Plan

The direct main-spec amendment is a proposed target contract, paired with this unarchived delta so implementation remains visible as pending. Before shipping, add the explicit supported native profile, persist bootstrap origin/opt-out, and preserve existing saved selections. Rollback disables the new adapter without deleting credentials, downloaded artifacts or embedding indexes; a now-unavailable selected provider yields explicit recovery/manual setup. No deployment or configuration migration is part of this commit.

## Sources checked 2026-11-15

- [Apple Foundation Models updates](https://developer.apple.com/documentation/updates/foundationmodels.md): custom `LanguageModel` protocol and MLX/Core AI integrations.
- [SystemLanguageModel availability](https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel/availability-swift.property.md): runtime availability API and platform availability.
- [MLXFoundationModels README](https://github.com/ml-explore/mlx-swift-lm/blob/main/Libraries/MLXFoundationModels/README.md): direct initializer, custom weights/downloader, explicit capabilities, SDK trait/27 SDK requirements.
- [Core AI Models](https://github.com/apple/coreai-models): exported resources and 27.0 platform/toolchain requirements.
- [Core AI Qwen3 recipe](https://github.com/apple/coreai-models/blob/main/models/qwen3/README.md) and [model catalog](https://github.com/apple/coreai-models/blob/main/models/README.md): `resourcesAt`, catalog limits and macOS 27.2 FP4/FP8 presets. Upstream examples are not UAR acceptance evidence.
