# Remaining graph and budget driver authoring

Phase Execute; this writer used source inspection and implementation/documentation
writes only. No tests, imports, checks, builds, network calls or formatter ran.
The lead owns the scoped local execution loop. Shared checkout edits were retained.

## Exact owned changes

- `tests/fixtures/client_tools_provider.py`: graph mode identifies the tool-free
  router from system/developer/native-system prompt identity and emits exactly
  `general-purpose`. Worker calls require their configured prompt marker, then use
  the existing caller-tool/result loop. Added controlled `provider_usage` with
  nonnegative integer `input_tokens` and `output_tokens`, preserved in request
  trace receipts and emitted on actual OpenAI JSON/SSE and Anthropic JSON/SSE
  responses. Existing default usage remains 2 input/3 output. The control endpoint
  rejects malformed synthetic usage instead of emitting incompatible wire data.
- `scripts/release/client_tools_acceptance.py`: existing graph CLI flags identify
  router (`--graph-before-marker`) and worker (`--graph-after-marker`), require
  `orchestrator-agent`, inspect prompt identity rather than arbitrary user text,
  and require router/worker/worker. Caller declaration assertions inspect the
  worker request. Cold trace matching uses the per-driver marker stem, since the
  router receives current input rather than the earlier conversation. Graph mode
  omits the ordinary-agent earlier-history assertion: the installed AgentNode
  explicitly defaults to HistoryForkMode::None. All ordinary matrix scenarios and
  their assertions remain in place. Added `--budget-provider-usage-json`, required
  when configuring warm/cold budget fixtures. Removed the false below-10-token
  provisioning guidance; no numeric ceiling or synthetic usage is chosen here.

## Supervisor interface

Provision `orchestrator-agent` through its existing route and persist the real
`general-purpose` specialist's verified loopback model and unique worker prompt,
preserving other fields. Pass router marker:
`You are a routing assistant. Respond with exactly one option from the provided list.`
Pass the unique worker system marker as the existing after-marker option. Keep
peer/settings/traces alive across an actual candidate stop/join/restart. Graph
counts require one router contact and two contacts at the selected worker.

For budget modes, pass a private JSON file containing exactly `input_tokens` and
`output_tokens`; values must be derived from the reservation proof below. The
fixture's control/trace surface affects only synthetic provider responses, never
the runtime ledger, private counters or persisted continuation rows.

## Unresolved budget bounds

`ModelCallBudget::begin_call` (`src/uar/runtime/cost_budget.rs`) reserves the sum
returned by `request_budget_bound` when a final-wire contract exists. Otherwise
it reserves catalog context-window plus maximum-output bounds. The request-bound
function (`src/llm/liter_driver.rs`) builds the real validated wire request,
counts those serialized bytes with the contract counter, rejects approximate
counting, adds input uncertainty, and reserves output plus additional reasoning.
Consequently the fixture's default reported 5 tokens cannot determine admission.

Actual first/resumed serialized bytes, contract uncertainty/output/reasoning
limits and verified source-catalog provider/model metadata remain unobserved.
No runnable cumulative-budget proof is claimed. The lead must establish R1 and
R2, then choose L and settled provider usage S satisfying R1 <= L, R2 <= L and
S + R2 > L, with usage inside the admitted response contract. A custom fixture
model cannot be assumed to supply catalog metadata. Retain the real wire usage
and admission evidence before crediting budget acceptance.

## Verification limits

All authored Python and graph/cold-continuation behavior remain unexecuted and
unverified. No scenario was removed from the existing matrix. Independent Python
review and lead-authorized production execution remain outstanding. This handoff
does not open the verification gate or authorize external publication.

## Gate005 observed driver failure repair

The lead reported `AttributeError` at the held disconnect path because global
`def http` shadows the package bound by `import http.client`. Aliased the module
to `http_client` and changed the HTTPSConnection/HTTPConnection selection only;
the actual socket disconnect and held upstream behavior are retained.

The failed driver did not reach end-of-run reporting, losing earlier completed
case receipts. Each recorded case now writes an explicitly incomplete report
checkpoint. An unexpected Exception records a failed case and fatal error,
writes retained observations, then re-raises instead of continuing the suite.
The normal final report alone marks execution_complete true; scenario failures
and coverage gaps still determine the existing nonzero exit statuses. No test
case was removed or converted to a successful result.

This repair is source-only: no Python import, execution, test, check, formatting,
build or network operation ran. The lead must rerun the authorized driver gate.

## Mounted OpenAI legacy message alias regression

The lead reported a reviewed production defect: the caller-tool branch screened
legacy `message` while executing `messages`. The existing OpenAI reject:history
case now first captures a fresh native caller-tool request through the existing
CapturedRequest mechanism, adds a conflicting legacy `message`, and submits it
to the actual captured HTTP route with its authenticated token/headers. It
requires a 4xx response and unchanged real provider contact count, then continues
the existing durable-history tampering regression. The case receipt includes
the alias rejection status; no new case or transport interface was added.

Only `scripts/release/client_tools_acceptance.py` and this handoff changed for
this regression. All 95 matrix cases remain. No execution, import, check,
network request, formatting or build ran; the authored regression remains
unverified pending the lead's full matrix against build007. The source-only
inspection/edit commands do not establish the runtime branch is repaired.

## Ordinary cold ACP fresh-run regression

After ordinary ACP restart-resume completes the pending batch and its identical
retry without provider execution, the driver now sends an actual runs/create RPC
on that same restored session with a unique new input and no client_tools field.
It requires a fresh real provider request containing the new input, response
client_tools history containing that user marker, and new durable caller IDs
disjoint from the completed batch. This exercises restored session declaration
defaults without replaying saved old history or overriding the selected agent.

The ordinary restart receipt retains acp_fresh_default_runs with session identity,
new marker, provider-contact count, new caller IDs and omission/history assertions.
Graph and budget restart variants do not execute this extra case. No original
matrix case or safety case changed. The new handoff is intentionally left pending
after observing its public caller boundary; no host effect is requested.

Only the acceptance driver and this handoff changed for this delta. No Python
execution/import, test, check, build, formatting or network command ran. The
regression remains unverified until the lead's authorized cold-production gate.
