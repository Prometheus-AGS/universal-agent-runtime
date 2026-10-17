# UAR 1.1.1 phase integration

This program exercises the known outstanding-issues phase scenarios through the
real runtime, authenticated routes, provider and MCP wire peers and durable
stores. It is run locally after all production source, migrations, fixtures,
configuration and required docs are implemented and the latest whole-phase
inventory is recorded. For the R11 repair cohort that inventory is L9;
historical L4/L8 inventories do not authorize its gates. Missing prerequisites are blocked, not
skipped. No unit, formatting, lint, schema-validation or broad historical
conformance sweep is part of this phase program.

The six required groups are binding, streams, mcp-outcomes, credentials, recovery
and combined. Their exact observable controls are in the phase verification.md
and adapter REQUIRED_SCENARIOS inventories. Controlled peers measure runtime
protocol behavior; they do not establish external-model quality or customer
cluster recovery. Local source integration for internal SDK seams must retain
real provider/storage collaborators and record the exact selected case argv.
The credentials group also requires the compiler REST profile adapter: both
aliases and every registered operation deny before effects in remote or shared
credential mode, with two admitted principals, retained resources after restart,
and a positive local single-owner control. This is a restriction observation,
not proof of owner-scoped compiler storage or closure of #387.

The local runner selects four exact real-host cases, serially in the relevant
groups: `phase_runtime_binding_refusals_use_real_host_store_and_provider` and
`phase_runtime_native_incidental_error_retains_success` from
`bauar_full_harness_cursor`, and
`phase_native_sdk_supplied_credentials_use_real_provider_and_durable_store` and
`phase_memory_credential_migration_uses_real_http_and_atomic_store` from
`credentials_api_integration_test`. It passes `--exact --test-threads=1` and
requires one actual passing case per invocation. These internal SDK/native
observations and the memory HTTP contract supplement the installed-image groups and receive no published-image
or architecture qualification credit.

## Private inputs and ownership

Copy scripts/release/uar_1_1_1_manifest.example.json into the phase evidence/gates
location, remove its template-only fields, and populate real values after L9.
The template is intentionally non-executable. The accompanying schema is a field
reference, not an authorization to run a validator. Runtime checks bind the actual
inventory digest, source/fixture identity, stage, architecture and required groups.

Private infrastructure JSON supplies `surreal` and `postgres` objects with
`image`, `username` and `password`. Surreal must use the immutable 3.3.0 image in
versions.toml. PostgreSQL must use a digest for the supported image with the
extensions required by the shipped schema. The runner creates disposable database
containers/volumes on its own Docker network; these are not live endpoint inputs.
Keep the file outside public receipts with mode 0600. The resource root must be a
new private directory; the evidence root is a separate new retained directory.
No real user credential store, know-me database or deployment namespace is used.
The disposable fixture disables request throttling for its exhaustive sequential
framing matrix. Authentication and run authority remain enabled. Rate-limit
qualification is outside this phase's selected scenarios.

The required qualified-supervisor case additionally needs a disposable isolated
deployment with an existing qualified independent controller. Production
know-me.tools is not an acceptance target. Put these fields only under
`supervisor` in the private infrastructure file: `qualification_file`,
`qualification_sha256`, `identity_receipt_file`, `identity_receipt_sha256`,
`controller_journal`, `runtime_url`, `host_token`, `ordinary_token`, `admin_token`,
`admin_key` and `team_owner_token`. File references are absolute; outer artifact
digests include `sha256:`. The admin token must be an authenticated owner without
`uar_instance_id`; it is distinct from trusted-host authority.

The immutable identity receipt binds the phase, isolated deployment, exact image,
platform, source/fixture hashes, actual runtime image content ID and endpoint. It
also binds the actual predecessor registration and fence, replacement fence and
all prepared team scopes with exact queued/uncertain attempt IDs. The adapter
reads the real controller journal with SQLite `mode=ro` and verifies physical
observations, epoch advancement and retained uncertainty. See the S4-integration
handoff for the complete setup identity contract and the existing
[supervisor recovery procedure](supervisor-recovery.md) for controller provisioning.
The adapter never manufactures or promotes qualification. Kubernetes exclusion
can inhibit the dedicated node; native exclusion requires an observed distinct
whole-host boot. No customer node may share that scratch scope. Absent genuine
inputs block this mandatory case and therefore withhold release acceptance.

The runner generates isolated JWT signing material and an administrative key.
Secrets stay in private resources and are redacted from retained command/log
receipts. Each adapter uses the provided scoped backend clients, owned process
helpers and runtime leases. Only owned containers, network, volumes and peer
process groups are stopped and joined. The private scratch root can contain
fixture credentials and must be removed by its owner when recovery evidence is
no longer needed. Do not publish that directory or the infrastructure input.

Disposable Surreal readiness is determined by HTTP 200 from `/health`, including
an empty successful response. A false probe replaces any older transient error
in the timeout diagnostic. Database startup logs and command receipts remain
separate evidence; a readiness timeout alone does not prove database failure.

## Candidate, published images and repair

From the repository root, the selected invocations are:

```text
python3 scripts/release/uar_1_1_1_acceptance.py --manifest .kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/local.json
python3 scripts/release/uar_1_1_1_acceptance.py --manifest .kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/arm64.json
python3 scripts/release/uar_1_1_1_acceptance.py --manifest .kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/amd64.json
```

The local stage builds through the production Dockerfile and exercises the new
image. Published stage pulls the immutable child, verifies membership/platform
in the released index and repeats all six groups without rebuilding UAR or
mounting replacement runtime binaries. Record the tag source commit, workflow
commit/run, index and both child digests. Companion images need not match UAR's
architecture; never set a global Docker platform override. Receipt labels
native/emulated execution from the Docker host and actual image architecture.

Published manifests also require `publication_receipt`, an absolute `path` and
the file's `sha256:` digest. The referenced JSON contains `schema_version: 1`,
`repository: "Prometheus-AGS/universal-agent-runtime"`, `tag: "v1.1.1"`,
`source_commit`, `index_digest`, integer `workflow_run_id` and
`workflow_run_attempt`, and `children` and `artifact_ids` maps keyed by
`linux/arm64` and `linux/amd64`. Children are complete `sha256:` digests;
artifact IDs are the actual GitHub Actions digest-export artifact IDs.

Before pulling the image, the runner independently reads the remote annotated
tag and its commit, then verifies the successful push run of
`.github/workflows/deploy.yml` has that exact source commit and run attempt.
It downloads that run's `digests-linux-arm64` and `digests-linux-amd64` artifacts
and compares their exported filenames with both receipt children and the actual
released index. A matching local tag or successful product behavior alone does
not establish source provenance. The retained receipt includes the observed run
URL, branch, attempt and artifact identities. The existing deployment workflow
retains digest exports for one day; collect the receipt and run published
acceptance within that window. Missing or expired artifacts block acceptance.
These checks use the [workflow run API](https://docs.github.com/en/rest/actions/workflow-runs),
[artifact API](https://docs.github.com/en/rest/actions/artifacts), and
[Git tag API](https://docs.github.com/en/rest/git/tags).

Source identity hashes tracked and new source/build inputs, including immutable
submodule commits. Fixture identity hashes tests and release scenario sources.
Both use sorted path plus content digests. The latest whole-phase inventory binds those values to the
complete implementation. Changed source/fixtures invalidate the freeze; update
the inventory and run affected integration again. No historical receipt certifies
new source.

Each group writes a retained observation result with mandatory scenario IDs,
positive/negative controls and durable readback. The program retains exact argv,
exit codes, sanitized transcripts, resource identity and file hashes. Exit 0
requires every selected mandatory scenario passed; exit 2 is blocked. Failed or
interrupted results are retained. Independent groups continue serially after a
group fails or blocks, so one missing prerequisite does not erase the other
required observations. Operator interruption stops the program. A single-group repair manifest references the
parent receipt and its digest, discloses changed source and uses a new evidence
root. It cannot replace the original complete program inventory. Broader tests
are not added after a passing gate unless an observed failure requires them.

The uncomfortable limit: correct admission can still refuse the customer profile
when its store is unattested. Isolated restart evidence cannot attest a different
volume, recover missing originals or prove another execution owner was fenced.
Keep those deployment facts explicit in issue and release handoff.

## Interrupted observer reads and owned cleanup

The second local program exposed two separate failures. Its AG-UI observer
requested a settled run without a replay cursor; that subscribes after the last
historical event. The fixture now explicitly requests `last_event_id=0` for
terminal replay. Every observer read also has a 240-second monotonic total
deadline in addition to its socket inactivity timeout. Heartbeats cannot extend
that total deadline. A deadline interrupts the actual socket and the runner
retains redacted partial stream bytes with the failed group. It never converts
a timeout or a closed connection into a successful terminal event.

Retained OrbStack logs separately show StorageFull writes at 09:36:59 CDT on
2026-10-16, followed by a forced VM stop. Recovery/combined did not reach their
product scenarios. Restore local disk capacity and the owned collaborators before
the next authorized program; neither this log nor an available socket qualifies
a supervisor. Container removal return codes are included in cleanup status;
successfully removed runtime leases leave the owned-container registry so final
cleanup does not report their intentional absence as another failure. No global
container/image/volume pruning is authorized.

These repairs are source implementation only until R9 and renewed L7 are complete.
Original local-02 receipts remain failed and immutable.
