# Backend and submission preparation after r8

Stage: Execute, preparation only. Verification is CLOSED. The operator requested
remaining backend/submission gates and an immutable artifact/pin handoff; this
does not authorize running any check, build, hook, service or publication.

## Assessment and analysis

R8 tree `f1deb3ccb29e75003c46abd8e545429e0c8512d7` has its own immutable source
manifest, ten passing identity tests, a successful customer build, and the client
gate's intentional exit 2 (`extended_passed_remaining_release_gates`). Those
receipts remain historical. They cannot certify a successor tree.

Source assessment identifies four remaining integration targets: agent_threads,
fresh_process_resume, stream_terminal_contract and approval_recovery. Their
infrastructure requires owned disposable PostgreSQL with pgvector and durable
SurrealDB, fresh namespaces, private settings and retained failure artifacts.
The PostgreSQL immutable reference is unresolved. A missing DATABASE_URL makes
one agent_threads test report SKIPPED and return success, so Cargo exit zero alone
cannot establish its acceptance.

The stream fixture still uses the old embedding constructor. Its HTTPS child is
also unowned during readiness/port parsing, which can leak Python after a panic.
Five observed Rust warnings have bounded source fixes. The vendor warning and
Darwin linker warning have no authorized minimal remedy; record them as holds.
Existing resource helpers put passwords on child argv and inherit the operator
environment. Reusing them unchanged violates this handoff's credential boundary.

## Complete implementation batch and ownership

One Rust writer owns these exact files, preserving all scenarios/assertions:

- `tests/stream_terminal_contract.rs`: current SwappableEmbeddingBackend wrapper
  with dimension384, original unavailable reason and threshold0.75; immediate
  private Drop child ownership through readiness and parsing. Preserve existing
  shutdown behavior and timing; disclose graph/provider cleanup limits.
- `src/uar/runtime/manager.rs`: remove only the two observed unused imports in
  the old approval_gate_tests module; preserve the r8 identity module.
- `src/server/tests.rs` and `src/uar/runtime/skills/service.rs`: remove only the
  redundant FutureExt and SkillOrigin imports, preserving parent visibility.
- `src/uar/rag/ingestion_admission.rs`: bounded non-secret Debug implementation
  without new generic bounds or queued descriptor disclosure.

The lead owns documentation, a new private-settings template and new scoped
gate supervisor source under `scripts/release/`, plus inventories/handoffs.
The supervisor must not invoke existing password-bearing resource argv. It must
use private environment files and scoped child environments, enforce immutable
backend images/platform, own resources before fallible setup, retain original
and cleanup failures, reject fixture-child bypasses and skipped cases, freeze
source before execution and stop on drift/failure. No operator store or model
account is an input. Setup/readiness/authentication operations, if implemented,
are inside the future exact gate, not independently authorized preflights.

Update `docs/operations/approval-recovery.md` from32 to34 groups. Add a successor
backend/submission handoff rather than rewriting r8 evidence. Preserve original
release/pin history, source manifests, all unsuccessful receipts and source
reviews. No dependency, vendor, toolchain, provider, schema or API changes.

## Proposed verification and publication boundaries

After all source/settings/docs are integrated, writers stop; capture complete
exact-path source inventory, preimages, staged tree and patch, reconcile canonical
KBD state, obtain separate source reviews of this preparation package, and then
ask for exact command approval. No verifier may execute code while CLOSED.

The serial proposed gate family is the four complete integration targets, with
their documented feature sets and one test thread, followed by the exact locked
server-full,postgres-backend customer build. Expected inventory is10 named
agent_threads tests with no skip;24 resume groups;83 stream rows across249 paths;
34 approval groups. Exact commands, private settings paths, evidence locations,
timeouts and cleanup limitations belong in the final request. The protocol11
receipt is retained and its target is not rerun. No automatic identity/client
rerun is added. New warning or fixture changes cannot inherit prior acceptance.

Submission additionally requires workspace format checking and applicable
commit hooks under their own explicitly named approval. Do not autoformat
unowned baseline files, bypass hooks or suppress unresolved warnings. The
whole-phase server-full suite is a broader pending certification, not implied
authority for this scoped package.

Publication remains a separate approval. Prepare the actual workflow/source
requirements, unchanged package version/default feature closure, commit/tree,
lock/toolchain/submodule identities, candidate/platform/index/provenance fields,
and consumer path/preimage requirements. Unknown commit/digest/run identities
remain pending. Never overwrite existing1.3.0/v1.3.0 tags. Genuine replacement
artifact, fresh owned runtime inputs, guarded Kubo integration, normal export
and Desktop assembly, renewed combined freeze and subsequent explicit consumer
verification approval precede PR readiness. Do not merge PR13.

The uncomfortable limit: this preparation cannot prove compilation, durable
cleanup, compatibility, image acceptance or merge readiness. Any unresolved
immutable input, implementation item, source drift or canonical disagreement
keeps verification CLOSED.

## Source-only settings reconciliation before freeze

Filesystem metadata shows an operator `.env` at `/Users/gqadonis/.env`; its
contents were not read. Current `src/main.rs` explicitly loads an env file only
through `--env-file` or UAR_ENV_FILE. The four target parents/direct support have
no implicit dotenv call; runtime fixture children use private cwd. Therefore
rejecting the source checkout merely because its ancestor has an unused `.env`
is unsupported. Remove that source-ancestry prerequisite, retain source hashing
of main/config/support, omit env-file flags and UAR_ENV_FILE from scoped child
execution, and place private work/settings under
`/Volumes/T7/pr13-native-identity-r9-private`, whose ancestor metadata has no
`.env`. No operator file is deleted, moved or edited. Private runtime ancestry
still must be clear. This reconciles actual source behavior, not a runtime claim.

The supervisor's source has a concrete failure-ownership defect: a receipt write
before resource cleanup can raise and skip the cleanup attempt, and per-resource
receipt writes can skip later owned resources. Existing memory records actual
external-volume write stalls. The same supervisor writer must place ownership
registration before fallible writes and make cleanup independent of receipt
availability, attempting every owned resource even if diagnostic writes fail.
Retain write failures in memory and persist when possible; do not claim that a
detached/full volume guarantees durable final evidence. Preserve original and
cleanup errors separately and fail closed if final diagnostics cannot be retained.

Plain Git source inventory identifies retained untracked handoff/review evidence
and typed child KBD projections; they are not ignored by Git. The supervisor
must distinguish those exact three metadata roots from untracked producer source
instead of refusing its own historical evidence. The allowed roots are
`.agent-team/uar-core/handoffs/kubo-native-identity/`,
`.agent-team/uar-core/reviews/kubo-native-identity-r7/`, and
`.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/children/kubo-native-tool-identity-pr13/`.
New r9 reviews will be kept in the same handoff root. The final manifest records
these fixed metadata exclusions; no caller-selected arbitrary prefix is allowed.
Tracked source, staged files and untracked producer source keep strict guards;
the supervisor and actual imported/build inputs remain individually hash-bound.

## Complete corrective source-review batch

Initial independent product review raises no actionable finding. Initial security
review raises S1 and S2; both are concrete failures of the required evidence-I/O
ownership boundary and will be corrected together before verification approval.

- S1, supervisor writer: attachment log I/O can make inherited join raise after
  Docker stop/wait, before container removal. Retain actual join and diagnostic
  failure separately, and still attempt owned container removal once termination
  is established. Preserve diagnostic failure as a blocker and retain each
  cleanup outcome. Never remove unrelated resources or add force-kill behavior.
- S2, Rust writer: `tests/support/approval_recovery_process.rs` writes process.json
  between raw child spawn and OwnedChild construction. Install the existing
  OwnedChild guard immediately on successful spawn before fallible serialization/
  writing, then obtain its PID through the guard. Preserve assertions, selection,
  timing and existing kill/wait semantics. This exact additional path is assigned
  to the Rust writer; no other fixture or production changes are included.

Preserve the candidate manifest/patch and both initial review reports. Stop both
writers, reconcile the complete successor source and perform only the one bounded
corrective source re-review per judge. Any unresolved actionable finding after
that cap remains a hold. No tests/builds/checks/probes are authorized by review.
