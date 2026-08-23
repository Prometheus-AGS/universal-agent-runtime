# Reflect — `uar-native-service-deployment`

**Date:** 2026-08-23
**Plan authority:** `plan.md` revision 23
**Evidence authority:** the five dated OpenSpec archives and append-only `.prometheus/logs/uar-native-service-deployment/`

## Plan-to-delivery delta

The planned native deployment capability was delivered, but the final change expanded in three material ways. First, the operator selected the newly released Alibaba model `qwen3.8-max`, replacing the planned older Qwen seed. Second, runtime visibility required advancing both catalog gitlinks and regenerating the reviewed compile-time catalog; advancing `models.dev` alone could not affect the binary because `catalog/provider_catalog.json` is the actual embedded input. Third, the OpenSpec archive gate found 17 legacy canonical-spec structure failures outside the five change directories. Applying the phase deltas removed two; the approved corrective pass normalized the remaining 15 and changed one stale GKE scenario so GitHub Actions remains deployment-only.

The plan expected one clean macOS restart. Delivery required additional corrective restarts after the malformed Alibaba credential reference, the stale embedded catalog, and a hung native SurrealDB dependency were observed. Those restarts did not consume additional inference requests. The underlying SurrealDB stall was not diagnosed; only the recovery procedure was established.

## Goal results

| Goal | Result | Observed evidence and limit |
|---|---|---|
| Build and install UAR 1.0.0 `server-full` as a loopback-only macOS LaunchAgent on port 1906. | MET | The installed binary reported `universal-agent-runtime 1.0.0`; launchd reported the job running; `/healthz` returned `ok`; `/readyz` returned `ready`; HTTP listened only on `127.0.0.1` and `::1`, and A2A gRPC only on `127.0.0.1`. This is a bounded macOS observation, not an availability-duration claim. |
| Ship launchd, systemd, and native Windows SCM packaging with graceful shutdown and `.prometheus` logs. | MET within stated platform limits | macOS installation, stop, restart, and log paths were observed. Ubuntu 24.04 accepted the systemd unit only after declared installation fixtures existed. Windows `server-full` compiled for MSVC and PowerShell entrypoints parsed; no Linux or Windows service runtime was observed. |
| Bootstrap least-privilege configuration for the local proxy, Kimi K3, MiniMax M3, Qwen, GLM, and Moonshot when matching credentials exist. | MET | The generated service environment contained only approved canonical names. Existing YAML, selected model, provider state, database authority, and backups were preserved. Alibaba/Qwen, Kimi, MiniMax, Z.AI, and local proxy entries were visible. Moonshot was correctly absent because no matching generated credential existed. No credential value entered YAML or retained evidence. |
| Document native installation and verify short genuine model inference without synthetic or long-running tests. | MET | README, product docs, and branded Docusaurus pages built successfully. Exactly six installed-boundary requests returned real responses: local proxy, Kimi K3, and MiniMax M3 through both API and shipped UI. Each request was limited to 120 seconds and 64 output tokens. No inference result transfers to Linux, Windows, another feature profile, or another model. |

## Delivered changes

1. `establish-native-service-deployment-contract` locked service, configuration, evidence, platform, and preservation contracts.
2. `implement-native-service-runtime-support` added explicit environment-file loading, file logging, shared HTTP/gRPC host selection, provider catalog enrichment, and native Windows SCM cancellation integration.
3. `package-native-service-installers` added lifecycle packages for a macOS user LaunchAgent, Linux systemd, and Windows SCM.
4. `bootstrap-native-provider-model-configuration` added allowlisted credential generation and additive provider/model configuration merging.
5. `document-and-deploy-native-services` completed documentation, the release build and local install, catalog refresh, bounded functional verification, recovery evidence, and archive artifacts.

## Failures and corrections

- An obsolete `QWEN_TOKENPLAN_API_KEY` reference prevented a clean restart. The correction migrated only the observed exact stale values to canonical `DASHSCOPE_API_KEY` and `alibaba/qwen3.8-max`; custom Alibaba configuration remained byte-identical in negative controls.
- Advancing `models.dev` did not update `/api/models`. Source inspection established that UAR embeds the offline catalog, so `liter-llm` was advanced and the deterministic 316-provider snapshot regenerated.
- Current `models.dev` contained two case-colliding filenames and made a default macOS checkout dirty. The pin moved to the newest clean ancestor that already contained Qwen 3.8-Max.
- The Unix merge parser ignored unquoted provider IDs and could duplicate Alibaba. History-blind artifact validation reproduced the defect; the minimal parser correction preserved an existing operator-owned block and emitted one provider.
- Windows GNU cross-checking failed because the pinned ORT distribution has no GNU prebuilt. Target-scoped MSVC cargo-xwin variables allowed host build dependencies to keep the macOS compiler and completed the Windows compile-only check.
- SurrealDB health briefly recovered and then stalled again. A full bootout/bootstrap sequence plus both HTTP health and a real WebSocket `RETURN 1;` gate produced a stable final readiness observation. This does not identify the database stall's root cause.
- OpenSpec could not archive cleanly while canonical specs retained delta-only headings and missing canonical sections. The approved normalization produced `101 passed, 0 failed` from `openspec validate --specs` before the five dated archive moves.

## Architecture integrity

Operator authority remained intact: native bootstrapping adds missing entries and never replaces the full YAML or persisted database. Runtime listeners derive from `server.host`; the macOS install demonstrated loopback binding for both transports. Provider/model visibility continues to use one compile-time catalog rather than an endpoint overlay. Windows SCM Stop/Shutdown enters the same graceful cancellation path as other shutdown sources. Operational logs use platform-specific `.prometheus` directories; database-engine metadata remains database state.

The uncomfortable limitation is that Linux and Windows packaging is shipped but not runtime-certified on those operating systems. The macOS host can support syntax, structure, cross-compilation, and source-trace claims only.

## Technical debt and residual risk

- Diagnose the SurrealDB/RocksDB stall separately. The recovery sequence is evidence-based but is not a root-cause fix.
- Run native Linux and Windows installation, SCM/systemd lifecycle, listener, persistence, and inference checks on those hosts before making platform-runtime claims.
- Only change 5 completed an artifact-refiner lifecycle. Changes 1–4 have strict OpenSpec and scoped static/functional evidence but no equivalent artifact-refiner receipt.
- `provider-model-settings-certification` retains a `TBD` Purpose line. It validates structurally but should be made descriptive when that capability is next changed.
- KBD projection refresh rewrites many unrelated legacy phase projections and can retain a stale `exactNextCommand`. Only canonical event transitions and active-phase projections were treated as authoritative; unrelated generated churn was excluded from the phase commit.

## Coordination and process

The serial worktree sequence prevented sideways merges and kept each later change based on the preceding commit. Functional verification began only after implementation was code-complete. Six real-inference requests replaced synthetic or multi-hour testing, and their provider/model paths were retained. Stop conditions worked: Alibaba configuration, catalog provenance, source cleanliness, and SurrealDB readiness were corrected only after observable failures and operator authorization.

The OpenSpec archive gate was broader than the phase surface. Stopping at that gate was correct because archiving while `openspec validate --specs` failed would have left the canonical contract invalid. The approved fix was structural except for the required GKE deployment-only policy reconciliation.

## Durable lessons

- A catalog submodule pointer is not necessarily the runtime catalog. Trace the binary's actual generated or embedded input before changing source pins.
- Case-fold collisions are a release constraint for repositories supporting default macOS filesystems; a semantically current commit can still be an unusable pin.
- A supervisor PID or one HTTP response is not dependency readiness. For this SurrealDB LaunchAgent, require a clean launch state and a real WebSocket query before starting UAR.
- OpenSpec archive readiness includes the canonical spec estate, not only the change being archived. Canonical specs must use `## Requirements`, while change deltas retain operation headings.
- Cross-compilation variables must be target-qualified when host build dependencies compile during a foreign-target build.

## Next phase recommendation

Do not broaden this completed native-deployment phase. The next product phase should be `uar-1-1-production-remediation` Wave 0: reconcile its proposed requirements against delivered 1.0 code, lock public contracts, and derive current estimates before implementation. A separate bounded child should own the SurrealDB stall diagnosis if it recurs. No successor phase is created or activated by this reflection.

## Artifact quality summary

The one retained artifact-refiner lifecycle initially failed on the unquoted-provider parser, stale README catalog count, and dirty `models.dev` pin. One correction iteration resolved all three and the final history-blind validation verdict was PASS. This is an artifact-quality result for change 5 only, not a product-readiness percentage or cross-platform verdict. Changes 1–4 have no artifact-refiner lifecycle to summarize.

## Sycophancy self-check

This reflection leads with deviations, preserves negative controls and platform limits, names the undiagnosed dependency failure, and does not convert compile/template evidence into Linux or Windows runtime claims. The producer is not the sole judge: a fresh history-blind artifact critic reviewed only this reflection and returned PASS before KBD completion. The retained receipt is `review/reflection/artifact-critic.md`.
