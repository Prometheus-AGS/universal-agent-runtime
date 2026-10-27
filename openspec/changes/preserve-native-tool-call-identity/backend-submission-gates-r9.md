# Remaining backend and submission gates

This is a preparation specification, not execution approval. Gate CLOSED.
The exact approval request will bind the final supervisor/settings/manifest
hashes and source tree after all required implementation and source review.

Prepared invocation (not authorized here), from the isolated UAR source root:

```bash
python3 scripts/release/native_identity_backend_gate.py --settings /Volumes/T7/pr13-native-identity-r9-private/backend-settings.json
```

The private settings retain the existing Cargo/toolchain caches and select the
local OrbStack Unix socket read from the operator's Docker context metadata;
daemon availability is unproven. Output and resources will use fresh separate
directories under that private parent. Its settings file will be0600, parent0700.
The runner's test/build argv also include absolute `--manifest-path` and explicit
`.cargo/config.toml`; the feature sets and test selection below are unchanged.

## Scoped backend family

The prepared supervisor `scripts/release/native_identity_backend_gate.py` will
own infrastructure setup, private settings, serialized execution, source-drift
checks, strict selection/outcome accounting, artifact retention and cleanup.
Those operations must be included in its exact approval; none is an extra
standalone preflight. It does not run the retained11/11 protocol target, r8
identity target, client gate or whole-phase acceptance driver.

The child command family, with exact paths/args finalized in the source runner,
is:

```bash
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test agent_threads -- --nocapture --test-threads=1
cargo test --locked --no-default-features --features server-full,postgres-backend --test fresh_process_resume -- --nocapture --test-threads=1
cargo test --locked --no-default-features --features server-full --test stream_terminal_contract -- --nocapture --test-threads=1
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test approval_recovery -- --nocapture --test-threads=1
cargo build --locked --no-default-features --features server-full,postgres-backend --bin universal-agent-runtime
```

| Target | Source-defined required selection/outcome |
| --- | --- |
| agent_threads |10 named tests, PostgreSQL configured; no skip/ignored/filtered case |
| fresh_process_resume |1 parent,24 groups;18 local/shared,2 corruption,1 shared race,3 unsupported controls |
| stream_terminal_contract |1 parent,83 scenario/protocol rows across249 legacy/local/shared paths |
| approval_recovery |1 parent,34 groups;26 profile cases,2 live-identity,2 unsupported,4 deadline groups |
| Customer build |Exact non-additive customer feature set, retained binary path/hash; compilation only |

Approval's two live-identity groups do not author the same acceptance.json
format as the other32 groups. Bind their actual readback artifacts and parent
assertions separately rather than inventing34 identical reports. Similarly,
resume progress lines alone do not enumerate all24 groups. The final runner
must describe the actual evidence objects it requires and preserve raw output.
Zero exit with missing selection or evidence does not qualify as a pass.

## Private settings and resource ownership

Select backend images by immutable digest and explicitly request linux/arm64.
SurrealDB uses its existing3.3.0 pinned image and a new owned RocksDB volume.
The operator authorized selecting a compatible immutable Supabase PostgreSQL
image from official metadata; selected facts are retained separately. Registry
metadata proves advertised identities only. Future approved setup must establish
actual platform, authenticated PostgreSQL access/pgvector and durable SurrealDB
configuration before setting the fixture durability attestation.

Selected backend reference is
`docker.io/supabase/postgres:17.6.1.173@sha256:da28f45caeb2d0857a5f53759654859a46c469112f257290e28bc2c291727506`.
Docker Hub's official tag metadata reports linux/arm64 manifest
`sha256:5fb0452350cbb4c0657acbb5e767a5d01dce6e6ce9973921138f190f4ef2ba53`
and linux/amd64 manifest
`sha256:edb20149b36e92c915f13b24ae04fed35e6e0a19a6bd6e06d54ed4afbce5c931`.
The selection record retains the metadata URL and distinguishes advertised
platforms from runtime observations. This is a fixture image selection, not the
replacement UAR release pin.

Use a new resource prefix/network/containers/volumes, private environment files,
fresh databases and separate namespaces. Record each owner before fallible setup.
Never use a supplied external database, broad cleanup, old PID or operator model
credential. A private JSON infrastructure file may contain the connection URLs;
public config and receipts may not. No password goes on child argv. Use a clean
allowlisted environment and private HOME/TMPDIR; preserve the existing approved
Cargo/toolchain caches and single build writer. Reject inherited child-job bypass
modes and prevent .env discovery from importing operator secrets.

Agent threads requires DATABASE_URL. Resume requires UAR_DOT06_INFRA_FILE,
fresh UAR_DOT06_NAMESPACE and UAR_DOT06_EVIDENCE_ROOT. Approval uses a fresh
UAR_DOT07_EVIDENCE_ROOT. Stream uses UAR_DOT05_INFRA_FILE and a distinct fresh
UAR_DOT05_NAMESPACE, VERTEXAI_PROJECT=dot05-fixture,
VERTEXAI_LOCATION=us-central1, and absent BEDROCK_BASE_URL/BEDROCK_CROSS_REGION.
There is no UAR_DOT05_EVIDENCE_ROOT; retain its TempDir under the owned TMPDIR.

Stop on the first nonzero command, missing/extra test selection, failed/missing
required report or source drift. Cleanup always attempts only owned resources,
retaining the original result and cleanup failures independently. Source-drift
and runtime operations are authorized only inside the exact approved runner.
Retain unsuccessful evidence and copy the exact customer binary before another
feature build overwrites it. No automatic retry, broader fallback or weaker
acceptance predicate is included.

Actual-expiry fixtures include two real five-minute waits. Existing operations
and shutdown have their own deadlines; some bootstrap and infrastructure
readiness/join paths still lack a finite cancellation/join guarantee. The stream
child guard preserves kill/wait behavior but does not prove whole-manager/provider
shutdown on every panic. Do not promise a finite overall run or fully graceful
cleanup absent actual evidence. Cancellation is not a passing receipt.

## Separate submission boundary

`cargo fmt --all -- --check` is required by the Rust phase rule but reaches
unowned workspace baseline files. Its exact approval is separate; no automatic
write-format repair is included. A failure requires source assessment, explicit
ownership/integration and renewed inventory before a repeat. There is no
source-confirmed mandatory custom Clippy command in the inspected repository
rules; do not add all-features Clippy from an unrelated example.

Five observed owned Rust warning sources have bounded repairs in this batch.
The vendored OnceLockExt warning and Darwin compact-unwind warning remain
submission holds. No vendor/dependency/profile edit or suppression is authorized.
The existing ignored provisioning case and broader server-full phase suite
remain unverified; no scoped receipt erases them.

Any future commit request includes DCO/Conventional Commits and its actual hooks:
pre-commit `pnpm github-actions-policy:validate`; commit-msg
`pnpm exec commitlint --edit <actual-message-file>`. Explicitly stage only the
owned manifest paths, never `git add -A`. No hook bypass or publication-triggering
push is included in backend gate approval.

The immutable image/release and consumer pin requirements are in
`immutable-artifact-pin-handoff-r9.md`. All unknown commit/version/run/index
fields stay pending. Completing this gate family alone cannot mark PR13 ready.
