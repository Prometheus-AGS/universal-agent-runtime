# Complete shared fixture ingestion repair and failed-gate continuation

Stage: Execute, source implementation only. Verification CLOSED after the first
r10 backend failure. Prior gate authority is consumed; no test/build/probe or
validator may run again without a complete successor freeze and exact instruction.

## Assessment and analysis

The approved r10 supervisor retained agent_threads10/10 with PostgreSQL exercised
and no skips. Fresh-process recovery compiled, then its first local predecessor
exited before readiness: `/data/ingest` is read-only on this Mac. The target had
0 passed/1 failed in26.67s; Cargo returned101 and the supervisor stopped with2.
Stream, approval and the separate customer build never started. All six owned
backend resources were removed (three containers, two volumes, one network),
without final source-drift, cleanup or diagnostic-write errors. A prior metadata
summary incorrectly said seven; the raw immutable receipt is authoritative.

Server startup creates the ingestion directory whenever persistence exists;
memory.enabled=false does not bypass it. Node::launch omits an override. Approval
uses that same Node helper, including predecessor/successor/race/unsupported
profiles. Agent threads and stream use stores/managers directly. Source inspection
of upload/model/memory/skills/policy/Wasm startup paths found no second required
directory override. Do not invent one or change the production default.

## Exact complete source batch and ownership

The Rust writer owns only `tests/support/fresh_process_resume_process.rs`:

1. Prepare an empty private `directory.join("ingest")` for every launched node.
2. Set server.ingest_directory to that exact owned path in its existing config.
3. Preserve all assertions, processes, persistence, watcher behavior, startup/
   shutdown deadlines, credentials, provider routes and supported/negative cases.

The gate writer owns only the new
`scripts/release/native_identity_backend_continuation_r11.py`. Keep the reviewed
baseline supervisor byte-identical. Its fixed adapter must:

- Bind the actual r10 receipt, manifest, command receipts/logs and source captures
  by exact paths/byte hashes from the retained continuation-input identities.
  Check the two-gate history, real successful10-name/no-skip prefix, failed resume
  exit101, clean final evidence/source outcomes and six removed resources.
- Reuse the unchanged baseline acceptance parser on the retained prefix log.
  A supplied success object, arbitrary skip selector/target/command or a forged
  source manifest cannot replace those actual bound artifacts.
- Enforce full source continuity: all previous source/control bytes, dependencies,
  external Cargo input, backend pins/platform and staged-tree paths stay bound.
  Only the reviewed helper repair and two new r11 files are allowed. Bind both
  helper preimage and the actual successor hash delivered by the Rust writer.
- Preserve every baseline SourceFreeze check, adding predecessor/delta binding
  before setup and at every capture. Select only the fixed remaining target
  suffix: fresh_process_resume, stream_terminal_contract, approval_recovery,
  then the unchanged exact customer build. Do not rerun agent_threads, protocol11,
  r8 identity/client or broader checks.
- Retain a separate authoritative continuation receipt distinguishing adopted
  historical prefix from actual suffix commands/outcomes. The unchanged baseline
  receipt's four-target scope label describes the original family; do not treat
  it as four new executions. Keep actual raw gates and original failure statuses.
- Use fresh private output/resources/settings, bounded receipt references and
  existing source, environment, resource ownership, cleanup and failure rules.
  No retries, weakened validators, forced cleanup or new overall timeout.

The lead owns this plan, immutable evidence/identity inventories, private settings,
typed KBD reconciliation and final handoff. Every path is assigned to one writer.
All historical r8/r9/r10 evidence and reviews remain unchanged. Explicit staging
only; no git add -A, dependency/vendor/toolchain/API edit or publication.

## Source integration, review and next gate

Finish the whole source/configuration batch, collect stopped-writer handoffs,
capture the complete cumulative inventory/patch/preimages/staged tree and exact
unchanged-prefix proof inputs, reconcile the typed child ledger, and obtain
independent artifact-only product/security reviews. Apply the normal bounded
review policy to this distinct runtime-failure correction. Source review cannot
certify the corrected startup or the continuation guard at runtime.

Only then request this exact future entrypoint with its fully populated private
settings, frozen hashes and actual argv: python3
scripts/release/native_identity_backend_continuation_r11.py --settings
/Volumes/T7/pr13-native-identity-r9-private/backend-settings-r11.json.
No validator, import/syntax check, test, build, format/lint, runtime probe or
negative diagnostic may execute during authoring/review. The planned real target
is the regression that exposed the configuration failure; no mirrored test or
weaker assertion is substituted for it.

The new sidecar AsyncFd deprecation warning is retained with the earlier vendor/
linker warnings as a separate submission hold. This plan does not change sidecar
or vendor code. Existing unbounded waits, incomplete shutdown guarantees under
permanent I/O failure, formatting/hooks, broader certification, immutable release
publication and consumer pin/runtime verification remain explicit holds.
