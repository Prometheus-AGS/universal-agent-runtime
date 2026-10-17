# R5 integration authoring handoff

Phase: `uar-outstanding-issues-v1-1-1`; Execute. Implementation authored;
**UNEXECUTED and pending independent review and completed-phase integration**.
The lead retains the authoritative waypoint, L4 inventory and gate ownership.

## Exact changed paths

- `scripts/release/uar_1_1_1_runtime.py`: replaces placeholder result labels with
  assertion-backed installed-runtime scenario bodies, closed mandatory inventories,
  provider administrative verification setup, native Anthropic selection, public
  run/status/events/AG-UI observations, actual MCP registration at the image's
  `/opt/uar/mcp.json`, canonical receipt readback, and team source controls.
- `tests/fixtures/uar_1_1_1_provider.py`: native Anthropic and OpenAI wire peer;
  recorded request/chunk/digest trace; atomic readiness/mode publication;
  real HTTP chunked writes; arbitrary two-way and fixed/bytewise partitions;
  CR/LF/CRLF, initial/partial BOM, Unicode, comments/unknown fields/multiline data,
  spaces, unknown event kinds, malformed known data, invalid UTF-8, overflow,
  terminal/EOF, cancellation and model-selected MCP modes.
- `tests/fixtures/truthful_tool_outcome_mcp.py`: timeout mode sleeps 90 seconds,
  longer than the runtime's 30-second MCP response deadline. All existing modes
  and fsynced effect trace are retained. Lead explicitly transferred this file.
- `tests/support/uar_1_1_1_runtime_fixture.rs` (NEW): real RunManager + SurrealKV +
  HTTP model source supplements for bindings and a native tool's incidental
  `isError` payload. Peer task is aborted and joined; manager graph/terminal/
  sandbox shutdown is explicit.
- `tests/bauar_full_harness_cursor.rs`: adds only two phase-specific integration
  entrypoints and the required `FullHarnessApiState.contexts` initializer for the
  current production structure. Existing cases are not selected by this phase.
- This handoff.

No dependency pin, manifest, vendor/submodule, production runtime or unrelated
fixture was edited. No other writer's work was reverted.

## Installed-image mandatory scenario inventory

`binding`:

- `binding-qualified-durable-control`: explicitly bounded actual model call,
  real accounting record, restart byte-equivalent accounting readback and a later
  valid bounded call. Uses a finite 1,000,000-token ceiling so the real full-request
  reservation can fit; this is fixture policy, not a changed product default.
- `binding-unattested-refusal`: public `orchestrator_start_failed`, safe diagnostic
  reference, trusted `remote_surreal_durability_not_attested`, zero model contact.
- `binding-16000-catalog-reservation-refusal`: added by the customer followup;
  see `R5-budget-followup.md` for the exact durable catalog-bound case. The prior
  PostgreSQL unsupported-storage claim was incorrect: PostgreSQL supports durable
  accounting. That image row is removed; actual unsupported InMemoryProvider
  coverage is a local source supplement in the existing binding case.
- `binding-v1-messages-policy`: shared denied and exact-grant allowed direct native
  Messages route, exact zero/one contact assertion and complete permitted stream.
- `binding-shared-alias-and-credential-free`: mismatched alias grant denies,
  exact alias grant succeeds; explicit credential-free capture emits no ambient
  key despite a synthetic `OPENAI_API_KEY` in the child environment.
- `binding-fallback-policy`: denied fallback source refuses before primary I/O;
  allowed captured fallback answers after real primary HTTP 503.
- `binding-timeout-inheritance`: per-agent catalog OpenAI selection inherits
  2-second timeout and zero retries; delayed provider responds after 6 seconds,
  requiring earlier failed settlement and exactly one wire request.
- `binding-team-source-policy`: sealed definition/package/binding installation,
  actual team creation/task claim/admission, allowed and denied shared-source
  capture, run projection, contact counts and durable execution-attempt readback.
- `binding-summary-capture-policy`: summarize strategy selects captured primary,
  exact allowed/denied source policy controls, and zero extra helper requests.
- `binding-invalid-budget-refusal`: negative cost budget, safe public diagnostic,
  trusted `budget_admission_failed`, zero provider effects. Consumes the lead's
  narrow `PolicyIntersectionError` classification fix.

`streams`:

- `streams-native-partition-matrix`: native `/v1/messages` is asserted for every
  actual request; every byte cut, whole and bytewise writes for LF/CRLF/CR with
  and without BOM; equal normalized event-name sequence and exact Unicode text.
- `streams-sse-fields-fixed-partitions`: comments, ignored fields, multiline JSON
  data and spaces, with BOM and 1/2/7/31-byte writes for all three newline forms.
- `streams-eof-matrix`: empty, metadata-only, text-only, partial-tool,
  complete-tool EOF and missing terminal fail with no pending tool effects.
- `streams-invalid-matrix`: known malformed JSON/shape, overflow, partial BOM,
  invalid UTF-8 and post-terminal data fail; unknown future event succeeds.
- `streams-tool-terminal-control`: real native model-selected MCP loop succeeds,
  one effect and one canonical successful receipt.
- `streams-cancel-cleanup`: cancel after peer contact, cancelled terminal
  precedence, then a separate valid run succeeds.
- `streams-graph-inherited-control`: installed built-in orchestrator graph,
  actual child-start projection and complete terminal/text control.

`mcp-outcomes`:

- `mcp-application-error`, `mcp-explicit-success-and-omitted-status`,
  `mcp-protocol-and-timeout`, `mcp-uncertain-effect-preserved`: actual model-selected
  MCP application true/false/omitted result, dispatched JSON-RPC error, deadline
  and lost response. Assertions compare ToolEnd, public outcome and canonical
  receipt, preserve text/image/resource/structured/meta payloads, count exactly
  one original effect, prohibit model replay for unknown outcome, restart,
  execute later independent success, and compare original typed receipt unchanged.
  Dispatched protocol/timeout/disconnect are `outcome_unknown`, not proof of no
  effect. The fixture records and asserts that exact production classification.

`combined`:

- `combined-failed-tool-incomplete-stream`: application-failed MCP followed by
  incomplete native Anthropic final stream settles error and preserves failed
  receipt through restart and independent recovery.
- `combined-valid-control`: separate terminal native tool-loop succeeds.

## Exact local source supplements for lead runner

Target: `bauar_full_harness_cursor`; names passed with `--exact` and one test thread:

- `phase_runtime_binding_refusals_use_real_host_store_and_provider` (`binding`).
  Requires the subprocess environment `UAR_LOCAL_ONLY=1`, asserted by the test.
  That makes malformed synthetic endpoint `http://[phase-constructor-secret`
  fail in the actual provider construction egress guard, whose internal error
  includes the offending URL. Public events must omit both endpoint marker and
  provider key. Real local peer remains eligible. Also exercises absent verified
  accounting principal, malformed budget and qualified durable positive control.
  The customer followup adds actual unsupported InMemoryProvider coverage with
  the `in-memory-backend` feature; this source seam is never image evidence.
- `phase_runtime_native_incidental_error_retains_success` (`mcp-outcomes`).
  Actual model-selected NativeSkill through normal host manager/persistence;
  `{"isError":true,"native_payload":42}` remains successful native data in
  ToolEnd and canonical receipt, with exactly two model calls.

The separate trust-owned native SDK supplied-driver case remains in the
`credentials` group. Its source case is never represented as a published-image
scenario; the previous fake `binding-safe-constructor-refusal` source-pointer row
has been removed.

## Interfaces and prerequisites

Consumes lead `base_config`, `new_runtime`, `jwt`, `resources.start_process`,
`resources.relay`, `resources.surreal_query`, `resources.redact`, `wait_for` and
`require`. Returns `status` and exact `scenarios[{id,status,observations}]`.
Provider traces stay private under the runner scratch root; public evidence is
redacted and chmod 0600. Peers use synthetic keys, never operator credentials.
MCP effect directory is writable by image uid65532 under the private runner root.

Actual Docker runtime, isolated attested durable Surreal service, supported local
KV, pinned PostgreSQL collaborator and Python in the installed image are required.
A missing collaborator fails/blocks the group; no environment absence creates a
passing row. Lead support must override image default persistence environment
with the requested effective backend; otherwise exact SQL readback will expose
that the installed image used another store. Lead was notified of this source
configuration concern; no runtime finding is claimed.

## Deliberate limits / uncomfortable case

Production summarization currently supplies neither final-wire helper budget
contract nor host-authorized prose marks. It deliberately performs no helper
inference. The summarize-selected real run proves capture and absence of an
ungranted helper; it does **not** claim summary inference qualification or enable
that dormant path.

HTTP chunk writes are controlled and traced; downstream TCP chunk coalescing is
not observable or guaranteed. Exhaustive two-way writes exercise real HTTP input,
not a promise that the OS delivers each application write as a separate read.

The full byte-split matrix is intentionally large. Parent runner must budget for
all declared cases and retain interrupted/failed receipts; it must not silently
sample the matrix or substitute an old passing stream suite. Source supplements
cannot certify published images. Controlled fixtures do not attest customer
storage or third-party model availability.

## Verification and ownership return

No test, check, build, formatter, lint, compiler/import probe, schema validator,
review or acceptance command ran. Commands used only read repository source or
write the assigned fixtures/handoff. There is no passing runtime evidence yet.
All acceptance and independent review remain with the lead after L4. Added
assertions trace to the phase's explicit R1–R5/T2 contracts; no speculative product
hardening or unrelated feature was added.
