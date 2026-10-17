# R7 runtime repairs — implementation authored, unexecuted

Phase: `uar-outstanding-issues-v1-1-1`, Execute. The lead reclosed the entire
implementation boundary after the round-1 BLOCK. This handoff covers assigned
V1-01, V1-02 and the runtime-adapter portion of V1-03. It does not clear the
review verdict or certify any runtime result. V1-04 and shared support/provenance
remain lead-owned.

## Exact changed paths

- `src/uar/runtime/turn/bindings.rs`: the settings-profile fallback branch now
  uses existing `Orchestrator::fallback_llm_config` and explicitly sets
  `resolved_provider_id` from the captured fallback endpoint profile before
  applying the fallback credential decision. This prevents primary identity
  from contaminating a distinct provider's authorized fallback capture.
- `src/uar/runtime/manager.rs`: the fallback credential-decision block returns
  an empty decision map when effective failover is disabled. Populated inert
  fallback configuration no longer resolves credentials or adds a dependency to
  an authorized primary run. Enabled fallback resolution still propagates denial.
- `scripts/release/uar_1_1_1_runtime.py`: extends actual fallback/team scenarios,
  switches secret-leak checks to raw in-memory logs, and uses a pinned native
  model alias that supports the existing durable budget control.
- `tests/fixtures/uar_1_1_1_provider.py`: updates the native response model alias
  to match the pinned fixture selection; no parser behavior is replaced.
- This handoff.

No Rust support fixture, manifest, dependency pin, workflow, shared acceptance
runner or shared support module was edited for this repair. Others' source edits
are retained. The lead explicitly transferred the two production Rust files.

## Required real integration controls

All cases remain mandatory inside the existing installed `binding` group and
its closed `REQUIRED_SCENARIOS`. No helper-only test or source reference earns an
installed pass. No extra runner/source case registration is needed.

`binding-team-source-policy` now requires four actual team executions:

- `primary-denied`: primary source ungranted; established
  `TEAM_PROVIDER_REQUEST_REJECTED` with diagnostic reference, zero endpoint calls.
- `primary-allowed`: exact OpenAI config-source grant; one primary request and
  successful terminal/text/AG-UI observations.
- `distinct-fallback-denied`: primary `openai/gpt-4o-mini` granted, fallback
  `azure/gpt-4o-mini` ungranted; zero primary and fallback calls, established team
  refusal and durable execution-attempt readback.
- `distinct-fallback-allowed`: both exact source tuples granted, separately
  configured and administratively verified providers, captured settings-profile
  routes. The primary peer returns HTTP 503, then exactly one fallback request
  reaches the separate peer. Assertions require call order, admitted bare model
  alias, `/chat/completions` on both routes, complete terminal/text/AG-UI success
  and retained team execution attempt.

The fixture installs a sealed package with an explicit fallback model role and
alias, installs both deployment model bindings with settings-profile revision1,
creates a team/task, claims it, and admits the actual team attempt over public
HTTP. Both provider/model pairs exist in the pinned catalog with finite pricing
and 128000 context + 16384 max-output capacity. The existing finite1million-token
team reservation accommodates both conservative requests; budgets are retained.

`binding-fallback-policy` now requires:

- `enabled-denied`: ungranted fallback rejects capture before either endpoint.
- `enabled-allowed`: one primary HTTP 503 followed by one authorized fallback
  request, with exact request-order and success assertions.
- `disabled-denied-inert`: identical populated ungranted fallback configuration,
  effective failover disabled, successful primary, exactly one primary request
  and zero fallback requests. This is the actual V1-02 regression control.

## Raw-log negative evidence

The adapter consumes the lead-authored `RuntimeLease.logs(redact=False)` only
inside transient membership checks. Binding, exact16000 budget and team cases
assert absence of the synthetic fixture key against raw runtime logs. They retain
only a `raw_log_secret_absent` boolean. Error messages are fixed safe descriptions
and never interpolate raw logs or secret values. Trusted diagnostic lines and
all saved receipts still use the default redacted log path and `_save` redaction.
The shared resources layer retains only redacted command/container output.

The prior check over already-redacted logs could not detect a leak. This repair
changes what the negative assertion observes; it is not evidence that production
logs previously leaked a credential.

## Fixture catalog correction

During source selection, the old native fixture alias
`claude-sonnet-4-20250514` was found absent from the pinned catalog. The lead
explicitly approved switching the fixture to `claude-haiku-4-5-20251001`, present
with 200000 context + 64000 max output. The native response alias matches. This
keeps the existing finite1million positive bound meaningful without changing a
production budget or reservation algorithm. The native framing matrix computes
its expected bytes from the same fixture encoder and still exercises the actual
Anthropic HTTP/stream parser.

## Verification and unresolved limits

Only source inspection and file authoring occurred. No tests, imports, builds,
compiler probes, checks, lint, formatting, validators or independent review ran.
No result from the earlier candidate certifies these changes. Compileability,
team fallback dispatch, disabled-fallback compatibility and raw-log observations
remain unverified until the lead records a fresh implementation-complete inventory
and runs the authorized phase gate. Fresh independent artifact-only review remains
pending and must include these production and fixture changes.

The guard added in manager traces directly to V1-02. The provider identity fix
traces to V1-01. Raw-log detection traces to V1-03. The fixture alias correction
traces to an observed missing pinned catalog entry and was explicitly approved.
There are no speculative production guards or unrelated additions.

Use the existing phase acceptance entrypoint with the lead's new manifest after
the new boundary. Existing source supplements and the six mandatory groups are
unchanged. No external prerequisite was introduced beyond the already-required
actual image, disposable service/store/relay collaborators and lead-controlled
verification/review availability.
