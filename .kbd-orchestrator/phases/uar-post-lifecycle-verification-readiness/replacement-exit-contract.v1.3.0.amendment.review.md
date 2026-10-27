# UAR 1.3.0 — actual amendment and exact execution approvals

Product gate CLOSED. Published source `774469746fee4de99be41dc1cd231857067e4487`; inventory `770fd322db7b4d4ec4209755d3698101d04fcc0f78ee1d649f0eedaa7aa4aa7a`. Canonical337/plan109; native76/78 tasks and OpenSpec81/83; acceptance5.1/review5.2 remain pending.

This Markdown displays the actual generated amendment and complete command/effect proposals. Sign the actual JSON file, not this Markdown. Do not edit or reformat the JSON.

[Actual amendment JSON](replacement-exit-contract.v1.3.0.amendment.json) · [Replacement contract](replacement-exit-contract.v1.3.0.proposed.json) · [Source inventory](v1.3.0-source-inventory.json)

## Actual generated amendment

```json
{
  "action": "amend-contract",
  "approver": "travis@tribehealthsolutions.com",
  "contractPath": ".kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/replacement-exit-contract.v1.3.0.proposed.json",
  "kind": "kbd.scope-authority",
  "namespace": "prometheus-kbd-scope-amendment",
  "newContractDigest": "d00c3c5bd3057ad70ce7324efb6d4476066271081f87833ed811f2929e521551",
  "nonce": "95d43181a199df6771bc719224e20006",
  "phaseId": "uar-post-lifecycle-verification-readiness",
  "previousContractDigest": "dab4675cd1d2889fbe627cf130ff2875ac2b2ab8c0dde1dddafb9042a648d0cb",
  "projectId": "c8d33ac7-e3cd-4db6-b938-1425ec19ee9a",
  "reason": "Operator approves UAR1.3.0 replacement, finite BTHREE disposition and explicit verification-only budget epoch",
  "runId": "uar-outstanding-issues-v1-1-1-20261021T115743Z",
  "scenarioChanges": {
    "added": [
      "BUILD-01",
      "INGEST-01",
      "REVIEW-01",
      "SPEC-01"
    ],
    "modified": [
      "BTHREE-01",
      "CAS-01",
      "COUNT-01",
      "ERROR-01",
      "ERROR-02",
      "HISTORY-01",
      "IDENTITY-01",
      "KB-01",
      "LIAB-01",
      "LIAB-02"
    ],
    "removed": []
  },
  "schemaVersion": "1",
  "timestamp": "2026-10-24T13:04:38Z"
}
```

## Signature, adoption and F/V decisions

# UAR 1.3.0 signature and execution approvals

Prepared against published production source `774469746fee4de99be41dc1cd231857067e4487` (`v1.3.0`), image index `sha256:f67a36f1a6fb886379c6947f605a88945a94481c8b3c4e1d998b9b481cea0697`. Deployment40091743841 passed amd64/arm64 native release builds and manifest merge. Main7ee9bf31 adds evidence only. Publication is complete; no image rebuild/retag is requested.

Active phase Execute, canonical337/plan109: native76/78 tasks, OpenSpec81/83; only issue426/5.1 acceptance and5.2 whole-phase independent review remain pending. The five-row difference is historical caller-tools outcomes. Run144/167 counts changes, not phase tasks. Source/config/fixture authoring is complete; runtime acceptance remains unverified. Product gate **CLOSED**.

## A — approved preparation; actual operator signature still required

Your prior approval A remains effective: finite BTHREE historical-disposition semantics, a new verification-only240-active-minute/240-call/two-repair/three-no-progress allowance with historical effort UNKNOWN, public signing-key registration, and exact template/adoption commands. Publication condition is met. The approved public key was registered at `/Users/gqadonis/.config/prometheus/kbd-scope-allowed-signers`, restricted to namespace `prometheus-kbd-scope-amendment`; no different existing file was overwritten. No private key has been read or used.

The new contract is explicitly newly authored, not reconstructed old bytes. It preserves the original ten obligations and adds INGEST-01, BUILD-01, REVIEW-01 and SPEC-01. BTHREE obligation PASS is accurate historical disposition only; underlying runtime stays NOT_EXERCISED. COUNT remains trusted-builder evidence, with standalone-main exact-fit UNSUPPORTED. Historical effort is not reset to zero; the new verification epoch starts only after signed adoption AND exact F/V instruction. Current canonical consumptionHighWater is empty; preserve future floors.

Review the actual generated amendment at `replacement-exit-contract.v1.3.0.amendment.json` and full replacement `replacement-exit-contract.v1.3.0.proposed.json`. The template computes the new digest and exact scenario changes from canonical binding; no old signature is replayed. Signing is your action, from repository cwd:

```sh
ssh-keygen -Y sign -f /Users/gqadonis/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/replacement-exit-contract.v1.3.0.amendment.json
```

After the actual signature exists, the previously approved adoption command is:

```sh
bash /Users/gqadonis/Projects/prometheus/prometheus-skill-system/shared/scripts/kbd-contract-adopt.sh .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/replacement-exit-contract.v1.3.0.proposed.json --supersedes contract-adoption-uar-post-lifecycle-verification-readiness-dab4675c --amendment .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/replacement-exit-contract.v1.3.0.amendment.json --signature .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/replacement-exit-contract.v1.3.0.amendment.json.sig
```

This verifies the real signature, writes the normalized adopted snapshot and canonical binding, and records the superseding adoption. It does not open product verification. Old bound digest remains `dab4675cd1d2889fbe627cf130ff2875ac2b2ab8c0dde1dddafb9042a648d0cb` until successful adoption. No private-key operation will be delegated to an agent.

## F — explicit isolated fixture/effect approval needed

Approve exact lifecycle A–E and full launcher C in [v1.3.0-isolated-fixture-approval.md](v1.3.0-isolated-fixture-approval.md), and the Issue426 allowlisted launcher in [v1.3.0-verification-packet.md](v1.3.0-verification-packet.md). Those files contain the complete commands, with no implicit readiness/import/source probe.

F must expressly authorize:

- New loopback main49310, controlled peer49311, disk-backed pinned SurrealDB49312 and gRPC49313; main up to3 starts, controlled peer1 start, plus trusted COUNT owned ephemeral loopback peer and awaited cleanup.
- Store `surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20`, owned `target/uar-post-lifecycle-v130-001/store`, main namespace/database `uar_post_lifecycle_v130_001`, and issue426 generated `api_key_<uuid-simple>` namespaces/database `ingestion`. Retain data; no broad drop/purge.
- Public synthetic fixture credentials/JWTs only, allowlisted environment and private fixture HOME/TMPDIR, loopback provider/model/embedding verification and dimension384. No paid service, operator credential/config reuse or remote agent.
- Owner-scoped history/KB/agent/accounting operations, native_echo/local spawn/wait tool decisions, bounded exact-row CAS/event contention (100ms, at most12 competing writes), snapshot-fenced cleanup and actual main/peer joins.
- Your trusted assertion that Docker resolves to a **local daemon** and that the authored mounted RocksDB backend is **disk-backed**, including `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1` for the exact owned fixture. This is permission to assert the configuration, not a durability PASS.
- Exact owned store stop/down D/E even after failure. Preserve store disk/evidence. No volume purge, shared-service stop or unrelated-process cleanup.

## V — exact one-time serial verification/review approval needed

Approve the final source-inventory digest and adopted contract digest, and the full serial packet in [v1.3.0-verification-packet.md](v1.3.0-verification-packet.md): fixture A/B/C; issue426 launcher; fixture D/E; six supported-profile commands; fresh artifact-only Rust and code reviews with their two named local screening commands; seven OpenSpec validations. The main launcher is the only main-boundary invocation; do not add its Cargo children or bare runner again.

The six profiles remain minimal host binary, server-full host binary, embedded-mobile host library, iOS arm64 library, Android arm64 library, and local Linux amd64 Docker build. Publication builds do not waive the separately adopted local build obligation. Missing SDK/toolchain/Docker prerequisite blocks; no hidden install or additional matrix is authorized. Native critics use fresh artifact-only contexts; disclose same-family fallback when no approved cross-family route exists. No paid/remote review dispatch.

F/V must also permit the operational sealing of the proposed manifest: actual `contractDigest`, `sourceInventoryDigest`, operator F/V references, `implementationComplete=true`, `readyToRun=true`, `cleanupAuthorized=true`, and status reflecting actual approval. Only those authority fields may change without new source work; record the final exact manifest SHA in the execution receipt before launch. Do not invent approval references. If adopted stamps or any bound source/input bytes differ from this packet, stop and renew the inventory/instruction.

One initial invocation per named gate. First failure stops later stages NOT_RUN; exact owned cleanup authority remains. Any corrective source/fixture edit closes execution approval and requires a renewed whole-phase inventory and exact failed/invalidated gate instruction. Two authorized repairs, one corrective review pass or three consecutive no-progress checkpoints are hard limits, not permission to auto-rerun. Record actual active time/calls/repairs from epoch start; unknown enforceable consumption blocks closure.

## Completion only after actual exit criteria pass

Use actual receipts to complete canonical5.1/5.2, synchronize/archive the seven accepted OpenSpec changes with normal validation, evaluate typed closure from actual `boundary-results.json`, then write the phase reflection. Exact conditional archive/evaluator commands are in the packet. No task counter, publication result, agent-authored flag or missing evidence can stand in for accepted scenario/safety results. No successor or new publication is included.

The uncomfortable fact: current code compiles and its image is published, but this main/ingestion fixture, cleanup, profile qualification and whole-phase review have not been exercised. Signing alone does not certify them.

## Exact final identities for this approval

- Source: `774469746fee4de99be41dc1cd231857067e4487`; canonical337/plan109.
- Source inventory SHA256: `770fd322db7b4d4ec4209755d3698101d04fcc0f78ee1d649f0eedaa7aa4aa7a` (`32054` project inputs plus `6` explicit public/tool inputs).
- Proposed normalized contract: `d00c3c5bd3057ad70ce7324efb6d4476066271081f87833ed811f2929e521551`; previous `dab4675cd1d2889fbe627cf130ff2875ac2b2ab8c0dde1dddafb9042a648d0cb`.
- Actual amendment: `replacement-exit-contract.v1.3.0.amendment.json`, generated by the approved canonical template. Added BUILD-01/INGEST-01/REVIEW-01/SPEC-01; ten original records modified; no removals.
- Proposed manifest: `boundary-manifest.v1.3.0.proposed.json`; input binding ready, execution/cleanup/approval fields CLOSED. Final manifest SHA is recorded after actual adopted/F/V sealing; this instruction does not authorize changing any source/input bytes.

After signing and successful adoption, the exact approval may state: **Approve F and V for inventory `770fd322db7b4d4ec4209755d3698101d04fcc0f78ee1d649f0eedaa7aa4aa7a` and contract `d00c3c5bd3057ad70ce7324efb6d4476066271081f87833ed811f2929e521551`, including the full bound A–E/main/Issue426 launchers and effects, local-Docker/disk-backed assertions, six profiles, independent reviews/screening, seven validations and conditional accepted completion actions.** This is proposed decision text, not an approval attributed to you.

Source-object reads/hashing authored the inventory. The scoped preparation critic resolved both documentation findings in one corrective review. No product verification, automated screening, service operation, signature or adoption ran. Prepared artifacts are local/uncommitted; no hooks or unrelated checks were invoked.


## Complete fixture lifecycle/effect proposal

# UAR 1.3.0 isolated local fixture — proposed approval scope

Phase `uar-post-lifecycle-verification-readiness`, Execute; waypoint revision337 / plan109. Verification CLOSED. This packet authors a new owned local fixture. No process, container, token issuance, HTTP request, port probe, validator, build, test or signing operation has run. The lead owns canonical inventory, source/image identity, manifest binding and final operator approval. These files cannot grant approval.

The uncomfortable fact is that launch configuration and a durability attestation do not demonstrate durability or passing behavior. The proposed store uses the repository-pinned Surreal image and `rocksdb:/data/surreal.db` on an owned bind mount. `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1` is a trusted supervisor assertion about that exact launch, not a test result. The operator must explicitly authorize making that assertion; omitting it correctly blocks durable scenarios. No existing store or operator configuration is reused.

## Concrete fixture

| Resource | Proposed exact identity |
|---|---|
| Main config | `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-fixtures/local-main.yaml` |
| Store Compose | `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-fixtures/local-store.yaml` |
| Environment names/data | `v1.3.0-fixtures/local-fixture.env.example`; all nonempty values are public synthetic fixture data |
| Main / companion gRPC | `127.0.0.1:49310` / `49313`; owned child only |
| Controlled LLM/embedding peer | `127.0.0.1:49311`, existing `scripts/post_lifecycle/controlled_peer.py`, dimension384, hold20 seconds |
| Owned store | `ws://127.0.0.1:49312`; container `uar-post-lifecycle-v130-001-store`; namespace/database both `uar_post_lifecycle_v130_001` |
| Disk mount | Repo `target/uar-post-lifecycle-v130-001/store` to container `/data`; `rocksdb:/data/surreal.db` |
| Store pin | `surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20` from `docker-compose.prod.yaml` |
| Provider / model | `fixture` / `gpt-4o-mini`; qualified `fixture/gpt-4o-mini`; only loopback transport, public `local-fixture-only` key |
| Embeddings | OpenAI-compatible protocol at controlled peer `/v1`, model `text-embedding-3-small`, dimension384; no OpenAI service request |
| Auth issuer / audience / tenant | Existing synthetic helper constants `uar-client-tools-local-gate` / `uar-client-tools-candidate` / `client-tools-local-tenant` |
| Owner / foreign owner / host | `uar-post-lifecycle-owner` / `uar-post-lifecycle-other` / `uar-post-lifecycle-host`; separate signed fixture JWTs, not issued yet |
| Workspace | `uar-post-lifecycle-v130-001-workspace` |
| Host authority | Exact configured issuer/subject/tenant mapping to `uar-post-lifecycle-v130-001-host`, not inferred from an admin role |

`local-main.yaml` reuses only source-supported fields from `src/config.rs` and the historical `configuration()` constructor in `scripts/release/client_tools_local_gate.py`. Public JWT test secret `synthetic-policy-scenario` is already used in `src/uar/security/authority.rs` source scenarios. This deliberate synthetic value is suitable only inside this newly owned loopback fixture. No operator secret is read, copied or changed. Settings-admin and store values are public fixture-only strings. JWTs have the helper's six-hour expiry and five-second nbf allowance.

Main startup explicitly selects this config. Environment is allowlisted before the runner because `main_boundary.py` and `Boundary::launch` inherit environment. Do not merely append overrides to an operator shell. Private HOME/TMPDIR prevent operator settings/catalog state reuse. No `UAR_ENV_FILE`, `CONFIG_FILE`, proxy variables, provider credentials, legacy LLM variables, remote launch authority or telemetry exporter variables are inherited. Source `main.rs` does not implicitly load `.env`. `UAR_LOCAL_ONLY=true` additionally restricts inference egress to loopback. Memory is disabled; file/web/terminal/session-search tools are disabled; MCP shared transports are disabled so source loads destination catalogs without connecting them. Native echo/spawn/wait remain the explicitly admitted scenario tools. Repository `policies` are read from project cwd as the production server requires.

## Current source bindings; lead must renew before requesting execution

The following fixture values are already authored in the proposal; this is a source description, not an instruction to mutate them again. The lead has renewed sourceInventoryDigest/boundInputs and proposal authority metadata after collecting the complete preparation packet. Actual signature/adoption and F/V instruction remain pending. Current published source is `774469746fee4de99be41dc1cd231857067e4487`; published index is `sha256:f67a36f1a6fb886379c6947f605a88945a94481c8b3c4e1d998b9b481cea0697`. Publication receipt records success for linux/amd64 and linux/arm64, not product acceptance.

1. Main argv is the fixed compiled candidate, `--config` followed by the absolute `local-main.yaml` path, then `--port`, `49310`; expected/max starts3. Peer argv keeps the existing bound Python/script, port49311 and evidence paths, uses string `384`; expected/max starts1. Preserve full peer executable/script byte bindings.
2. Store endpoint is `ws://127.0.0.1:49312`, namespace/database `uar_post_lifecycle_v130_001`, `userEnv=UAR_BOUNDARY_STORE_USER`, `passwordEnv=UAR_BOUNDARY_STORE_PASSWORD`. Bind these authored config/env files and docs with whole-phase source identity. Never bind generated bearer bytes into a manifest.
3. Authored fixture variables are `approvedProviderId=fixture`, `approvedModelId=gpt-4o-mini`. Proposed `existingKbConfig` is `{"embedding_provider":"openai","embedding_model":"text-embedding-3-small","vector_dimensions":384,"file_processor":"kreuzberg","chunk_strategy":"document","retrieval_top_k":5,"retrieval_min_score":0.0}`. These fields are `KbConfigRequest`; document chunking retains each marker-bearing file as a whole. The name `existingKbConfig` is a recipe variable, not a claim about an operator KB.
4. Exact source-framed owner key: `v1:k:issuer:i:27:uar-client-tools-local-gate:t:25:client-tools-local-tenant:s:24:uar-post-lifecycle-owner`. URL-encode that key for `ownerStorageKeyUrlEncoded`. It follows `CredentialPrincipal::storage_key`; runtime matching remains unverified. All captured resource IDs remain driver-produced.
5. The authored proposal includes one source-supported `request` setup step after initial main health: `POST /api/providers/fixture/test`, headers `Authorization: UAR_BOUNDARY_OWNER_AUTHORIZATION`, `X-UAR-Admin-Key: UAR_BOUNDARY_SETTINGS_ADMIN_KEY`, body `{"model":"gpt-4o-mini"}`, status200. Assert `/ok=true`, `/received_text=true`, `/provider_id=fixture`, `/model_id=gpt-4o-mini`. This real protected inference call persists the current model-verification fingerprint. Do not insert a fabricated verified flag or reuse019 receipts. It produces an additional controlled peer model request; initial zero-request probe must remain before this step.
6. Retain the authored D/E files with `UAR_HOLD:kb-race:` and the recipe arm/release controls. Main config dimensions, KB config and peer dimensions all384. Synthetic embeddings are fixture evidence only.
7. `fixtureTerminalEvidenceRef` must label the exact synthetic store collaborator evidence (not attest an actual paid provider charge); e.g. `uar-post-lifecycle-v130-001:synthetic-accounting-terminal-evidence`. The LIAB recipe itself labels direct store usage as collaborator evidence.
8. Keep `implementationComplete=false`, `cleanupAuthorized=false`, and approval references unset until the lead has reconciled the entire phase and the operator explicitly approves the final commands/effects. This document does not set them.

## Exact proposed lifecycle commands — DO NOT RUN

All commands below are proposed for project cwd `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. These are separate approval items; approving the main boundary alone does not approve Docker/setup/stop. No availability/readiness/preflight command is implied. If a fixed port or container name collides, stop and retain that failure; do not probe, stop unrelated processes or silently choose another identity.

A. Prepare only owned directories (does not issue credentials):

```bash
mkdir -p target/uar-post-lifecycle-v130-001/store target/uar-post-lifecycle-v130-001/home target/uar-post-lifecycle-v130-001/tmp target/uar-post-lifecycle-v130-001/ingest
```

B. Start only the owned pinned disk-backed store, without healthcheck or wait gate:

```bash
docker compose --project-name uar-post-lifecycle-v130-001 --file .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-fixtures/local-store.yaml up --detach --no-deps store
```

Compose may fetch the exact pinned image if absent; this is a local fixture pull/start proposal, not an image-publication approval. Bind path is relative to the Compose file and resolves to the exact owned repo target directory. No shared volume, environment file, restart loop, privileged mode or Docker socket is exposed. Store initialization/readiness failure is retained by the approved real gate; no separate curl/docker-inspect/exec/ping is added.

C. Issue only fixture JWTs in memory, construct the allowlisted environment and invoke the exact boundary once. This entire heredoc is a proposed exact command; it has not run:

```bash
/Users/gqadonis/.pyenv/versions/3.11.6/bin/python3.11 - <<'PY'
import os
from pathlib import Path
import runpy
import sys
root = Path('/Users/gqadonis/Projects/prometheus/universal-agent-runtime')
fixture = root / 'target/uar-post-lifecycle-v130-001'
original_home = os.environ['HOME']
env = {key: os.environ[key] for key in ('PATH', 'LANG', 'LC_ALL') if key in os.environ}
env.update({
    'HOME': str(fixture / 'home'),
    'TMPDIR': str(fixture / 'tmp'),
    'CARGO_HOME': os.environ.get('CARGO_HOME', str(Path(original_home) / '.cargo')),
    'RUSTUP_HOME': os.environ.get('RUSTUP_HOME', str(Path(original_home) / '.rustup')),
    'RUST_LOG': 'info',
    'UAR_LOCAL_ONLY': 'true',
    'UAR_BOUNDARY_STORE_USER': 'root',
    'UAR_BOUNDARY_STORE_PASSWORD': 'uar-post-lifecycle-v130-001-public-fixture-store',
    'UAR_BOUNDARY_EXISTING_WORKSPACE': 'uar-post-lifecycle-v130-001-workspace',
    'UAR_BOUNDARY_SETTINGS_ADMIN_KEY': 'uar-post-lifecycle-v130-001-public-fixture-admin',
    'UAR_REMOTE_SURREAL_DURABILITY_ATTESTED': '1',
})
helper = runpy.run_path(str(root / 'scripts/release/client_tools_local_gate.py'))
for key, subject, roles in (
    ('UAR_BOUNDARY_OWNER_AUTHORIZATION', 'uar-post-lifecycle-owner', ['admin']),
    ('UAR_BOUNDARY_OTHER_OWNER_AUTHORIZATION', 'uar-post-lifecycle-other', []),
    ('UAR_BOUNDARY_EXISTING_HOST_AUTHORIZATION', 'uar-post-lifecycle-host', ['admin']),
):
    env[key] = 'Bearer ' + helper['jwt']('synthetic-policy-scenario', subject, roles)
os.chdir(root)
os.execve(sys.executable, [sys.executable, 'scripts/post_lifecycle/main_boundary.py',
    '--manifest', '.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/boundary-manifest.v1.3.0.proposed.json',
    '--evidence-root', '.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-001'], env)
PY
```

`runpy.run_path` loads the already inspected fixture helper without running its main entry. Only its `jwt` function is used; it neither provisions operator IAM nor persists token bytes. Its source bytes must join boundInputs. PATH/Cargo/Rustup cache choices support the declared Cargo children; these are the only inherited toolchain locations. The supervisor sets the remote durability assertion only for this exact new owned rocksdb fixture; that explicit attestation is part of approval C. JWT environment variables are private process memory; no shell export or credential file is produced. Descendants inherit them only as the existing driver requires. The main/peer runtime receipt content must not print bearer values.

D. Stop only the owned store after main/peer teardown receipts are retained:

```bash
docker compose --project-name uar-post-lifecycle-v130-001 --file .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-fixtures/local-store.yaml stop --timeout 30 store
```

E. Remove only the stopped owned container/network; preserve disk data/evidence:

```bash
docker compose --project-name uar-post-lifecycle-v130-001 --file .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-fixtures/local-store.yaml down --timeout 30
```

No `rm -rf`, volume purge, namespace/database/table drop, credential revocation or broad Docker cleanup is proposed. Retained target data remains an explicit cleanup obligation until the operator requests deletion. Main/peer cleanup stays inside the boundary: `/bin/kill -TERM {ownedPid}`, awaited normal exit, bounded emergency termination only with BLOCKED receipt. Docker exit code alone does not prove the main/peer exited cleanly.

## Explicit mutation/effect scope requiring approval

Approval must name A–E and the final bound manifest, its Cargo build/test argv and evidence destination. It must cover this new local fixture authority only: store root login, schema initialization/migrations and settings/provider persistence; actual controlled model verification; main up to3 starts, controlled peer1 start and shutdown, and the trusted COUNT helper's owned ephemeral `127.0.0.1:0` peer with awaited join; all scenario HTTP/gRPC effects through the production main; owned agent create/delete; A–E upload/index/search/replacement/delete; native_echo plus admitted local spawn_agent/wait_agents; direct accounting seed and synthetic evidence reconciliation; exact CAS synchronous event on `agui_runs`, target/contention rows,100ms hold, at most12 competing writes, event removal; owner-separated history reads; exact captured-row cleanup/absence readback. No paid provider, remote agent, operator service/config/credential, or external publication falls within this scope.

CAS scheduling/winner-order observations may fail and must remain BLOCKED. Exact-row cleanup is constrained to the helper's existing allowlist and recorded ownership/full-snapshot comparison; never replace it with prefix deletion. Container teardown preserves the dedicated disk fixture even if row cleanup fails. First failure stops all later product gates. It does not cancel explicitly approved cleanup D/E for the exact owned store, nor authorize a retry, new readiness check, automatic repair or deletion. Run D/E after the main and Issue426 gate on the passing path; on failure stop later gates and retain main/peer teardown obligations before owned store teardown. A timed-out Cargo descendant remains an unresolved obligation; Docker teardown does not prove it exited. Existing timeout/descendant limits in `v1.3.0-fixture-requirements.md` still apply.

## Additional Issue426 effects in the same owned store

The separate serial Issue426 command in `v1.3.0-verification-packet.md` uses `tests/support/api_key_process.rs::Database::shared()`. It creates a generated namespace `api_key_<uuid-simple>` and database `ingestion` on this exact store, alongside temporary embedded SurrealKV databases. This is outside the main manifest's `uar_post_lifecycle_v130_001` namespace/database and must be expressly included in F. The durable target invokes the same test executable in bounded child stages (`unattested` for remote store, then `accept`, `claim`, `publish`), constructs `UAR_DOT08_*` internally, and sets the trusted remote durability attestation only for the staged durable cases. No PostgreSQL fixture is selected by `server-full,test-probes`. No arbitrary namespace authority is inferred for the main driver; its scope remains the exact bound namespace/database. Generated Issue426 namespaces and disk data remain retained in the owned store; no broad namespace drop or purge is authorized.

The operator must assert that Docker resolves to a local daemon for this exact new fixture and that the selected store backend is disk-backed as authored. The proposed command does not establish either claim through a preflight. Store image availability, port freedom, Python executable usability and toolchain/SDK availability remain unverified. No hidden installation or separate readiness command is approved.

## State and remaining decisions

Concrete local config, store pin/mount, public fixture auth source, loopback routes, namespace/database, dimensions and proposed launch/stop commands are authored. No existing operator input is needed to invent an isolated identity. Remaining operator choice is whether to approve this exact new fixture, its deliberately public test authority, local Docker and trusted disk-backend assertions, generated Issue426 namespaces, exact cleanup and final bounded gate/effects. Prior A approval covers authority policy and preparation; actual signing/adoption remains pending and does not grant F/V execution. Lead has bound the current source/image/inventory/proposed contract and authored input bytes, while actual contract authority and F/V remain pending. Any source/manifest edit closes the gate and needs renewed operator instruction.

Source inspection only: `cat`, `rg`, `sed` reads and local heredoc file writes. No acceptance command was executed. Config parsing, image availability, Docker behavior, ports, authentication, provider proof, KB/CAS semantics, compilation, child exit, store release and cleanup are unverified because the gate remains CLOSED.


## Complete verification/completion command proposal

# UAR 1.3.0 whole-phase verification packet

PROPOSED. Product verification F/V CLOSED. Active phase: `uar-post-lifecycle-verification-readiness`, Execute; waypoint revision337 / plan109. Published source is `774469746fee4de99be41dc1cd231857067e4487`. Published image index is `sha256:f67a36f1a6fb886379c6947f605a88945a94481c8b3c4e1d998b9b481cea0697`. The lead must renew `v1.3.0-source-inventory.json`, manifest boundInputs and whole-phase inventory after all packet edits; the current proposal is not runnable. Canonical authorship is76/78 phase tasks complete, with5.1/5.2 pending; run-wide144/167 is a different scope. OpenSpec81/83 differs by five historical caller-tools coordination rows. No old1.2.3 receipt certifies this candidate.

## Current approvals and identities

`v1.3.0-publication-receipt.json` records deployment40091743841 success, native linux/amd64 and linux/arm64 builds and the merged index. All four image tags (`1.3.0`, `v1.3.0`, `latest`, `sha-774469746fee4de99be41dc1cd231857067e4487`) resolve to that index. Earlier failed attempts remain recorded. Publication and the successful local `server-full,postgres-backend` build are separate evidence; they do not certify the runner's `server-full` profile, six supported-profile gates or product behavior.

The authority decisions are in [v1.3.0-unblock-approvals.md](v1.3.0-unblock-approvals.md). Prior A policy approval is conditional on actual operator signature and adoption; no signature, trust registration or contract adoption is inferred. F exact local fixture effects/cleanup and V final-inventory-bound serial product gates/reviews require fresh explicit instruction. The old adopted body remains missing historical lineage. No agent-authored field grants authority. `contractDigest`, approval refs and runnable/cleanup flags must reflect actual adoption and instruction only.

The lead must reconcile the whole phase ledger and all writer handoffs, record the exact final source/input inventory, present remaining work and commands, and wait for the operator. Approval names only the exact commands named; no extra source-identity, Docker, import, readiness, compiler or policy-validator preflight is implied. Any subsequent implementation edit closes approval and requires renewed inventory/instruction.

## A. Existing post-lifecycle main acceptance

Approve the complete exact lifecycle and allowlisted launcher heredoc in [v1.3.0-isolated-fixture-approval.md](v1.3.0-isolated-fixture-approval.md). It prepares owned directories, starts only the pinned disk-backed loopback SurrealDB, issues synthetic fixture JWTs in memory and invokes the existing main boundary entry once. The approved launcher allowlists environment names; it does not inherit personal application/provider configuration. Existing Cargo/Rustup caches are explicitly retained by path. Main49310, peer49311, store49312, gRPC49313; namespace/database uar_post_lifecycle_v130_001, dimensions384.

The entry invoked ONLY through that launcher is:

```sh
/Users/gqadonis/.pyenv/versions/3.11.6/bin/python3.11 scripts/post_lifecycle/main_boundary.py --manifest .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/boundary-manifest.v1.3.0.proposed.json --evidence-root .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-001
```

Its exact declared children, serially with CARGO_TARGET_DIR=target/uar-post-lifecycle-main:

```sh
cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
cargo test --locked --no-default-features --features server-full --test post_lifecycle_main -- --test-threads=1
```

Do not invoke these separately or add a standalone readiness/import/source check. The runner internally binds source/build/boot/fixture identities and owns ten LIAB/ERROR/COUNT/HISTORY/KB/CAS/BTHREE/IDENTITY cases and awaited main/peer cleanup. COUNT is trusted_rust_builder only; standalone-main exact fit stays UNSUPPORTED. The recommended signed replacement makes BTHREE a finite historical-disposition obligation; underlying runner runtimeOutcome remains NOT_EXERCISED and is never relabelled product PASS. Exact scoped store cleanup and bounded CAS schema effects are part of F. Wrong identity, zero tests, missing summary, unexpected child, timeout or failed teardown cannot pass.

## B. Issue426, durable ingestion and lifetime regression

Run only after the main gate passes, while the exact owned fixture store remains available. This proposed exact launcher allowlists the environment for the Issue426 gate too; do not substitute an ambient shell invocation:

```bash
/Users/gqadonis/.pyenv/versions/3.11.6/bin/python3.11 - <<'PY_GATE'
import os
from pathlib import Path
root = Path('/Users/gqadonis/Projects/prometheus/universal-agent-runtime')
fixture = root / 'target/uar-post-lifecycle-v130-001'
original_home = os.environ['HOME']
env = {key: os.environ[key] for key in ('PATH', 'LANG', 'LC_ALL') if key in os.environ}
env.update({
    'HOME': str(fixture / 'home'),
    'TMPDIR': str(fixture / 'tmp'),
    'CARGO_HOME': os.environ.get('CARGO_HOME', str(Path(original_home) / '.cargo')),
    'RUSTUP_HOME': os.environ.get('RUSTUP_HOME', str(Path(original_home) / '.rustup')),
    'UAR_LOCAL_ONLY': 'true',
    'UAR_API_KEY_TEST_SURREAL_URL': 'ws://127.0.0.1:49312',
    'UAR_API_KEY_TEST_SURREAL_USER': 'root',
    'UAR_API_KEY_TEST_SURREAL_PASS': 'uar-post-lifecycle-v130-001-public-fixture-store',
    'CARGO_TARGET_DIR': 'target/uar-post-lifecycle-main',
})
os.chdir(root)
os.execvpe('cargo', ['cargo', 'test', '--locked', '--no-default-features',
    '--features', 'server-full,test-probes', '--test', 'ingestion_admission',
    '--test', 'durable_ingestion', '--test', 'lifecycle_owned_resources',
    '--test', 'run_retention', '--', '--test-threads=1'], env)
PY_GATE
```

F must explicitly include `Database::shared()` generated `api_key_<uuid-simple>` namespaces/database `ingestion`, in addition to temporary embedded SurrealKV fixtures. These are generated by the driver, not unbounded permission for the main manifest. The target spawns its own cleared-environment crash-stage children and supplies `UAR_DOT08_*`/durability assertions internally. The exact owned disk store retains their data; no namespace purge is added.

All four targets must execute nonzero required cases and pass. Record case counts, feature identity, queue/FIFO/oversized/permit-wake results, held-worker settlement, durable publication and owner/observer lifetime. test-probes profile differs from main acceptance. No PostgreSQL functional certification is implied. Afterwards execute owned store stop/down D/E from F, also on failure as required cleanup; do not let cleanup permission restart a failed packet.

## C. Supported artifacts, serially

```sh
CARGO_TARGET_DIR=target/uar-post-lifecycle-main cargo build --locked --no-default-features --features minimal --bin universal-agent-runtime
CARGO_TARGET_DIR=target/uar-post-lifecycle-main cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
CARGO_TARGET_DIR=target/uar-post-lifecycle-main cargo build --locked --no-default-features --features embedded-mobile --lib
CARGO_TARGET_DIR=target/uar-post-lifecycle-main cargo build --locked --no-default-features --features embedded-mobile --lib --target aarch64-apple-ios
CARGO_TARGET_DIR=target/uar-post-lifecycle-main cargo build --locked --no-default-features --features embedded-mobile --lib --target aarch64-linux-android
docker build --platform linux/amd64 --tag uar-issue-426:1.3.0 .
```

Record each binary/library/image identity before another profile overwrites its output path. Missing toolchain/SDK/Docker prerequisite is BLOCKED, not silently installed, waived or replaced by an easier target. One Cargo/build owner per target directory. A successful deployment image may be referenced for its exact source/platform claims, but does not silently waive a separately required local build command; any reuse must be explicitly reviewed in the approval. No Docker run/push/deploy or extra preflight is implied by the final local build. Builds do not prove mobile device, PostgreSQL functional, ARM runtime, WASM, paid-provider or whole-estate behavior. The Boss remains excluded.

## D. Independent artifact-only review and spec validation

V includes fresh independent Rust and code critics, using final artifacts/diff, requirements, source identity and actual A-C receipts only. Reports live under .agent-team/uar-core/reviews/v1.3.0/. They run no extra tests/compilers/probes. Dependency-only review already recorded for publication is not either whole-phase review and earns no product acceptance credit. Preserve review-family and screening limitations honestly; include report screening under the installed protocol explicitly. No remote or paid review dispatch. At most one corrective review pass; unresolved required findings block acceptance.

Exact local spec validations:

```sh
openspec validate pl-main-budget-reconciliation
openspec validate pl-main-compatible-budget-errors
openspec validate pl-main-destination-budget-contract
openspec validate pl-main-persistence-discriminators
openspec validate pl-main-boundary-evidence
openspec validate openai-client-tools
openspec validate update-parking-lot-admission
```

The exact local report-screening commands included in V are:

```sh
prometheus sycophancy detect .agent-team/uar-core/reviews/v1.3.0/whole-phase-rust-review.md --strictness standard
prometheus sycophancy detect .agent-team/uar-core/reviews/v1.3.0/whole-phase-code-review.md --strictness standard
```

These inspect the reports locally; inspected CLI source uses its local detector, without a model API. Record score, classifications and tool status; manually assess report assertions. No automatic rewrite or passing verdict is inferred from exit zero. Fresh native critics use artifact-only contexts; where no approved cross-family route is available, label the same-family fallback and its limits.

No listed product command or whole-phase independent review has run. No formatting, Clippy, full unit suite, audit, additional feature matrix or GitHub Actions test campaign is added. The obsolete tasks.json readiness helper is not part of this packet; typed canonical task coordinates govern.

## Finite exit and receipts

The full newly authored proposal is replacement-exit-contract.v1.3.0.proposed.json; v1.3.0-exit-contract.md provides its readable checklist. Recommended policies require signed approval: BTHREE finite-disposition separation and a new verification-only epoch of240 active minutes/240 calls/two repairs/three consecutive no-progress checkpoints, no new spend. Historical effort remains UNKNOWN separately, excluded only by explicit signed amendment; preserve actual canonical high-water floors. Unknown enforceable consumption inside the new epoch blocks closure. No automatic counter reset or extension.

Initial serial order: fixture A directory preparation, B store start, C main launcher; this packet B Issue426 launcher; fixture D stop and E down; six supported-profile commands in displayed order; approved independent Rust/code review and seven OpenSpec commands in displayed order. Product stages require both F and V plus actual A adoption. D/E cleanup authority survives the first gate failure solely for the exact owned resources. No loop, parallel build, readiness probe or automatic retry is authorized.

One initial invocation per exact gate. First failure stops later stages NOT_RUN. Preserve argv/profile/source/contract/fixture identity, elapsed/exit/output/case counts, actual owned teardown and output artifacts. A corrective implementation edit closes the gate; refresh whole-phase inventory and obtain exact failed/invalidated command approval. Preserve original failures and unaffected applicable evidence; do not repeat passing unrelated gates or expand scope.

Only actual applicable evidence for every contract obligation and safety control permits canonical5.1/5.2 completion, KBD-owned OpenSpec sync/archive, typed phase closure and reflection. Stop there. No successor or extra publication follows implicitly. The uncomfortable fact: publication is complete for both Linux architectures, but the original ten-scenario product boundary, Issue426 regression/profile gates and independent whole-phase review are still pending. Source inspection confirms command paths/selectors only; it cannot prove that those commands pass.

## Source-inspection limits before approval

Runner `BUILD`/`TEST` and manifest use `server-full`; the Dockerfile and successful publication use `server-full,postgres-backend`. Preserve that distinction. `minimal`, `server-full`, `embedded-mobile` and `test-probes` are declared Cargo features; integration target files and all seven OpenSpec change directories exist. Python peer flags, fixed ports, fixture paths, JWT helper and three main/one peer start bounds match the authored proposal. The lead source inventory records current Git-blob/byte bindings from source inspection; the wrapper enforces its bound input hashes at execution. No product or runtime gate ran in this preparation.

The lead renews the proposed manifest source/input bindings for this packet; readyToRun, cleanup and approval references remain closed until actual adoption and explicit F/V instruction. `v1.3.0-fixture-requirements.md` contains historical unresolved-existing-fixture wording; the isolated-fixture packet supersedes those identity choices, while its timeout, ownership, scenario and cleanup constraints remain applicable. Runtime config parsing, authentication, synthetic provider verification, durable eligibility, scheduling, child teardown and cleanup remain unverified. Fixture approval must explicitly authorize the local-Docker and disk-backend assertions; no command here proves them in advance. No fixture/runner source correction was made by this documentation writer.

## Accepted completion actions — proposed only after all exit criteria pass

Record actual scenario and safety results with source/contract identities and measured verification-epoch consumption in `boundary-results.json`. No prefilled PASS or historical runtime waiver is allowed. Complete canonical5.1/5.2 only from those actual receipts, and archive the seven accepted changes through the lead-owned workflow:

```sh
openspec archive pl-main-budget-reconciliation --yes
openspec archive pl-main-compatible-budget-errors --yes
openspec archive pl-main-destination-budget-contract --yes
openspec archive pl-main-persistence-discriminators --yes
openspec archive pl-main-boundary-evidence --yes
openspec archive openai-client-tools --yes
openspec archive update-parking-lot-admission --yes
bash /Users/gqadonis/Projects/prometheus/prometheus-skill-system/shared/scripts/kbd-phase-close-eval.sh --phase uar-post-lifecycle-verification-readiness --evidence-file .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/boundary-results.json
```

Archive retains normal spec synchronization/validation; do not skip either. The evaluator records canonical disposition using the actual adopted snapshot and results. A refusal is not completion. Write the phase reflection only after accepted canonical closure, leading with plan-versus-proof deltas and unsupported/unexercised limits. These are conditional completion actions, not permission to close now.

## Frozen proposed authority

Canonical revision337/plan109; production source `774469746fee4de99be41dc1cd231857067e4487`. The actual unsigned amendment proposes normalized contract `d00c3c5bd3057ad70ce7324efb6d4476066271081f87833ed811f2929e521551` over previous `dab4675cd1d2889fbe627cf130ff2875ac2b2ab8c0dde1dddafb9042a648d0cb`. Actual signature/adoption and F/V instruction remain pending. The proposal is pre-stamped with digest-neutral identity fields using the adoption helper's exact jq serialization; do not revise plan/contract/bound inputs between signing and adoption. Any raw byte difference after adoption requires renewed inventory/instruction. The cover approval file records the final machine-inventory digest; it is excluded from that inventory to avoid self-reference.

