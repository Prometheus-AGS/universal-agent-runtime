# Supporting input preparation

`prepare_supporting_inputs.py` replaces the failed M1 ad-hoc supporting fixture authoring with one explicit, frozen configuration. It uses only Python's standard library. Importing this file has no effects. **The operator verification lock remains CLOSED. Authoring this script does not authorize running it.**

The uncomfortable fact is that existing partial007 inputs, historical source010, packet002's validity window and retained006 recipes cannot establish current runnable authority. The lead must bind current source bytes, a new packet with approved current validity, actual installed Python/runtime metadata, and explicit preparation permission. Nothing here repairs or overwrites partial007.

## Invocation and ownership

The CLI accepts exactly one argument, an absolute path to an owner-private JSON configuration:

```text
python3 scripts/acceptance/prepare_supporting_inputs.py /absolute/path/to/private-config.json
```

The eventual executable must be the operator-selected frozen Python, replacing `python3` in the actual approval command. This is a CLI grammar example, not an approved command or existing configuration. The lead seals the actual absolute argv in the approval packet; no profile sourcing, helper import, process launch or shell evaluation is performed by this script.

One host writer owns each destination. `output_version` is a three-digit string greater than `007`. `output_root` must equal this checkout's absolute `target/uar-user-outcomes-{output_version}/supporting` path. Its parent must already exist, be owned by the current user, have no group/other permissions, and contain no symlink components. The script exclusively creates the supporting root, mode0700; every file is exclusive0600. An existing directory, including an empty directory left by an earlier attempt, blocks. No cleanup, reset, retry or automatic version allocation occurs. The script never writes into partial007 or any existing store.

## Configuration contract

The configuration is a JSON object with all the fields below. A frozen reference is exactly `{"path": absolute_regular_file_path, "sha256": digest_string}`. Digests accept either64lowercase hex or `sha256:` plus64lowercase hex. File bytes must match. Symlinks, including symlinked ancestors, are refused. Configuration and private input JSON files must be owner-private. Public source/recipe references need not be private. JSON reads are bounded to128MiB; source/executable hashing streams files up to16GiB.

| Field | Required value |
| --- | --- |
| `schema_version` | `uar.supporting-input-preparation.v1` |
| `executable` | `true`; explicit preparation request, never a runtime activation flag |
| `epoch_id` | `uar-user-outcomes-original-20261024` |
| `output_version`, `output_root` | New version and absolute fresh destination described above |
| `source_inventory` | Frozen current canonical source inventory reference; its `root`, `files` and `source_identity` are consumed |
| `supporting_plan` | Frozen `user-outcome-supporting-final-001.plan.json` recipe or its explicitly approved successor preserving this interface |
| `preparation_packet` | Frozen `user-outcome-p1-packet-002.json` packet or its explicitly approved successor; supplies exact `jwt_validity` |
| `templates` | Object with `manifest`, `config`, `store` frozen references; paths must match the plan's `inert_drafts` |
| `documents` | Object with exactly `a`, `b`, `c`, `d`, `e` frozen references; paths must match the plan's five `document_materialization` sources in order |
| `private_environment_path` | Absolute existing newly prepared `env.json`; no keyfile or shell-profile lookup |
| `private_inputs` | Frozen matching private `inputs.json` reference in the same directory |
| `private_receipt` | Frozen matching private `receipt.json` reference in the same directory; env/inputs paths and input hash must agree |
| `jwt_validity` | Object containing integer `not_before_unix`, `expires_unix`; exactly the packet values, current at execution and spanning at most86400seconds |
| `python` | Frozen resolved regular executable file reference for the explicitly selected installed Python |
| `installed_runtime_inputs` | Nonempty array of frozen regular runtime closure/manifest references, explicitly selected by lead; no dependency discovery or executable probe |
| `toolchain_environment` | String map restricted to `PATH`, `LANG`, `LC_ALL`, `CARGO_HOME`, `RUSTUP_HOME`; PATH/CARGO_HOME/RUSTUP_HOME required; locale entries optional; no ambient environment inheritance |
| `contract_digest` | Exact current contract SHA256, selected by lead; preparation does not sign or adopt it |
| `preparation_approval` | Frozen owner-private approval reference matching the exact projection below |

The approval file must equal this object, with actual configuration-derived values:

```text
operation = "prepare-uar-supporting-inputs"
operator_approved = true
preparation_projection_sha256 = SHA256(canonical(config without preparation_approval))
output_root = config.output_root
effects = [
  "read-selected-existing-private-fixture-inputs",
  "mint-three-supporting-only-hs256-tokens",
  "create-exclusive-private-versioned-supporting-scope",
  "materialize-inert-supporting-metadata"
]
```

Canonical means sorted compact JSON, finite numbers only, UTF8 bytes using Python `json.dumps` defaults. The SHA256 projection excludes only the approval reference to avoid a circular preimage. The record attests the operator's explicit instruction; it is not cryptographic adoption or evidence that the gate opened. The lead must obtain that actual instruction before writing an executable configuration and invoking it.

## Source and secret boundaries

Only inventory rows whose `kind` equals `regular` enter the current-source projection. Each hash is syntactically validated and checked against its regular file. The canonical sorted regular projection must reproduce `source_identity`. Symlink and missing rows, including nullable digests, are retained verbatim in the receipt's `nonregular_source_entries`; their hashes are never split or parsed as executable bindings. No new source inventory is inferred or generated here.

These exact source paths must exist as regular rows: `scripts/post_lifecycle/main_boundary.py`, `scripts/post_lifecycle/controlled_peer.py`, `tests/post_lifecycle_main.rs`, `tests/support/post_lifecycle_count.rs`, `tests/support/post_lifecycle_store.rs`, and this preparer. Selected Python must also be a resolved regular executable with matching bytes. Installed runtime references are file bindings; their actual completeness and compatibility remain unverified.

The preparer reads the existing private environment only in host memory, selects `UAR_FIXTURE_JWT_SECRET`, `UAR_FIXTURE_ADMIN_KEY`, `UAR_FIXTURE_SURREAL_USERNAME`, and `UAR_FIXTURE_SURREAL_PASSWORD`, then discards the remaining map. It never sources shell files, copies upstream keys, changes the two product tokens, or hashes individual secret values. The source private receipt binds `inputs.json`; the private env itself is not publicly hashed or placed in `boundInputs`.

Exactly three supporting JWTs are minted after the exclusive destination claim. Issuer/audience/tenant come from the original primary claims; subjects/roles/workspace/host use the selected recipe with the explicitly approved output version substituted for007. The approved packet's bounded `iat`/`nbf`/`exp` replace any obsolete source validity; no implicit extension occurs. Tokens use sorted compact JSON, unpadded base64url and HMAC-SHA256. Accounting/history owner keys use the source-defined UTF8 byte lengths, with full percent encoding for the accounting URL key. Predicted keys and claims are metadata, not authentication results.

## Produced files and ordering

The writer fills complete JSON documents in `local-main.yaml` and `local-store.yaml`; there is no assumed runtime `@env` expansion. Config contains only the selected fixture credentials and controlled loopback provider. Store config retains its inert `x-uar-draft` extension. Ports49310–49313 remain recipe selections, not observed availability. Fresh compose project/container, namespace/database, identity and document marker names derive from the approved version. The retained main Cargo target remains `target/uar-post-lifecycle-main`.

Outputs are `documents/a.txt` through `e.txt`, `local-main.yaml`, `local-store.yaml`, `supporting.env.json`, `boundary-manifest.json`, and finally `inputs-receipt.json`. A/B/C replace the exact source markers; D/E must retain `UAR_HOLD:kb-race:`. The full ten-scenario manifest is copied from the frozen template, preserving setup/body/cleanup steps and controls. Path relocation is constrained to the known supporting root and007 fixture identity strings.

`boundInputs` contains the validated regular current source denominator, source inventory, source-backed recipe/packet, selected Python/runtime references, private input/receipt identities and actual config/compose/document identities. It excludes the manifest itself, both secret environment files, future signatures, future composition output and receipt selfreferences. Duplicate paths with conflicting hashes refuse. Secret-bearing config and compose file hashes are recorded as whole-artifact identities; no individual key or token hashes are emitted. The final receipt reports separate nonregular source entries, selected aliases, claims, validity, file identities and explicit unverified outcomes. Stdout contains only a completion status, receipt path and CLOSED; failure stdout contains only a closed error code/type.

`evidence/` is created, but `evidence/main/` is deliberately absent because `main_boundary.py` exclusively owns its future evidence directory. Empty owned store/home/tmp/ingest/models/reviews directories are created without starting anything. All manifest activation booleans remain false, verification/store approval references remain null, and final signing/adoption is absent. A failure retains its partial scope; absence of the final receipt means M1 is not complete. No downstream authority signing or composer invocation is permitted merely because some files were written. Even a successful materialization receipt requires separate explicit instruction for those later operations and for final V.

## Verification status

Source authored only under CLOSED. No invocation, import probe, syntax check, test, review gate, signing, composition, service, provider call or target materialization has run. The lead must inventory these source bytes with the other writers before proposing exact future preparation/verification commands. A future materialization operation performs the described frozen-reference comparisons as part of that exact approved operation, not as an independently authorized preflight.
