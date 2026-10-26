# Serial platform input preparation

Source only; verification gate **CLOSED**. This controller has not run, been imported, compiled, checked or tested. Authoring it grants no exception to the operator lock. The lead must seal actual private config/approval bytes and exact argv, then obtain explicit operator instruction covering all nested preparation commands/effects. No executable configuration is included.

The uncomfortable limitation is that successful builds/extraction establish prepared artifacts, not platform runtime behavior or a complete composed fixture. Historical closures are usable only when their selected existing trees still match the explicit approved hashes. Missing dependencies, drift, source changes, build failures and incomplete cleanup stop preparation; no retry, additional install, repair, fallback artifact or new provider authorization is inferred.

## CLI and sequence

Exactly one argument is accepted:

```text
FROZEN_PYTHON ABSOLUTE_REPO/scripts/acceptance/prepare_platform_inputs.py ABSOLUTE_PRIVATE_CONFIG
```

This line defines argv grammar; the lead must replace all three with actual absolute paths in the eventual exact approval packet. Import performs no I/O. No shell interprets argv.

The six mandatory `commands` entries run in fixed order:

1. `artifacts`: existing `prepare_artifacts.py` with one frozen private configuration. Its three stage IDs must be exactly `native`, `rust_sdk`, `grpc`; no TypeScript/frontend/classifier guest rebuild or acceptance execution. Unchanged TypeScript emitted output, frontend resources and classifier WASM must use explicitly retained existing trees and their approved hashes. All12 resource closures remain explicitly selected in that helper configuration. Already-produced resources use existing tree paths and adopted tree hashes; no new install/discovery operation is introduced.
2. `docker_build`: selected frozen Docker `build`, exact local argv with one `--iidfile`, one selected `--platform linux/arm64` or `linux/amd64`, absolute repository context. The image identity comes exclusively from the successful build's new iidfile. No tag lookup, pull-only substitution, push, container run or service startup follows.
3. `linux_prepare`: frozen Python plus existing `linux_runtime.py --prepare --config … --approval …`. The controller authors the private successful image build receipt, Linux config and exact extraction approval only after successful build exit and valid iidfile. `image_id` and `image_source_id` are both that actual local image ID. The parent approval explicitly includes this metadata derivation; the derived approval is not an independent operator instruction. The helper inspects the exact image, creates an owned stopped extraction container, copies the Linux binary/Python and removes that exact container. It never starts the container.
4. `ios_prepare`: frozen Python plus existing `ios/runner.py prepare --selection … --timeout-seconds N --udid …`. Metadata selection is constrained to the explicit configured UDID; no simulator boot occurs. Actual selected target comes from `ios-selection.json`.
5. `ios_target`: frozen `/Users/gqadonis/.cargo/bin/rustup target add @selected-target@ --toolchain nightly-2026-07-18-x86_64-apple-darwin`, at most600seconds. This is an explicitly proposed network component installation for only the observed selected simulator target under the pinned toolchain, never a toolchain upgrade or another target install. The lead packet previews the expected observed target as `x86_64-apple-ios`; the actual command expands only the selection receipt. It requires explicit operator installation approval and does not run now.
6. `ios_build`: same runner's `build --selection … --timeout-seconds N --output … --source-inventory …`. Its nested locked SDK staticlib build and Xcode `build-for-testing` use the actual selection. No XCTest execution, app install, simulator boot, service or provider call is included.

The exact helper argv shapes are enforced by source. Their order matches `ORDER` in the controller. Docker's remaining build options are fully explicit in configuration and operator approval, including selected pins/toolchain/build arguments/cache settings. Existing Dockerfile nested build/install/network effects must be named in that approval; this program does not hide them or authorize independent preflight commands. The final context must be the repository's absolute path.

## Exact configuration contract

All paths are absolute, normalized and resolved without symlink components. A frozen reference is exactly `{path,sha256}`, where SHA is `sha256:` plus64lowercase hex. Reference comparisons stream regular files, maximum16GiB/file. JSON inputs are regular, at most64MiB. Configuration and approval plus artifact helper config must be owner-private. Artifact references/receipts are not fabricated.

| Field | Value |
| --- | --- |
| `schema_version` | `uar.platform-input-preparation.v1` |
| `executable` | `true` only in the later actually approved config |
| `approval` | Frozen private approval reference matching the projection below |
| `output_root` | Fresh absolute private controller directory under an existing0700owner-owned parent; lead selects the new008scope; no existing root overwrite |
| `source_inventory` | Frozen canonical current inventory; source regular rows and required helper rows must agree |
| `source_identity` | Exact current regular-source canonical identity, identical to inventory and artifact helper config |
| `remaining_global_seconds` | Positive integer representing actual remaining original global preparation allowance selected by lead; never a new allowance |
| `limits` | `total_seconds` positive integer no greater than remaining global; `output_bytes` positive integer at most67108864 for aggregate captured outer command stdout/stderr |
| `build_owner_lock` | Same existing shared private lock path selected in artifact helper config, outside controller output root |
| `artifact_config` | Frozen private `prepare_artifacts.py` config reference; output must be `output_root/artifacts`; native+RustSDK+gRPC stages only; same source rows/identity/lock |
| `tools` | Object of frozen executable/tool refs; requires `python`, `docker`, `cargo`, `git`, `xcrun`, `xcodebuild`, `xcode_select`, `rustup`; also declare Node or any tool named by an artifact stage |
| `inputs` | Explicit array of frozen other inputs, e.g. toolchain/closure/resource/source-backed manifests; no implicit environment or directory discovery |
| `ios_udid` | Explicit36-character selected installed simulator UUID; prepare must return the same UDID |
| `linux` | Exactly `architecture`, `container_binary`, `container_python`, `extraction_container_name`, `command_seconds`, `output_bytes` as below |
| `commands` | Object containing exactly `artifacts`, `docker_build`, `linux_prepare`, `ios_prepare`, `ios_target`, `ios_build` specifications |

Each command specification is `{argv,cwd,environment,timeout_seconds}`. `argv` is a nonempty string array. `cwd` must equal the repository path. `environment` is an explicit literal string map, never inherited; `PATH` is required and fixed. Names containing API_KEY/TOKEN/PASSWORD/SECRET are refused. No secrets, profile sourcing, shell `-c`, eval, arbitrary shell interpolation, unnamed install preflight or external agent dispatch is introduced. Nested artifact stage executables must match an explicit frozen tool and cannot be shell executables. Artifact helper `environment_env` maps only values explicitly supplied by this controller's artifact stage environment; freeze loader/toolchain/cache paths in that approved map.

Only closed `@image-id@` and `@selected-target@` substitutions are recognized in declared argv/environment values. They are available only after successful Docker build/metadata selection respectively. Any unresolved or other named token refuses. The six enforced helper CLI shapes have no dynamic target flag: iOS build reads its target from selection. The target substitution may be used in an explicitly declared later environment entry; the controller never guesses it. No original budget or credential value is substituted.

Per-stage maxima are14400seconds for artifacts,7200for Docker build,1200for Linux preparation,120for iOS metadata,600for the explicitly selected target installation and7200for iOS build. The configured overall remaining-global deadline may be tighter. Artifact helper's own total must fit its outer stage, and its configured output ceiling must fit the outer configured output ceiling. Its individual build limits remain at most7200seconds under its existing contract. Linux nested command seconds are positive integers at most120 and per-command output at most2097152bytes. iOS helper retains its existing nested bounded logs/probes (16MiB per build log,4MiB per metadata command); outer captured stdout/stderr additionally share the controller's at-most64MiB budget. Nested private build logs are retained by their owning helper rather than relayed into controller stdout.

Linux `architecture` is `arm64` or `amd64`; `container_python` must be `/usr/local/bin/python3`; container binary is the explicit absolute image path. `extraction_container_name` must match `uar-acceptance-linux-extract-` plus32lowercase hex, selected exclusively for this scope. Generated config sets `state_parent=output_root`, `output=output_root/linux-artifact`, frozen Docker client, current source and actual build receipt. Generated extraction approval exactly mirrors existing `linux_runtime.prepare_commands`: image inspect, owned stopped create, inspected actual CID, binary/Python copy, exact removal and absence lookup. Its internal `@owned-container-id@` is resolved only by that existing helper from the actual owned container, never by arbitrary controller interpolation.

Fixed controller-derived paths used by exact command argv are:

- Artifact config path from the frozen reference; output `output_root/artifacts`.
- Docker iidfile `output_root/image.iid`.
- Linux config `output_root/linux-config.json`, approval `output_root/linux-approval.json`, build receipt `output_root/linux-image-build.json`, extraction output `output_root/linux-artifact`.
- iOS selection `output_root/ios-selection.json`, output `output_root/ios-artifact`, canonical inventory path from its frozen reference.

The approval JSON must equal `{operation:"prepare-uar-platform-inputs",operator_approved:true,projection_sha256:SHA256(canonical(config excluding approval)),effects:EFFECTS}`. `EFFECTS` is the exact eight-string constant in the controller. Canonical serialization is sorted compact JSON with finite values, Python defaults, UTF8 bytes. The projection includes every nested command, input, tool, limit, source identity and Linux metadata derivation recipe; no dynamic produced hash is inserted into its own preimage. This local record is not cryptographic authority and must reflect the real external operator instruction.

## Ownership, receipts and stopping

The artifact helper independently acquires the configured common flock during its stage. The controller must not hold that same lock while launching it. After helper success the controller nonblocking-acquires the **same** lock before Docker/Linux/iOS work, retaining it until completion/failure. If another writer wins between stages, the controller fails rather than running concurrently. There is no hidden parallel subprocess or second build target writer. Tools that ignore this flock still require lead scheduling discipline.

Every command has exclusive private stdout/stderr files and a command receipt with exact command, matching approved command, source-bound context, exit/status, output references, timestamps, selected environment names and owned process-group cleanup attempts. No raw output/secret/exception is printed. Timeout/output overflow/failed exit/invalid output stops all later stages. Only the exact created process group is signaled and its leader joined; separate groups created by nested helpers are not falsely certified absent. Linux extraction failure/timeout can therefore leave an owned stopped extraction container whose cleanup needs its retained helper receipts and a separately approved exact cleanup, never a name/prefix purge.

No automatic retry, repair, deletion, image removal, root reuse, provider attempt, platform launch or budget reset occurs. Partial roots and private logs remain. `failed.json` records incomplete preparation and completed command receipts. The final `platform-artifacts.json` binds actual native/SDK/gRPC preparation, Linux image/extracted artifact receipt, iOS selection/artifact receipt, command receipts and current source; it explicitly sets `fixture_composed=false`, `product_verification=false`, gate CLOSED. Actual service/runtime/platform outcomes remain pending.

The lead still owns selected existing closure tree hashes, all toolchain/cache paths, unsigned config/approval packet, external operator instruction, whole-phase inventory, exact later private/supporting preparation and composition. Those operations are intentionally absent until their actual bytes and effects are sealed. No proof of runtime portability, Linux container behavior, simulator behavior, credential correctness, composed fixture completeness or user acceptance follows from authored controller source or eventual build success.

The host Cargo launcher at `/Users/gqadonis/.cargo/bin/cargo` is a rustup symlink and is not a resolved regular tool reference. Keep the regular-file boundary: native/RustSDK/gRPC artifact stage argv explicitly dispatch through the frozen regular `/Users/gqadonis/.cargo/bin/rustup run nightly-2026-07-18-x86_64-apple-darwin cargo`, followed by the original exact build/no-run arguments. `tools.cargo` may identify that actual regular rustup executable with the same hash as `tools.rustup`; Cargo JSON output selection is unchanged. The existing iOS helper's source-defined nested `cargo` lookup uses the explicitly approved PATH and `RUSTUP_TOOLCHAIN=nightly-2026-07-18-x86_64-apple-darwin` environment; that source-backed nested lookup does not relax frozen controller/artifact-stage executable references or introduce a compiler probe.
