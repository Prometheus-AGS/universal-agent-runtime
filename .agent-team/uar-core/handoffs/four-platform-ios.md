# task2.22 — actual iOS simulator embedded acceptance host

Phase: uar-post-lifecycle-verification-readiness / Execute. Gate: CLOSED.
Authoring status: source implementation complete in this ownership scope. No
verification, build, compiler probe, formatter, simulator metadata call, simulator
boot, service/network call, or external gate has executed. All Rust/Xcode/Swift
and actual runtime outcome claims remain unverified. This handoff is not phase
completion, acceptance evidence, or permission to execute.

## Ownership and files

Owned NEW scripts/acceptance/ios/** only, plus this handoff. Other teammates share
the checkout; no existing source or another writer's changes were reverted.

- rust/lib.rs: SDK staticlib example ABI with process owner retaining executor,
  SDK Runtime, actual LiterLlmDriver, existing durable SurrealDbProvider, live
  receivers, SDK admin/run calls, canonical event replay/live drain, cancellation,
  explicit awaited shutdown and engine acknowledgement, FFI panic containment,
  bounded inputs/events, secret-free closed error codes.
- Host/App.swift, Bridge.h, Info.plist: minimal noninteractive UIKit XCTest host,
  exact C bridge and localhost networking allowance. No customer UI design.
- Tests/EmbeddedAcceptance.swift: fixed arithmetic JSON/result predicates,
  canonical live stream ordering/terminal, two-call contextual arithmetic,
  default/orchestrator deletion refusal/readback, live-catalog specialist routing
  and AgentSelected, actual manifest effect and SkillActivated, reduction/token
  allowance accounting, cancellation/terminal, shutdown acknowledgement, second
  process persisted agent/settings/history equality.
- Acceptance.xcodeproj/project.pbxproj and shared UARAcceptance scheme: checked-in
  app + hosted XCTest targets, linked SDK staticlib, serialized build scheme.
- policies/acceptance.cedar: narrow orchestrator execute_tool spawn_agent /
  wait_agents authority; existing StandaloneToolAdmissionPort supplies host seam.
- runner.py: future explicit prepare/build/run actions; deterministic installed
  metadata selection freezes runtime build/UDID/SDK/Xcode/host-supported architecture;
  no boot in prepare. Serial locked staticlib and Xcode build-for-testing; six
  actual hosted XCTest invocations with terminate between seed/restore. Captures
  xcresults, source closure/product identity, PID/store/run/native receipts and
  existing shared budget ledger before/after without resetting it.
- scenarios.json, private-config.schema.json, receipt.schema.json, README.md:
  fixed predicates, private input contract, receipt structure and exact future
  command/prerequisite documentation. No populated credentials committed.

Lead owns shared SDK/root Cargo manifests/lock and planning/spec artifacts. Requested
and accepted SDK feature ios-acceptance combining embedded-mobile, existing
surreal-backend and direct optional same-source Liter dependency; staticlib
[[example]] ios_acceptance points to this rust/lib.rs. No independent workspace,
Cargo manifest, or lock generated in this scope. Lead also corrects observed SDK
policy namespace/effective_config signature drift. Bridge awaits effective_config
as SDK Result per lead's shared correction.

## Source contracts and decisions

Actual SDK/runtime sources were read. SDK builder requires explicit governance and
admission; both supplied. Embedded settings seeds only run_policy.global, so the
persisted setting mutates that real namespace. SDK save_skill persists but does
not refresh the live registry; harness explicitly refreshes existing SkillService
and requires exact saved skill/overlay loaded before running it. Agent is read
back canonically before the restart baseline to include serialized optional fields.
The app owns its provider connection and awaits existing close_scope after runtime
release. A dropped runtime object alone is never restart or shutdown evidence.

The uncomfortable constraint: existing budget_proxy forwards chunks directly to
its client; genuine cancellation may yield BrokenPipe and unknown reservation,
blocking that shared epoch. The iOS runner refuses PASS rather than discarding
unknown charges. This shared proxy behavior was reported to the lead for central
resolution. No proxy source changed here. The runner's final forwarding predicate
requires actual completed reservations, outbound start, exact approved binding,
same epoch and cumulative counters within approved caps. It retains reservations
and does not claim measured actual cost where only conservative ceilings exist.

Guards trace to real boundaries: C pointer/JSON admission, repeated process owner,
input/event byte and timeout bounds, secret redaction, localhost proxy endpoint,
canonical source and installed product identity, selected simulator metadata,
private input permissions, and cumulative budget ownership. No unrelated feature,
new database implementation, customer UI, mock/inmemory positive path, or server
listener was added.

## Exact proposed commands; NOT RUN

After the lead reconciles every active-phase implementation item, freezes a new
source inventory including these files and the shared manifest edits, and obtains
explicit operator instruction naming commands:

1. python3 scripts/acceptance/ios/runner.py prepare --timeout-seconds 120 --selection /PRIVATE/ios-selection.json
2. python3 scripts/acceptance/ios/runner.py build --timeout-seconds 7200 --selection /PRIVATE/ios-selection.json --source-inventory /CANONICAL/source.json --output /PRIVATE/ios-artifact
3. python3 scripts/acceptance/ios/runner.py run --timeout-seconds 7200 --selection /PRIVATE/ios-selection.json --source-inventory /CANONICAL/source.json --output /PRIVATE/ios-artifact --private-config /PRIVATE/ios-config.json

Paths above are packet-bound future parameters; no executable authority was
invented. build includes exact Cargo locked no-default-features ios-acceptance
staticlib target and Xcode build-for-testing. run includes its frozen metadata,
full source inventory comparison using the existing source_inventory selection rules with bounded source_rows enumeration
(including its source-only git ls-files calls), installed-product comparisons,
boot/install/container lookup, test-without-building, explicit terminate, and
existing budget ledger ownership/cap comparisons. Every nested command is in
runner.py; no commands have run merely because they were documented.

Prerequisites: installed actual Xcode iOS SDK/runtime/device, selected architecture
Rust target, resolved SDK lock covering lead's feature closure, writable new private
artifact directory, immutable canonical source snapshot, already running approved
budget proxy/real provider with appropriate cumulative allowances and exact model
alias,0600 private-config.json, explicit gate authorization. Existing proxy unknown
reservation behavior must be resolved/reconciled centrally for a cancelled-stream
acceptance to pass. Physical-device signing is out of scope.

## Current evidence and unresolved verification

Actual commands executed in this scope were cat/sed/rg/head/tail/ls source
inspection, mkdir, heredoc source/document writes, and Python scripts whose only
actions read/replace/write authored files. No check, parser, test, compiler,
xcrun/simulator action, network action, artifact build, or review-driven gate ran.
There is no compiled staticlib, installed app, xcresult or PASS receipt.

Unverified: Cargo feature/SDK lock resolution, iOS native dependency/platform ABI,
Xcode project/linker framework closure, Swift compiler correctness, app-container
persistence over two actual process lifetimes, real model/routing/skill/context
predicates, cancellation transport settlement, joined shutdown/engine receipt,
three-repeat runtime acceptance. Source-only inspection cannot certify them and
the operator lock prevents commands until separately authorized.

## Shared cancellation settlement update

Lead reports a source-authored budget_proxy correction that drains the same
already-dispatched bounded upstream response after known downstream disconnects,
then settles only actual terminal usage. No retry/refund/reset is introduced.
iOS runner now performs a finite90second read-only ledger poll for its new pending
reservations before requiring completed forwarding. Unknown accounting still
fails immediately. This source correction and the poll have not executed; the
previous observed conflict remains a risk pending real cancellation evidence.

Finite execution and ownership update: all subprocesses are spawned into a new
owned process group, use bounded captured output (4MiB metadata,16MiB logs), and
are terminated as that exact group on expiry/failure. Overall explicit CLI
timeouts are capped at120seconds preparation and7200seconds build/run; run is
also clamped to the already approved epoch absolute deadline. Metadata-only
subcommands have a120second ceiling. Simulator selection admits actual iPhone
device types only. Run refuses any pre-existing Documents data, creates an
exclusive source/artifact/epoch/scope ownership marker, copies policies once, and
never replaces prior stores/receipts. Only the exact unchanged current private
input owned by that marker is deleted in cleanup. Populated private config must
include operator-supplied fresh scope_id. These bounds/ownership guards are
source-authored and have not executed.

The runner also refuses an already installed harness bundle before installation,
so it cannot upgrade an unowned app while inspecting ownership. A fresh simulator
is an operator prerequisite. Model context_window is an explicit host-admitted
private input; the output ceiling is copied from the exact existing approved
proxy binding. No context capacity or paid output allowance is guessed.

Exact model semantics: private.model is the bare approved wire alias (`gpt-5.4`
for the lead's planned binding), never `openai/gpt-5.4`. Provider registry ID is
openai and default_model is the bare alias. LlmConfig.model is explicitly
qualified as openai/<alias>; EmbeddedRuntime preserves that route. The real
LiterLlmDriver::new receives the bare alias with an explicit endpoint, selecting
the pinned generic OpenAI-compatible adapter. Its source-defined prefix stripping
only removes a matching prefix plus slash, absent from a bare alias. Startup
checks mounted SDK provider_registry.default_model and inner.llm_config before
returning readiness; no HTTP verification endpoint or mock is used. Actual wire
model is additionally enforced by the existing exact approved proxy binding.

## Required default-agent fresh-input coverage correction

Tests/EmbeddedAcceptance.swift now invokes genuine default-agent immediately
after protected-agent deletion refusal/readback in seed, and again after actual
process relaunch and persisted-state readback in restore. Each phase/repetition
uses a new UUID and fresh bounded random arithmetic operands; expected nonce and
integer arithmetic result are computed on the host and compared by exact JSON
equality after real canonical drain (RunDone, no Error/Cancelled). The arithmetic
answer is not supplied in the prompt. The restore UUID must differ from seed.
Both seed and restore receipts preserve nonce, operands, host expectation, run ID
and actual events under default_agent_challenge. scenarios.json and README.md
record these exact predicates. This adds six genuine model attempts across the
three repetitions; existing shared epoch ceilings still govern every attempt.

Only the owned XCTest source, scenarios.json, README.md and this handoff changed
for the correction. Source inspection and file authoring only; no syntax/parser,
Swift/Xcode/Cargo, simulator, model, test or verification command ran. Historical
source snapshots do not cover this correction; lead must freeze the new whole-
phase source inventory before requesting exact gate authorization. Gate CLOSED.
