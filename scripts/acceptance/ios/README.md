# Actual embedded iOS simulator acceptance

Gate CLOSED. These are authored sources, not a simulator result. No simulator,
Cargo, Xcode, syntax checker, or acceptance command has run for this harness.
The SDK staticlib example links `embedded-mobile` plus the existing
`surreal-backend` adapter. It never selects `server`, a mock LLM, or an in-memory
positive backend. The existing LiterLlmDriver forwards real model traffic through
the already approved host budget proxy. The trusted app owns a SurrealKV provider
inside Documents; this is not a new database implementation.

The uncomfortable limitation is that a successful cross-compile cannot establish
simulator behavior. This project requires an actual installed iOS simulator and
six successful hosted XCTest process invocations. All current claims are source
implementation claims; linking, simulator ABI, system frameworks, real routing,
skill activation, context reduction, and shutdown are unverified until authorized
execution. Physical-device signing and customer UI are outside this harness.

## Source contracts

`rust/lib.rs` is the SDK's `ios_acceptance` staticlib example. `Host/Bridge.h`
exposes the exact C allocation/free ABI to Swift. A serialized process owner keeps
the Tokio executor, SDK Runtime, store, and live broadcast receivers alive across
FFI calls. Explicit shutdown awaits the runtime lifecycle and the store engine
acknowledgement before releasing ownership. Both synchronous and async panics are
contained at the ABI; failures expose closed error codes, never provider errors.
The XCTest owns cleanup on failures. The host installs the exact checked-in Cedar
spawn/wait policy and existing StandaloneToolAdmissionPort; no permit-all engine
is synthesized.

`Tests/EmbeddedAcceptance.swift` tests arithmetic JSON equality, canonical live
stream terminals and ordering, a second call using the previous subtotal,
protected default/orchestrator deletion refusal and readback, a newly persisted
invoice specialist selected by the actual orchestrator with AgentSelected event,
a selected manifest that changes computed tax and emits SkillActivated,
structural reduction plus token allowance accounting, cancellation acknowledgement
plus Cancelled terminal, and joined runtime/store shutdown. A second XCTest
process reads the exact custom agent prompt/metadata, persisted global setting,
and every event of the earlier run from the same store. The runner terminates the
app between the invocations and checks different process IDs. This is repeated
three times with separate store directories; the epoch budget is never reset.

## Explicit future commands

The lead must present these exact commands with a reconciled whole-phase source
inventory and obtain operator approval before running them. Paths are supplied by
the approved authority packet, not inferred or copied from this README.

1. `python3 scripts/acceptance/ios/runner.py prepare --timeout-seconds 120 --selection /PRIVATE/ios-selection.json`
   Reads actual available devices/runtimes, SDKSettings, host architecture, Xcode
   version and developer directory. Picks newest available iOS runtime, then
   lexical runtime ID/UDID. Optional `--udid` constrains to an actual installed
   device. Does not boot a simulator. The receipt freezes SDK path/version,
   runtime version/build, UDID and exact supported host architecture/Rust target.
2. `python3 scripts/acceptance/ios/runner.py build --timeout-seconds 7200 --selection /PRIVATE/ios-selection.json --source-inventory /CANONICAL/source.json --output /PRIVATE/ios-artifact`
   Serially invokes locked Cargo SDK staticlib build and Xcode build-for-testing.
   No unit tests run during build. Saves private logs, staticlib digest and exact
   installed test-product file closure. An output directory must be new.
3. `python3 scripts/acceptance/ios/runner.py run --timeout-seconds 7200 --selection /PRIVATE/ios-selection.json --source-inventory /CANONICAL/source.json --output /PRIVATE/ios-artifact --private-config /PRIVATE/ios-config.json`
   Boots only the frozen simulator, installs the frozen app, writes private config
   to its named Documents/private-config.json, and runs seed then restore via
   test-without-building for repetitions1–3. Preserves xcresult/logs/native receipts
   and the shared budget ledger before/after. Removes the app private config in a
   finally block. It does not start a service or the budget proxy.

The build command contains `cargo build --locked --manifest-path
sdks/rust/Cargo.toml --no-default-features --features ios-acceptance --example
ios_acceptance --target <frozen target>` and selects a fresh serialized target
folder. `xcodebuild` commands and `xcrun` actions are explicit in runner.py. Every
metadata/source/artifact comparison executed by these commands is part of the
command, not an additional implicit permission request. No command auto-opens the
operator verification gate.

## Private setup and evidence

Create a0600 JSON file matching private-config.schema.json outside the checkout.
`base_url` must equal the exact `/v1` loopback binding recognized by existing
budget_proxy.owned_hop. `proxy_token` is its ingress bearer token, never an upstream
provider credential. `model` is the approved exact model/wire alias; account for
Liter's source-defined leading-provider transform when composing that alias.
`budget_proxy_config` names the approved, already-running host proxy configuration.
No example secrets or permissive budget values are committed. The proxy must have
sufficient cumulative allowance for all three repetitions and prep consumption;
unknown reservations or exhausted caps fail acceptance. The shared proxy drains
already-dispatched upstream responses after downstream cancellation. The runner
waits up to90seconds for new reservations to settle without issuing another model
call. If upstream cannot settle, the unknown reservation fails the evidence run;
there is no accounting reset or fabricated successful cancellation receipt.
Known terminal non-success HTTP responses (including401and500) may settle only
with the approved binding/epoch/input identity, response bytes and hash, and finite
ordered reservation/outbound/settlement timestamps. They retain the full proxy
reservation and make their invocation FAIL, including a zero-exit XCTest invocation;
settlement alone never earns useful-success evidence. UNKNOWN, malformed terminals
and unresolved pending requests still block. The existing90second/overall deadline
bound remains unchanged, and successful forwarding still requires actual output usage.
Each read-only ledger snapshot now receives the remaining monotonic settlement
allowance and stays within the original overall deadline, including snapshots
outside settlement. SQLite connection/lock waits are clipped to the remaining
allowance (and the existing five-second SQLite ceiling); a progress handler
interrupts database reads at expiry, and the read connection closes deterministically.
Settlement checks both deadlines after snapshot acquisition and immediately before
returning, including a snapshot with no reservations. Polling sleeps are clipped
to the remaining allowance. Late completed receipts cannot earn settled success.
These deadline changes and five additional focused regression methods are authored
only; timing, SQLite interruption, and runtime behavior remain UNVERIFIED.
The runner and server share the pure `budget_proxy.terminal_receipt_error`
validator. The iOS snapshot projects each actual reservation and the approved
response ceiling: response bytes and successful output usage must fit both that
reservation and the binding ceiling. Terminal validation never changes ledger
accounting or replaces the existing native useful-success and per-invocation
failure requirements. These source changes and authored regressions are UNVERIFIED.

Receipts bind canonical source identity and inventory bytes, product closure,
SDK/runtime/UDID/architecture, repetition, process, app store, native run IDs and
canonical events. Source rows are compared to the frozen inventory at build/run;
installed products are compared before and after execution. The runner requires
actual new completed forwarding reservations from the same approved binding and
epoch, outbound-start evidence, and cumulative counters within existing caps.
A final PASS exists only after every XCTest invocation exits successfully and all
receipt/ledger predicates hold. No echo, nonempty-response, or compiler-only
substitute is accepted. No result currently exists.

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

Default-agent functionality now has a fresh challenge in both seed and restore
for every repetition. After protected-agent deletion refusal/readback, the seed
invokes the genuine default-agent with a new UUID and random arithmetic operands.
The restored process invokes default-agent again with a new UUID and fresh
operands after persisted state readback. Each response must equal exactly
`{"nonce": <fresh UUID>, "result": operand*multiplier+increment}`, computed on
the host; drain also requires the real RunDone terminal and no Error/Cancelled.
The expected arithmetic answer is absent from the model prompt. Both native
receipts retain nonce, operands, host expectation, run ID and canonical events.
This adds six real model calls across three repetitions to the approved cumulative
budget requirement. Source authored only; no result is claimed.

## Bounded same-item collection correction (source authored, CLOSED)

The run command now additionally requires `--results /PRIVATE/NEW-IOS-RESULTS`.
`--output` remains the built artifact; results must be a separate new, direct,
absolute directory. Existing result directories are refused before app installation.
The optional `--collection-control /PRIVATE/CONTROL.json` is a private host-written
input, not repair authority by itself. The suite writes it exclusively with its
run ID, same epoch, already consumed corrections, unchanged shared correction cap,
immutable controller/helper references and optional frozen correction authority.
The default standalone run has no correction authority or repair slots.

Six stable items are `ios:seed:1`, `ios:restore:1`, through repetition3. Each
restore depends only on its own successful seed. There are still11predicate
groups times3, using six initial XCTest invocations. A failed seed blocks its
restore. After confirmed owned cleanup and settled same-epoch accounting, a later
independent repetition may continue. Runtime/store cleanup requires successful
owned termination or the current identity-bound native receipt's explicit clean
shutdown predicates. A failure before that native receipt can remain BLOCKED when
termination is unconfirmed; process exit alone is insufficient evidence.

Each attempt gets a new seed/restore/repetition/revision directory with its log,
xcresult, pre-item baseline, invocation result and terminal receipt. Native bytes
are retained exactly before any same-item removal. Store-N must be a direct owned
directory without symlinks; baseline retention is bounded to10000entries,
16MiB per file and64MiB total, under the original deadline. Existing store baselines
are copied into the private attempt scope; an absent seed baseline is recorded as
absent. The runner never restores or deletes a store. The host must restore the
exact pre-item state before rerun. Subsequent repetitions never reuse store-N.

Optional actual authority is a separately frozen private JSON reference with
`schema_version:1`, `operator_approved:true`,
`operation:"correct-uar-ios-item"`, nonempty `authority_id`, exact initial
`source_identity`, `artifact_identity`, `epoch_id`, private owned
`receipt_directory`, bounded `wait_seconds` (at most3600), `max_corrections`
(at most6), `refreshable_artifact_roots`, `refreshable_inventory_paths`, and
`approved_effects` exactly `{refresh_source_inventory:true,
build_fresh_artifact:true,install_owned_app_preserving_container:true,
restore_exact_owned_store_baseline:true}`. These effect declarations describe
separately authorized host work; they never authorize this authoring session or
cause the runner to dispatch a repair/build/install command.

The existing failure_sweep receipt protocol emits an exact failed-item request,
then waits for a private host-produced self-sealed receipt. The seal supplies
tamper evidence, not authority. Existing required receipt fields remain:
`schema_version`, `authority_id`, `run_id`, `item_id`, `correction_number:1`,
exact `failed_receipt`, old/new source and artifact identities,
`targeted_correction_completed:true`, `artifact_refresh_completed:true`, and a
nonempty `description`. The iOS extension adds matching `approved_effects`,
`owned_baseline_restored` exactly matching the retained baseline descriptor, and
`refreshed_ios:{artifact_root,source_inventory:{path,sha256}}`. The actual current
store bytes must equal that baseline. Artifact roots must be distinct and
explicitly admitted. Source inventory bytes, actual source closure, selection,
product closure and new identities must match. Live Python/JavaScript helpers,
runner, interpreter, selection and private control/config remain immutable.
Any source/controller edit outside these rules requires a fresh approved packet.

At most one correction and one exact same-item rerun occur. There is no whole-run
retry, synthesized repair or automatic command execution from a receipt. Durable
`correction-consumed-N.json` is written before refresh admission; rejected refreshes
still consume that slot. The suite recovers these charges even without a child
summary. Global time, repair caps, forwarding reservations and epoch counters
carry all earlier macOS/Linux work. UNKNOWN accounting/ownership/cleanup stops
all dispatch. The result retains unresolved failures and original identities;
mixed successful source/product identities are BLOCKED. Exit0/1/2 represents
PASS/FAIL/BLOCKED and the final receipt remains consumable for ordinary failures.
No new source is certified by an earlier built XCTest product. Runtime behavior,
including every new correction transition, remains UNVERIFIED.

The authority file itself remains immutable while waiting. The installed app's
actual file closure is compared with the admitted built `.app` before and after
each invocation, in addition to the native receipt and test-product closure.
An updated source wave is reported through final_source_identity and
final_artifact_identity. Even a coherent new-source iOS PASS makes the suite
BLOCKED/requires_platform_rebind when macOS/Linux still name the earlier source.
Earlier platform receipts remain retained with their own identities.
