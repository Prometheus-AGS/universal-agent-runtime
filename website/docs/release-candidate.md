---
sidebar_position: 10
title: Use UAR 1.4.0-rc.4
description: Choose, install, exercise, and recover the Linux amd64 release candidate in a controlled environment.
---

# Use UAR 1.4.0-rc.4

Use this release candidate to evaluate the packaged UAR APIs on **Linux amd64** behind a trusted host or gateway. The qualified container profile uses the `server-full,postgres-backend` image and defaults to **embedded SurrealDB 4.0.0 on SurrealKV**. PostgreSQL 18 is a separate restricted lane. Choose embedded SurrealKV for the team and checkpoint scenarios covered here.

Use an isolated environment, synthetic or disposable inputs, and a fresh volume for a first evaluation. For existing data, follow the [upgrade and rollback procedure](./upgrade-guide.md). Keep the previous artifact and verified recovery point available. The [historical RC3 guide](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/v1.4.0-rc.3/docs/releases/v1.4.0-rc.3.md) describes the changes and binds available evidence to its original artifact. The [support matrix](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/product-support-matrix.md) defines the release claims and exclusions.

## Install and check the candidate

1. Use a Linux amd64 Docker host. Obtain the candidate tag from the registry and record the digest supplied by the release operator. Compare it with the actual pulled image. Use that digest for subsequent starts.
2. Prepare the candidate's configuration and secrets on the trusted host. Provision verified identity, workspace mapping, model/provider access, approval policy and an isolated network. The gateway owns credentials for functional scenarios. A test receives the endpoint, workspace and model selectors.
3. Give the candidate a fresh volume mounted at `/var/lib/uar`. The image runs as `65532:65532`; for a host bind mount, grant that identity access only to its owned data. Leave the default storage endpoint at `surrealkv:///var/lib/uar/data/uar.db` for the embedded lane.
4. Bind the API on loopback at the host boundary and attach the trusted gateway. Keep any additional transports private unless their specific admission and policy have been reviewed. Configure the actual port to match your host setup.

```sh
docker pull --platform linux/amd64 ghcr.io/prometheus-ags/universal-agent-runtime@sha256:0b6d94d6a776d7de8abd00085856c68dddd12e12251750f1aab8e1f1dc79780e
docker image inspect \
  --format '{{.Os}}/{{.Architecture}} {{json .RepoDigests}}' \
  ghcr.io/prometheus-ags/universal-agent-runtime@sha256:0b6d94d6a776d7de8abd00085856c68dddd12e12251750f1aab8e1f1dc79780e
```

For a host-managed pilot, set `UAR_IMAGE` to the reviewed digest reference and `UAR_HOST_ENV` to the path to a host-owned environment file containing the effective settings. Keep secrets out of the repository and receipts. Use a new volume and container name for each independent run. The sample assumes port 1906 inside the container and an unused host port 1906.

```sh
docker volume create uar-rc3-pilot-data
docker run -d --name uar-rc3-pilot --platform linux/amd64 \
  --env-file "$UAR_HOST_ENV" \
  --env UAR_SERVER__HOST=0.0.0.0 --env UAR_SERVER__PORT=1906 \
  --publish 127.0.0.1:1906:1906 \
  --mount type=volume,source=uar-rc3-pilot-data,target=/var/lib/uar \
  "$UAR_IMAGE"
curl --fail --silent --show-error http://127.0.0.1:1906/healthz
curl --fail --silent --show-error http://127.0.0.1:1906/readyz
```

Liveness shows that the process responds. Readiness checks its configured dependencies. Through the provisioned gateway, complete an owner-scoped run and read its result back. Use a bounded synthetic provider first. Exercise an approved tool and a denied tool if your application uses tools. Check the terminal event and the actual downstream effect or refusal. Record the image digest, config identity, workspace, run ID and result. See [deployment](./deployment.md) and [run operations](./operations/runs.md).

## Send images and constrain model routing

Send inline PNG or JPEG parts using standard padded base64 on the documented OpenAI-compatible chat path when the chosen route supports vision. Preserve the order of text and images. Remote URLs and arbitrary file references are outside this path. Image transport validates the data source, base64, declared media type and content. The selected capability and policy are applied before dispatch, then the runtime recompiles the request and its context and output budgets for the actual destination.

Ordinary chat using an inherited, verified model can use a configured and authorized fallback when the primary fails before semantic output. Explicit model selections, saved conversation targets and bound calls keep their original scope. A fallback cannot bypass image, tool, format, credential, approval or budget constraints. Once output starts, treat a later failure as that run's partial/error outcome.

Use the [provider guide](./providers/inference.md) and the [HTTP compatibility guide](./protocols/http-compatibility.md) to choose your request path. To evaluate this behavior, configure a synthetic capable peer and a disallowed peer, then compare actual requests. Exercise ordered image parts, malformed base64, an ineligible destination, an inherited fallback, and a fixed target that must refuse fallback. Confirm invalid input causes zero provider arrivals, and compare the successful destination's wire content and budget. Understanding a picture requires separate evidence from a real vision model.

## Recover chat turns and A2UI actions

Use native chat turn IDs when a client must resolve a lost response without submitting the user's intent twice. Send stable `client_turn_id` and `user_message_id` together with the original session. Use the original owner and workspace for a read of `GET /api/uar/chat-turns/{client_turn_id}?session_id=...`. Caller-owned tools use their separate continuation identity. Omit the `tools` field for native no-tools requests; an empty array still selects caller-tool handling.

Render completed history from the canonical session and run order. Preserve distinct assistant segments and media across reload. A disconnect or failed save can leave durable native and wire evidence at different boundaries. Read the original turn and history before rendering a terminal outcome or offering a retry. Report explicit gaps when retained events are unavailable.

Use stable A2UI interaction IDs for clicks on rendered surfaces. An action is scoped to its original owner, run, source surface, generation, component and semantic input. After a response is lost, read the original receipt before considering another action. A changed payload or stale generation must refuse. A repeated read cannot authorize a new effect. Use the [chat guide](./product/chat.md), [A2UI guide](./product/a2ui.md), and [A2UI test guide](./product/a2ui-testing.md).

For a disposable UAR API recovery check, record the original turn or action receipt. After interrupting the client response, read the original receipt and history through the same owner context. Confirm the original IDs and verify no new provider or tool effect occurred. Browser and renderer behavior are outside RC4 qualification.

## Run tools, team tasks and protocol peers

Use MCP/A2A integration only for the implemented roles and transports in their [MCP](./protocols/mcp.md) and [A2A](./protocols/a2a.md) guides. Check capability discovery and approval against the effective policy. A claimed MCP call with lost confirmation remains outcome unknown. Cancellation of its transport does not prove what the peer did. Preserve its original invocation correlation, and use the trusted operator's evidence procedure before resolving it. Shutdown can explicitly refuse with unresolved owned effects; check a separate known-outcome control for clean shutdown.

Team execution uses Surreal-backed persistence. Attempt, role, directed edges and governance determine which controls can be offered. A benign control refusal can be returned as a structured non-effect result. Policy denial, uncertain effects and durable accounting failures require their actual error dispositions. At restart, verify the original settled sibling, original pending interrupted call and the exact IDs of explicit replacement tasks. See [recovery and shutdown](./operations/recovery-and-shutdown.md) and [team control reconciliation](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/v1.4.0-rc.4/docs/operations/team-control-reconciliation.md).

Local embedded accounting serializes the owned provider's admission and usage mutations where they share scope records. The lock ends before model I/O; the accounting transaction retains generation, identity and compare-and-swap checks. Review original durable calls through `GET /api/uar/accounting/calls/{run_id}`. Keep pending liability when the host lacks authoritative terminal evidence. Billing settlement and connector-effect settlement are separate, and neither releases a team reservation automatically. See [cost and accounting](./operations/cost.md).

For the Wasm packaging lane, use a compatible plain component and the declared tool policy. The pinned build/runtime versions align for that component. Treat arbitrary native cache input as untrusted. Governed Wasm execution and AOT certification are separate from staging and direct component checks. See the [component trust policy](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/v1.4.0-rc.4/docs/operations/wasm-component-trust.md).

## Index documents, embed data and use memory

For an owned knowledge base, upload a small document with a unique synthetic fact using multipart field `file` at `POST /api/uar/knowledge-bases/{kb}/documents`. Poll `GET /api/uar/knowledge-bases/{kb}/documents/{document}/ingestion` for the actual terminal state, then retrieve that fact and its source. Set `kreuzberg.ocr_enabled` to enable local OCR for this profile. The packaged English OCR data is available without an extraction-time download. A deterministic malformed input or empty extraction must reach an honest failed state without publishing an indexed generation. A provider's transient failure follows the configured retry budget. Test text, malformed input and a readable scan independently. See the [knowledge guides](./knowledge/overview.md).

When using the embeddings API, inspect usage availability and provenance alongside vectors. Preserve reported zero, unavailable, and provider- or host-observed usage as separate outcomes. Batch index or response mismatches fail before returning partial or mislabeled vectors. A replacement embedding model needs index regeneration even when vector width stays the same. Test the mounted endpoint with independently observed inputs, usage present/absent, and the regenerated index. See the [embedding accounting guide](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/v1.4.0-rc.4/docs/operations/embedding-usage-accounting.md).

Automatic memory injection is independent of memory-tool recall. Enable the service and run policy, enable context injection and configure a finite `memory.injection_min_similarity` between `-1` and `1`. The host bounds retrieval, checks owner and scope, compares current-model embeddings, and rechecks record availability. Omission of the cutoff refuses automatic selection with `memory_relevance_not_configured`. Use a fresh fact, empty session history, and an agent with memory tools disabled. Observe the final provider input. Test no cutoff, below-threshold results, turn opt-out, foreign ownership, deleted data and provider refusal separately. See [agent memory](./memory/overview.md).

## Read usage and telemetry correctly

On the supported Anthropic Messages lanes, distinguish native and bridged requests, streaming and nonstreaming responses, and complete versus cancelled runs. Where the provider reports usage, preserve the observed counters and their origin. Where it omits usage, report unavailable. Use the original run and tool IDs when reading replay. A replay must preserve the original accounting and effect counts. See [provider inference](./providers/inference.md).

Correlate owned run, request, retry and tool attempts across structured logs, metrics and configured tracing. Route labels are bounded and avoid request IDs; use traces or logs for correlation. An embedding host may already own global telemetry. A host-owned metrics recorder cannot be scraped through UAR as if UAR owned it. Confirm what reached your actual exporter and inspect logs without private prompts, tokens or raw tool data. See [observability](./operations/observability.md).

The selected synthetic measurement runs three rounds at concurrency 1 and three at concurrency 4, with eight requests per round. Its acceptance threshold is **zero errors in each round**. Retain semantic response/complete timings, the provider journal, raw result, and the original settled call readbacks. Latency and throughput are observations tied to this workload and artifact. No production latency target, capacity, paid model quality or cross-runtime comparison follows from the result.

## Repeat the existing verification scenarios

Start with the exact tagged checkout, initialize recursive submodules and install its pinned dependencies. Provision the scenario's required services independently. Use only the disposable workspace, controlled provider and endpoint intended for that scenario. Keep a copy of the actual report and record prerequisites, skipped cases and the precise reason for any failure. Run scenario targets serially when they share a host, volume or port.

| Question | Existing repository scenario or check | Evidence to inspect |
|---|---|---|
| Does the exact Rust source compile on all required profiles? | `bash scripts/verify-rust-test-baseline.sh --acceptance` | Clean committed source and pinned submodules, same source before/after, all three profile harnesses. |
| Does the eligible destination actually receive the intended input? | `scripts/behaviors/capability-dispatch.cucumber.mjs` | Mounted policy, final provider request and negative zero-arrival evidence. |
| Are images, embeddings, document ingestion and protocol peers working? | `scripts/behaviors/production-artifact-qualification.cucumber.mjs` and `scripts/behaviors/embedding-usage.cucumber.mjs` | Relevant selected scenario, exact peer, live request and original source/usage or refusal. |
| Does the memory fact reach the prepared provider call? | `scripts/behaviors/memory-context.cucumber.mjs` | Independent fresh-fact witness, owner/scope and cutoff controls. |
| Does owned telemetry preserve actual attribution? | `scripts/behaviors/telemetry-ownership.cucumber.mjs` | Owned recorder/profile, structured evidence, original retry/tool facts and privacy. |
| Does team execution recover honestly? | `scripts/behaviors/team-follow-up.cucumber.mjs` and `scripts/behaviors/team-recovery.cucumber.mjs` | Original settled and uncertain records, actual stop/reopen, exact provider arrivals. |
| Did all 48 controlled requests meet the threshold? | `scripts/behaviors/runtime-measurement.cucumber.mjs` | Frozen manifest, all six zero-error rounds, unique completed requests and read-only settlement. |

After provisioning a supported fixture, pass its Cucumber configuration to the installed runner. For example, the measurement configuration requires its frozen manifest and a separately provisioned controlled peer.

```sh
UAR_BASE_URL="$UAR_PILOT_GATEWAY" \
UAR_WORKSPACE_ID="$UAR_PILOT_WORKSPACE" \
UAR_RUNTIME_MEASUREMENT_MANIFEST="$UAR_PILOT_MANIFEST" \
  ./node_modules/.bin/cucumber-js \
  --config scripts/behaviors/runtime-measurement.cucumber.mjs --fail-fast
```

## Preserve and recover data

Stop every owner and freeze writes before taking a cold snapshot. Inventory all enabled embedded stores, including a separately configured memory store. Ordinary 4.0 startup refuses an unconverted eligible 3.3 store. Convert only a fresh separate destination with the companion runner from the matching source and lockfile, keep the original untouched, and demonstrate parity and clean reopen before cutover. On rollback restore the retained 3.3 recovery point into a different destination with the prior image. Record accepted writes on the candidate and reconcile their disposition before restoring service. Follow [upgrade and rollback](./upgrade-guide.md) and the [embedded SurrealDB upgrade runbook](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/v1.4.0-rc.4/docs/operations/embedded-surrealdb-4-upgrade.md).

Use [backup and restore](./backup-and-restore.md) to plan separate datastore, uploads, configuration and policy recovery. Full harness uses `process_ephemeral`, `unsupported_after_restart`, and `steer_supported:false`. Durable native session or team records do not turn a full-harness run into a restartable execution. On PostgreSQL, read `GET /api/uar/persistence/capabilities` and use the supported owner-scoped paths. Team execution, follow-ups, schedules and observers have capability limits. Checkpoint resume returns the distinct `checkpoint_resume_unsupported` error. The independent Surreal-backed memory service retains its own boundary. Keep a separate owned profile for each backend.

## Evidence to carry forward

RC4 passed the selected Linux amd64 UAR API checks, including streaming, cancellation, owned restart, documents, independent MCP and A2A, and old-epoch refusal. Signed archives, checksums, image and attestations verified. Earlier supported migration and restricted PostgreSQL receipts retain their original artifact identities. Other architectures, browser and application UI, governed WASM/AOT, native runtime installation and real-provider quality are outside the qualified scope. [Signed release](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/tag/v1.4.0-rc.4) · [Qualification record](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/465#issuecomment-6875246846).
