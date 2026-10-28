# Owned datastore partial-startup cleanup handoff

Phase: `uar-post-lifecycle-verification-readiness`, implementation. Operator gate
CLOSED. This packet authors source only and does not mark phase completion.

## Files and ownership

- `scripts/acceptance/observers_runtime.py`: PG prepared/cleaned handshake;
  derived four-label ownership and Surreal CID transport; exact immutable
  container inspection; genuine forensic capture separated from lifecycle
  teardown; truthful retained-state/active-residual receipt. Existing root
  inference, forwarding, MCP, static-serving and team behavior preserved.
- `scripts/acceptance/README.md`: native receipt contract, cleanup failure and
  retention semantics, exact environment aliases and unexecuted status.
- This handoff. No other files changed by this bounded task.

Source ownership is released to root after this handoff. Composer owns
`compose_fixture.py`; PG helper writer owns `stores_fixture.py`. Their current
source contracts were read and aligned, without executing either program.

## Actual contract

PG preparation now requires private `postgres-fixture-prepared.json` before
CREATE SCHEMA. Its string schema version, prepared status, run/scope/nonce and
config SHA must match the actual frozen `runtime.postgres_store.config_path`.
Observer connection and psql path must match that native configuration. Actual
CID, image, four labels, owned mount, host port, SQL database/user/data directory,
container port and pgvector readiness are required. Cleanup invokes the paired
PG helper even if Store construction/capture/SQL drop fails, then requires its
matching cleaned receipt and actual exact-name absence listing digest. Helper
allows validated no-container absence after partial claim, refuses unclaimed
existing container deletion, and retains the owned disk.

Surreal environment aliases are derived from this invocation:
`UAR_ACCEPTANCE_OWNED_LABEL=uar.acceptance.scope=<scope>`,
`UAR_ACCEPTANCE_OWNED_RUN_LABEL=uar.acceptance.run=<run_id>`,
`UAR_ACCEPTANCE_OWNED_NONCE_LABEL=uar.acceptance.nonce=<ownership nonce>`,
`UAR_ACCEPTANCE_OWNED_BACKEND_LABEL=uar.acceptance.backend=<backend>`, and
`UAR_ACCEPTANCE_SURREAL_CID_FILE=<owned root>/surreal-container.cid`.
An ambient `UAR_ACCEPTANCE_SURREAL_CONTAINER_ID` is discarded. Actual inspect
sets it only after exact immutable CID/name/labels/image/argv/mount/port checks.
Cleanup argv must be the frozen docker binary followed by `rm --force` and that
single derived CID reference. Stopped/failed-start containers can be inspected.
Actual name, all-four-label and recorded-CID listings must agree on the owned
container; absence requires successful bounded diagnostic-free listings before
and after removal. Creation failure before CID/container existence is handled
with actual absence evidence, without invented IDs or blind name removal.

Cleanup first retains bounded redacted genuine counts and typed ingestion job
metadata. Capture failure records null counts/jobs, unavailable status and an
opaque-error digest, never a fake empty job set. Capture/save/drop failure does
not skip exact-owned lifecycle teardown. Those failures retain private root and
disk; active absence may still be confirmed. `residuals` remains nonempty for
retained state, preventing packet success. Full private-directory deletion needs
saved available capture, confirmed schema/database drop and confirmed service
absence. Recorded live runtime PIDs still prevent teardown.

## Evidence, limits and unfinished inputs

Commands run were source reads (`sed`, `rg`) and explicit file authoring through
`python3`/`cat`, all with `login:false`. One source search named nonexistent
`fixture_composer.py`; the actual composer is `compose_fixture.py`, subsequently
read. No edited module was imported or executed. No test, syntax/type/compiler
check, format/lint, review verification, Docker, datastore, server, browser or
provider operation ran. There is no passing runtime evidence or new source hash
claim. All parsing, lifecycle execution, failure paths and both backend behaviors
remain unverified until the operator approves the exact renewed whole-phase
packet.

No unresolved source input is required from the composer or PG writer: current
argv, native config path and cleaned absence receipt match this integration.
Frozen real tools/images, private fixture configuration, Docker availability and
approved future execution remain runtime prerequisites. A failed cleanup is
retained and stops the packet; no automatic retry was added. Raw upstream causes
remain intentionally unrecovered where only digests are retained.

Guards address the reported prepare/readiness failure skipping teardown and the
real ownership boundary around destructive container/disk operations. No new
user feature, external probe hook, broad infrastructure refactor or unrelated
change was added. The uncomfortable remaining risk is that this source has not
been executed: implementation is authored, not accepted.
