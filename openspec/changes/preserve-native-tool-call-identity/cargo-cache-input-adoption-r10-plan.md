# Adopt the existing Cargo cache inputs without ambient credentials

Stage: Execute, source preparation only. All verification remains CLOSED.

## Assessment and analysis

The completed r9 corrective reviews are preserved and their review cap is spent.
They establish source resolution of S1/S2, not execution acceptance. Final
operational-input inspection subsequently identified a distinct blocker: the
selected existing `/Users/gqadonis/.cargo/config.toml` has `[env]` entries
SCCACHE_DIR and SCCACHE_CACHE_SIZE. The runner currently rejects every `[env]`
table. Thus its prepared settings cannot use the existing cache. No gate was run
to discover this; only configuration section/key metadata was read. No secret
value was displayed.

Creating another CARGO_HOME would lose the intended sharing/fingerprints, and
editing the operator's configuration would change an unrelated owned input.
Instead adopt the actual existing configuration as an immutable trusted build
input, accepting only its two cache environment keys. Provider credentials,
runner injection and arbitrary new environment configuration remain forbidden.

## Exact source batch and ownership

The existing supervisor writer owns only
`scripts/release/native_identity_backend_gate.py` and
`scripts/release/native_identity_backend_settings.example.json`:

- Add a required `cargo_config_sha256` setting for the actual existing
  CARGO_HOME/config.toml; bind its bytes during configuration and every source
  capture, without displaying its contents.
- Parse TOML as source data. Only the global configuration's SCCACHE_DIR and
  SCCACHE_CACHE_SIZE string entries are permitted; no other env key, arbitrary
  runner, value table or multiline value is accepted. Repository env injection
  remains forbidden. The exact configuration hash binds its build/cache/profile
  settings; do not rewrite or copy the operator file.
- Preserve existing CARGO_HOME, RUSTUP_HOME, target/cache directory, feature
  commands, all acceptance predicates and resource cleanup. No dependency,
  vendor, compiler/profile, production behavior or API change is included.

The lead owns this plan, the private settings hash, source inventory, canonical
task reconciliation and artifact handoff. Capture this external input separately
from relative producer source. No credential-bearing Cargo file is published.

## Integration, review and gate

Finish the entire input-adoption batch, stop the writer, and freeze the complete
cumulative successor including all r9 source. Obtain one independent artifact-only
product and security review of this distinct input-adoption task. These are its
initial reviews, not another r9 corrective round or an attempt to change a prior
verdict. Preserve all r9 initial/corrective reports unchanged. Apply the normal
one-initial/one-corrective cap to this task and retain any unresolved finding.

No tests, builds, lint, format/import/syntax checks, validators, runtime probes,
Docker or hooks may run. After all scoped implementation and settings are complete,
reconcile the actual typed child ledger, record exact full source/external-input
identities, and ask for the exact future command approval. No existing r8 or r9
review/receipt certifies these new bytes or whole-phase/PR readiness.
