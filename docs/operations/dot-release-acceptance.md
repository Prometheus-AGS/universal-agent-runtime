# Dot acceptance operations

Status: source authored, pending testing. The uncomfortable fact is that none of the runner, production paths, installed images or qualification resources has been verified for this combined source. Source assignments are not passing evidence; missing infrastructure and any failed observation block certification.

The lead owns `.kbd-orchestrator/phases/dot-feature-support/implementation-sequencing.json`. No command below is authorized until the lead records every production entry, including this runner, as source complete in `production-complete-inventory.json` **and an explicit testing continuation authorizes execution**. Code complete/pending testing keeps `verification_authorized: false`; inventory creation does not start a test. The inventory binds the sequencing digest, all production change/backend task pairs (24 after the additional owner correction), handoff files, exact Git head, dirty patch, untracked source hashes and recursive submodule states. `verification_authorized: true` alone does not open the gate. Do not execute the fingerprint helper during source authoring. Root must record this identity at the actual boundary.

## Entry point and boundaries

After the boundary, the single entry point is:

```sh
python3 scripts/dot/acceptance.py STAGE --inventory /absolute/private/inventory.json --config /absolute/private/config.json --evidence /absolute/private/new-evidence-directory
```

Choose a new evidence directory for every invocation. Files are private, exclusive-created and append-only. Never replace a failed receipt. Stages run serially. There are no user-defined shell commands or scenario callbacks.

1. `candidate-build`: fixed frozen pnpm installs, frontend build/static validation and locked release Cargo `server-full` build. `candidate_build.target_directory` owns one build directory. It records `candidate-runtime-artifact`; its receipt is `candidate_build_receipt` for later local stages.
2. `source-integration`: complete real integration targets for credentials, API keys, tool outcomes, stream protocol, fresh-process resume, approvals, ingestion, accounting and memory. This is source evidence, not independently installed conformance. Existing storage-only subcases never substitute for the installed composition stages.
3. `journeys`: real local SurrealKV and two actual installed know-me runtime Pods on one durable networked SurrealDB. Includes credentials/revocation, budgets, memory controls, actual worker ingestion, logical instance/schedule/follow-up, real Required effects, A2A, AG-UI, discovered tools and MCP elicitation. MCP ends with abrupt host loss; only cold interaction reads and ingestion worker work follow it. Same-boot logical recovery is not fabricated.
4. `streams`: independent profile data; valid fragmented Unicode then incomplete EOF. The incomplete call retains liability after a joined restart and must deny another provider dispatch. This stage deliberately leaves an unresolved call and never manufactures usage reconciliation.
5. `settings`: independent fresh default and configured hosts; real no-model outcome, packet-captured no remote egress, actual listener inventory, CORS and persisted/live settings boundaries.
6. `calendar` and `memory-capture`: separate isolated Linux VM stages with real guest/database clock changes. Read `docs/dot-calendar-qualification.md` and `docs/dot-memory-capture-qualification.md`. Never use the developer host, shared remote database clock, container-only fake clock, or application test clock. Tokens/certificates must remain valid across the documented calendar range. Guest clocks are restored on cleanup.
7. `supervisor`: dedicated native service or qualified Kubernetes controller cases. Native preparation deliberately blocks until an independently authorized operator reboot of the named dedicated machine. Completion requires stable machine identity, changed boot identity, dead original child and real exclusion. No automatic reboot exists in this runner. Root controller security stage uses an actually trusted-CA but unallowlisted client certificate; a network error is not a successful rejection.
8. `recovery-faults`: staged real controller/persistence/network faults. Read `docs/operations/dot-recovery-fault-qualification.md`. Preparation/offline observations are blocked stages until measured restoration. The separate `stale-epoch` whole target exercises actual production closed-epoch writes after qualified physical exclusion; it does not use a public executor-authority hook.
9. `governance` and `upgrade`: dedicated profiles; their owned module documentation specifies actual current-authority, warning, prior-version binary and setting inputs. Governance Off never removes descriptor Required, Ask/Deny or caller restrictions.
10. `reconcile`: reads the checked-in matrix, exact same-inventory receipts and immutable observation files. Any incomplete driver, absent required observation or missing profile blocks acceptance. `reconciliation.boundary` is `local_product` or `published_product`; `receipts` is an explicit list of absolute `receipt.json` paths. Published acceptance also requires `local_acceptance`, `customer_build_receipt` and the separate `clean_release_inputs_receipt`.
11. `customer-build`: only after complete local reconciliation. `customer_build` supplies `local_acceptance`, `boss_checkout` and exact new output file paths in `artifacts`. Runs the required `pnpm build:mac:arm64` locally and records command, result and actual artifact checksums. Existing output is rejected to prevent stale artifact credit.
12. `installed`: independently installed, actually published runtime. Supply `artifact_boundary: "published"` for each additional calendar/settings/protocol qualification stage so it also verifies published provenance. All nine issue379 requirements must pass against that artifact; prior local source evidence is only a prerequisite.

No stage publishes, signs, promotes, deploys, reboots, installs a service, changes a firewall or starts a GitHub Actions product suite. Deployment-health evidence is a separate deployment operation and is not inferred from product acceptance.

## Private configuration

Top-level `profiles` must contain `local_surreal_kv` and `shared_two_runtime_surreal_db`. Allocate distinct data namespaces per stage. Each profile's `hosts` maps stable labels to one local or two shared Host records:

- Local Host: `base_url`, `workspace`, `binary`, `binary_sha256`, `config` (existing private runtime JSON), `working_directory`, `lifecycle: "child_process"`. Installed shared Hosts use the exact `kubernetes` contract in [installed Pod qualification](dot-installed-pod-qualification.md).
- `credentials` maps `owner`, `other_tenant`, `other_issuer`, `admin`, `host` to environment variable **names**. Tokens are genuinely issued by the configured trusted issuer. Collision principals use the same subject with different verified tenant/issuer. Host accounting authority is a real trusted host credential, never an admin-role substitute.
- `environment` contains explicit non-secret child configuration; `inherit_environment` names the minimum necessary secret variables. `settings_admin_key_env` is optional only for legacy per-key administrative mutation routes. Effective settings require Bearer admin alone.

The shared profile requires identical actual persistence configurations, remote durability attestation and `durability_evidence` pointing to retained infrastructure evidence. A running Surreal process is insufficient: its underlying storage must be durable. Each runtime's memory endpoint/namespace/database must match. No embedded in-memory backend qualifies.

Composition profiles also require `provider_port`, `artifact` (real admitted agent artifact), `run_credentials`, `deployment_binding_id`, `effect_trace`, `credential_provider_id`, `a2a_agent`, `agui_agent`, `effective_settings_expected`, `secret_marker_environment`, and configured real MCP servers. The controlled provider is an upstream wire peer: only the runtime executes host tools, makes policy decisions or persists authority. Its trace independently counts dispatched model requests. The effect fixture independently fsyncs real external effects.

Configure `tests/fixtures/dot17_effect_mcp.py TRACE` as a real stdio MCP server and permit its actual tool descriptor in the artifact. It can emit success, application failure, transport-unknown effects, or hold a detached descendant. Dedicated supervisor fixtures must preserve the source layout so the effect child can import the production controller identity functions. Linux container images need an image-local `/etc/machine-id` and visible boot identity; do not add a forbidden privileged hostPath merely to supply it. Node identity comes independently from the node observer.

`mcp_transports` must include both fixed names `stdio` and `http`. Each record has `name`, `transport`, `server_name`, exact prefixed `elicit_tool`/`effect_tool`, absolute `trace`/`disconnect_file`; HTTP additionally has `http_port`. Configure the real runtime to use `dot19_mcp_peer.py` with matching paths/transport. Optional per-entry `artifact` and `run_credentials` select admitted capabilities. The helper owns HTTP peers; the runtime owns stdio peers. An accepted elicitation never stands in for the separate Required effect approval.

The source integration stage requires the concrete environment variables listed in `acceptance.source_gates`; these include actual Surreal/Postgres endpoints and retained Dot04–07 infrastructure manifests. Missing inputs block before invoking Cargo. Ambient child-test selectors are rejected so the parent production lifecycle cannot be bypassed.

## Publication identity

Each profile's `publication` contains `tag`, `source_revision`, `release_manifest_url`, `installed_binary`, `archive_name`, `downloaded_archive`, `image_index_url`, and `image_manifest_urls` with `amd64` and `arm64`. Optional `registry_token_env` names an environment variable. The manifest/index/architecture metadata is fetched from actual HTTPS publication endpoints and hashed. The installed executable must be outside the source checkout and match the selected published archive's executable checksum; the downloaded archive checksum must match too. Both Linux architecture manifests must match the actual published OCI index. No local build is presented as publication. Published evidence requires committed source, including the runner and submodule states.

Signatures, release policy resolution, publication execution and narrow deployment health remain separate required release operations. The runner reports exactly what it observed; it does not turn a supplied receipt string into deployment authority.

## Evidence and unresolved work

The matrix is `.kbd-orchestrator/phases/dot-feature-support/implementation/dot17-scenario-matrix.json`. It enumerates 109 spec scenarios and nine issue379 requirement groups. `driver.implementation` describes source completeness. `status: not_executed` remains until actual evidence exists. Source integration assignments have explicit `source_only` scope. Partial drivers carry a concrete `source_gap`; clearing that field requires implementing the missing action and observation, not changing a label.

The implementation handoff separates completed source from unavailable execution prerequisites. Missing prerequisites or failed controls prevent acceptance even when a driver is authored. No compilation, syntax check, test, formatting, product review, build, publication or infrastructure mutation was executed during source authoring.

## Additional source-authored stages

`release-inputs` invokes the repository's real toolchain-policy and release-input provenance programs against the immutable source; mismatch blocks without editing policy. `oci-build` uses `oci_build.builder`, an already installed named Buildx builder, and optional `github_token_file` to build Linux amd64 and arm64 into a local OCI tar. It records actual BuildKit metadata/provenance and hashes every exported layer and both architecture manifests. It never pushes an image. Flags follow the [official Buildx build reference](https://github.com/docker/buildx/blob/master/docs/reference/buildx_build.md), retrieved through Context7 during authoring. Mutable distro downloads mean this is locked-input provenance, not a byte-for-byte reproducibility claim.

`oci_build.cargo_cache_key` is required: supply the actual authorized source archive's SHA-256 as 64 lowercase hexadecimal characters. The runner passes explicit `--build-arg UAR_CARGO_BUILD_JOBS=1` and `--build-arg UAR_CARGO_CACHE_KEY=<configured-value>` and records both in the existing command observation. Validation checks the key's format only; it does not hash an archive or prove that the current checkout equals its contents. The operator must bind the archive and current authorized inputs separately. Changing the source archive requires its new digest; retaining an old key just to avoid compilation misidentifies this namespace. No arbitrary build arguments are accepted.

Cache reuse requires the actual same surviving backend/cache state and the Dockerfile's same toolchain, architecture, build profile and key. A named local builder does not transfer cache bytes from a private Pod, and an amd64 target cache cannot supply independently materialized arm64 code. Jobs1 bounds each Cargo invocation; separate platform solves and architecture-specific locked cache mounts do not establish a global concurrency or memory limit. Confirm the selected existing backend's actual resource and platform support before the authorized gate. This wiring neither configures a builder nor guarantees cache hits, zero recompilation or prevention of every OOM. Both platforms, maximum provenance, frozen/locked build paths and exported-content verification remain required.

`followup-clock` uses the same explicit disposable guest authorization as calendar. Its fixed range is November 7, 2027 02:50–14:00 UTC. It covers Chicago 22:00 quiet entry, both actual 01:30 fall-fold instants, exact 07:00 release, current policy edited to 08:00, and all scanners killed after the durable source/defer commit before one logical inbox delivery. External connectors remain unsupported; this stage makes no external-delivery recovery claim.

`unsupported` uses `spec.unsupported` and the separately owned unsupported qualification guide. `deployment-health` is read-only against already deployed selected endpoints and requires `artifact_boundary: "published"`. It records HTTP health, matching card version and anonymous private-route rejection. It does not claim rollout inspection or the remote process's running image identity from HTTP health.

The planned concrete final environments are this machine and the existing `know-me` LKE context through its Envoy gateway. This document does not authorize provisioning, deployment, service changes, guest clock changes or reboot. Source templates distinguish unfilled prerequisites from observed qualification.

## Completion boundary and installed Pod extension

The latest plan has 24 production task pairs. The guard derives the exact pair
set from the source-bound sequencing ledger; an old 23-entry receipt cannot
satisfy the expanded phase. The lead inventory must still explicitly authorize
testing after source completion. `verification_authorized: false` is the normal
code-complete/pending-testing state and cannot launch any stage.

The installed Pod adapter, controlled peer resources, private configuration
shape and source/publication binding are documented in
[installed Pod qualification](dot-installed-pod-qualification.md). `k8s` is
included in the source fingerprint. The normal shared `journeys` stage requires
two actual named runtime Pods and both independent effect traces. Separate
stage overrides retain the originally required dedicated VM, supervisor,
controlled clock, prior-version and fresh settings qualifications.

Additional concrete stages are `schedule-release`, `http-capabilities` and
`stale-epoch`. The HTTP stage reconciles actual available discovery operations
against same-source live and protected supervisor receipts. The stale epoch
whole integration target requires actual source-bound supervisor exclusion
artifacts for each profile; the local runtime must first be stopped and joined
before the target opens the same embedded store. It attempts real closed-epoch
storage writes and expects the specific production `ExecutorClosed` rejection.
These are later local product gates, never deployment checks or CI jobs.

`oci-build` writes both architecture OCI content locally using the inventoried
Dockerfile's frozen/locked build paths; it never pushes. `deployment-health`
remains narrowly separate from installed product conformance. A currently
running tag, prior source result, or successful HTTP health response cannot
substitute for final published tag/source/index/architecture/archive identities.

Use private storage fault case name `storage-commit` for both backend profiles;
the matrix requires its actual `storage-exercise` and `storage-cold` completed
observations. Local `embedded_enospc` uses a bounded disposable guest volume and
requires actual upload and settlement transaction ENOSPC errors. Shared network
storage uses observed real connection failure. Uncertainty blocks either case;
free-space readings or a deliberate failed probe alone never prove the runtime
transaction failed. See the recovery fault qualification guide for the exact
private observer and staged operator transition contract.

The node/controller partition private case name is `supervisor-node-partition`.
Its restored-stage receipt contains the retained measured offline refusal and
subsequent qualified exclusion. The scenario matrix does not interpret a deleted
Pod or lost HTTP connection as physical exclusion.

Stage overrides may also supply a `candidate_build_receipt` or
`oci_build_receipt` for that stage's actual source-bound architecture. A developer
machine executable cannot stand in for an executable installed in a Linux clock
VM. The receipt must have the same full-phase inventory and the actual boot path
and checksum; the runner does not infer compatible builds from one source tag.

## Candidate inputs and commit-only finalization

Local candidate qualification can precede the commit required for clean release
inputs. `oci-build` saves an exact copy of the authorized full-phase inventory
and invokes the fixed provenance generator with `--candidate-inventory` pointing
to that copy. The generator permits the inventoried dirty root only in this
mode, still requires clean pinned submodules, and checks actual policy, versions,
locks and workspace sources. The runner requires `input_boundary: local_candidate`,
the copy's checksum and the exact candidate source identity. This emits
`candidate-locked-inputs`; it does not satisfy `locked-release-inputs`.

After all local product gates and the Boss customer build pass, a commit that
changes only Git identity can retain that evidence. The fixed `finalize-source`
stage requires a new authorized inventory for the now-clean source. Its
`source_finalization` configuration contains absolute paths `tested_inventory`,
`tested_acceptance`, `tested_reconciliation_receipt` and
`tested_customer_build_receipt`. These name the original retained files, not
rewritten copies with a new source revision.

`source.content_sha256` binds actual scoped file paths, bytes, executable bits,
symlink targets and recursive submodule content and pins. Root Git HEAD and
tracked/untracked labels are excluded from this content digest, while the normal
inventory continues to bind both. Finalization requires identical content and
submodule pins, unchanged production list, sequencing and scenario matrix, clean
current source descending from the tested checkout, the complete original
evidence-file closure and unchanged customer artifact bytes. Any content change
blocks this path and needs the affected planned gate under a new inventory.

The resulting `source_finalization/finalized-acceptance.json` keeps the complete
`original_tested_acceptance` and `lineage.tested_source` alongside the separate
`lineage.finalized_source`. It records metadata equivalence, not new product
execution. Later published reconciliation uses this file as `local_acceptance`
and the retained original customer receipt as `customer_build_receipt`.
The finalization receipt and observation must also remain intact.

Run the separate `release-inputs` stage against the final clean inventory and
provide its receipt as `clean_release_inputs_receipt`. Published reconciliation
checks its actual provenance file and release boundary; candidate provenance
cannot replace it. All nine issue379 requirements still require independently
installed published-artifact conformance under the final source identity. A
commit-only metadata receipt never promotes a candidate image to publication.

MCP qualification includes both real transports' native child/root isolation,
ordinary timeout suspension, actual ten-minute input lifetime, cancellation,
request loss and cold expiry. HTTP additionally exercises actual admitted grant
revocation/expiry and independent request-stream abandonment while the peer
process remains alive. The configured HTTP entry requires a real authorized
`credential_grant` template and both transports require the registered child
artifact described in the MCP child guide. Allocate at least 2700 seconds for
the MCP block per profile; elapsed-time cases do not alter the runtime clock or
extend credential authority. See [MCP peer qualification](dot-mcp-lke-qualification.md)
and [MCP child qualification](dot-mcp-child-qualification.md).


## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


### Linux candidate executable from the OCI build

The local OCI build exports the actual regular ELF runtime executable from each verified image filesystem. Its `local-two-architecture-image` observation records `source_content_sha256` and `runtime_binaries` for amd64 and arm64, including each executable path, SHA-256, size, architecture and image digest. These are candidate artifacts; no publication is claimed.

For Linux guest qualification, install the selected arm64 executable and run the source-bound acceptance driver inside that guest. Copy the genuine same-inventory OCI receipt, observation, archive and exported executable without relabeling their paths or results. The host configuration selects `candidate_runtime_origin: "oci"`, `os: "linux"`, `architecture: "arm64"` and the genuine `oci_build_receipt`. The runner checks the receipt/source binding, archive, exported executable, actual installed executable bytes and current host architecture. The guest's installed path may differ from the export path. Native macOS candidate validation retains its existing local build receipt. Source/plan inventory and every operational stage still require their actual inputs and observations; exporting an executable proves no recovery behavior.


## Superseding delivery acceptance — operator decision 2026-10-14

The operator directed that code-complete UAR delivery stop being blocked by qualification suites and instead be accepted through actual use by agents and dependent projects. The 109-case qualification matrix, synthetic/conformance/fault suites, native reboot and clock/disk qualification, and review-suite certification are no longer prerequisites for this release. Preserve their actual historical passes/failures and unexecuted status. This decision does not mark a cancelled check passed and does not remove any implemented feature or any of the nine reliability or nine issue #379 requirements.

The release exit criteria are usable production implementation, a build of the actual current source, installation on existing authorized capacity, and a useful task executed by an existing configured agent/provider through UAR. Record the real turn, the actual tools/approvals used and their independently observed result, memory/document/accounting persistence actually exercised, exact artifact/source identity and consumer availability. Do not force unused features into artificial scenarios. Report unused protocols, adversarial edge cases and crash/reboot guarantees as unverified rather than blocking unrelated consumers. Keep actual defects, failed builds and failed consumer use actionable; code presence alone is not a passing consumer result.

Necessary artifact building and deployment health remain delivery work. All production implementation and this scope reconciliation precede any build or consumer-use execution. Gates remain serial; no new qualification scripts, matrix expansion, cache-copy workaround or paid resources. Existing node/volume/VM approvals and immutable cleanup deadlines remain. Independent teammates may inspect and fix observed code problems without making their review a new release gate. Merged-main releases for other projects may proceed using their explicitly selected source; unfinished Dot qualification is not a dependency-wide freeze.

Cargo cache IDs are reusable namespace identifiers, not source-attestation hashes. The delivery may reuse the genuinely populated existing architecture/toolchain/profile namespace. Actual source identity, dependency pins, Cargo fingerprints and produced artifact digests remain independently recorded. Do not relabel an old source archive, copy compiled outputs under a fabricated cache identity, prune caches or claim reproducibility from cache reuse.
