# Boss 2.2.40 UAR source preparation — 2026-10-19

Phase `uar-outstanding-issues-v1-1-1`, Execute. Isolated checkout
`/Users/gqadonis/.claude/worktrees/uar-liter-boss-2240` starts at committed
candidate `38f0b5d85960e3cb8445607750538ca500efdf17`. The operator verification
gate remains **CLOSED**. This is a source-only handoff, not release acceptance.

## Scoped source

- `vendor/git/liter-llm` points to frozen GQAdonis fork commit
  `31c7e63236aa96914fccae2a5c25c4cda9ffa430` (2.3.0).
  `versions.toml` names that version and commit. `Cargo.lock` resolves Liter
  2.3.0 with its required transitive updates through `cargo update -p liter-llm
  --offline`; this was lockfile authoring, not a check or build.
- `catalog/provider_catalog.json` was regenerated from that fork's provider and
  model schemas with `node scripts/refresh-provider-catalog.mjs`: 358 providers,
  SHA-256 `6befbcf6a5ccda605daed188b2b59affa64930eddb9c6a001348a0e37a90e668`.
  `catalog/SNAPSHOT.md` records the frozen input hashes and current provenance;
  `scripts/refresh-provider-catalog.mjs` corrects its comment because Liter
  2.3.0 now contains the exact GPT-6.1 Sol model.
- The observed concurrent Surreal credential-migration failure motivated the
  exact `src/uar/security/credentials/principal_store.rs` repair from the dirty
  primary checkout. On an untyped racing transaction failure, it reads an
  already committed journal and replays the immutable receipt under the
  existing authorization checks. Typed rejections retain priority.
- Transferred the release qualification contract and controlled fixture from
  the dirty primary checkout: `scripts/release/uar_1_1_1_acceptance.py`,
  `uar_1_1_1_compiler.py`, `uar_1_1_1_credentials.py`,
  `uar_1_1_1_manifest.example.json`, `uar_1_1_1_manifest.schema.json`,
  `uar_1_1_1_recovery.py`, `uar_1_1_1_runtime.py`,
  `uar_1_1_1_support.py`, `tests/fixtures/uar_1_1_1_provider.py`,
  `docs/operations/uar-1.1.1-acceptance.md`, and `docs/releases/v1.1.1.md`.
  The primary checkout's FC2–FC4 handoffs identify these as the function-first
  1.1.1 acceptance reduction and its fixture.
- Round-1 qualification B3 reported an admitted run hanging through store
  outage until the HTTP timeout. Its bounded admission repair is transferred
  surgically in `src/config.rs`, `src/server.rs`,
  `src/uar/runtime/manager.rs`,
  `src/uar/security/credentials/mod.rs`, and
  `src/uar/security/credentials/resolver.rs`. The only unrelated primary
  hunks in those mixed files remain excluded. `tests/settings_persistence.rs`
  supplies the required field in its explicit `ResilienceConfig` literal.
- Round-2 qualification reports a passing real-path post-restart stream after
  the durable run-history repair. Transferred its changes in
  `src/uar/runtime/manager.rs`, `src/uar/persistence/mod.rs`,
  `src/uar/persistence/providers/{postgres,surreal,surreal_host}.rs`,
  new `src/uar/persistence/run_history.rs`,
  `src/uar/persistence/providers/run_history/{postgres,surreal}.rs`,
  `migrations/20261018000000_run_history.sql`, and
  `migrations/surrealdb/run_history.surql`.
- Round-5 qualification verified Surreal `Value::None` migration-status
  handling at its formerly failing assertion. Transferred the typed conversion
  repair in `src/uar/persistence/providers/surreal.rs` and its
  `tests/surreal_value_conversion.rs` source test. The later concurrent
  migration 503 remains unverified against this assembled source; the
  `principal_store.rs` repair is a candidate, not a passing receipt.
- Copied the matching 1.1.1 OpenSpec proposal/design/tasks/verification and
  two changed capability deltas; parent phase plan, ownership and three
  contract fragments; child plan/spec/scenario disposition/supervisor runbook,
  FC1–FC6 handoffs; and the five historical qualification reports. Historical
  reports are not current acceptance. Unrelated dirty Rust, generated tool
  integrations, private credentials, stale inventories and gate receipts were
  not copied.

## Canonical state and remaining work

This checkout's `tasks.md` still marks 8.8/L11 pending. Its committed waypoint
and ledger predate the primary checkout's L11 completion. The primary L11
inventory binds old source and fixture trees and cannot certify these Liter,
catalog, runner, fixture or credential edits. Reconcile 8.8 in canonical KBD
state only after the lead collects the complete phase implementation inventory,
including the selected function-first child artifacts and any further repair.
The active phase also has 6.2 local acceptance and 7.1–7.3 publication,
published-architecture acceptance and receipts open. The latest inspected
function-first local r3 receipt failed credential `POST /sql` with 401 and
recovery on a supervisor runtime/source identity mismatch. The transferred
code and new Liter pin have **not** been tested against those failures.

Source inspection classifies the r3 401 as a fixture defect already repaired
in the transferred `uar_1_1_1_credentials.py`: the old metadata helper reopened
the existing SurrealKV volume with a fresh random password, whereas UAR used
its configured `surreal_user`/`surreal_pass` or root/root. The current helper
reuses those credentials; that correction has no acceptance result. The
recovery mismatch is a qualification prerequisite: `_qualified_supervisor`
requires the retained identity's image, platform, source tree and fixture tree
to equal the manifest before scenario work. R3's source/fixture trees differ
from the retained 04/05/06 identities, and this new Liter source differs again.
A fresh same-image supervisor preparation, physical boot and identity receipt
are required for the final assembled source. No additional product-code fix
for these two r3 failures was identified through source inspection.

The lead approved and ran the exact identity command below after source
assembly. It produced production tree
`sha256:d41e765337e653766f3afedd95b674b47359cf702704fc132103cfd9ccfac351`
and fixture tree
`sha256:fd6ae4bf47dd85435104f1bc6c9662b6b6894d8eb757e1262a5d74d2679d81de`.
The proposed 36-row whole-parent/child inventory is
`implementation-inventory-function-first.json` with digest
`sha256:2fdda0d9ba9b49ca0917913d80affd2f4123e5aec4ba8c9bdbefa5a97689d6a9`.
It includes B3, C1, all report-backed fixture rounds, the round-five value
conversion, the candidate concurrent replay repair and the frozen Liter pin.
The child view, FC5 handoff, operator packet and three non-executable gate
templates now bind these trees. The lead must review that inventory, then
update canonical task 8.8 and child FC5 state through the KBD CLI. Generated
child `progress.json`/`tasks.md` were not copied or hand-edited. The lead must
present exact build/preparation/gate commands for separate operator
instruction. Do not relabel any old inventory or reuse FC6 identity-04/05/06.

The saved private infrastructure document has the required supervisor field
names, including scoped tokens and a Lima journal transport. The runbook
describes export, install, staged predecessor, physical boot and replacement.
These are source inputs, not a ready candidate: all retained identities bind
older source/fixture trees and image IDs. Fresh candidate image bytes,
preparation receipt, installed guest bytes, staged work, boot transition and
unused proof are still needed. Operability of the private guest, credentials
and services has not been checked while the gate is CLOSED.

The exact source and fixture identity command already approved and run by the
lead was:

```bash
python3 -c 'import sys; sys.path.insert(0, "scripts/release"); from uar_1_1_1_acceptance import SOURCE_PATHS, FIXTURE_PATHS, source_identity; print(source_identity(SOURCE_PATHS)); print(source_identity(FIXTURE_PATHS))'
```

It computed the runner's production and fixture tree digests; it did not prove
buildability or runtime behavior. The local manifest is still a non-executable
template until a freshly prepared image and same-image supervisor receipts
are recorded. A prospective local command for the completed manifest is:

```bash
python3 scripts/release/uar_1_1_1_acceptance.py --manifest .kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/function-first-local.json
```

It would run the complete selected local functional program and report its
actual result; it would not certify either published architecture. Do not run
it until the whole-phase inventory reconciles and the operator explicitly
names that command. New implementation edits require a renewed inventory and
new instruction. No test, lint, format, typecheck, Cargo check/build, packaging,
review gate, `git diff --check`, commit or push was performed here.

## Startup configuration child integration — renewal required

The primary startup-config child finished its finite diagnosis at canonical
revision 3097. Its `handoff-out.md` and
`receipts/config-lifecycle-map-2026-10-19.md` support the v1-v4 lifecycle
repairs. I selectively transferred those product hunks to `src/embedded.rs`,
`src/server.rs`, `src/uar/api/{knowledge,providers,settings}.rs`,
`src/uar/rag/embeddings/{mod,swappable}.rs`, `src/uar/rag/ingest.rs`,
`src/uar/runtime/matching/vector.rs`, and `src/uar/settings/manager.rs`.
Only the required compatibility interfaces were added: the shared-backend
`new_shared` constructors for ingest and vector matching, while retaining
their prior public `new` form, plus `SettingsApiState` literal fields in the
settings helper and `tests/settings_persistence.rs`. The latter fixture also
passes the explicit credential-retention flag on one provider update. The
primary checkout's unrelated config/memory expansion was excluded.

The earlier production tree `d41e7653…`, fixture tree `fd6ae4bf…`, and 36-row
inventory digest `2fdda0d9…` above are historical after that transfer. The
newly approved exact identity command recorded the current production tree
`sha256:7c14facb2a2e8f6e7aca230bb7e30581ed4b01abe0d8ea2bd09bdad0f4ec1d41`
and fixture tree
`sha256:4de62ff30d467ab9da134ad7f988c04d167fb15c95e89da7e7706739e5b42025`.
The revised 37-row inventory's former digest `1debdb3e…` is superseded by its
identity update; its current document digest is pending. The three gate
manifests remain non-executable templates. The parent query-time embedding-space
mismatch guard remains **NOT READY**, since a second supported credentialed
embedding space is unavailable. The child diagnosis PASS does not qualify this
assembled source or release. Publication remains stopped. No behavior
verification, build, commit, or push followed the transfer.
