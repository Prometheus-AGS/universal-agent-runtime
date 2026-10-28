# Task2.1 core source handoff

2026-10-25. Active phase uar-post-lifecycle-verification-readiness, Execute.
Verification CLOSED. Source authoring only; no imports, runner, checks, tests,
builds, services, model calls or review gates executed. Shared checkout preserved.

Owned authored files: scripts/acceptance/core.py, user_outcomes.py,
cases_prompt.py, fixtures/fixture.example.json, README.md and this handoff.
No production source or canonical state edited. No out-of-scope additions.

API and precise baseline observer input/output are in scripts/acceptance/README.md.
Agent-state received the API before writing its separate cases_agents.py.
Case extension API: cases()->list[Case]; Case(id,family,callable,repetitions=3,
durable=False); callable(ctx)->Outcome(Predicate(bool,detail),Predicate(bool,detail),
facts). Context provides HTTP request/record/observe/restart and owned process APIs.
Prompt helper returns observed content/native run/session/inspection/accounting/
provenance; requires separate useful-result grading. expected_model/send_model
support saved-default/session precedence without adding a request model selector.

Baseline hooks to be implemented by integration owner in observers_runtime.py:
prepare, ready, admission, inference, session, cleanup. They are mandatory argv
JSON programs, not placeholders that count as implementation-complete. See the
README table for exact inputs and fields. Envelope run_id is packet UUID;
inference values.run_id is actual chatcmpl suffix/native run identity. Relevant
mounted sources read: server.rs ChatCompletionRequest and api_chat_completion
(/v1/chat/completions); api/providers.rs ProviderConfig CRUD and /{id}/test;
api/routes.rs RunInspection GET /api/uar/runs/{id}; api/accounting.rs GET
/api/uar/accounting/calls/{run_id}; persistence/accounting.rs AccountingCall.

Observed source schemas: chat response id=chatcmpl-{native_run_id}, model is
provider/model, session_id UUID, choices[0].finish_reason=stop; run inspection
status=done, agent_id, agent_revision, conversation_id and effective_model
{provider_id,model_id}; accounting calls list admission.identity, admission.model,
state=settled, observed_tokens and reconciliation_evidence. Provider GET returns
ProviderView directly and never api_key. POST create is201 or409 if existing;
helper GETs first, creates only if404, refuses mismatching configured fields,
then performs genuine /test with {model:model_id}. That extra inference must be
counted in the future authority packet. No configured binding is overwritten.

Proof gaps remain: actual runtime observer implementations and exact disposable
DB lifecycle/driver settings; session canonical storage receipt extractor;
genuine endpoint provenance extractor; executable frozen fixture and source/
installed-artifact/host-budget identities; full25-family instantiated manifest;
remaining case modules and all runtime verification. The example intentionally
has executable=false and empty argv/frozen_files; these block rather than pass.
Task2.1 is source-authored foundation, not a whole-phase implementation-complete
or acceptance claim. Root owns runtime observers and integration; state teammate
owns startup observers. U02 covers warm+cold private fact/canonical order and
real process restart; owner isolation remains its separately required family.

Guards trace to named plan failures: synthetic positive responses, RUN_ERROR
accepted as success, missing Postgres green skips, provider mismatches,
unimplemented scenario PASS, and cleanup before awaited owned work. Redaction,
loopback origin, exact frozen inputs and owned process/resource cleanup enforce
real credential/network/fixture trust boundaries. No speculative runtime guards.

Commands used: source inspection via cat/sed/rg and authorized mkdir/heredoc/
apply_patch writes only. No verification result exists. Syntax/import behavior,
mounted runtime compatibility, observers, both backend journeys and useful
provider outcomes remain unverified because the operator lock is CLOSED.

Integration follow-up: optional startup_initial_capture runs after prepare before
initial server launch. Named private mode600 temporary server stdout/stderr paths
are exported to observer environment as UAR_ACCEPTANCE_SERVER_STDOUT_PATH and
UAR_ACCEPTANCE_SERVER_STDERR_PATH; each restart updates them. Structured SSE
JSON events, terminals and typed errors are retained (private reasoning removed);
malformed opaque blocks are explicitly evidence-incomplete/unrecovered. Cleanup
has a separate finite cleanup_seconds allowance after work wall exhaustion.
Process teardown now retains bounded redacted JSON/NDJSON stdout/stderr receipts
before closing private temporary files. Nonstructured text remains hashed with
explicit evidence-incomplete/unrecovered state. Fixture evidence binds the exact
bytes loaded, rather than a later reread; failed admission retains only the raw
fixture digest and failure class so unadmitted secret fields cannot leak.
