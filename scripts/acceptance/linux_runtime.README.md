# Linux runtime binding — source authored, gate CLOSED

This module makes Linux a real server execution target while retaining the macOS controller, SDK/browser clients, host datastore observers, and packet budget proxy. It does not certify all97 cases. No Docker command, import, compile, build, service startup, or acceptance execution was performed when authoring it.

`runtime_platform` is optional. Its absent/default value is `macos-native`; `linux-container` selects the container path in the composer and both backend profiles. The installed `server_binary` and final installed artifact must then be the Linux ELF extracted from the immutable image, rather than the host Mach-O executable. The image ID seals the runtime libraries and Linux Python. macOS client/tool closures remain separate from that Linux server identity.

## Inert binding example

These placeholders grant no authority. Fill exact approved references after the separately approved image build and extraction; retain the full existing bindings contract.

```json
{
  "runtime_platform": "linux-container",
  "linux_runtime": {
    "schema_version": 1,
    "executable": false,
    "image_id": "REPLACE_WITH_ACTUAL_sha256_IMAGE_ID",
    "image_source_id": "REPLACE_WITH_SAME_LOCAL_IMAGE_ID_OR_ACTUAL_REPODIGEST",
    "architecture": "arm64",
    "container_binary": "/usr/local/bin/universal-agent-runtime",
    "container_python": "/usr/local/bin/python3",
    "docker_client": {"path": "/REPLACE/frozen/docker", "sha256": "REPLACE"},
    "image_receipt": {"path": "/REPLACE/outputroot008/linux-artifact/image-receipt.json", "sha256": "REPLACE"},
    "repo_root": "/Users/gqadonis/Projects/prometheus/universal-agent-runtime",
    "readonly_resource_roots": [],
    "host_alias": "host.docker.internal",
    "owned_endpoint_routes": [
      {"scheme": "http", "host": "127.0.0.1", "port": 49421},
      {"scheme": "postgresql", "host": "127.0.0.1", "port": 49403}
    ]
  }
}
```

The example route list is intentionally incomplete. Author every exact owned endpoint from the prepared port inventory: all model, embedding and memory budget proxy ingress ports, structured metadata proxy, negative provider peer, PostgreSQL and Surreal published host endpoints. Include the actual URL scheme (`postgres` and `postgresql` are distinct). The exact route allowlist rewrites only those loopback destinations to the declared alias. It preserves credentials, escaped URL bytes, scope/search-path queries and public upstream hosts. An undeclared loopback URL refuses composition/launch instead of silently using container localhost. Controller metadata and observer store connections keep their actual host loopback origins. Genuine-origin proof compares the corresponding approved runtime alias; it does not infer provider authenticity from the alias.

The Colima hostResolver declaration for `host.docker.internal -> host.lima.internal` is configuration evidence only. It does not prove that a Linux container can reach macOS services bound to127.0.0.1. That route remains a potential blocker until explicitly approved real-path startup/execution proves it. No automatic network rebinding, public exposure, shared service mutation, gateway guessing, alternate credential, or upstream bypass is performed.

## Future approved image extraction

A separately approved image build must produce its immutable image ID and an actual successful build receipt with `source_identity`, `image_id`, `image_source_id`, exact `command`, identical `operator_approved_command`, and `exit_code:0`. A local source-built image may use its own immutable image ID as `image_source_id`; no registry publication or fabricated RepoDigest is required. A registry source must match an actual image RepoDigest. The module never builds, pulls, installs packages, starts the extraction container, or deletes an image/cache.

Author a private regular config file with this inert shape, then request explicit approval for the concrete config hash and full expanded command list:

```json
{
  "schema_version": 1,
  "executable": false,
  "source_identity": "REPLACE_WITH_FINAL_SOURCE_sha256",
  "image_id": "REPLACE_WITH_ACTUAL_sha256_IMAGE_ID",
  "image_source_id": "REPLACE_WITH_SAME_LOCAL_IMAGE_ID_OR_ACTUAL_REPODIGEST",
  "architecture": "arm64",
  "container_binary": "/usr/local/bin/universal-agent-runtime",
  "container_python": "/usr/local/bin/python3",
  "docker_client": {"path": "/REPLACE/frozen/docker", "sha256": "REPLACE"},
  "build_receipt": {"path": "/REPLACE/linux-image-build-receipt.json", "sha256": "REPLACE"},
  "state_parent": "/REPLACE/outputroot008",
  "output": "/REPLACE/outputroot008/linux-artifact",
  "extraction_container_name": "uar-acceptance-linux-extract-REPLACE_WITH32LOWERHEX",
  "command_seconds": 120,
  "output_bytes": 2097152
}
```

The existing parent must be owned and mode0700; output must be new. `prepare_commands(config, output)` defines the exact future packet: frozen Docker `image inspect IMAGEID`; `create --name OWNEDNAME --cidfile OUTPUT/extract.cid --mount type=tmpfs,destination=/var/lib/uar --no-healthcheck --entrypoint IMAGEBINARY IMAGEID`; `inspect @owned-container-id@`; two `cp -L` commands for the image binary and `/usr/local/bin/python3`; `rm @owned-container-id@`; and `ps --all --no-trunc --filter id=@owned-container-id@ --format {{.ID}}`. The placeholder is replaced only with the exact64hex ID retained by that create. Temporary tmpfs replaces the Dockerfile anonymous volume. No shared container selector is used. Failed extraction retains private source/command receipts and attempts only the already approved exact-ID cleanup.

Approval JSON must equal `{schema_version:1, operation:"prepare-linux-runtime-artifact", operator_approved:true, config_sha256:ACTUAL_CONFIG_SHA256, source_identity:ACTUAL_SOURCE_ID, commands:EXACT_PREPARE_COMMANDS}`. Use actual JSON strings and arrays; the notation in this sentence is descriptive. Approval of the entry command includes only that exact listed internal packet; an image build and later runtime execution need their own explicit approval.

The future entry command, with every absolute path resolved in the operator packet, is:

```text
<FROZEN_HOST_PYTHON> <REPO>/scripts/acceptance/linux_runtime.py --prepare --config <PRIVATE_CONFIG_JSON> --approval <EXACT_OPERATOR_APPROVAL_JSON>
```

Preparation validates frozen Docker bytes, the approved prior build receipt, actual image OS/architecture/source, exact unstarted extraction container identity, and both extracted ELF machine types. It emits `uar`, `image-python3`, `image-receipt.json`, matching `build-receipt.json`, and private ownership/command records. Status is `LINUX_IMAGE_PREPARED_NOT_RUNTIME_ACCEPTANCE`. Freeze the extracted Linux ELF, image receipt, matching build receipt, client bytes and all generated configuration before composition. This command has not run.

## Future runtime ownership and cleanup

The wrapper execs the frozen foreground Docker client, preserving the controller's server PID. Docker uses the exact immutable image ID with `--pull=never`, an exact CID file, nonce-bearing run/scope/backend/runtime labels, no image healthcheck side effects, explicit hostUID:GID, and a private0600 env file. Only explicitly selected runtime environment values enter that file; secrets are never command arguments or launch-receipt values.

The image's explicit Python executes the same-path frozen `linux_runtime.py --container-exec` helper, removes inherited image UAR/LLM and legacy persistence/config overrides, then execs the actual attested Linux binary. This is necessary because the current Dockerfile sets a Surreal persistence override that would redirect a PostgreSQL fixture. MCP also uses `/usr/local/bin/python3` and the same repository script path. Neither path installs dependencies.

Mounts are exact: same-path repository RO; same-path owned runtime root RW; owned `root/image-state` RW at `/var/lib/uar` to replace the image's anonymous volume; and any explicitly listed, disjoint readonly resource roots for frozen static/model/WASM assets outside the repository. HTTP/gRPC are published only on host127.0.0.1 at their prepared ports. Actual inspect must match every mount, command/config path, numeric user, mapped port, image ID, immutable container ID and ownership label; start time must correlate with the launch receipt. The helper's own bytes, configuration/MCP bytes and extracted binary remain frozen.

Every restart and final cleanup calls the `runtime_stop` observer before joining the Docker CLI. It inspects the exact immutable container ID and labels, stops and waits for that container, removes only that ID, then proves its absence. Store teardown additionally proves absence for every recorded Linux launch before dropping owned store data. A missing or ambiguous CID/ownership receipt blocks teardown; a dead Docker CLI is insufficient evidence. Containers/images/shared services outside these exact ownership selectors are untouched.

Remaining uncertainties include Colima loopback reachability, macOS bind-mount UID behavior, image Python/standard-library availability at execution, complete current image build success, and all97 functional outcomes. A valid preparation receipt proves extraction/identity only. Only the operator-approved production execution packet can supply runtime evidence.

Per-platform composition also gives the existing-service observer a unique receipt leaf derived from both the shared budget epoch and that composition's absolute output directory. macOS/Linux keep identical budget configuration content and the shared persisted epoch ledger. They do not reuse or overwrite an earlier service observer's exclusive receipt. This changes receipt ownership only; it does not reset the spend ledger or authorize a second service listener.

## Preserved private inputs and two composition outputs

Default composition still requires the exact original private preparation config's `composition_output_dir`. To reuse preserved private inputs/receipt/config bytes for new macOS/Linux output directories, bindings may instead name a frozen `composition_scope_approval:{path,sha256}`. The referenced private owned regular document must have `schema_version:1`, `operator_approved:true`, `operation:"extend-uar-private-composition-outputs"`, the exact frozen `private_inputs` and `private_config` references, current `source_identity`, an existing absolute canonical `output_root` chosen for packet008, and `outputs` with exactly `macos-native` and `linux-container` mapped to distinct canonical direct children of that root. The selected platform's output must match exactly and remain fresh at its invocation. The other selected path may already contain the earlier platform composition; it is never overwritten or used as authority.

Both the frozen final `operator_approval` document and the frozen budget `approval_packet` document must contain `composition_scope_approval` equal to that exact extension reference. Thus its true flag alone grants no permission. Retain the operator signature/resource-authority adoption and final explicit execution approval. The extension document is a derived authority input excluded from the source denominator; it never rewrites an old config/receipt, copies private key values, mints product JWTs, changes the budget epoch, or opens a runtime gate.
